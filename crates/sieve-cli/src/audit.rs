//! 审计日志（关联审计数据模型 + 双层防御日志设计）。
//!
//! Week 5 起接入 SQLite append-only 存储。
//! Week 6（v2.0）：schema v2 加 caller_pid / caller_exe 两列。
//!
//! 设计约束：
//! - SQLite append-only：BEFORE UPDATE / DELETE 触发器拒绝修改。
//! - 异步写入接口：`tokio::task::spawn_blocking` + internal `Mutex` 串行化。
//! - 不暴露 `rusqlite` 类型到 crate 外部。
//! - Schema 版本通过 `PRAGMA user_version` 跟踪；打开数据库时自动迁移（v1→v2）。

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

// ─────────────────────────── CallerContext ─────────────────────────────────

/// 调用方上下文（v2.0）。
///
/// 记录触发审计事件的进程 PID 和可执行文件路径，用于追溯 Claude Code
/// 或其他接入方的身份。两个字段均为 NULL 兼容（接入方未提供时为 None）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CallerContext {
    /// 调用方进程 PID（NULL 表示未知）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<i32>,
    /// 调用方可执行文件路径（NULL 表示未知）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exe: Option<String>,
}

// ─────────────────────────── AuditEvent ────────────────────────────────────

/// 审计事件枚举（关联处置矩阵 + 双层防御日志需求）。
///
/// v2.0：每个 variant 含 `caller: CallerContext`，
/// 记录 caller_pid / caller_exe；`#[serde(default)]` 保证旧 raw_json 反序列化兼容。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuditEvent {
    /// 出站请求中检测到敏感内容并脱敏。
    OutboundRedacted {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
        /// 调用方上下文；旧 JSON 缺失时 Default 为空。
        #[serde(default)]
        caller: CallerContext,
    },
    /// 入站响应 hook 标记了疑似高危工具调用。
    InboundHookMarked {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
        /// 调用方上下文；旧 JSON 缺失时 Default 为空。
        #[serde(default)]
        caller: CallerContext,
    },
    /// 入站高危工具调用等待用户决策。
    InboundDecisionRequested {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
        /// 调用方上下文；旧 JSON 缺失时 Default 为空。
        #[serde(default)]
        caller: CallerContext,
    },
    /// 用户对高危工具调用给出决策（Allow / Block）。
    InboundDecisionResolved {
        rule_id: String,
        severity: String,
        decision: String,
        request_id: String,
        raw_json: Option<String>,
        /// 调用方上下文；旧 JSON 缺失时 Default 为空。
        #[serde(default)]
        caller: CallerContext,
    },
    /// 状态栏通知已发送。
    StatusBarNotified {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
        /// 调用方上下文；旧 JSON 缺失时 Default 为空。
        #[serde(default)]
        caller: CallerContext,
    },
    // ── v2.0 新增事件变体 ──────────────────────
    /// 入站工具调用被用户决策（Allow/Deny）处置完成。
    DecisionMade {
        rule_id: String,
        /// "allow" | "deny" | "redact_and_allow"
        decision: String,
        severity: String,
        /// true = 用户点击 Allow；false = 超时 / 系统回退
        by_user: bool,
        request_id: String,
        #[serde(default)]
        caller: CallerContext,
    },
    /// 灰名单条目已写入。
    GraylistAdded {
        rule_id: String,
        fingerprint: String,
        request_id: String,
        #[serde(default)]
        caller: CallerContext,
    },
    /// 灰名单写入被 Critical 锁拒绝（fail-closed 二次校验）。
    GraylistCriticalRejected {
        rule_id: String,
        request_id: String,
        #[serde(default)]
        caller: CallerContext,
    },
    /// 灰名单命中，跳过 IPC 弹窗直接 Allow。
    GraylistHit {
        rule_id: String,
        fingerprint: String,
        request_id: String,
        #[serde(default)]
        caller: CallerContext,
    },
    /// 灰名单写入失败（磁盘满 / 权限错 / 序列化错）。
    ///
    /// 写入失败不影响本次 Allow 决策（fail-soft），但必须记录到 audit 供事后排查。
    GraylistAddFailed {
        rule_id: String,
        /// 错误描述（`e.to_string()`）
        error: String,
        request_id: String,
        #[serde(default)]
        caller: CallerContext,
    },
    /// 行为序列检测命中（IN-SEQ-*）。
    SequenceHit {
        rule_id: String,
        description: String,
        path_label: String,
        #[serde(default)]
        caller: CallerContext,
    },
    /// 用户规则 reload 结果。
    UserRulesReloaded {
        /// reload 是否成功
        success: bool,
        /// 成功时的规则数量
        #[serde(default)]
        rule_count: Option<usize>,
        /// 失败时的错误信息
        #[serde(default)]
        error: Option<String>,
        /// 触发 reload 的 trigger_id（来自 sieve rules edit）
        #[serde(default)]
        trigger_id: Option<String>,
    },
    // ── v2.1 GUI 控制面 ─────────────────────
    /// 操作被 critical_lock 拒绝（防线二的非 graylist 出口，如 set_preset_overrides）。
    CriticalLockBlocked {
        rule_id: String,
        /// `"ipc_set_overrides"` | `"ipc_response"` | `"cli"` | …
        source: String,
    },
    /// preset 模式变化（CLI / GUI / config_reload 触发）。
    PresetChanged {
        from_mode: String,
        to_mode: String,
        /// `"cli"` | `"gui"` | `"config_reload"`。
        source: String,
    },
    /// 单条 preset override 应用成功。
    PresetOverrideApplied {
        rule_id: String,
        timeout_seconds: u32,
        /// `"block"` | `"allow"` | `"redact"`。
        default_on_timeout: String,
        source: String,
    },
    /// 单条 preset override 被拒绝。
    PresetOverrideRejected {
        rule_id: String,
        /// `"critical_lock"` | `"unknown_rule"` | `"invalid_value"`。
        reason: String,
        source: String,
    },
    /// 暂停状态被设置或清除。
    PausedSet {
        /// 暂停截止时间 RFC3339；None = 立刻恢复。
        #[serde(default)]
        until: Option<String>,
        source: String,
    },
    /// `sieve.toml` + 用户规则 reload 完成。
    ConfigReloaded {
        user_rules_errors_count: usize,
        source: String,
    },
    /// 灰名单条目被删除（用户主动 / 过期）。
    GraylistRemoved {
        fingerprint: String,
        rule_id: String,
        /// `"gui_user_action"` | `"cli"` | `"expired"`。
        removed_by: String,
    },
    /// 暂停期间命中非 Critical 规则触发自动处置，跳过弹窗（SPEC-002 §9.1）。
    ///
    /// 与 paused 状态绑定的特殊审计事件——记录"用户暂停时被 Sieve 替我拒了什么"，
    /// 让用户在恢复后能查询暂停期间发生的事件。
    AutoDecidedPaused {
        /// 触发的所有 rule_id（多条命中时逗号分隔）。
        rule_ids: String,
        /// 应用的决策：`"allow"` | `"deny"` | `"redact_and_allow"`。
        decision: String,
        request_id: String,
        #[serde(default)]
        caller: CallerContext,
    },
    /// 入站 Critical 工具调用被 fail-closed 自动拦截（无用户决策）。
    ///
    /// 接线背景：入站 block 路径（SSE + JSON、Anthropic + OpenAI）此前一律不落 audit
    /// （真机 dogfood 抓出，2026-06-18），`sieve audit query` 查不到任何拦截记录。
    /// 仅记录元数据（无 payload），整体序列化天然零 secret。
    InboundBlocked {
        rule_id: String,
        severity: String,
        request_id: String,
        /// "anthropic_sse" | "openai_sse" | "anthropic_json" | "openai_json"
        path_label: String,
        #[serde(default)]
        caller: CallerContext,
    },
    /// IPC socket 收到超大帧，关闭连接（SPEC-005 §1.1 / §1.3.1）。
    ///
    /// **禁止**记录任何 raw payload；只记录 `peer / size_bytes / closed_at_ms`。
    IpcOversizeFrame {
        /// 对端标识（如 socket 路径或 peer addr；无法获取时为 `"unknown"`）。
        peer: String,
        /// 超限帧的字节数（完整帧含 `\n`，或 partial remainder 的字节数）。
        size_bytes: u64,
        /// 关闭连接的时间（unix 毫秒）。
        closed_at_ms: i64,
    },
    /// GUI 用户触发了清空历史（Touch ID 确认后，开始执行前）。SPEC-005 §11B。
    PurgeHistoryStarted {
        /// GUI 端 Touch ID 通过的时刻（unix ms）。
        confirmed_at_ms: i64,
    },
    /// 清空历史执行完成。SPEC-005 §11B。
    PurgeHistoryCompleted {
        /// 本次删除的行数。
        rows_deleted: u64,
        /// 执行完成的时刻（unix ms）。
        purged_at_ms: i64,
    },
}

