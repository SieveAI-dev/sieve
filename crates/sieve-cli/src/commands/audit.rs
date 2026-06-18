//! `sieve audit` 子命令实现（ADR-028 TODO-5，unix-pipeable）。
//!
//! 直接读 `~/.sieve/audit.db` SQLite，输出 jsonl 格式方便接 jq / fluentd / vector。
//! 不修改 `crates/sieve-cli/src/audit.rs` 的写入路径，本文件是**只读查询路径**。
//!
//! ## 子命令
//!
//! - `tail [-f|--follow] [--format jsonl|pretty] [--limit N]`
//! - `query [--since DUR] [--severity SEV] [--rule-id RULE] [--provider-id PROVIDER] [--format jsonl|pretty]`
//! - `show <id>`

use anyhow::{anyhow, Context, Result};
use chrono::{Duration as ChronoDuration, Utc};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cli::{AuditArgs, AuditCommand, OutputFormat, Severity};

// ─────────────────────────── 路径辅助 ──────────────────────────────────────

fn audit_db_path() -> Result<PathBuf> {
    let home = sieve_ipc::paths::sieve_home().context("获取 sieve home 失败")?;
    Ok(home.join("audit.db"))
}

// ─────────────────────────── 输出 schema ───────────────────────────────────

/// audit_events 表单行输出 schema（jsonl 格式）。
///
/// 字段对照 v3 schema（含 provider_id）。
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditRow {
    pub id: i64,
    pub timestamp: String,
    pub direction: String,
    pub rule_id: String,
    pub severity: String,
    pub disposition: String,
    pub decision: Option<String>,
    pub request_id: String,
    pub provider_id: String,
    pub caller_pid: Option<i64>,
    pub caller_exe: Option<String>,
    pub raw_json: Option<String>,
}

impl AuditRow {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            direction: row.get(2)?,
            rule_id: row.get(3)?,
            severity: row.get(4)?,
            disposition: row.get(5)?,
            decision: row.get(6)?,
            request_id: row.get(7)?,
            raw_json: row.get(8)?,
            caller_pid: row.get(9)?,
            caller_exe: row.get(10)?,
            provider_id: row
                .get::<_, Option<String>>(11)?
                .unwrap_or_else(|| "unknown".to_owned()),
        })
    }
}

fn print_rows(rows: &[AuditRow], format: OutputFormat) {
    match format {
        OutputFormat::Jsonl => {
            for row in rows {
                if let Ok(s) = serde_json::to_string(row) {
                    println!("{s}");
                }
            }
        }
        OutputFormat::Pretty => {
            for row in rows {
                if let Ok(s) = serde_json::to_string_pretty(row) {
                    println!("{s}");
                }
            }
        }
    }
}

// ─────────────────────────── SELECT helper ─────────────────────────────────

const SELECT_ALL_COLS: &str = r#"
SELECT id, timestamp_rfc3339, direction, rule_id, severity, disposition,
       decision, request_id, raw_json, caller_pid, caller_exe, provider_id
FROM audit_events
"#;

fn query_rows(
    conn: &Connection,
    where_clause: &str,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<AuditRow>> {
    let sql = format!("{SELECT_ALL_COLS} {where_clause}");
    let mut stmt = conn.prepare(&sql).context("准备 SQL 语句失败")?;
    let rows = stmt
        .query_map(
            rusqlite::params_from_iter(params.iter()),
            AuditRow::from_row,
        )
        .context("查询 audit_events 失败")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("读取行失败")?;
    Ok(rows)
}

// ─────────────────────────── tail ──────────────────────────────────────────

async fn run_tail(follow: bool, format: OutputFormat, limit: u32) -> Result<()> {
    let db_path = audit_db_path()?;
    if !db_path.exists() {
        return Err(anyhow!(
            "审计数据库不存在（{}）；请先启动 sieve daemon",
            db_path.display()
        ));
    }

    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("打开审计数据库失败（{}）", db_path.display()))?;

    // 初始输出：最后 limit 条
    let rows = query_rows(&conn, &format!("ORDER BY id DESC LIMIT {limit}"), &[])?;
    // 倒序后逆转为时间正序输出
    let mut rows_asc = rows;
    rows_asc.reverse();
    print_rows(&rows_asc, format);

    if !follow {
        return Ok(());
    }

    // --follow 模式：轮询新记录
    let mut last_id: i64 = rows_asc.last().map(|r| r.id).unwrap_or(0);

    // 捕获 Ctrl+C，优雅退出
    let ctrlc = tokio::signal::ctrl_c();
    tokio::pin!(ctrlc);

    loop {
        // 500ms 间隔轮询
        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                let new_rows = query_rows(
                    &conn,
                    "WHERE id > ?1 ORDER BY id LIMIT 100",
                    &[&last_id],
                )?;
                if !new_rows.is_empty() {
                    last_id = new_rows.last().unwrap().id;
                    print_rows(&new_rows, format);
                }
            }
            _ = &mut ctrlc => {
                break;
            }
        }
    }

    Ok(())
}

