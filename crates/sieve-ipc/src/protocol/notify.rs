//! 通知（daemon → GUI 单向推送）wire schema。
//!
//! 包含 StatusBarNotify / NotifyKind / PausedChangedNotify /
//! PresetChangedNotify 等 daemon → GUI fan-out 通知类型，
//! 对应 SPEC-005 §5.7–§5.9（通知推送）。
//!
//! **零 IO 约束**：本文件仅 import serde / chrono / uuid / std，
//! 禁止引入 tokio / fd-lock / 任何 IO / 异步 / 运行时依赖。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::rules::PresetOverride;

// ── StatusBar 通知 ────────────────────────────────────────────────────────────

/// 状态栏通知（单向 daemon → GUI），用于 IN-SEQ-* 序列检测 + 出站脱敏 + 其他不打断的提示。
///
/// JSON-RPC 2.0 method = `"sieve.notify_status_bar"`，fire-and-forget（无 id 字段）。
///
/// 关联：PRD v2.0 §5.7（行为序列 StatusBar 通知）+ §5.4.3（GUI 接口预留）+ ADR-013。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarNotify {
    /// 全局唯一通知 ID（UUIDv7，便于追踪 + 去重）。
    pub notify_id: Uuid,
    /// 创建时间（UTC，毫秒精度 + Z 后缀，SPEC-005 §4A）。
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub created_at: DateTime<Utc>,
    /// 通知类型枚举。
    pub kind: NotifyKind,
    /// 简短文案（GUI 状态栏显示，< 80 字符）。
    pub title: String,
    /// 详情（GUI 点击后展开，可选）。
    #[serde(default)]
    pub detail: Option<String>,
    /// 关联规则 ID（如 IN-SEQ-01-RECON-EXFIL / OUT-01-API-KEY）。
    #[serde(default)]
    pub rule_id: Option<String>,
    /// 自动消失秒数（0 = 不自动消失，用户手动关闭）。
    pub auto_dismiss_seconds: u32,
}

/// 状态栏通知类型。
///
/// 关联：PRD v2.0 §5.7.2（SequenceHit）+ §9 #13（OutboundRedacted）+ §9 #14（UserRules*）+ SPEC-005 §5.9（HookTerminal）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyKind {
    /// 行为序列检测命中（PRD §5.7.2）。
    SequenceHit,
    /// 出站自动脱敏（OUT-01~05/12，PRD §9 #13）。
    OutboundRedacted,
    /// 用户规则文件加载失败（PRD §9 #14 fail-safe，daemon 仍正常启动）。
    UserRulesLoadFailed,
    /// 用户规则 reload 成功（sieve rules edit 后 daemon 接收到 reload 通知并成功加载）。
    UserRulesReloaded,
    /// hook 终端处置路径被触发（SPEC-005 §5.9）。
    ///
    /// daemon 在 PreToolUse hook 终端路径决策完成后推送此通知，让 GUI 状态栏
    /// 显示"hook 已处理"事件（不打断工作流，5s 自动消失）。
    HookTerminal,
    /// 其他通用提示。
    Generic,
}

// ── 控制面变更通知 ────────────────────────────────────────────────────────────

/// `sieve.preset_changed` 通知（daemon → GUI fan-out）。
///
/// 关联：ADR-013 §S.3 / SPEC-002 §9.2。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetChangedNotify {
    pub mode: String,
    /// 仅 `mode == "custom"` 时有意义；其他模式可为空 map。
    #[serde(default)]
    pub overrides: std::collections::HashMap<String, PresetOverride>,
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub changed_at: DateTime<Utc>,
    /// `"cli"` | `"gui"` | `"config_reload"`。
    pub source: String,
    /// 触发本次变更的原始 GUI 请求 ID（SPEC-005 §10.0.2）。
    ///
    /// GUI 触发的 mutating request → 填对应 request id；CLI/daemon 自身触发 → `None`。
    /// GUI 端可据此识别"本地回声"（自己发的 set_preset 导致的广播，无需弹窗）。
    #[serde(default)]
    pub origin_request_id: Option<Uuid>,
}

/// `sieve.paused_changed` 通知（daemon → GUI fan-out）。
///
/// `applies_to` **永远不包含** Critical 锁规则的 disposition——暂停不影响内置 Critical 拦截。
/// 关联：ADR-013 §S.3 / SPEC-002 §9.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausedChangedNotify {
    pub paused: bool,
    /// 暂停截止时间（UTC，SPEC-005 §4A）；未暂停时为 None。
    #[serde(default, serialize_with = "crate::ts_serde::serialize_opt_utc_millis")]
    pub paused_until: Option<DateTime<Utc>>,
    /// `"user_request"` | `"auto_resumed"` | `"daemon_restart"`。
    pub reason: String,
    pub applies_to: Vec<String>,
    /// 触发来源（SPEC-005 §10.2 required）：`"cli"` | `"gui"` | `"config_reload"` | `"daemon"`。
    ///
    /// 此前漏发此字段导致 GUI `PausedChangedParams` 解码失败（required）→ disconnected
    /// （跨仓 fixture 一致性测试抓出，2026-06-18）。
    pub source: String,
    /// 触发本次变更的原始 GUI 请求 ID（SPEC-005 §10.0.2）。
    ///
    /// GUI 触发的 mutating request → 填对应 request id；CLI/daemon 自身触发 → `None`。
    /// GUI 端可据此识别"本地回声"，避免重复展示操作确认。
    #[serde(default)]
    pub origin_request_id: Option<Uuid>,
}