impl AuditEvent {
    fn direction(&self) -> &'static str {
        match self {
            Self::OutboundRedacted { .. } => "outbound",
            Self::InboundHookMarked { .. }
            | Self::InboundDecisionRequested { .. }
            | Self::InboundDecisionResolved { .. }
            | Self::StatusBarNotified { .. }
            | Self::DecisionMade { .. }
            | Self::GraylistAdded { .. }
            | Self::GraylistCriticalRejected { .. }
            | Self::GraylistHit { .. }
            | Self::GraylistAddFailed { .. }
            | Self::SequenceHit { .. }
            | Self::InboundBlocked { .. }
            | Self::AutoDecidedPaused { .. } => "inbound",
            Self::UserRulesReloaded { .. }
            | Self::CriticalLockBlocked { .. }
            | Self::PresetChanged { .. }
            | Self::PresetOverrideApplied { .. }
            | Self::PresetOverrideRejected { .. }
            | Self::PausedSet { .. }
            | Self::ConfigReloaded { .. }
            | Self::GraylistRemoved { .. }
            | Self::IpcOversizeFrame { .. }
            | Self::PurgeHistoryStarted { .. }
            | Self::PurgeHistoryCompleted { .. } => "system",
        }
    }

    fn rule_id(&self) -> &str {
        match self {
            Self::OutboundRedacted { rule_id, .. }
            | Self::InboundHookMarked { rule_id, .. }
            | Self::InboundDecisionRequested { rule_id, .. }
            | Self::InboundDecisionResolved { rule_id, .. }
            | Self::StatusBarNotified { rule_id, .. }
            | Self::DecisionMade { rule_id, .. }
            | Self::GraylistAdded { rule_id, .. }
            | Self::GraylistCriticalRejected { rule_id, .. }
            | Self::GraylistHit { rule_id, .. }
            | Self::GraylistAddFailed { rule_id, .. }
            | Self::SequenceHit { rule_id, .. }
            | Self::InboundBlocked { rule_id, .. }
            | Self::CriticalLockBlocked { rule_id, .. }
            | Self::PresetOverrideApplied { rule_id, .. }
            | Self::PresetOverrideRejected { rule_id, .. }
            | Self::GraylistRemoved { rule_id, .. } => rule_id,
            Self::AutoDecidedPaused { rule_ids, .. } => rule_ids,
            Self::UserRulesReloaded { .. } => "system.user_rules_reload",
            Self::PresetChanged { .. } => "system.preset_changed",
            Self::PausedSet { .. } => "system.paused_set",
            Self::ConfigReloaded { .. } => "system.config_reloaded",
            Self::IpcOversizeFrame { .. } => "system.ipc_oversize_frame",
            Self::PurgeHistoryStarted { .. } => "system.purge_history_started",
            Self::PurgeHistoryCompleted { .. } => "system.purge_history_completed",
        }
    }

    fn severity(&self) -> &str {
        match self {
            Self::OutboundRedacted { severity, .. }
            | Self::InboundHookMarked { severity, .. }
            | Self::InboundDecisionRequested { severity, .. }
            | Self::InboundDecisionResolved { severity, .. }
            | Self::StatusBarNotified { severity, .. }
            | Self::DecisionMade { severity, .. }
            | Self::InboundBlocked { severity, .. } => severity,
            Self::GraylistAdded { .. }
            | Self::GraylistCriticalRejected { .. }
            | Self::GraylistHit { .. }
            | Self::GraylistAddFailed { .. }
            | Self::SequenceHit { .. }
            | Self::UserRulesReloaded { .. }
            | Self::PresetChanged { .. }
            | Self::PresetOverrideApplied { .. }
            | Self::PresetOverrideRejected { .. }
            | Self::PausedSet { .. }
            | Self::ConfigReloaded { .. }
            | Self::GraylistRemoved { .. }
            | Self::AutoDecidedPaused { .. }
            | Self::IpcOversizeFrame { .. }
            | Self::PurgeHistoryStarted { .. }
            | Self::PurgeHistoryCompleted { .. } => "info",
            Self::CriticalLockBlocked { .. } => "critical",
        }
    }

    fn disposition(&self) -> &'static str {
        match self {
            Self::OutboundRedacted { .. } => "redact",
            Self::InboundHookMarked { .. } => "mark",
            Self::InboundDecisionRequested { .. } => "pending",
            Self::InboundDecisionResolved { .. } => "resolved",
            Self::StatusBarNotified { .. } => "notify",
            Self::DecisionMade { .. } => "decision_made",
            Self::GraylistAdded { .. } => "graylist_added",
            Self::GraylistCriticalRejected { .. } => "graylist_critical_rejected",
            Self::GraylistHit { .. } => "graylist_hit",
            Self::GraylistAddFailed { .. } => "graylist_add_failed",
            Self::SequenceHit { .. } => "sequence_hit",
            Self::InboundBlocked { .. } => "blocked",
            Self::UserRulesReloaded { .. } => "user_rules_reloaded",
            Self::CriticalLockBlocked { .. } => "critical_lock_blocked",
            Self::PresetChanged { .. } => "preset_changed",
            Self::PresetOverrideApplied { .. } => "preset_override_applied",
            Self::PresetOverrideRejected { .. } => "preset_override_rejected",
            Self::PausedSet { .. } => "paused_set",
            Self::ConfigReloaded { .. } => "config_reloaded",
            Self::GraylistRemoved { .. } => "graylist_removed",
            Self::AutoDecidedPaused { .. } => "auto_decided_paused",
            Self::IpcOversizeFrame { .. } => "ipc_oversize_frame",
            Self::PurgeHistoryStarted { .. } => "purge_history_started",
            Self::PurgeHistoryCompleted { .. } => "purge_history_completed",
        }
    }

    fn decision(&self) -> Option<&str> {
        match self {
            Self::InboundDecisionResolved { decision, .. }
            | Self::DecisionMade { decision, .. }
            | Self::AutoDecidedPaused { decision, .. } => Some(decision),
            _ => None,
        }
    }

    fn request_id(&self) -> &str {
        match self {
            Self::OutboundRedacted { request_id, .. }
            | Self::InboundHookMarked { request_id, .. }
            | Self::InboundDecisionRequested { request_id, .. }
            | Self::InboundDecisionResolved { request_id, .. }
            | Self::StatusBarNotified { request_id, .. }
            | Self::DecisionMade { request_id, .. }
            | Self::GraylistAdded { request_id, .. }
            | Self::GraylistCriticalRejected { request_id, .. }
            | Self::GraylistHit { request_id, .. }
            | Self::GraylistAddFailed { request_id, .. }
            | Self::InboundBlocked { request_id, .. }
            | Self::AutoDecidedPaused { request_id, .. } => request_id,
            Self::SequenceHit { .. }
            | Self::UserRulesReloaded { .. }
            | Self::CriticalLockBlocked { .. }
            | Self::PresetChanged { .. }
            | Self::PresetOverrideApplied { .. }
            | Self::PresetOverrideRejected { .. }
            | Self::PausedSet { .. }
            | Self::ConfigReloaded { .. }
            | Self::GraylistRemoved { .. }
            | Self::IpcOversizeFrame { .. }
            | Self::PurgeHistoryStarted { .. }
            | Self::PurgeHistoryCompleted { .. } => "",
        }
    }

    fn raw_json(&self) -> Option<&str> {
        match self {
            Self::OutboundRedacted { raw_json, .. }
            | Self::InboundHookMarked { raw_json, .. }
            | Self::InboundDecisionRequested { raw_json, .. }
            | Self::InboundDecisionResolved { raw_json, .. }
            | Self::StatusBarNotified { raw_json, .. } => raw_json.as_deref(),
            _ => None,
        }
    }

    /// 提取调用方 PID。
    fn caller_pid(&self) -> Option<i32> {
        match self {
            Self::OutboundRedacted { caller, .. }
            | Self::InboundHookMarked { caller, .. }
            | Self::InboundDecisionRequested { caller, .. }
            | Self::InboundDecisionResolved { caller, .. }
            | Self::StatusBarNotified { caller, .. }
            | Self::DecisionMade { caller, .. }
            | Self::GraylistAdded { caller, .. }
            | Self::GraylistCriticalRejected { caller, .. }
            | Self::GraylistHit { caller, .. }
            | Self::GraylistAddFailed { caller, .. }
            | Self::SequenceHit { caller, .. }
            | Self::InboundBlocked { caller, .. }
            | Self::AutoDecidedPaused { caller, .. } => caller.pid,
            Self::UserRulesReloaded { .. }
            | Self::CriticalLockBlocked { .. }
            | Self::PresetChanged { .. }
            | Self::PresetOverrideApplied { .. }
            | Self::PresetOverrideRejected { .. }
            | Self::PausedSet { .. }
            | Self::ConfigReloaded { .. }
            | Self::GraylistRemoved { .. }
            | Self::IpcOversizeFrame { .. }
            | Self::PurgeHistoryStarted { .. }
            | Self::PurgeHistoryCompleted { .. } => None,
        }
    }

    /// 提取调用方可执行路径。
    fn caller_exe(&self) -> Option<&str> {
        match self {
            Self::OutboundRedacted { caller, .. }
            | Self::InboundHookMarked { caller, .. }
            | Self::InboundDecisionRequested { caller, .. }
            | Self::InboundDecisionResolved { caller, .. }
            | Self::StatusBarNotified { caller, .. }
            | Self::DecisionMade { caller, .. }
            | Self::GraylistAdded { caller, .. }
            | Self::GraylistCriticalRejected { caller, .. }
            | Self::GraylistHit { caller, .. }
            | Self::GraylistAddFailed { caller, .. }
            | Self::SequenceHit { caller, .. }
            | Self::InboundBlocked { caller, .. }
            | Self::AutoDecidedPaused { caller, .. } => caller.exe.as_deref(),
            Self::UserRulesReloaded { .. }
            | Self::CriticalLockBlocked { .. }
            | Self::PresetChanged { .. }
            | Self::PresetOverrideApplied { .. }
            | Self::PresetOverrideRejected { .. }
            | Self::PausedSet { .. }
            | Self::ConfigReloaded { .. }
            | Self::GraylistRemoved { .. }
            | Self::IpcOversizeFrame { .. }
            | Self::PurgeHistoryStarted { .. }
            | Self::PurgeHistoryCompleted { .. } => None,
        }
    }
}

