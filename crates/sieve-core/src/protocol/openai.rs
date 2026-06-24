//! OpenAI Chat Completions 协议适配层。
//!
//! 实现服务端接收视角的 schema 解析和到 [`UnifiedMessage`] 的转换。
//! sieve-core 新增 OpenAI Chat Completions 协议适配层。
//!
//! # 设计原则
//!
//! - 只解析 Sieve 检测所需字段；无关字段（temperature 等）通过 `#[serde(flatten)]`
//!   保留在 `extra` 中以便无损转发。
//! - 不引入 async-openai / openai-api-rs 等大型外部 crate。
//! - 错误类型统一用 `thiserror`，禁 `anyhow`（库 crate 约束）。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::unified_message::{ContentBlock, MessageMetadata, Role, ToolUseBlock, UnifiedMessage};

// ── 请求 schema ───────────────────────────────────────────────────────────────

/// OpenAI Chat Completions 请求体（服务端接收视角）。
///
/// 关联 schema 设计。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIRequest {
    /// 模型名（如 "gpt-4o"、"gpt-4"）。
    pub model: String,
    /// 消息列表。
    #[serde(default)]
    pub messages: Vec<OpenAIMessage>,
    /// 是否流式（SSE）输出。
    #[serde(default)]
    pub stream: bool,
    /// 工具定义列表（function calling）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAITool>>,
    /// 最大生成 token 数。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// 采样温度（Sieve 不使用，但保留以无损转发）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// 兜底未知字段，确保向后兼容上游协议演进。
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// OpenAI Chat Completions 单条消息。
///
/// `content` 可以是纯字符串或 content part 数组（含 image_url 等），
/// 统一用 `serde_json::Value` 接收以兼容两种形式（content 多态）。
///
/// `extra` 通过 `#[serde(flatten)]` 兜底，保留 legacy `function_call` 字段
/// 及厂商私有扩展字段，确保 AutoRedact 重序列化时不丢失原始内容
/// （修复 Codex review R8-#4）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    /// 角色：`"system"` / `"user"` / `"assistant"` / `"tool"`。
    pub role: String,
    /// 消息内容（字符串或 content part 数组）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<serde_json::Value>,
    /// 可选名称（multi-agent 场景中标识发言者）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 工具调用列表（assistant 消息含 tool_calls 时填充）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
    /// 关联的工具调用 ID（role="tool" 的消息填充）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// 兜底其他厂商扩展字段（legacy function_call / vendor extensions），
    /// 保证 AutoRedact 重序列化不丢失原始字段。
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// OpenAI 工具调用（出现在 assistant 消息中）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCall {
    /// 工具调用 ID（由上游生成，用于 tool 消息关联）。
    pub id: String,
    /// 类型，目前固定为 `"function"`。
    #[serde(rename = "type")]
    pub call_type: String,
    /// 具体函数调用信息。
    pub function: OpenAIFunctionCall,
}

/// OpenAI 函数调用的名称和参数（完整版，非流式）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCall {
    /// 函数名。
    pub name: String,
    /// 函数参数（JSON 字符串，需要二次解析）。
    pub arguments: String,
}

/// OpenAI 工具定义（请求体中的 `tools` 字段）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITool {
    /// 工具类型，目前固定为 `"function"`。
    #[serde(rename = "type")]
    pub tool_type: String,
    /// 函数定义。
    pub function: OpenAIFunctionDef,
}

/// OpenAI 函数定义（工具注册信息）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionDef {
    /// 函数名。
    pub name: String,
    /// 函数功能描述（用于模型理解）。
    #[serde(default)]
    pub description: Option<String>,
    /// 参数 JSON Schema。
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

// ── 流式 SSE delta schema ─────────────────────────────────────────────────────

