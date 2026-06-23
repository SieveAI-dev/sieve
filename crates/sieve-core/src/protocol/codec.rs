//! A1 ProviderCodec：把出站请求的 provider schema 操作（解析 / 提取待检文本 / 写回脱敏）
//! 收编成可插拔 codec，使检测核心只面对统一接口。
//!
//! 网关上游分层（A1）：新增一个上游 = 实现一个 [`ProviderCodec`]，daemon 出站编排不再重复
//! provider 专属逻辑。检测核心（`OutboundFilter` + 脱敏）只调用 [`DecodedRequest`] 的
//! provider 无关方法。关联 ADR-004（Anthropic 优先统一接口）/ ADR-018（OpenAI 协议适配）。

use crate::error::{SieveCoreError, SieveCoreResult};
use crate::protocol::anthropic::{AnthropicMessage, AnthropicRequest};
use crate::protocol::openai::{OpenAIMessage, OpenAIRequest};
use crate::protocol::unified_message::UpstreamProvider;

/// 出站 codec：每家上游一个 impl，把 provider schema ↔ 统一操作隔离。
pub trait ProviderCodec: Send + Sync {
    /// 该 codec 对应的上游 provider 标记。
    fn provider(&self) -> UpstreamProvider;

    /// 解析出站请求 body → 可提取待检文本 / 可写回脱敏的 [`DecodedRequest`]。
    ///
    /// # Errors
    /// body 不是该 provider 合法请求 schema 时返回 [`SieveCoreError::Protocol`]，
    /// 调用方据此回退到原样转发（让上游自行返 4xx）。
    fn decode_request(&self, body: &[u8]) -> SieveCoreResult<DecodedRequest>;
}

/// 已解码的出站请求（provider 无关接口）。持有具体 schema，只暴露统一操作。
pub enum DecodedRequest {
    /// Anthropic Messages API 请求。
    Anthropic(AnthropicRequest),
    /// OpenAI Chat Completions 请求。
    OpenAi(OpenAIRequest),
}

impl DecodedRequest {
    /// 上游 provider 标记。
    #[must_use]
    pub fn provider(&self) -> UpstreamProvider {
        match self {
            Self::Anthropic(_) => UpstreamProvider::Anthropic,
            Self::OpenAi(_) => UpstreamProvider::OpenAI,
        }
    }

    /// 请求声明的 model（billing / 审计用）。
    #[must_use]
    pub fn model(&self) -> &str {
        match self {
            Self::Anthropic(r) => &r.model,
            Self::OpenAi(r) => &r.model,
        }
    }

    /// 提取待检文本段：`(累计字节偏移, 文本)` 列表（顺序即检测 / 写回的 segment 顺序）。
    #[must_use]
    pub fn extract_text_content(&self) -> Vec<(usize, String)> {
        match self {
            Self::Anthropic(r) => r.extract_text_content(),
            Self::OpenAi(r) => r.extract_text_content(),
        }
    }

    /// 把脱敏后的文本段写回请求，返回新 body 字节（保持原 schema）。
    ///
    /// `original_texts` 为 [`Self::extract_text_content`] 的原始段列表；`redacted_texts` 为脱敏后
    /// 文本（顺序对应）。遍历顺序与 `extract_text_content` 严格一致。
    ///
    /// # Errors
    /// `redacted_texts` 与 `original_texts` 长度不一致，或重序列化失败时返回 [`SieveCoreError::Protocol`]。
    pub fn apply_redacted_texts(
        &self,
        original_texts: &[(usize, String)],
        redacted_texts: &[String],
    ) -> SieveCoreResult<Vec<u8>> {
        match self {
            Self::Anthropic(r) => {
                let new_req = apply_redacted_anthropic(r, original_texts, redacted_texts)?;
                serde_json::to_vec(&new_req)
                    .map_err(|e| SieveCoreError::Protocol(format!("reserialize anthropic: {e}")))
            }
            Self::OpenAi(r) => {
                let new_req = apply_redacted_openai(r, original_texts, redacted_texts)?;
                serde_json::to_vec(&new_req)
                    .map_err(|e| SieveCoreError::Protocol(format!("reserialize openai: {e}")))
            }
        }
    }
}

/// Anthropic Messages API 出站 codec。
#[derive(Debug, Clone, Copy, Default)]
pub struct AnthropicCodec;

impl ProviderCodec for AnthropicCodec {
    fn provider(&self) -> UpstreamProvider {
        UpstreamProvider::Anthropic
    }

    fn decode_request(&self, body: &[u8]) -> SieveCoreResult<DecodedRequest> {
        serde_json::from_slice::<AnthropicRequest>(body)
            .map(DecodedRequest::Anthropic)
            .map_err(|e| SieveCoreError::Protocol(format!("anthropic request: {e}")))
    }
}

/// OpenAI Chat Completions 出站 codec。
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenAiCodec;

impl ProviderCodec for OpenAiCodec {
    fn provider(&self) -> UpstreamProvider {
        UpstreamProvider::OpenAI
    }