// ─────────────────────────── Schema migration ──────────────────────────────

/// 当前 schema 版本（v3：加 provider_id 列）。
const CURRENT_SCHEMA_VERSION: u32 = 3;

/// 幂等补列：仅当 `audit_events` 实表缺 `col` 时才 `ALTER TABLE ADD COLUMN`。
///
/// SQLite 无 `ADD COLUMN IF NOT EXISTS`，对已存在列再 ALTER 会 "duplicate column
/// name" 报错并毒化整个迁移事务，故必须先查 `PRAGMA table_info` 实列。这是迁移
/// 以「实表列」为真相源（而非 user_version 整数）的基石（见 [`migrate`]）。
///
/// # Errors
/// `PRAGMA table_info` 或 `ALTER TABLE` 执行失败时返回错误。
fn ensure_column(conn: &Connection, col: &str, col_ddl: &str) -> Result<()> {
    let exists = conn
        .prepare("PRAGMA table_info(audit_events)")?
        .query_map([], |r| r.get::<_, String>(1))?
        .filter_map(std::result::Result::ok)
        .any(|name| name == col);
    if !exists {
        conn.execute_batch(&format!(
            "ALTER TABLE audit_events ADD COLUMN {col} {col_ddl};"
        ))
        .with_context(|| format!("补列 {col} 失败"))?;
    }
    Ok(())
}