// ─────────────────────────── duration 解析 ─────────────────────────────────

/// 解析 duration 字符串（"1h" / "30m" / "7d"）为 chrono::Duration。
pub fn parse_duration(s: &str) -> Result<ChronoDuration> {
    let s = s.trim();
    if let Some(n) = s.strip_suffix('h') {
        let hours: i64 = n.parse().with_context(|| format!("无效小时数: {n}"))?;
        return Ok(ChronoDuration::hours(hours));
    }
    if let Some(n) = s.strip_suffix('m') {
        let mins: i64 = n.parse().with_context(|| format!("无效分钟数: {n}"))?;
        return Ok(ChronoDuration::minutes(mins));
    }
    if let Some(n) = s.strip_suffix('d') {
        let days: i64 = n.parse().with_context(|| format!("无效天数: {n}"))?;
        return Ok(ChronoDuration::days(days));
    }
    Err(anyhow!("无法解析 duration（支持格式：30m / 1h / 7d）: {s}"))
}

// ─────────────────────────── query ─────────────────────────────────────────

async fn run_query(
    since: Option<String>,
    severity: Option<Severity>,
    rule_id: Option<String>,
    provider_id: Option<String>,
    format: OutputFormat,
) -> Result<()> {
    let db_path = audit_db_path()?;
    if !db_path.exists() {
        return Err(anyhow!(
            "审计数据库不存在（{}）；请先启动 sieve daemon",
            db_path.display()
        ));
    }

    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("打开审计数据库失败（{}）", db_path.display()))?;

    // 动态构造 WHERE 子句
    let mut conditions: Vec<String> = Vec::new();
    let mut param_values: Vec<String> = Vec::new();

    if let Some(ref since_str) = since {
        let dur = parse_duration(since_str)?;
        let cutoff = Utc::now() - dur;
        conditions.push(format!("timestamp_rfc3339 >= ?{}", param_values.len() + 1));
        param_values.push(cutoff.to_rfc3339());
    }

    if let Some(ref sev) = severity {
        let sev_str = match sev {
            Severity::Critical => "Critical",
            Severity::High => "High",
            Severity::Medium => "Medium",
            Severity::Low => "Low",
        };
        conditions.push(format!("severity = ?{}", param_values.len() + 1));
        param_values.push(sev_str.to_owned());
    }

    if let Some(ref rid) = rule_id {
        conditions.push(format!("rule_id = ?{}", param_values.len() + 1));
        param_values.push(rid.clone());
    }

    if let Some(ref pid) = provider_id {
        conditions.push(format!("provider_id = ?{}", param_values.len() + 1));
        param_values.push(pid.clone());
    }

    let where_clause = if conditions.is_empty() {
        "ORDER BY id".to_owned()
    } else {
        format!("WHERE {} ORDER BY id", conditions.join(" AND "))
    };

    // 用 rusqlite 动态参数（不能用 query_rows helper 因为参数个数动态）
    let sql = format!("{SELECT_ALL_COLS} {where_clause}");
    let mut stmt = conn.prepare(&sql).context("准备 SQL 语句失败")?;

    let params_ref: Vec<&dyn rusqlite::ToSql> = param_values
        .iter()
        .map(|s| s as &dyn rusqlite::ToSql)
        .collect();

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_ref), AuditRow::from_row)
        .context("查询失败")?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("读取行失败")?;

    print_rows(&rows, format);
    Ok(())
}

