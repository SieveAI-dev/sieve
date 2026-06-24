//! Anthropic Messages API 请求/响应 schema（子集）。
//!
//! 文档: <https://docs.anthropic.com/en/api/messages>
//! 关联 Phase 1 边界。
//!
//! 只实现 Phase 1 需要的字段；extra 字段通过 `#[serde(flatten)]` 保留，
//! 确保原始 body 可无损转发到上游。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// POST /v1/messages 请求 body。
///
/// Phase 1 只解析 Anthropic 格式，其他 provider 预留。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicRequest {
    /// 模型名（如 claude-sonnet-4-6）。
    pub model: String,
    /// 最大生成 token 数。
    pub max_tokens: u32,
    /// 消息列表。
    pub messages: Vec<AnthropicMessage>,
    /// 是否流式（SSE）。
    #[serde(default)]
    pub stream: bool,
    /// 系统提示（string 或 content blocks）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<serde_json::Value>,
    /// 工具定义列表。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<serde_json::Value>,
    /// 工具选择策略。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    /// 其他字段（向前兼容，不在乎也不丢弃）。
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Anthropic Messages API 单条消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    /// 角色（"user" 或 "assistant"）。
    pub role: String,
    /// 内容（string 或 content block 数组）。
    pub content: serde_json::Value,
}

impl AnthropicRequest {
    /// 提取所有 message content 中的文本（string content 或 type=text content block）。
    ///
    /// 返回 `(近似 body 字节偏移, text)` 列表。Phase 1 偏移仅供审计参考；精确 span 由
    /// vectorscan 在单条文本内 scan 时给出（start/end 是相对该 text 的偏移）。
    ///
    /// 同时追加 `system` 字段中的文本（string 或 content blocks）。
    pub fn extract_text_content(&self) -> Vec<(usize, String)> {
        let mut result = Vec::new();
        let mut cursor = 0usize;
        for msg in &self.messages {
            match &msg.content {
                serde_json::Value::String(s) => {
                    result.push((cursor, s.clone()));
                    cursor += s.len();
                }
                serde_json::Value::Array(blocks) => {
                    for block in blocks {
                        if let Some(block_obj) = block.as_object() {
                            if block_obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                                if let Some(text) = block_obj.get("text").and_then(|v| v.as_str()) {
                                    result.push((cursor, text.to_string()));
                                    cursor += text.len();
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        // 同时扫 system prompt（若有）
        if let Some(system) = &self.system {
            if let Some(s) = system.as_str() {
                result.push((cursor, s.to_string()));
            } else if let Some(blocks) = system.as_array() {
                for block in blocks {
                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                        result.push((cursor, text.to_string()));
                        cursor += text.len();
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_request() {
        let json = r#"{"model":"claude-sonnet-4-6","max_tokens":1024,"messages":[{"role":"user","content":"hi"}]}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "claude-sonnet-4-6");
        assert_eq!(req.messages.len(), 1);
        assert!(!req.stream);
        assert!(req.extra.is_empty());
    }

    #[test]
    fn parse_streaming_request_with_tools() {
        let json = r#"{
            "model": "claude-opus-4-5",
            "max_tokens": 4096,
            "stream": true,
            "messages": [{"role": "user", "content": "hello"}],
            "tools": [{"name": "bash", "description": "run shell"}],
            "unknown_future_field": 42
        }"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        assert!(req.stream);
        assert!(req.tools.is_some());
        // 未知字段被 flatten 保留，不丢弃
        assert!(req.extra.contains_key("unknown_future_field"));
    }

    #[test]
    fn roundtrip_preserves_extra_fields() {
        let json = r#"{"model":"claude-sonnet-4-6","max_tokens":1,"messages":[],"custom_key":"custom_value"}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        let re_serialized = serde_json::to_string(&req).unwrap();
        assert!(re_serialized.contains("custom_key"));
        assert!(re_serialized.contains("custom_value"));
    }
}

#[cfg(test)]
mod tests_extract {
    use super::*;

    #[test]
    fn extract_simple_string_content() {
        let json = r#"{"model":"x","max_tokens":1,"messages":[{"role":"user","content":"hello"}]}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].1, "hello");
    }

    #[test]
    fn extract_content_blocks() {
        let json = r#"{"model":"x","max_tokens":1,"messages":[{"role":"user","content":[{"type":"text","text":"hi"},{"type":"text","text":"world"}]}]}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 2);
        assert_eq!(texts[0].1, "hi");
        assert_eq!(texts[1].1, "world");
    }

    #[test]
    fn extract_with_system_prompt() {
        let json = r#"{"model":"x","max_tokens":1,"system":"You are helpful","messages":[{"role":"user","content":"q"}]}"#;
        let req: AnthropicRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 2);
        // system 在最后一项
        assert!(texts.iter().any(|(_, t)| t == "You are helpful"));
    }
}