/// 打开数据库后执行一次 schema 迁移，把 `audit_events` 升到 [`CURRENT_SCHEMA_VERSION`]。
///
/// 以「实表列」为真相源逐列补齐，而非用 `PRAGMA user_version` 整数推断结构。
///
/// 历史教训：`user_version = 0` 有两种语义截然不同的来源——
/// (A) 全新库，`CREATE TABLE` 刚建出完整 v3 表；(B) pre-f7f794d 早期遗留库，表已
/// 存在但只有 9 个 v1 列，且当时代码从不写 user_version 故停在 SQLite 默认 0。旧实现
/// 的 `current == 0` 快速路径把 (B) 误判为 (A)，盲目 stamp version=3 却跳过 ADD COLUMN
/// → 表缺列 + 版本号永久说谎 + 之后每次 append（固定 11 列 INSERT）全失败。改为实列
/// 驱动后：全新库三个 `ensure_column` 都是 no-op（CREATE 已建全列），遗留库（v0/v1/v2）
/// 按需补缺列，两者都收敛到 v3。未来加列照抄一行 `ensure_column` 即可，迁移不会再因
/// 忘记递增/盖错版本而漂移。
///
/// 迁移在单一事务内执行，任一步失败整体回滚（`ALTER TABLE` 与 `PRAGMA user_version`
/// 均受 SQLite 事务保护）。
///
/// # Errors
/// SQLite 执行失败时返回错误。
fn migrate(conn: &Connection) -> Result<()> {
    let current: u32 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .context("读取 PRAGMA user_version 失败")?;

    if current >= CURRENT_SCHEMA_VERSION {
        // 已是最新版本，无需迁移。
        return Ok(());
    }

    // BEGIN IMMEDIATE 而非延迟 BEGIN：迁移含「先读 table_info（SHARED）再写 ALTER/PRAGMA
    // （需把锁升级到 RESERVED）」，延迟 BEGIN 的读后升级是 SQLite 最易死锁/SQLITE_BUSY 的路径。
    // 前置 IMMEDIATE 直接拿 RESERVED 写锁，规避升级争用（与兄弟方法 delete_all_events 一致）。
    conn.execute_batch("BEGIN IMMEDIATE;")
        .context("开启 schema 迁移事务失败")?;

    // ALTER TABLE ADD COLUMN 在 SQLite 中是 O(1)（不重写表）；新列对现有行为 NULL 或
    // 取 DEFAULT，不触发 NOT NULL 失败；BEFORE UPDATE/DELETE 触发器基于行操作，ADD
    // COLUMN 不使其失效（append-only 不变量安全）。列顺序对应 v1→v2→v3 历史增量，但
    // 不再依赖 version 分支精确性——存在即跳过。
    let migrated = (|| -> Result<()> {
        ensure_column(conn, "caller_pid", "INTEGER")?;
        ensure_column(conn, "caller_exe", "TEXT")?;
        ensure_column(conn, "provider_id", "TEXT NOT NULL DEFAULT 'unknown'")?;
        conn.execute_batch(&format!("PRAGMA user_version = {CURRENT_SCHEMA_VERSION};"))
            .context("设置 user_version 失败")?;
        Ok(())
    })();

    match migrated {
        Ok(()) => {
            conn.execute_batch("COMMIT;")
                .context("提交 schema 迁移事务失败")?;
            Ok(())
        }
        Err(e) => {
            // 回滚尽力而为；即便回滚失败也返回原始迁移错误（诊断价值更高）。
            let _ = conn.execute_batch("ROLLBACK;");
            Err(e)
        }
    }
}

// ─────────────────────────── AuditStore ────────────────────────────────────

