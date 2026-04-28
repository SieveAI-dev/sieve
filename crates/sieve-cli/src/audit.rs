//! 审计日志（关联 data-model.md §审计 + ADR-007 + ADR-014）。
//!
//! Week 5 起接入 SQLite append-only 存储。
//!
//! 设计约束（ADR-007 / ADR-014）：
//! - SQLite append-only：BEFORE UPDATE / DELETE 触发器拒绝修改。
//! - 异步写入接口：`tokio::task::spawn_blocking` + internal `Mutex` 串行化。
//! - 不暴露 `rusqlite` 类型到 crate 外部。

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};

// ─────────────────────────── AuditEvent ────────────────────────────────────

/// 审计事件枚举（关联 PRD §5.4 处置矩阵 + ADR-014 双层防御日志需求）。
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
    },
    /// 入站响应 hook 标记了疑似高危工具调用。
    InboundHookMarked {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
    },
    /// 入站高危工具调用等待用户决策。
    InboundDecisionRequested {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
    },
    /// 用户对高危工具调用给出决策（Allow / Block）。
    InboundDecisionResolved {
        rule_id: String,
        severity: String,
        decision: String,
        request_id: String,
        raw_json: Option<String>,
    },
    /// 状态栏通知已发送。
    StatusBarNotified {
        rule_id: String,
        severity: String,
        request_id: String,
        raw_json: Option<String>,
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
    /// 初始化审计存储：打开 SQLite，创建表，安装 append-only 触发器。
    ///
    /// 幂等——文件已存在时不重建表。
    ///
    /// # Errors
    /// SQLite 打开或 DDL 执行失败时返回错误。
    pub fn init(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建审计目录 {} 失败", parent.display()))?;
        }

        let conn = Connection::open(path)
            .with_context(|| format!("打开审计数据库 {} 失败", path.display()))?;

        // 建表
        conn.execute_batch(CREATE_TABLE_DDL)
            .context("创建 audit_events 表失败")?;

        // 安装 append-only 触发器（幂等：IF NOT EXISTS 不会重建）
        conn.execute_batch(APPEND_ONLY_TRIGGERS_DDL)
            .context("安装 append-only 触发器失败")?;

        tracing::debug!(path = %path.display(), "audit store initialized (SQLite)");
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
    raw_json            TEXT
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
    (timestamp_rfc3339, direction, rule_id, severity, disposition, decision, request_id, raw_json)
VALUES
    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
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
        }
    }

    fn make_decision_event() -> AuditEvent {
        AuditEvent::InboundDecisionResolved {
            rule_id: "IN-CR-01".to_string(),
            severity: "Critical".to_string(),
            decision: "Block".to_string(),
            request_id: "req-decision".to_string(),
            raw_json: None,
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
}