    fn decode_request(&self, body: &[u8]) -> SieveCoreResult<DecodedRequest> {
        serde_json::from_slice::<OpenAIRequest>(body)
            .map(DecodedRequest::OpenAi)
            .map_err(|e| SieveCoreError::Protocol(format!("openai request: {e}")))
    }
}

// ─── 脱敏写回（逐字搬自 sieve-cli daemon.rs apply_redacted_texts_to_*，A1 收编）──────
// 行为字节级等价：anyhow! → SieveCoreError::Protocol，类型路径改 crate::，逻辑一字不动。

/// 把脱敏后文本段写回 [`AnthropicRequest`]（修 #1 AutoRedact 偏移修复；遍历顺序须与
/// `extract_text_content` 一致）。
fn apply_redacted_anthropic(
    req: &AnthropicRequest,
    original_texts: &[(usize, String)],
    redacted_texts: &[String],
) -> SieveCoreResult<AnthropicRequest> {
    if original_texts.len() != redacted_texts.len() {
        return Err(SieveCoreError::Protocol(format!(
            "redacted_texts 长度 {} 与 original_texts 长度 {} 不一致",
            redacted_texts.len(),
            original_texts.len()
        )));
    }

    let mut seg_idx = 0usize;

    let mut new_messages: Vec<AnthropicMessage> = Vec::new();
    for msg in &req.messages {
        let new_content = match &msg.content {
            serde_json::Value::String(_) => {
                let replacement = redacted_texts
                    .get(seg_idx)
                    .cloned()
                    .unwrap_or_else(|| msg.content.as_str().unwrap_or("").to_string());
                seg_idx += 1;
                serde_json::Value::String(replacement)
            }
            serde_json::Value::Array(blocks) => {
                let mut new_blocks = Vec::with_capacity(blocks.len());
                for block in blocks {
                    if let Some(block_obj) = block.as_object() {
                        if block_obj.get("type").and_then(|v| v.as_str()) == Some("text")
                            && block_obj.get("text").and_then(|v| v.as_str()).is_some()
                        {
                            let replacement =
                                redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                                    block_obj
                                        .get("text")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string()
                                });
                            seg_idx += 1;
                            let mut new_obj = block_obj.clone();
                            new_obj
                                .insert("text".to_string(), serde_json::Value::String(replacement));
                            new_blocks.push(serde_json::Value::Object(new_obj));
                            continue;
                        }
                    }
                    new_blocks.push(block.clone());
                }
                serde_json::Value::Array(new_blocks)
            }
            other => other.clone(),
        };
        new_messages.push(AnthropicMessage {
            role: msg.role.clone(),
            content: new_content,
        });
    }

    let new_system = if let Some(system) = &req.system {
        if system.as_str().is_some() {
            let replacement = redacted_texts
                .get(seg_idx)
                .cloned()
                .unwrap_or_else(|| system.as_str().unwrap_or("").to_string());
            seg_idx += 1;
            Some(serde_json::Value::String(replacement))
        } else if let Some(blocks) = system.as_array() {
            let mut new_blocks = Vec::with_capacity(blocks.len());
            for block in blocks {
                if block.get("text").and_then(|v| v.as_str()).is_some() {
                    let replacement = redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                        block
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    });
                    seg_idx += 1;
                    let mut new_obj = block.as_object().cloned().unwrap_or_default();
                    new_obj.insert("text".to_string(), serde_json::Value::String(replacement));
                    new_blocks.push(serde_json::Value::Object(new_obj));
                } else {
                    new_blocks.push(block.clone());
                }
            }
            Some(serde_json::Value::Array(new_blocks))
        } else {
            Some(system.clone())
        }
    } else {
        None
    };

    let _ = seg_idx; // 消除 unused variable 警告

    Ok(AnthropicRequest {
        model: req.model.clone(),
        max_tokens: req.max_tokens,
        messages: new_messages,
        stream: req.stream,
        system: new_system,
        tools: req.tools.clone(),
        tool_choice: req.tool_choice.clone(),
        extra: req.extra.clone(),
    })
}