/// 审计存储句柄（SQLite append-only）。
///
/// Week 5 起持有真实 SQLite 连接；线程安全通过 `Arc<Mutex<Connection>>` 实现。
/// 关联双层防御日志需求。
/// 审计存储句柄（SQLite append-only）。
pub struct AuditStore {
    conn: Arc<Mutex<Connection>>,
}

impl AuditStore {
    /// 初始化审计存储：打开 SQLite，执行 schema 迁移，安装 append-only 触发器。
    ///
    /// 幂等——文件已存在时执行 schema 迁移（v1→v2），不重建表。
    ///
    /// # Errors
    /// SQLite 打开、迁移或 DDL 执行失败时返回错误。
    pub fn init(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建审计目录 {} 失败", parent.display()))?;
        }

        let conn = Connection::open(path)
            .with_context(|| format!("打开审计数据库 {} 失败", path.display()))?;

        // busy_timeout：默认 0 → 锁争用立即 SQLITE_BUSY。daemon 启动 migrate 时若恰有
        // `sieve audit` 读连接持 SHARED 锁，migrate 的写锁升级会瞬时失败 → init 失败 →
        // daemon fail-closed 起不来。给 5s 重试窗口让启动期争用收敛。
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .context("设置 busy_timeout 失败")?;

        // 建表（全新 DB）或幂等（已存在时跳过）
        conn.execute_batch(CREATE_TABLE_DDL)
            .context("创建 audit_events 表失败")?;

        // schema 迁移（v1→v2）
        migrate(&conn).context("schema 迁移失败")?;

        // 安装 append-only 触发器（幂等：IF NOT EXISTS 不会重建）
        conn.execute_batch(APPEND_ONLY_TRIGGERS_DDL)
            .context("安装 append-only 触发器失败")?;

        tracing::debug!(path = %path.display(), "audit store initialized (SQLite, schema v{CURRENT_SCHEMA_VERSION})");
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 返回实库 `PRAGMA user_version`（而非编译期常量），供 sieve.hello 握手通知填充
    /// `audit_db_user_version` 字段（SPEC-005 §3）。
    ///
    /// 读实库而非返回 [`CURRENT_SCHEMA_VERSION`]：正常流程下 init 迁移成功后实库必达常量值，
    /// 但旧 daemon 打开未来版本库（user_version > CURRENT，migrate 早退不动库）时，握手必须
    /// 上报真实版本而非常量，否则 GUI/审计据此判断 schema 会被误导。
    /// 读取失败（极罕见）回退到 [`CURRENT_SCHEMA_VERSION`]，不让握手因此失败。
    pub fn schema_version(&self) -> u32 {
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap_or(CURRENT_SCHEMA_VERSION)
    }

    /// 删除所有审计事件行，保留表结构（`sieve.purge_history` SPEC-005 §11B 专用）。
    ///
    /// 因 append-only 触发器阻止 DELETE，此方法先 DROP 触发器，执行删除，再重建触发器。
    /// 整个操作包裹在 SQLite 事务中，保证原子性。
    ///
    /// # 审计记录
    ///
    /// 调用方需在调用前后各写一条 `purge_started` / `purge_completed` 事件
    /// （由 `handle_purge_history` 负责，本方法不写审计）。
    ///
    /// # Returns
    ///
    /// 返回实际删除的行数。
    pub async fn delete_all_events(&self) -> Result<u64> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let guard = conn
                .lock()
                .map_err(|e| anyhow::anyhow!("audit mutex poisoned: {e}"))?;

            // 事务包裹：DROP 触发器 + DELETE + 重建触发器，原子执行。
            guard.execute_batch("BEGIN IMMEDIATE;")?;

            // 暂时移除 append-only 触发器（purge_history 专用路径）
            guard.execute_batch(
                "DROP TRIGGER IF EXISTS no_delete; DROP TRIGGER IF EXISTS no_update;",
            )?;

            // 执行全量删除（不 DROP TABLE，保留 schema）
            let rows_deleted = guard.execute("DELETE FROM audit_events", [])?;

            // 重建 append-only 触发器（DDL 幂等，IF NOT EXISTS）
            guard.execute_batch(APPEND_ONLY_TRIGGERS_DDL)?;

            guard.execute_batch("COMMIT;")?;

            Ok::<u64, anyhow::Error>(rows_deleted as u64)
        })
        .await
        .context("spawn_blocking failed")?
    }

    /// 异步写入一条审计事件（spawn_blocking + Mutex 串行化）。
    ///
    /// # Errors
    /// SQLite 写入失败时返回错误。
    pub async fn append(&self, event: AuditEvent, provider_id: &str) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let provider_id = provider_id.to_owned();
        tokio::task::spawn_blocking(move || {
            let guard = conn
                .lock()
                .map_err(|e| anyhow::anyhow!("audit mutex poisoned: {e}"))?;
            let timestamp = Utc::now().to_rfc3339();
            let raw_json = serde_json::to_string(&event).ok();
            guard.execute(
                INSERT_SQL,
                params![
                    timestamp,
                    event.direction(),
                    event.rule_id(),
                    event.severity(),
                    event.disposition(),
                    event.decision(),
                    event.request_id(),
                    // 优先使用事件自带的 raw_json，否则用序列化整个事件
                    event.raw_json().or(raw_json.as_deref()),
                    event.caller_pid(),
                    event.caller_exe(),
                    provider_id,
                ],
            )?;
            Ok::<(), anyhow::Error>(())
        })
        .await
        .context("spawn_blocking failed")??;
        Ok(())
    }
}

/// daemon 系统级审计事件的默认 provider_id（无 listener 上下文）。
///
/// 用于 control plane 调用、规则 reload、preset 变更等不属于任何具体上游的事件。
/// `audit_events.provider_id` 列 NOT NULL，所以系统事件需用此常量而非空字符串。
///
pub const SYSTEM_PROVIDER_ID: &str = "_system";

/// 测试 / 兜底场景的 provider_id 常量（同样 NOT NULL 占位）。
#[cfg_attr(not(test), allow(dead_code))]
pub const UNKNOWN_PROVIDER_ID: &str = "unknown";

