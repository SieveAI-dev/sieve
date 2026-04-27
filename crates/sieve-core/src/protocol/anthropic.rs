//! Anthropic Messages API 请求/响应 schema（子集）。
//!
//! 文档: <https://docs.anthropic.com/en/api/messages>
//! 关联 PRD §6.1 Phase 1 边界。
//!
//! 只实现 Phase 1 需要的字段；extra 字段通过 `#[serde(flatten)]` 保留，
//! 确保原始 body 可无损转发到上游。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// POST /v1/messages 请求 body。
///
/// 关联 PRD §6.1：Phase 1 只解析 Anthropic 格式，其他 provider 预留 (ADR-004)。
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