/// 把脱敏后文本段写回 [`OpenAIRequest`]（修 A2-#1；`message.content` string / content-part array
/// 两形态，image_url 等非文本 part 原样保留不计 segment）。
fn apply_redacted_openai(
    req: &OpenAIRequest,
    original_texts: &[(usize, String)],
    redacted_texts: &[String],
) -> SieveCoreResult<OpenAIRequest> {
    if original_texts.len() != redacted_texts.len() {
        return Err(SieveCoreError::Protocol(format!(
            "redacted_texts 长度 {} 与 original_texts 长度 {} 不一致",
            redacted_texts.len(),
            original_texts.len()
        )));
    }

    let mut seg_idx = 0usize;
    let mut new_messages: Vec<OpenAIMessage> = Vec::new();

    for msg in &req.messages {
        let new_content = match &msg.content {
            Some(serde_json::Value::String(_)) => {
                let replacement = redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                    msg.content
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                });
                seg_idx += 1;
                Some(serde_json::Value::String(replacement))
            }
            Some(serde_json::Value::Array(parts)) => {
                let mut new_parts = Vec::with_capacity(parts.len());
                for part in parts {
                    if let Some(obj) = part.as_object() {
                        if obj.get("type").and_then(|v| v.as_str()) == Some("text")
                            && obj.get("text").and_then(|v| v.as_str()).is_some()
                        {
                            let replacement =
                                redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                                    obj.get("text")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string()
                                });
                            seg_idx += 1;
                            let mut new_obj = obj.clone();
                            new_obj
                                .insert("text".to_string(), serde_json::Value::String(replacement));
                            new_parts.push(serde_json::Value::Object(new_obj));
                            continue;
                        }
                    }
                    new_parts.push(part.clone());
                }
                Some(serde_json::Value::Array(new_parts))
            }
            other => other.clone(),
        };
        new_messages.push(OpenAIMessage {
            role: msg.role.clone(),
            content: new_content,
            name: msg.name.clone(),
            tool_calls: msg.tool_calls.clone(),
            tool_call_id: msg.tool_call_id.clone(),
            extra: msg.extra.clone(),
        });
    }

    let _ = seg_idx; // 消除 unused variable 警告

    Ok(OpenAIRequest {
        model: req.model.clone(),
        messages: new_messages,
        stream: req.stream,
        tools: req.tools.clone(),
        max_tokens: req.max_tokens,
        temperature: req.temperature,
        extra: req.extra.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 迁自 daemon.rs（A1 收编）：OpenAI string content 的 secret 经 codec 脱敏写回。
    #[test]
    fn openai_redact_string_content() {
        let raw_token = "sk-ant-api03-AABBCCDD1234";
        let json = format!(
            r#"{{"model":"gpt-4","messages":[{{"role":"user","content":"my key is {raw_token}"}}]}}"#
        );
        let decoded = OpenAiCodec.decode_request(json.as_bytes()).expect("valid openai");
        let texts = decoded.extract_text_content();
        assert_eq!(texts.len(), 1);
        let redacted = vec!["my key is [REDACTED:OUT-01]".to_string()];
        let new_body = decoded
            .apply_redacted_texts(&texts, &redacted)
            .expect("apply ok");
        let new_json = String::from_utf8(new_body).unwrap();
        assert!(!new_json.contains(raw_token), "脱敏后不应含原 token: {new_json}");
        assert!(new_json.contains("[REDACTED:OUT-01]"), "应含占位符: {new_json}");
    }

    /// 迁自 daemon.rs：array-of-content-parts，text part 脱敏、image_url part 原样保留。
    #[test]
    fn openai_redact_array_content_parts() {
        let raw_token = "sk-ant-api03-XXYZZY9876";
        let json = format!(
            r#"{{"model":"gpt-4","messages":[{{"role":"user","content":[{{"type":"text","text":"key={raw_token}"}},{{"type":"image_url","image_url":{{"url":"https://example.com/img.png"}}}}]}}]}}"#
        );
        let decoded = OpenAiCodec.decode_request(json.as_bytes()).expect("valid openai");
        let texts = decoded.extract_text_content();
        assert_eq!(texts.len(), 1, "只有 text part 计 segment");
        let redacted = vec!["key=[REDACTED:OUT-01]".to_string()];
        let new_json =
            String::from_utf8(decoded.apply_redacted_texts(&texts, &redacted).unwrap()).unwrap();
        assert!(!new_json.contains(raw_token));
        assert!(new_json.contains("[REDACTED:OUT-01]"));
        assert!(new_json.contains("image_url"), "image_url 应保留: {new_json}");
    }

    /// 迁自 daemon.rs：长度不一致返回错误（不 silent fail）。
    #[test]
    fn openai_redact_mismatched_lengths_returns_error() {
        let json = r#"{"model":"gpt-4","messages":[{"role":"user","content":"hello"}]}"#;
        let decoded = OpenAiCodec.decode_request(json.as_bytes()).expect("valid openai");
        let texts = decoded.extract_text_content();
        let bad: Vec<String> = vec![];
        assert!(decoded.apply_redacted_texts(&texts, &bad).is_err());
    }

    /// Anthropic 侧 codec 对称单测：decode → extract → apply 往返。
    #[test]
    fn anthropic_decode_extract_redact_roundtrip() {
        let json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"secret here"}]}"#;
        let decoded = AnthropicCodec
            .decode_request(json.as_bytes())
            .expect("valid anthropic");
        assert!(matches!(decoded.provider(), UpstreamProvider::Anthropic));
        assert_eq!(decoded.model(), "claude-sonnet-4-5");
        let texts = decoded.extract_text_content();
        assert_eq!(texts.len(), 1);
        let new_json = String::from_utf8(
            decoded
                .apply_redacted_texts(&texts, &["[REDACTED:OUT-01]".to_string()])
                .unwrap(),
        )
        .unwrap();
        assert!(new_json.contains("[REDACTED:OUT-01]"));
        assert!(!new_json.contains("secret here"));
    }

    /// 非法 body → decode_request 返回错误（调用方据此回退原样转发）。
    #[test]
    fn decode_invalid_body_errors() {
        assert!(AnthropicCodec.decode_request(b"not json").is_err());
        assert!(OpenAiCodec.decode_request(b"not json").is_err());
    }
}
