//! 审计日志（关联 data-model.md §审计 + ADR-007 + ADR-014）。
//!
//! Week 5 起接入 SQLite append-only 存储。
//! Week 6（PRD §5.6.1 v2.0）：schema v2 加 caller_pid / caller_exe 两列。
//!
//! 设计约束（ADR-007 / ADR-014）：
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

/// 调用方上下文（PRD §5.6.1 v2.0）。
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

/// 审计事件枚举（关联 PRD §5.4 处置矩阵 + ADR-014 双层防御日志需求）。
///
/// PRD §5.6.1 v2.0：每个 variant 含 `caller: CallerContext`，
/// 记录 caller_pid / caller_exe；`#[serde(default)]` 保证旧 raw_json 反序列化兼容。
// 方法在 daemon 完整接入前不被调用；Week 6 移除此 allow。
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuditEvent {
    /// 出站请求中检测到敏感内容并脱敏。
    OutboundRedacted {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
        /// 调用方上下文（PRD §5.6.1）；旧 JSON 缺失时 Default 为空。
        #[serde(default)]
        caller: CallerContext,
    },
    /// 入站响应 hook 标记了疑似高危工具调用。
    InboundHookMarked {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
        /// 调用方上下文（PRD §5.6.1）；旧 JSON 缺失时 Default 为空。
        #[serde(default)]
        caller: CallerContext,
    },
    /// 入站高危工具调用等待用户决策。
    InboundDecisionRequested {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
        /// 调用方上下文（PRD §5.6.1）；旧 JSON 缺失时 Default 为空。
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
        /// 调用方上下文（PRD §5.6.1）；旧 JSON 缺失时 Default 为空。
        #[serde(default)]
        caller: CallerContext,
    },
    /// 状态栏通知已发送。
    StatusBarNotified {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
        /// 调用方上下文（PRD §5.6.1）；旧 JSON 缺失时 Default 为空。
        #[serde(default)]
        caller: CallerContext,
    },
}

// impl 方法仅在 tests 和 append 中使用；Week 6 接入后移除此 allow。
#[allow(dead_code)]
impl AuditEvent {
    fn direction(&self) -> &'static str {
        match self {
            Self::OutboundRedacted { .. } => "outbound",
            Self::InboundHookMarked { .. }
            | Self::InboundDecisionRequested { .. }
            | Self::InboundDecisionResolved { .. }
            | Self::StatusBarNotified { .. } => "inbound",
        }
    }

    fn rule_id(&self) -> &str {
        match self {
            Self::OutboundRedacted { rule_id, .. }
            | Self::InboundHookMarked { rule_id, .. }
            | Self::InboundDecisionRequested { rule_id, .. }
            | Self::InboundDecisionResolved { rule_id, .. }
            | Self::StatusBarNotified { rule_id, .. } => rule_id,
        }
    }

    fn severity(&self) -> &str {
        match self {
            Self::OutboundRedacted { severity, .. }
            | Self::InboundHookMarked { severity, .. }
            | Self::InboundDecisionRequested { severity, .. }
            | Self::InboundDecisionResolved { severity, .. }
            | Self::StatusBarNotified { severity, .. } => severity,
        }
    }

    fn disposition(&self) -> &'static str {
        match self {
            Self::OutboundRedacted { .. } => "redact",
            Self::InboundHookMarked { .. } => "mark",
            Self::InboundDecisionRequested { .. } => "pending",
            Self::InboundDecisionResolved { .. } => "resolved",
            Self::StatusBarNotified { .. } => "notify",
        }
    }

    fn decision(&self) -> Option<&str> {
        if let Self::InboundDecisionResolved { decision, .. } = self {
            Some(decision)
        } else {
            None
        }
    }

    fn request_id(&self) -> &str {
        match self {
            Self::OutboundRedacted { request_id, .. }
            | Self::InboundHookMarked { request_id, .. }
            | Self::InboundDecisionRequested { request_id, .. }
            | Self::InboundDecisionResolved { request_id, .. }
            | Self::StatusBarNotified { request_id, .. } => request_id,
        }
    }

    fn raw_json(&self) -> Option<&str> {
        match self {
            Self::OutboundRedacted { raw_json, .. }
            | Self::InboundHookMarked { raw_json, .. }
            | Self::InboundDecisionRequested { raw_json, .. }
            | Self::InboundDecisionResolved { raw_json, .. }
            | Self::StatusBarNotified { raw_json, .. } => raw_json.as_deref(),
        }
    }

    /// 提取调用方 PID（PRD §5.6.1）。
    fn caller_pid(&self) -> Option<i32> {
        match self {
            Self::OutboundRedacted { caller, .. }
            | Self::InboundHookMarked { caller, .. }
            | Self::InboundDecisionRequested { caller, .. }
            | Self::InboundDecisionResolved { caller, .. }
            | Self::StatusBarNotified { caller, .. } => caller.pid,
        }
    }

    /// 提取调用方可执行路径（PRD §5.6.1）。
    fn caller_exe(&self) -> Option<&str> {
        match self {
            Self::OutboundRedacted { caller, .. }
            | Self::InboundHookMarked { caller, .. }
            | Self::InboundDecisionRequested { caller, .. }
            | Self::InboundDecisionResolved { caller, .. }
            | Self::StatusBarNotified { caller, .. } => caller.exe.as_deref(),
        }
    }
}