/// OpenAI SSE 流式 delta chunk（每条 `data:` 行的 JSON 结构）。
///
/// 关联流式解析。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamingChunk {
    /// chunk ID。
    pub id: String,
    /// 对象类型，固定为 `"chat.completion.chunk"`。
    pub object: String,
    /// 创建时间（UNIX 时间戳秒数）。
    pub created: u64,
    /// 模型名。
    pub model: String,
    /// 候选输出列表（通常只有 index=0 一条）。
    pub choices: Vec<OpenAIChoiceDelta>,
    /// token 用量（仅 `stream_options.include_usage=true` 时，在 `choices` 为空的
    /// 末尾 chunk 出现：`{prompt_tokens, completion_tokens, total_tokens}`）。原始 JSON
    /// 透传，供超额计费观测消费；缺省 `None`（绝大多数 chunk 无此字段）。
    #[serde(default)]
    pub usage: Option<serde_json::Value>,
}

/// 流式 chunk 中的单个候选输出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChoiceDelta {
    /// 候选下标（通常为 0）。
    pub index: u32,
    /// 增量内容。
    pub delta: OpenAIDelta,
    /// 停止原因（流式结束时填充，如 `"stop"` / `"tool_calls"`）。
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// 流式 chunk 的增量数据（content 或 tool_calls 之一）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIDelta {
    /// 角色（首个 chunk 填充，后续 chunk 省略）。
    #[serde(default)]
    pub role: Option<String>,
    /// 文本增量（普通对话时填充）。
    #[serde(default)]
    pub content: Option<String>,
    /// 工具调用增量（function calling 时填充）。
    #[serde(default)]
    pub tool_calls: Option<Vec<OpenAIToolCallDelta>>,
}

/// 流式工具调用增量。
///
/// `index` 用于跨 chunk 聚合同一工具调用；`id` 和 `name` 只在首个 chunk 出现，
/// `arguments` 在后续 chunk 中增量追加（流式聚合）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCallDelta {
    /// 工具调用下标（用于多工具并发时区分）。
    pub index: u32,
    /// 工具调用 ID（首个 chunk 填充）。
    #[serde(default)]
    pub id: Option<String>,
    /// 工具类型（首个 chunk 填充，固定 `"function"`）。
    #[serde(default)]
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    /// 函数调用增量（name + arguments 分批到达）。
    #[serde(default)]
    pub function: Option<OpenAIFunctionCallDelta>,
}

/// 流式函数调用增量（name 首个 chunk，arguments 逐 chunk 追加）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionCallDelta {
    /// 函数名（首个 chunk 填充）。
    #[serde(default)]
    pub name: Option<String>,
    /// arguments JSON 字符串片段（逐 chunk 拼接）。
    #[serde(default)]
    pub arguments: Option<String>,
}

// ── 转换到 UnifiedMessage ─────────────────────────────────────────────────────