// ─────────────────────────── show ──────────────────────────────────────────

async fn run_show(id: i64) -> Result<()> {
    let db_path = audit_db_path()?;
    if !db_path.exists() {
        return Err(anyhow!(
            "审计数据库不存在（{}）；请先启动 sieve daemon",
            db_path.display()
        ));
    }

    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("打开审计数据库失败（{}）", db_path.display()))?;

    let rows = query_rows(&conn, "WHERE id = ?1", &[&id])?;
    if rows.is_empty() {
        return Err(anyhow!("未找到 id={id} 的审计事件"));
    }
    print_rows(&rows, OutputFormat::Pretty);
    Ok(())
}

// ─────────────────────────── 入口 ──────────────────────────────────────────

/// `sieve audit` 命令入口。
///
/// 必须是 `async`：`main` 是 `#[tokio::main]`，本命令在该 runtime 内被 `.await`。
/// 旧实现在此 `Builder::new_current_thread().block_on(...)` 会触发
/// "Cannot start a runtime from within a runtime" panic（headless dogfood e2e 抓出，
/// 见 tasks/lessons.md 2026-06-18）——`sieve audit` 任一子命令直接崩溃、exit 134。
pub async fn run(args: AuditArgs) -> Result<()> {
    run_async(args).await
}

async fn run_async(args: AuditArgs) -> Result<()> {
    match args.command {
        AuditCommand::Tail {
            follow,
            format,
            limit,
        } => run_tail(follow, format.unwrap_or(OutputFormat::Jsonl), limit).await,
        AuditCommand::Query {
            since,
            severity,
            rule_id,
            provider_id,
            format,
        } => {
            run_query(
                since,
                severity,
                rule_id,
                provider_id,
                format.unwrap_or(OutputFormat::Jsonl),
            )
            .await
        }
        AuditCommand::Show { id } => run_show(id).await,
    }
}