// ─────────────────────────── SQL 常量 ──────────────────────────────────────

/// 建表 DDL（含 v3 全部新列：caller_pid / caller_exe / provider_id）。
///
/// - v2 列：caller_pid / caller_exe
/// - v3 列：provider_id —— 标注本次审计事件命中哪个 listener 上游
const CREATE_TABLE_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS audit_events (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp_rfc3339   TEXT    NOT NULL,
    direction           TEXT    NOT NULL,   -- 'outbound' | 'inbound'
    rule_id             TEXT    NOT NULL,
    severity            TEXT    NOT NULL,   -- 'Critical' | 'High' | 'Medium' | 'Low'
    disposition         TEXT    NOT NULL,   -- 'redact' | 'mark' | 'pending' | 'resolved' | 'notify'
    decision            TEXT,               -- 'Allow' | 'Block' | NULL
    request_id          TEXT    NOT NULL,
    raw_json            TEXT,
    caller_pid          INTEGER,                                        -- 调用方 PID（NULL 表示未知）
    caller_exe          TEXT,                                           -- 调用方可执行路径（NULL 表示未知）
    provider_id         TEXT    NOT NULL DEFAULT 'unknown'              -- 上游身份标识（'unknown' 表示无 listener 上下文）
);
"#;

/// append-only 触发器：拒绝 UPDATE / DELETE。
const APPEND_ONLY_TRIGGERS_DDL: &str = r#"
CREATE TRIGGER IF NOT EXISTS no_update
BEFORE UPDATE ON audit_events
BEGIN
    SELECT RAISE(FAIL, 'audit_events is append-only: UPDATE is forbidden');
END;

CREATE TRIGGER IF NOT EXISTS no_delete
BEFORE DELETE ON audit_events
BEGIN
    SELECT RAISE(FAIL, 'audit_events is append-only: DELETE is forbidden');
END;
"#;

// Week 6 接入后移除此 allow。
#[allow(dead_code)]
const INSERT_SQL: &str = r#"
INSERT INTO audit_events
    (timestamp_rfc3339, direction, rule_id, severity, disposition, decision,
     request_id, raw_json, caller_pid, caller_exe, provider_id)
VALUES
    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
"#;

