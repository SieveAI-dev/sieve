//! UnifiedMessage：Sieve 内部统一消息表示（关联 PRD §6.1）。
//!
//! Phase 1 仅 Anthropic 实现，UpstreamProvider::Relay variant 预留但不实现解析。
//! 设计依据：data-model.md §1 + ADR-004。

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// 消息角色（对应 Anthropic 的 user/assistant + Sieve 内部 system/tool）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// 系统提示角色。
    System,
    /// 用户角色。
    User,
    /// 助手角色。
    Assistant,
    /// 工具结果角色。
    Tool,
}

/// 消息方向（出站=客户端→上游，入站=上游→客户端）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// 客户端发往上游。
    Outbound,
    /// 上游返回给客户端。
    Inbound,
}

/// 上游 provider（Phase 1 仅 Anthropic；Phase 1 Week 6 新增 OpenAI；Relay 预留，见 ADR-004 + ADR-018）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum UpstreamProvider {
    /// Anthropic Messages API（Phase 1 唯一实现，ADR-004）。
    Anthropic,
    /// OpenAI Chat Completions API（Phase 1 Week 6 新增，ADR-018）。
    OpenAI,
    /// 中转站（Phase 2 预留，不实现解析）。
    Relay(String),
}

/// 内容块在原始 body 中的字节区间（half-open [start, end)）。
///
/// 用于规则匹配后精确定位告警位置（Week 2 起使用）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ContentSpan {
    /// 起始字节偏移。
    pub start: usize,
    /// 结束字节偏移（exclusive）。
    pub end: usize,
}

/// 内容块（Anthropic content block 的 Sieve 统一映射）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// 文本块。
    Text {
        /// 文本内容。
        text: String,
        /// 块在原始 body 中的字节区间（用于规则匹配定位，Week 2 起使用）。
        #[serde(default)]
        span: Option<ContentSpan>,
    },
    /// 图片块（Phase 1 透传不扫描）。
    Image {
        /// 媒体类型（如 image/png）。
        media_type: String,
    },
}

/// 工具调用块（Anthropic tool_use）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    /// 工具调用 ID（toolu_xxx）。
    pub id: String,
    /// 工具名。
    pub name: String,
    /// 参数（已解析的 JSON）。
    pub input: serde_json::Value,
    /// SSE 流式累积的原始 partial_json（Week 3 入站工具调用聚合时填充）。
    #[serde(default)]
    pub raw_partial: Option<String>,
}

/// 工具结果块（Anthropic tool_result）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlock {
    /// 关联的工具调用 ID。
    pub tool_use_id: String,
    /// 结果文本（简化版；Week 4 后扩展为 ContentBlock[]）。
    pub content: String,
}

/// 消息元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// 当前会话 ID（进程启动时随机生成，不跨进程关联，见 data-model.md §6.3）。
    pub session_id: String,
    /// 方向。
    pub direction: Direction,
    /// 上游 provider。
    pub upstream_provider: UpstreamProvider,
    /// 接收时间（UNIX epoch 秒数）。
    #[serde(with = "systemtime_as_secs")]
    pub received_at: SystemTime,
}

mod systemtime_as_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S: Serializer>(t: &SystemTime, s: S) -> Result<S::Ok, S::Error> {
        let secs = t
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        s.serialize_u64(secs)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<SystemTime, D::Error> {
        let secs = u64::deserialize(d)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

/// Sieve 内部统一消息（Phase 1 仅 Anthropic 实现解析，见 PRD §6.1）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    /// 角色。
    pub role: Role,
    /// 内容块列表。
    pub content_blocks: Vec<ContentBlock>,
    /// 工具调用列表。
    #[serde(default)]
    pub tool_uses: Vec<ToolUseBlock>,
    /// 工具结果列表。
    #[serde(default)]
    pub tool_results: Vec<ToolResultBlock>,
    /// 元数据。
    pub metadata: MessageMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_serde_lowercase() {
        let json = serde_json::to_string(&Role::User).unwrap();
        assert_eq!(json, "\"user\"");
    }

    #[test]
    fn upstream_provider_anthropic_serde() {
        let p = UpstreamProvider::Anthropic;
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("anthropic"));
    }

    #[test]
    fn content_block_text_roundtrip() {
        let block = ContentBlock::Text {
            text: "hello".to_owned(),
            span: Some(ContentSpan { start: 0, end: 5 }),
        };
        let json = serde_json::to_string(&block).unwrap();
        let decoded: ContentBlock = serde_json::from_str(&json).unwrap();
        match decoded {
            ContentBlock::Text { text, .. } => assert_eq!(text, "hello"),
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[test]
    fn direction_serde() {
        assert_eq!(
            serde_json::to_string(&Direction::Outbound).unwrap(),
            "\"outbound\""
        );
        assert_eq!(
            serde_json::to_string(&Direction::Inbound).unwrap(),
            "\"inbound\""
        );
    }
}
