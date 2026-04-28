// hook 侧轻量协议结构体，与 sieve-ipc 的 protocol.rs 保持字段对齐，
// 但独立定义避免 tokio/tracing 等依赖拖入二进制。
//
// 关联：SPEC-001 §3（文件协议 schema）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultOnTimeout {
    Redact,
    Block,
    Allow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionPayload {
    pub rule_id: String,
    pub severity: String,
    pub disposition: String,
    pub title: String,
    pub one_line_summary: String,
    pub details: serde_json::Value,
}

/// 与 sieve-ipc 的 DecisionRequest 字段完全对齐，用于反序列化 pending 文件。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub request_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub timeout_seconds: u32,
    pub default_on_timeout: DefaultOnTimeout,
    pub detections: Vec<DetectionPayload>,
}

/// 写入 decisions/<id>.json 的结构。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub request_id: Uuid,
    pub decision: String, // "allow" | "deny"
    pub decided_at: DateTime<Utc>,
    pub by_user: bool,
    pub remember: bool,
}