// ─────────────────────────── 单元测试 ──────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rusqlite::{params, Connection};
    use tempfile::tempdir;

    fn create_test_db(dir: &std::path::Path) -> Connection {
        let db_path = dir.join("audit.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS audit_events (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp_rfc3339   TEXT    NOT NULL,
                direction           TEXT    NOT NULL,
                rule_id             TEXT    NOT NULL,
                severity            TEXT    NOT NULL,
                disposition         TEXT    NOT NULL,
                decision            TEXT,
                request_id          TEXT    NOT NULL,
                raw_json            TEXT,
                caller_pid          INTEGER,
                caller_exe          TEXT,
                provider_id         TEXT    NOT NULL DEFAULT 'unknown'
            );
        "#,
        )
        .unwrap();
        conn
    }

    fn insert_test_row(conn: &Connection, rule_id: &str, severity: &str, provider_id: &str) {
        conn.execute(
            r#"INSERT INTO audit_events
               (timestamp_rfc3339, direction, rule_id, severity, disposition,
                decision, request_id, raw_json, caller_pid, caller_exe, provider_id)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                Utc::now().to_rfc3339(),
                "outbound",
                rule_id,
                severity,
                "redact",
                Option::<String>::None,
                "req-001",
                Option::<String>::None,
                Option::<i64>::None,
                Option::<String>::None,
                provider_id,
            ],
        )
        .unwrap();
    }

    /// tail --limit N 返回最多 N 条记录。
    #[test]
    fn tail_limit_returns_at_most_n_rows() {
        let dir = tempdir().unwrap();
        let conn = create_test_db(dir.path());
        for i in 0..10 {
            insert_test_row(&conn, &format!("OUT-0{i}"), "Critical", "anthropic");
        }

        // 用 query_rows 直接测 LIMIT 逻辑
        let rows = query_rows(&conn, "ORDER BY id DESC LIMIT 5", &[]).unwrap();
        assert_eq!(rows.len(), 5, "tail --limit 5 应返回 5 条");
    }

    /// query --since 解析：1h 应过滤出最近 1 小时的记录。
    #[test]
    fn query_since_1h_parses_correctly() {
        let dur = parse_duration("1h").unwrap();
        assert_eq!(dur.num_hours(), 1, "1h 应解析为 1 小时");

        let dur30m = parse_duration("30m").unwrap();
        assert_eq!(dur30m.num_minutes(), 30, "30m 应解析为 30 分钟");

        let dur7d = parse_duration("7d").unwrap();
        assert_eq!(dur7d.num_days(), 7, "7d 应解析为 7 天");
    }

    /// parse_duration 无效输入应返回错误。
    #[test]
    fn parse_duration_invalid_returns_error() {
        assert!(parse_duration("abc").is_err(), "无效 duration 应返回 Err");
        assert!(parse_duration("1x").is_err(), "未知单位应返回 Err");
        assert!(parse_duration("").is_err(), "空字符串应返回 Err");
    }

    /// jsonl 字段对齐：AuditRow 序列化应含所有必要字段。
    #[test]
    fn audit_row_jsonl_fields_aligned() {
        let row = AuditRow {
            id: 1,
            timestamp: "2026-05-05T12:34:56Z".to_owned(),
            direction: "outbound".to_owned(),
            rule_id: "OUT-01".to_owned(),
            severity: "Critical".to_owned(),
            disposition: "redact".to_owned(),
            decision: None,
            request_id: "req-001".to_owned(),
            provider_id: "anthropic".to_owned(),
            caller_pid: Some(1234),
            caller_exe: Some("/usr/bin/claude".to_owned()),
            raw_json: Some("{\"test\":1}".to_owned()),
        };

        let json = serde_json::to_string(&row).expect("序列化应成功");
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // 验证所有必要字段存在
        for field in &[
            "id",
            "timestamp",
            "direction",
            "rule_id",
            "severity",
            "disposition",
            "request_id",
            "provider_id",
            "caller_pid",
            "caller_exe",
        ] {
            assert!(parsed.get(field).is_some(), "jsonl 输出应含 {field} 字段");
        }

        // jsonl 格式：无换行
        assert!(!json.contains('\n'), "jsonl 格式不应含换行符");

        // 验证字段值
        assert_eq!(parsed["id"].as_i64(), Some(1));
        assert_eq!(parsed["rule_id"].as_str(), Some("OUT-01"));
        assert_eq!(parsed["provider_id"].as_str(), Some("anthropic"));
        assert_eq!(parsed["caller_pid"].as_i64(), Some(1234));
    }

    /// query 按 provider_id 过滤。
    #[test]
    fn query_provider_id_filter() {
        let dir = tempdir().unwrap();
        let conn = create_test_db(dir.path());
        insert_test_row(&conn, "OUT-01", "Critical", "anthropic");
        insert_test_row(&conn, "OUT-02", "High", "openai");
        insert_test_row(&conn, "OUT-03", "Medium", "anthropic");

        let rows = query_rows(
            &conn,
            "WHERE provider_id = ?1 ORDER BY id",
            &[&"anthropic" as &dyn rusqlite::ToSql],
        )
        .unwrap();
        assert_eq!(rows.len(), 2, "应返回 2 条 anthropic 记录");
        assert!(rows.iter().all(|r| r.provider_id == "anthropic"));
    }

    /// query 按 severity 过滤（大小写敏感，schema 存储为首字母大写）。
    #[test]
    fn query_severity_filter() {
        let dir = tempdir().unwrap();
        let conn = create_test_db(dir.path());
        insert_test_row(&conn, "OUT-01", "Critical", "anthropic");
        insert_test_row(&conn, "OUT-02", "High", "anthropic");
        insert_test_row(&conn, "OUT-03", "Critical", "anthropic");

        let rows = query_rows(
            &conn,
            "WHERE severity = ?1 ORDER BY id",
            &[&"Critical" as &dyn rusqlite::ToSql],
        )
        .unwrap();
        assert_eq!(rows.len(), 2, "应返回 2 条 Critical 记录");
    }
}