impl OpenAIRequest {
    /// 提取所有 message content 中的文本片段，行为与 `AnthropicRequest::extract_text_content` 一致。
    ///
    /// 返回 `(segment_index, text_chunk)` 列表，供规则匹配引擎使用。
    /// 关联检测兼容性。
    pub fn extract_text_content(&self) -> Vec<(usize, String)> {
        let mut result = Vec::new();
        let mut cursor = 0usize;
        for msg in &self.messages {
            match &msg.content {
                Some(serde_json::Value::String(s)) => {
                    result.push((cursor, s.clone()));
                    cursor += s.len();
                }
                Some(serde_json::Value::Array(parts)) => {
                    for part in parts {
                        // content part 数组：{ "type": "text", "text": "..." }
                        if let Some(obj) = part.as_object() {
                            if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                                if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                                    result.push((cursor, text.to_owned()));
                                    cursor += text.len();
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        result
    }

    /// 将 OpenAI 请求转换为 Sieve 内部统一消息表示。
    ///
    /// 转换策略（UnifiedMessage 映射）：
    /// - `system` role → `ContentBlock::Text` + `Role::System`（合并为首条）
    /// - `user` / `assistant` / `tool` role → 对应 `Role` variant
    /// - `tool_calls` 中的 function 调用 → `ToolUseBlock`（arguments 字符串解析为 JSON）
    /// - 无法解析的 arguments → 保留为 `serde_json::Value::String`
    ///
    /// 注意：返回的是**最后一条非 system 消息**对应的 UnifiedMessage（代理检测场景下
    /// 规则引擎逐消息调用，此处返回 messages 末尾用户/助手消息；完整会话扫描由调用方
    /// 迭代 `self.messages` 并逐条转换，扫描粒度）。
    pub fn into_unified(self, metadata: MessageMetadata) -> UnifiedMessage {
        // 取最后一条消息作为主体；若列表为空则生成空 user 消息
        let last = self.messages.into_iter().next_back();
        let msg = match last {
            Some(m) => m,
            None => {
                return UnifiedMessage {
                    role: Role::User,
                    content_blocks: vec![],
                    tool_uses: vec![],
                    tool_results: vec![],
                    metadata,
                };
            }
        };

        let role = match msg.role.as_str() {
            "system" => Role::System,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            _ => Role::User,
        };

        let mut content_blocks = Vec::new();
        match &msg.content {
            Some(serde_json::Value::String(s)) if !s.is_empty() => {
                content_blocks.push(ContentBlock::Text {
                    text: s.clone(),
                    span: None,
                });
            }
            Some(serde_json::Value::Array(parts)) => {
                for part in parts {
                    if let Some(obj) = part.as_object() {
                        if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                            if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                                content_blocks.push(ContentBlock::Text {
                                    text: text.to_owned(),
                                    span: None,
                                });
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        // 工具调用转换：OpenAI tool_calls → ToolUseBlock
        let tool_uses: Vec<ToolUseBlock> = msg
            .tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tc| {
                // arguments 是 JSON 字符串，尝试二次解析；失败则保留为字符串值
                let input = serde_json::from_str::<serde_json::Value>(&tc.function.arguments)
                    .unwrap_or_else(|_| serde_json::Value::String(tc.function.arguments.clone()));
                ToolUseBlock {
                    id: tc.id,
                    name: tc.function.name,
                    input,
                    raw_partial: None,
                }
            })
            .collect();

        UnifiedMessage {
            role,
            content_blocks,
            tool_uses,
            tool_results: vec![],
            metadata,
        }
    }
}

/// `From<OpenAIRequest>` 无法携带 `MessageMetadata`（需要 session_id / received_at），
/// 因此提供 `Into<UnifiedMessage>` 的辅助方法而非 std trait 实现。
///
/// 调用方应使用 [`OpenAIRequest::into_unified`] 并传入 metadata。
/// 此处保留 trait stub 以满足规范要求，内部用默认 metadata（仅测试用）。
#[cfg(test)]
impl From<OpenAIRequest> for UnifiedMessage {
    fn from(req: OpenAIRequest) -> Self {
        use super::unified_message::{Direction, UpstreamProvider};
        use std::time::SystemTime;
        let metadata = MessageMetadata {
            session_id: "test-session".to_owned(),
            direction: Direction::Outbound,
            upstream_provider: UpstreamProvider::OpenAI,
            received_at: SystemTime::UNIX_EPOCH,
        };
        req.into_unified(metadata)
    }
}

// ── 单元测试 ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::super::unified_message::{Direction, UpstreamProvider};
    use super::*;
    use std::time::SystemTime;

    fn test_metadata() -> MessageMetadata {
        MessageMetadata {
            session_id: "test".to_owned(),
            direction: Direction::Outbound,
            upstream_provider: UpstreamProvider::OpenAI,
            received_at: SystemTime::UNIX_EPOCH,
        }
    }

    // ── 测试 1：解析最简请求 ──────────────────────────────────────────────────

    #[test]
    fn parse_minimal_request() {
        let json = r#"{"model":"gpt-4","messages":[{"role":"user","content":"hi"}]}"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "gpt-4");
        assert_eq!(req.messages.len(), 1);
        assert!(!req.stream);
        assert!(req.tools.is_none());
    }

    // ── 测试 2：解析含 tools 的请求 ──────────────────────────────────────────

    #[test]
    fn parse_request_with_tools() {
        let json = r#"{
            "model": "gpt-4o",
            "messages": [{"role": "user", "content": "call bash"}],
            "tools": [{
                "type": "function",
                "function": {
                    "name": "bash",
                    "description": "run shell command",
                    "parameters": {"type": "object", "properties": {"cmd": {"type": "string"}}}
                }
            }],
            "stream": true
        }"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        assert!(req.stream);
        let tools = req.tools.as_ref().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.name, "bash");
        assert_eq!(tools[0].tool_type, "function");
        assert!(tools[0].function.description.is_some());
        assert!(tools[0].function.parameters.is_some());
    }

    // ── 测试 3：解析含 tool_calls 的 assistant 消息 ───────────────────────────

    #[test]
    fn parse_message_with_tool_calls() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [{
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_abc123",
                    "type": "function",
                    "function": {
                        "name": "transfer",
                        "arguments": "{\"to\":\"0xDEAD\",\"amount\":1}"
                    }
                }]
            }]
        }"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let tc = &req.messages[0].tool_calls.as_ref().unwrap()[0];
        assert_eq!(tc.id, "call_abc123");
        assert_eq!(tc.call_type, "function");
        assert_eq!(tc.function.name, "transfer");
        assert!(tc.function.arguments.contains("0xDEAD"));
    }

    // ── 测试 4：解析流式 chunk ────────────────────────────────────────────────

    #[test]
    fn parse_streaming_chunk() {
        let json = r#"{
            "id": "chatcmpl-xyz",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {"content": "hello"},
                "finish_reason": null
            }]
        }"#;
        let chunk: OpenAIStreamingChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.id, "chatcmpl-xyz");
        assert_eq!(chunk.object, "chat.completion.chunk");
        assert_eq!(chunk.choices[0].index, 0);
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("hello"));
        assert!(chunk.choices[0].finish_reason.is_none());
    }

    // ── 测试 5：解析流式 tool_calls delta ────────────────────────────────────

    #[test]
    fn parse_tool_calls_delta() {
        let json = r#"{
            "id": "chatcmpl-tc1",
            "object": "chat.completion.chunk",
            "created": 0,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {
                    "role": "assistant",
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_001",
                        "type": "function",
                        "function": {"name": "bash", "arguments": "{\"cmd\":\"ls"}
                    }]
                },
                "finish_reason": null
            }]
        }"#;
        let chunk: OpenAIStreamingChunk = serde_json::from_str(json).unwrap();
        let tc = &chunk.choices[0].delta.tool_calls.as_ref().unwrap()[0];
        assert_eq!(tc.index, 0);
        assert_eq!(tc.id.as_deref(), Some("call_001"));
        assert_eq!(tc.call_type.as_deref(), Some("function"));
        let func = tc.function.as_ref().unwrap();
        assert_eq!(func.name.as_deref(), Some("bash"));
        assert!(func.arguments.as_ref().unwrap().contains("cmd"));
    }

    // ── 测试 6：roundtrip 保留 extra 字段 ────────────────────────────────────

    #[test]
    fn roundtrip_preserves_extra_fields() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [],
            "custom_vendor_field": "sieve_test",
            "numeric_extra": 42
        }"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        assert!(req.extra.contains_key("custom_vendor_field"));
        assert!(req.extra.contains_key("numeric_extra"));
        let re = serde_json::to_string(&req).unwrap();
        assert!(re.contains("custom_vendor_field"));
        assert!(re.contains("sieve_test"));
        assert!(re.contains("numeric_extra"));
    }

    // ── 测试 7：extract_text_content 简单字符串 ──────────────────────────────

    #[test]
    fn extract_text_content_simple_string() {
        let json = r#"{"model":"gpt-4","messages":[{"role":"user","content":"hi"}]}"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].1, "hi");
    }

    // ── 测试 8：extract_text_content 多条 messages ───────────────────────────

    #[test]
    fn extract_text_content_multiple_messages() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [
                {"role": "system", "content": "You are helpful"},
                {"role": "user", "content": "question"},
                {"role": "assistant", "content": "answer"}
            ]
        }"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 3);
        assert_eq!(texts[0].1, "You are helpful");
        assert_eq!(texts[1].1, "question");
        assert_eq!(texts[2].1, "answer");
    }

    // ── 测试 9：into_unified 字段映射正确 ────────────────────────────────────

    #[test]
    fn into_unified_field_mapping() {
        let json = r#"{
            "model": "gpt-4o",
            "messages": [
                {"role": "user", "content": "send 1 ETH to 0xDEAD"}
            ],
            "stream": false
        }"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let unified: UnifiedMessage = req.into();
        assert_eq!(unified.role, Role::User);
        assert_eq!(unified.content_blocks.len(), 1);
        match &unified.content_blocks[0] {
            ContentBlock::Text { text, .. } => {
                assert!(text.contains("0xDEAD"));
            }
            other => panic!("unexpected block: {other:?}"),
        }
        assert!(unified.tool_uses.is_empty());
        assert_eq!(unified.metadata.upstream_provider, UpstreamProvider::OpenAI);
    }

    // ── 补充：tool_calls 转换为 ToolUseBlock ─────────────────────────────────

    #[test]
    fn into_unified_tool_calls_become_tool_uses() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [{
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {"name": "sign_tx", "arguments": "{\"hash\":\"0xABC\"}"}
                }]
            }]
        }"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let unified = req.into_unified(test_metadata());
        assert_eq!(unified.role, Role::Assistant);
        assert_eq!(unified.tool_uses.len(), 1);
        assert_eq!(unified.tool_uses[0].name, "sign_tx");
        assert_eq!(unified.tool_uses[0].id, "call_1");
        // arguments 应被解析为 JSON 对象
        assert!(unified.tool_uses[0].input.is_object());
    }

    // ── 测试 R6-#5a：minimal request 序列化不含 null 字段 ────────────────────

    #[test]
    fn serialize_minimal_request_no_null_fields() {
        let json = r#"{"model":"gpt-4","messages":[{"role":"user","content":"hi"}]}"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&req).unwrap();
        // Option::None 字段不应序列化为 "null"
        assert!(
            !serialized.contains(":null"),
            "serialized minimal request contains null field: {serialized}"
        );
        // 确认必要字段存在
        assert!(serialized.contains("\"model\":\"gpt-4\""));
        assert!(serialized.contains("\"messages\""));
    }

    // ── 测试 R6-#5b：含所有 Option 字段的 roundtrip 保持一致 ────────────────

    #[test]
    fn roundtrip_full_request_option_fields_consistent() {
        let json = r#"{
            "model": "gpt-4o",
            "messages": [{
                "role": "assistant",
                "content": null,
                "name": "agent",
                "tool_calls": [{
                    "id": "call_abc",
                    "type": "function",
                    "function": {"name": "bash", "arguments": "{\"cmd\":\"ls\"}"}
                }],
                "tool_call_id": null
            }],
            "tools": [{
                "type": "function",
                "function": {"name": "bash", "description": "run bash", "parameters": null}
            }],
            "max_tokens": 1024,
            "temperature": 0.7,
            "stream": true
        }"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        // content=null 和 tool_call_id=null 应反序列化为 None
        assert!(req.messages[0].content.is_none());
        assert!(req.messages[0].tool_call_id.is_none());
        // 有值字段应正常保留
        assert_eq!(req.messages[0].name.as_deref(), Some("agent"));
        assert_eq!(req.max_tokens, Some(1024));
        assert!((req.temperature.unwrap() - 0.7_f32).abs() < 1e-5);
        // 序列化后 None 字段不含 null，有值字段保留
        let serialized = serde_json::to_string(&req).unwrap();
        // content=null → skip
        assert!(!serialized.contains("\"content\":null"));
        // tool_call_id=null → skip
        assert!(!serialized.contains("\"tool_call_id\":null"));
        // name="agent" 保留
        assert!(serialized.contains("\"name\":\"agent\""));
        // max_tokens=1024 保留
        assert!(serialized.contains("\"max_tokens\":1024"));
    }

    // ── 测试 R8-#4a：legacy function_call 字段在 message.extra 中保留 ─────────

    #[test]
    fn message_extra_preserves_legacy_function_call() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [{
                "role": "assistant",
                "content": null,
                "function_call": {"name": "transfer", "arguments": "{\"to\":\"0xDEAD\"}"}
            }]
        }"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let msg = &req.messages[0];
        // legacy function_call 应落入 extra 而不是被丢弃
        assert!(
            msg.extra.contains_key("function_call"),
            "legacy function_call 字段未出现在 message.extra"
        );
        // roundtrip 序列化后字段仍在
        let re = serde_json::to_string(&req).unwrap();
        assert!(
            re.contains("\"function_call\""),
            "roundtrip 后 function_call 丢失: {re}"
        );
        assert!(
            re.contains("0xDEAD"),
            "roundtrip 后 function_call 参数丢失: {re}"
        );
    }

    // ── 测试 R8-#4b：厂商私有扩展字段在 message.extra 中保留 ────────────────

    #[test]
    fn message_extra_preserves_vendor_extension_fields() {
        let json = r#"{
            "model": "gpt-4",
            "messages": [{
                "role": "user",
                "content": "hello",
                "custom_vendor_field": "sieve_test_value",
                "x_meta": 99
            }]
        }"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let msg = &req.messages[0];
        assert!(
            msg.extra.contains_key("custom_vendor_field"),
            "custom_vendor_field 未出现在 message.extra"
        );
        assert!(
            msg.extra.contains_key("x_meta"),
            "x_meta 未出现在 message.extra"
        );
        // roundtrip 后字段仍在
        let re = serde_json::to_string(&req).unwrap();
        assert!(re.contains("custom_vendor_field"));
        assert!(re.contains("sieve_test_value"));
        assert!(re.contains("x_meta"));
    }

    // ── 测试 R8-#4c：AutoRedact 改写 content 后扩展字段不丢失 ───────────────
    //
    // 模拟 daemon apply_redacted_texts_to_openai_request 的 roundtrip：
    // 解析含 legacy function_call 的请求 → 替换 content → 重序列化
    // 验证 function_call 和厂商扩展字段在最终 body 中仍然存在。

    #[test]
    fn autoredact_roundtrip_preserves_message_extra() {
        let original_json = r#"{
            "model": "gpt-4",
            "messages": [{
                "role": "assistant",
                "content": "secret mnemonic: abandon abandon abandon",
                "function_call": {"name": "old_fn", "arguments": "{}"},
                "x_vendor_tag": "keep_me"
            }]
        }"#;
        let req: OpenAIRequest = serde_json::from_str(original_json).unwrap();

        // 模拟 AutoRedact：克隆并改写 content，保留其他字段
        let mut new_messages = Vec::new();
        for msg in &req.messages {
            let new_content = Some(serde_json::Value::String("[REDACTED:OUT-01]".to_string()));
            new_messages.push(OpenAIMessage {
                role: msg.role.clone(),
                content: new_content,
                name: msg.name.clone(),
                tool_calls: msg.tool_calls.clone(),
                tool_call_id: msg.tool_call_id.clone(),
                extra: msg.extra.clone(),
            });
        }
        let new_req = OpenAIRequest {
            model: req.model.clone(),
            messages: new_messages,
            stream: req.stream,
            tools: req.tools.clone(),
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            extra: req.extra.clone(),
        };

        // 重序列化
        let body = serde_json::to_string(&new_req).unwrap();

        // content 已被替换
        assert!(
            body.contains("[REDACTED:OUT-01]"),
            "redacted content 未出现: {body}"
        );
        assert!(!body.contains("abandon"), "原始敏感词未被替换: {body}");
        // legacy function_call 仍在
        assert!(
            body.contains("\"function_call\""),
            "legacy function_call 在 autoredact 后丢失: {body}"
        );
        assert!(
            body.contains("old_fn"),
            "function_call.name 在 autoredact 后丢失: {body}"
        );
        // 厂商扩展字段仍在
        assert!(
            body.contains("x_vendor_tag"),
            "x_vendor_tag 在 autoredact 后丢失: {body}"
        );
        assert!(
            body.contains("keep_me"),
            "x_vendor_tag 值在 autoredact 后丢失: {body}"
        );
    }
}
