//! 握手协议 wire schema。
//!
//! 包含 sieve.hello 握手通知参数及 sieve.reload_user_rules 单向通知参数，
//! 对应 SPEC-005 §3（握手协议）。
//!
//! **零 IO 约束**：本文件仅 import serde / uuid / std，
//! 禁止引入 tokio / fd-lock / 任何 IO / 异步 / 运行时依赖。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// `sieve.hello` 握手通知参数（daemon → GUI，每次连接的第一条出站消息）。
///
/// 关联：SPEC-005 §3（握手协议）。
/// GUI 收到后应校验 `protocol_version == "v2"`，不兼容时关闭连接。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloParams {
    /// IPC 协议版本，当前固定为 `"v2"`。
    pub protocol_version: String,
    /// daemon 二进制版本（来自 Cargo.toml）。
    pub daemon_version: String,
    /// 当前是否处于暂停状态。
    pub paused: bool,
    /// 当前生效的 preset 名称（如 `"default"` / `"paranoid"` / `"custom"`）。
    pub preset: String,
    /// daemon 已运行秒数（启动时刻到连接时刻）。
    pub uptime_seconds: u64,
    /// audit.db 的 PRAGMA user_version（schema 版本）。
    pub audit_db_user_version: u32,
    /// daemon 本次启动时生成的唯一 UUID（整生命周期不变）。
    pub daemon_boot_id: Uuid,
}

/// 用户规则重新加载请求（单向 sieve rules edit 命令 → daemon）。
///
/// JSON-RPC 2.0 method = `"sieve.reload_user_rules"`，fire-and-forget（无 id 字段）。
///
/// 关联：PRD v2.0 §5.5.5（编辑器关闭后 lint + atomic backup + IPC reload）+ ADR-013。
///
/// daemon 收到后：
/// 1. 重新读取 `~/.sieve/rules/user.toml`
/// 2. lint + UserEngine::compile（fail-safe，PRD §9 #14：失败保留旧引擎）
/// 3. atomic swap LayeredEngine 内的 user 字段
/// 4. 成功 → 推一条 `NotifyKind::UserRulesReloaded` StatusBarNotify
/// 5. 失败 → 推一条 `NotifyKind::UserRulesLoadFailed`
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReloadUserRules {
    /// 触发 reload 的请求 ID（追踪用，可选）。
    #[serde(default)]
    pub trigger_id: Option<Uuid>,
}
