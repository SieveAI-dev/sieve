//! Health（健康状态）查询 wire schema。
//!
//! 包含 sieve.health 请求/响应及其所有子结构快照类型，
//! 对应 SPEC-005 §9（health RPC）。
//!
//! **零 IO 约束**：本文件仅 import serde / chrono / std，
//! 禁止引入 tokio / fd-lock / 任何 IO / 异步 / 运行时依赖。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::rules::PresetOverride;

/// `sieve.health` 请求参数（空对象）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthRequest {}

/// preset 快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetSnapshot {
    pub mode: String,
    pub overrides: std::collections::HashMap<String, PresetOverride>,
}

/// 监听地址快照（health 子结构）。
///
/// **ADR-026 后语义变化**：daemon 可同时绑定多个端口（[`HealthResult::listeners`]
/// 数组每项一个）。本结构仍代表单个 listener，但 [`HealthResult::listen`] 字段
/// 退化为 `listeners[0]` 的别名，仅保留向后兼容（旧 GUI 客户端读取）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenSnapshot {
    pub addr: String,
    pub port: u16,
}

/// 单 listener 完整快照（ADR-026 §决策 6 + Stage F）。
///
/// 比 [`ListenSnapshot`] 多带 `provider_id` + `protocol` 元信息，新版 GUI 客户端
/// 通过 [`HealthResult::listeners`] 数组消费，列出 daemon 所有 listener 及其上游身份。
///
/// 协议版本不 bump（v2 内向后兼容扩展）：旧客户端忽略本数组，新客户端读取它。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerSnapshot {
    /// 监听地址（强制 `127.0.0.1`，PRD §9 #2）。
    pub addr: String,
    /// 监听端口（multi-listener 各自唯一）。
    pub port: u16,
    /// 上游身份标识（来自 `sieve.toml [[upstream]] provider_id`，留空时从 URL host 派生）。
    pub provider_id: String,
    /// 协议声明（`"auto"` | `"anthropic"` | `"openai"`）。
    /// `"auto"`（未显式声明，含 legacy `upstream_url`）按请求 path 自适应，不做错位拒绝；
    /// 显式 `"anthropic"` / `"openai"` 的错位请求会被 daemon fail-closed 400（ADR-026 §决策 4）。
    pub protocol: String,
}

/// audit.db 快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditDbSnapshot {
    pub path: String,
    pub size_bytes: u64,
    pub schema_version: u32,
    pub events_total: u64,
    pub events_today: u64,
}

/// 规则集快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesSnapshot {
    pub system_count: u32,
    pub user_count: u32,
    #[serde(default, serialize_with = "crate::ts_serde::serialize_opt_utc_millis")]
    pub last_reload: Option<DateTime<Utc>>,
}

/// 灰名单快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraylistSnapshot {
    pub active_count: u32,
}

/// IPC 状态快照（health 子结构）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcSnapshot {
    pub connected_clients: u32,
    pub total_decisions_inflight: u32,
}

/// `sieve.health` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    pub daemon_version: String,
    pub protocol_version: String,
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub started_at: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub preset: PresetSnapshot,
    /// 当前是否处于暂停状态（SPEC-005 §9.5）。
    pub paused: bool,
    /// 暂停截止时间（UTC，SPEC-005 §4A）；`paused=false` 时为 None。
    #[serde(default, serialize_with = "crate::ts_serde::serialize_opt_utc_millis")]
    pub paused_until: Option<DateTime<Utc>>,
    /// **已废弃，向后兼容保留**：等价于 `listeners[0]`。
    /// ADR-026 多 listener 后单一 listen 字段语义不再唯一；新客户端应读 `listeners` 数组。
    pub listen: ListenSnapshot,
    /// 多 listener 完整快照（ADR-026 §决策 6 + Stage F）。
    ///
    /// daemon 同时绑定的所有端口及其元信息（含 provider_id / protocol）。
    /// `#[serde(default)]` 保证旧 daemon 不发本字段时新客户端拿到空 vec 不崩。
    /// 旧客户端忽略本字段、读 `listen` 单值即可继续工作。
    #[serde(default)]
    pub listeners: Vec<ListenerSnapshot>,
    pub audit_db: AuditDbSnapshot,
    pub rules: RulesSnapshot,
    pub graylist: GraylistSnapshot,
    pub ipc: IpcSnapshot,
}
