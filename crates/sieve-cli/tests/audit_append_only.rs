//! `AuditStore` append-only 集成测试（ADR-007 / ADR-014）。
//!
//! 验证：写 3 条 → SELECT 能读到；UPDATE / DELETE 被触发器拒绝。
//!
//! 注意：由于 sieve-cli 是纯 binary crate，这里通过子进程或直接用 rusqlite 验证。
//! audit.rs 中已有 `#[cfg(test)]` 单元测试覆盖同等逻辑；本集成测试作为补充验证。

use rusqlite::{params, Connection};
use tempfile::tempdir;

const CREATE_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS audit_events (
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
"#;

const TRIGGERS_DDL: &str = r#"
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

const INSERT_SQL: &str = r#"
INSERT INTO audit_events
    (timestamp_rfc3339, direction, rule_id, severity, disposition, decision, request_id, raw_json)
VALUES
    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
"#;

fn setup_db(path: &std::path::Path) -> Connection {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(CREATE_DDL).unwrap();
    conn.execute_batch(TRIGGERS_DDL).unwrap();
    conn
}

#[test]
fn write_3_events_and_read_back() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_audit.db");
    let conn = setup_db(&db_path);

    for i in 1..=3u32 {
        conn.execute(
            INSERT_SQL,
            params![
                format!("2026-04-27T00:0{i}:00Z"),
                "outbound",
                format!("OUT-0{i}"),
                "Critical",
                "redact",
                Option::<String>::None,
                format!("req-{i}"),
                Option::<String>::None,
            ],
        )
        .unwrap();
    }

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM audit_events", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 3, "应有 3 条记录");

    let rule_id: String = conn
        .query_row("SELECT rule_id FROM audit_events WHERE id = 2", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(rule_id, "OUT-02");
}

#[test]
fn update_is_rejected_by_trigger() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_trigger_update.db");
    let conn = setup_db(&db_path);

    conn.execute(
        INSERT_SQL,
        params![
            "2026-04-27T00:00:00Z",
            "inbound",
            "IN-CR-01",
            "Critical",
            "pending",
            Option::<String>::None,
            "req-x",
            Option::<String>::None,
        ],
    )
    .unwrap();

    let result = conn.execute(
        "UPDATE audit_events SET rule_id = 'tampered' WHERE id = 1",
        [],
    );
    assert!(result.is_err(), "UPDATE 应被触发器拒绝");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("append-only"),
        "错误信息应含 'append-only'，实际: {msg}"
    );
}

#[test]
fn delete_is_rejected_by_trigger() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_trigger_delete.db");
    let conn = setup_db(&db_path);

    conn.execute(
        INSERT_SQL,
        params![
            "2026-04-27T00:00:00Z",
            "inbound",
            "IN-CR-02",
            "Critical",
            "resolved",
            Some("Block"),
            "req-y",
            Option::<String>::None,
        ],
    )
    .unwrap();

    let result = conn.execute("DELETE FROM audit_events WHERE id = 1", []);
    assert!(result.is_err(), "DELETE 应被触发器拒绝");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("append-only"),
        "错误信息应含 'append-only'，实际: {msg}"
    );
}