// ─────────────────────────── Schema migration ──────────────────────────────

/// 当前 schema 版本（v2.0，PRD §5.6.1）。
const CURRENT_SCHEMA_VERSION: u32 = 2;

/// 打开数据库后执行一次 schema 迁移。
///
/// - 全新 DB（user_version = 0）：CREATE TABLE 已含新列，直接设版本号。
/// - v1 老 DB（user_version = 1）：`ALTER TABLE ADD COLUMN`，不重写表，不丢数据。
/// - v2 及以上：跳过。
///
/// 迁移在事务内执行，失败自动回滚（PRD §5.6.1）。
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

    if current == 0 {
        // 全新数据库：CREATE TABLE 已包含 caller_pid / caller_exe；
        // 直接将版本号设为最新。
        conn.execute_batch(&format!("PRAGMA user_version = {CURRENT_SCHEMA_VERSION};"))
            .context("设置 user_version 失败")?;
        return Ok(());
    }

    if current < CURRENT_SCHEMA_VERSION {
        // v1 → v2：为旧表加两列。
        // ALTER TABLE ADD COLUMN 在 SQLite 中是 O(1) 操作（不重写表），
        // 新列对现有行为 NULL，不触发 NOT NULL 约束失败（列定义无 NOT NULL）。
        // BEFORE UPDATE/DELETE 触发器基于行操作，ADD COLUMN 不失效触发器。
        conn.execute_batch(&format!(
            "BEGIN;
             ALTER TABLE audit_events ADD COLUMN caller_pid INTEGER;
             ALTER TABLE audit_events ADD COLUMN caller_exe TEXT;
             PRAGMA user_version = {CURRENT_SCHEMA_VERSION};
             COMMIT;"
        ))
        .context("v1→v2 schema 迁移失败")?;
    }

    Ok(())
}

// ─────────────────────────── AuditStore ────────────────────────────────────

/// 审计存储句柄（SQLite append-only）。
///
/// Week 5 起持有真实 SQLite 连接；线程安全通过 `Arc<Mutex<Connection>>` 实现。
/// 关联 ADR-014 双层防御日志需求。
// Week 5：`conn` / `append` 在 daemon 完整接入前不被调用，加 allow 避免 dead_code lint。
// Week 6 接入后移除这个属性。
#[allow(dead_code)]
pub struct AuditStore {
    conn: Arc<Mutex<Connection>>,
}

// `append` 在 daemon 完整接入前不被 main.rs 调用；Week 6 移除此 allow。
#[allow(dead_code)]
impl AuditStore {
    /// 初始化审计存储：打开 SQLite，执行 schema 迁移，安装 append-only 触发器。
    ///
    /// 幂等——文件已存在时执行 schema 迁移（v1→v2），不重建表（PRD §5.6.1）。
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

    /// 异步写入一条审计事件（spawn_blocking + Mutex 串行化）。
    ///
    /// # Errors
    /// SQLite 写入失败时返回错误。
    pub async fn append(&self, event: AuditEvent) -> Result<()> {
        let conn = Arc::clone(&self.conn);
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
                ],
            )?;
            Ok::<(), anyhow::Error>(())
        })
        .await
        .context("spawn_blocking failed")??;
        Ok(())
    }
}

// ─────────────────────────── SQL 常量 ──────────────────────────────────────

/// 建表 DDL（含 v2 新列 caller_pid / caller_exe，PRD §5.6.1）。
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
    caller_pid          INTEGER,            -- 调用方 PID（PRD §5.6.1，NULL 表示未知）
    caller_exe          TEXT                -- 调用方可执行路径（PRD §5.6.1，NULL 表示未知）
);
"#;

/// append-only 触发器：拒绝 UPDATE / DELETE（ADR-007 / ADR-014）。
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
     request_id, raw_json, caller_pid, caller_exe)
VALUES
    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
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
            store.append(make_event(i)).await.expect("append failed");
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

        store.append(make_decision_event()).await.unwrap();

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

    // ─── 新增：schema migration 测试（PRD §5.6.1）───────────────────────────

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

        // 调用迁移
        migrate(&conn).expect("migrate 应成功");

        // 验证 user_version = 2
        let ver: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ver, 2, "迁移后 user_version 应为 2");

        // 验证旧数据仍存在
        let rule_id: String = conn
            .query_row("SELECT rule_id FROM audit_events WHERE id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(rule_id, "OUT-01", "迁移后旧数据不应丢失");

        // 验证新列存在且旧行为 NULL
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
    }

    /// 全新数据库（通过 AuditStore::init）应直接从 schema v2 开始，
    /// 包含 caller_pid / caller_exe 列，PRAGMA user_version = 2。
    #[test]
    fn fresh_database_starts_at_v2() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("fresh.db");
        let _store = AuditStore::init(&db_path).expect("init failed");

        let conn = Connection::open(&db_path).unwrap();

        // 验证 user_version = 2
        let ver: u32 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ver, 2, "全新 DB 的 user_version 应为 2");

        // 验证 caller_pid / caller_exe 列存在（pragma table_info 返回列描述）
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
        store.append(event).await.expect("append failed");

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
        store.append(event).await.expect("append failed");

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
}