// ─────────────────────────── 单元测试 ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_event(n: u32) -> AuditEvent {
        AuditEvent::OutboundRedacted {
            rule_id: format!("OUT-0{n}"),
            severity: "Critical".to_string(),
            request_id: format!("req-{n}"),
            raw_json: Some(format!("{{\"test\":{n}}}")),
            caller: CallerContext::default(),
        }
    }

    fn make_decision_event() -> AuditEvent {
        AuditEvent::InboundDecisionResolved {
            rule_id: "IN-CR-01".to_string(),
            severity: "Critical".to_string(),
            decision: "Block".to_string(),
            request_id: "req-decision".to_string(),
            raw_json: None,
            caller: CallerContext::default(),
        }
    }

    #[tokio::test]
    async fn write_and_read_events() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("audit.db");
        let store = AuditStore::init(&db_path).expect("init failed");

        for i in 1..=5 {
            store
                .append(make_event(i), UNKNOWN_PROVIDER_ID)
                .await
                .expect("append failed");
        }

        // 直接用 rusqlite 验证
        let conn = Connection::open(&db_path).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 5, "应有 5 条记录");

        let rule_id: String = conn
            .query_row("SELECT rule_id FROM audit_events WHERE id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(rule_id, "OUT-01");
    }

    #[tokio::test]
    async fn decision_event_stores_decision_field() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("audit_decision.db");
        let store = AuditStore::init(&db_path).expect("init failed");

        store
            .append(make_decision_event(), UNKNOWN_PROVIDER_ID)
            .await
            .unwrap();

        let conn = Connection::open(&db_path).unwrap();
        let decision: Option<String> = conn
            .query_row("SELECT decision FROM audit_events WHERE id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(decision.as_deref(), Some("Block"));
    }

    #[test]
    fn update_trigger_blocks() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("audit_trigger.db");
        let store = AuditStore::init(&db_path).expect("init failed");

        // 同步插一条记录
        {
            let guard = store.conn.lock().unwrap();
            guard
                .execute(
                    INSERT_SQL,
                    params![
                        Utc::now().to_rfc3339(),
                        "outbound",
                        "OUT-01",
                        "Critical",
                        "redact",
                        Option::<String>::None,
                        "req-1",
                        Option::<String>::None,
                        Option::<i32>::None,    // caller_pid
                        Option::<String>::None, // caller_exe
                        UNKNOWN_PROVIDER_ID,    // provider_id (v3 schema)
                    ],
                )
                .unwrap();
        }

        // 尝试 UPDATE → 应该失败
        {
            let guard = store.conn.lock().unwrap();
            let result = guard.execute(
                "UPDATE audit_events SET rule_id = 'hacked' WHERE id = 1",
                [],
            );
            assert!(result.is_err(), "UPDATE 应该被触发器拒绝");
            let err_msg = result.unwrap_err().to_string();
            assert!(
                err_msg.contains("append-only"),
                "错误信息应含 append-only，实际: {err_msg}"
            );
        }

        // 尝试 DELETE → 应该失败
        {
            let guard = store.conn.lock().unwrap();
            let result = guard.execute("DELETE FROM audit_events WHERE id = 1", []);
            assert!(result.is_err(), "DELETE 应该被触发器拒绝");
        }
    }

    // ─── 新增：schema migration 测试 ───────────────────────────

    /// 构建一个模拟 v1 schema 的 in-memory 数据库（无 caller_pid/caller_exe 列，
    /// PRAGMA user_version = 1），插入一条记录，调用 migrate，验证数据无损 + 版本升到 2。
    #[test]
    fn migration_from_v1_preserves_data() {
        // 在 in-memory DB 手工建 v1 schema
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp_rfc3339   TEXT    NOT NULL,
                direction           TEXT    NOT NULL,
                rule_id             TEXT    NOT NULL,
                severity            TEXT    NOT NULL,
                disposition         TEXT    NOT NULL,
                decision            TEXT,
                request_id          TEXT    NOT NULL,
                raw_json            TEXT
            );
            PRAGMA user_version = 1;",
        )
        .unwrap();

        // 插入一条 v1 数据（8 列）
        conn.execute(
            "INSERT INTO audit_events
                (timestamp_rfc3339, direction, rule_id, severity, disposition,
                 decision, request_id, raw_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                "2026-04-27T00:00:00Z",
                "outbound",
                "OUT-01",
                "Critical",
                "redact",
                Option::<String>::None,
                "req-legacy",
                Option::<String>::None,
            ],
        )
        .unwrap();

        // 调用迁移（v1 → v3 一次性迁移）
        migrate(&conn).expect("migrate 应成功");

        // 验证 user_version = 3（v1 → v3 完整迁移）
        let ver: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ver, 3, "v1 迁移后 user_version 应为最新 v3");

        // 验证旧数据仍存在
        let rule_id: String = conn
            .query_row("SELECT rule_id FROM audit_events WHERE id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(rule_id, "OUT-01", "迁移后旧数据不应丢失");

        // 验证 v2 列存在且旧行为 NULL
        let pid: Option<i32> = conn
            .query_row(
                "SELECT caller_pid FROM audit_events WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let exe: Option<String> = conn
            .query_row(
                "SELECT caller_exe FROM audit_events WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(pid.is_none(), "迁移后旧行 caller_pid 应为 NULL");
        assert!(exe.is_none(), "迁移后旧行 caller_exe 应为 NULL");

        // 验证 v3 列存在且旧行默认 'unknown'
        let provider: String = conn
            .query_row(
                "SELECT provider_id FROM audit_events WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            provider, "unknown",
            "v1→v3 迁移后旧行 provider_id 应默认 'unknown'"
        );
    }

    /// 全新数据库（通过 AuditStore::init）应直接从 schema v3 开始，
    /// 包含 caller_pid / caller_exe / provider_id 列，PRAGMA user_version = 3。
    #[test]
    fn fresh_database_starts_at_v2() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("fresh.db");
        let _store = AuditStore::init(&db_path).expect("init failed");

        let conn = Connection::open(&db_path).unwrap();

        // 验证 user_version = 3
        let ver: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ver, 3, "全新 DB 的 user_version 应为最新 v3");

        // 验证 caller_pid / caller_exe / provider_id 列均存在
        let mut stmt = conn.prepare("PRAGMA table_info(audit_events)").unwrap();
        let cols: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(
            cols.contains(&"caller_pid".to_string()),
            "全新 DB 应含 caller_pid 列，实际列：{cols:?}"
        );
        assert!(
            cols.contains(&"caller_exe".to_string()),
            "全新 DB 应含 caller_exe 列，实际列：{cols:?}"
        );
        assert!(
            cols.contains(&"provider_id".to_string()),
            "全新 DB 应含 provider_id 列（v3），实际列：{cols:?}"
        );
    }

    /// 回归（pre-f7f794d 遗留库迁移）：user_version=0 的「遗留库」——表已存在但只有
    /// 9 个 v1 列（pre-f7f794d 早期构建从不写 user_version，停在 SQLite 默认 0）。
    /// 旧 migrate 的 `current == 0` 快速路径把它误判为「CREATE 刚建好的全列库」，
    /// 盲目 stamp version=3 却不跑 ADD COLUMN → 表缺列 + 版本号永久说谎 →
    /// 之后每次 append（固定 11 列 INSERT）都因 "no column named caller_pid" 失败，
    /// 表现为「审计写入全部失败」。修复后 migrate 以实表列为真相源逐列补齐。
    #[tokio::test]
    async fn migrate_legacy_v0_table_backfills_missing_columns() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("legacy_v0.db");

        // 播种遗留库：只含 9 个 v1 列，且不设 user_version（停在默认 0）。
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE audit_events (
                    id                INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp_rfc3339 TEXT NOT NULL,
                    direction         TEXT NOT NULL,
                    rule_id           TEXT NOT NULL,
                    severity          TEXT NOT NULL,
                    disposition       TEXT NOT NULL,
                    decision          TEXT,
                    request_id        TEXT NOT NULL,
                    raw_json          TEXT
                );
                INSERT INTO audit_events
                    (timestamp_rfc3339, direction, rule_id, severity, disposition, decision, request_id, raw_json)
                VALUES ('2026-01-01T00:00:00Z','outbound','OUT-01','Critical','redact',NULL,'legacy-req',NULL);",
            )
            .unwrap();
            let v: u32 = conn
                .query_row("PRAGMA user_version", [], |r| r.get(0))
                .unwrap();
            assert_eq!(v, 0, "遗留库应停在 user_version=0（前置版本管理从不写它）");
        }

        // 走完整公共路径：CREATE TABLE IF NOT EXISTS（对遗留表 no-op）+ migrate + triggers。
        let store = AuditStore::init(&db_path).expect("init on legacy v0 db failed");

        let conn = Connection::open(&db_path).unwrap();

        // (a) 版本收敛到 3。
        let ver: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ver, 3, "迁移后 user_version 应升到 v3");

        // (b) 三列被补齐。
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(audit_events)")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        for c in ["caller_pid", "caller_exe", "provider_id"] {
            assert!(
                cols.contains(&c.to_string()),
                "遗留库迁移后应含 {c} 列，实际：{cols:?}"
            );
        }

        // (c) 旧行 provider_id 取默认 'unknown'。
        let pid: String = conn
            .query_row(
                "SELECT provider_id FROM audit_events WHERE request_id = 'legacy-req'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(pid, "unknown", "补列后旧行 provider_id 应为默认 'unknown'");

        // (d) 核心断言：append 不再因缺列失败（修复前这里 "no column named caller_pid"）。
        store
            .append(make_event(2), "test-provider")
            .await
            .expect("append after legacy-v0 migration must succeed");

        let count: u32 = conn
            .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2, "遗留行 + 新 append 行共 2 条");
    }

    /// caller_pid / caller_exe 非 NULL 时写入后读出应一致。
    #[tokio::test]
    async fn caller_fields_persist_round_trip() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("caller_rt.db");
        let store = AuditStore::init(&db_path).expect("init failed");

        let event = AuditEvent::OutboundRedacted {
            rule_id: "OUT-01".to_string(),
            severity: "Critical".to_string(),
            request_id: "req-caller".to_string(),
            raw_json: None,
            caller: CallerContext {
                pid: Some(1234),
                exe: Some("/usr/bin/claude".to_string()),
            },
        };
        store
            .append(event, "test-provider")
            .await
            .expect("append failed");

        let conn = Connection::open(&db_path).unwrap();
        let (pid, exe): (Option<i32>, Option<String>) = conn
            .query_row(
                "SELECT caller_pid, caller_exe FROM audit_events WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(pid, Some(1234), "caller_pid 应为 1234");
        assert_eq!(
            exe.as_deref(),
            Some("/usr/bin/claude"),
            "caller_exe 应为 /usr/bin/claude"
        );
    }

    /// caller_pid / caller_exe 为 None 时写入后读出应仍为 NULL。
    #[tokio::test]
    async fn null_caller_fields_round_trip() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("null_caller.db");
        let store = AuditStore::init(&db_path).expect("init failed");

        let event = AuditEvent::InboundHookMarked {
            rule_id: "IN-CR-02".to_string(),
            severity: "Critical".to_string(),
            request_id: "req-no-caller".to_string(),
            raw_json: None,
            caller: CallerContext::default(),
        };
        store
            .append(event, UNKNOWN_PROVIDER_ID)
            .await
            .expect("append failed");

        let conn = Connection::open(&db_path).unwrap();
        let (pid, exe): (Option<i32>, Option<String>) = conn
            .query_row(
                "SELECT caller_pid, caller_exe FROM audit_events WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(pid.is_none(), "caller_pid 应为 NULL");
        assert!(exe.is_none(), "caller_exe 应为 NULL");
    }

    /// 从 v1 迁移到 v2 后，append-only 触发器依然阻止 UPDATE。
    #[test]
    fn append_only_trigger_still_blocks_update_after_migration() {
        // 构建 v1 DB
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp_rfc3339   TEXT    NOT NULL,
                direction           TEXT    NOT NULL,
                rule_id             TEXT    NOT NULL,
                severity            TEXT    NOT NULL,
                disposition         TEXT    NOT NULL,
                decision            TEXT,
                request_id          TEXT    NOT NULL,
                raw_json            TEXT
            );
            PRAGMA user_version = 1;",
        )
        .unwrap();

        // 安装触发器
        conn.execute_batch(APPEND_ONLY_TRIGGERS_DDL).unwrap();

        // 插入一条数据
        conn.execute(
            "INSERT INTO audit_events
                (timestamp_rfc3339, direction, rule_id, severity, disposition,
                 decision, request_id, raw_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                "2026-04-27T00:00:00Z",
                "outbound",
                "OUT-01",
                "Critical",
                "redact",
                Option::<String>::None,
                "req-x",
                Option::<String>::None,
            ],
        )
        .unwrap();

        // 执行 v1→v2 迁移
        migrate(&conn).expect("migrate 应成功");

        // 迁移后尝试 UPDATE → 应该仍然失败
        let result = conn.execute(
            "UPDATE audit_events SET rule_id = 'tampered' WHERE id = 1",
            [],
        );
        assert!(result.is_err(), "迁移后 UPDATE 应仍被触发器拒绝");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("append-only"),
            "错误信息应含 append-only，实际: {err_msg}"
        );
    }

    // ─── v2.1 GraylistAddFailed 变体测试 ──────────────────────────

    /// GraylistAddFailed 事件能够写入并从 SQLite 读回，字段完整。
    ///
    /// 验证：disposition = "graylist_add_failed"、rule_id、request_id、caller 字段全部持久化。
    #[tokio::test]
    async fn graylist_add_failed_event_persists() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("graylist_fail.db");
        let store = AuditStore::init(&db_path).expect("init failed");

        let event = AuditEvent::GraylistAddFailed {
            rule_id: "IN-GEN-04".to_string(),
            error: "磁盘空间不足: No space left on device".to_string(),
            request_id: "req-fail-001".to_string(),
            caller: CallerContext {
                pid: Some(4242),
                exe: Some("/usr/local/bin/claude".to_string()),
            },
        };

        store
            .append(event, "test-provider")
            .await
            .expect("GraylistAddFailed append 不应失败");

        let conn = Connection::open(&db_path).unwrap();
        let row: (String, String, String, Option<i32>, Option<String>) = conn
            .query_row(
                "SELECT rule_id, disposition, request_id, caller_pid, caller_exe
                 FROM audit_events WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();
        let (rule_id, disposition, request_id, caller_pid, caller_exe) = row;
        assert_eq!(rule_id, "IN-GEN-04", "rule_id 应持久化");
        assert_eq!(
            disposition, "graylist_add_failed",
            "disposition 应为 graylist_add_failed"
        );
        assert_eq!(request_id, "req-fail-001", "request_id 应持久化");
        assert_eq!(caller_pid, Some(4242), "caller_pid 应持久化");
        assert_eq!(
            caller_exe.as_deref(),
            Some("/usr/local/bin/claude"),
            "caller_exe 应持久化"
        );
    }

    /// GraylistAddFailed direction = "inbound"，severity = "info"。
    #[test]
    fn graylist_add_failed_metadata() {
        let event = AuditEvent::GraylistAddFailed {
            rule_id: "IN-GEN-04".to_string(),
            error: "测试错误".to_string(),
            request_id: "req-meta".to_string(),
            caller: CallerContext::default(),
        };
        // 验证内部方法返回正确元数据
        assert_eq!(event.rule_id(), "IN-GEN-04", "rule_id getter 应返回正确值");
        assert_eq!(
            event.severity(),
            "info",
            "GraylistAddFailed severity 应为 info（fail-soft 事件）"
        );
        assert_eq!(
            event.disposition(),
            "graylist_add_failed",
            "GraylistAddFailed disposition 应为 graylist_add_failed"
        );
    }
}
