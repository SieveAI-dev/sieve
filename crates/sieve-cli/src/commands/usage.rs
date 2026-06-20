//! `sieve usage` 子命令（ADR-038 / SPEC-010）——本地 token 用量与超额计费查询。
//!
//! **隐私红线**：只读 `~/.sieve/usage.db`（严格本地、永不上传）。本模块是**只读查询
//! 路径**，与 `billing::usage_store`（写入）分离，无任何出站。

use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::cli::{OutputFormat, UsageArgs, UsageCommand};

fn usage_db_path() -> Result<PathBuf> {
    let home = sieve_ipc::paths::sieve_home().context("获取 sieve home 失败")?;
    Ok(home.join("usage.db"))
}

/// usage_records 单行输出 schema（jsonl）。
#[derive(Debug, Serialize, Deserialize)]
struct UsageRow {
    id: i64,
    timestamp: String,
    request_id: Option<String>,
    provider_id: String,
    model: String,
    trust: String,
    independent_input: i64,
    independent_output: i64,
    claimed_input: Option<i64>,
    claimed_output: Option<i64>,
    deviation_pct: Option<f64>,
    is_estimate: bool,
    verdict: String,
    expected_cost_usd: Option<f64>,
    claimed_cost_usd: Option<f64>,
}

impl UsageRow {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            request_id: row.get(2)?,
            provider_id: row.get(3)?,
            model: row.get(4)?,
            trust: row.get(5)?,
            independent_input: row.get(6)?,
            independent_output: row.get(7)?,
            claimed_input: row.get(8)?,
            claimed_output: row.get(9)?,
            deviation_pct: row.get(10)?,
            is_estimate: row.get::<_, i64>(11)? != 0,
            verdict: row.get(12)?,
            expected_cost_usd: row.get(13)?,
            claimed_cost_usd: row.get(14)?,
        })
    }
}

/// `sieve usage` 入口。必须 async（main 是 `#[tokio::main]`，本命令被 `.await`，
/// 内部 block_on 会触发嵌套 runtime panic，见 tasks/lessons.md 2026-06-18）。
pub async fn run(args: UsageArgs) -> Result<()> {
    let command = args.command.unwrap_or(UsageCommand::List {
        limit: 20,
        overbilled_only: false,
        format: None,
    });
    match command {
        UsageCommand::List {
            limit,
            overbilled_only,
            format,
        } => run_list(
            limit,
            overbilled_only,
            format.unwrap_or(OutputFormat::Jsonl),
        ),
    }
}

fn run_list(limit: u32, overbilled_only: bool, format: OutputFormat) -> Result<()> {
    let path = usage_db_path()?;
    if !path.exists() {
        eprintln!(
            "usage.db 不存在（{}）。超额计费检测默认关闭；在 config.toml 设 \
             [billing_check].enabled = true 后才会产生记录。",
            path.display()
        );
        return Ok(());
    }
    let conn = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("打开 usage.db {} 失败", path.display()))?;

    let where_clause = if overbilled_only {
        "WHERE verdict = 'overbilled'"
    } else {
        ""
    };
    let sql = format!(
        "SELECT id, timestamp, request_id, provider_id, model, trust, \
         independent_input, independent_output, claimed_input, claimed_output, \
         deviation_pct, is_estimate, verdict, expected_cost_usd, claimed_cost_usd \
         FROM usage_records {where_clause} ORDER BY id DESC LIMIT ?1"
    );
    let mut stmt = conn.prepare(&sql).context("准备 usage 查询失败")?;
    let rows = stmt
        .query_map([limit], UsageRow::from_row)
        .context("执行 usage 查询失败")?;

    let mut records: Vec<UsageRow> = Vec::new();
    for r in rows {
        records.push(r.context("读取 usage 行失败")?);
    }
    // 时间正序展示（查询取最新 N 条，输出按 id 升序）。
    records.reverse();

    match format {
        OutputFormat::Jsonl => {
            for r in &records {
                println!(
                    "{}",
                    serde_json::to_string(r).context("序列化 usage 行失败")?
                );
            }
        }
        OutputFormat::Pretty => {
            if records.is_empty() {
                println!("(无 usage 记录)");
            }
            for r in &records {
                let flag = if r.verdict == "overbilled" {
                    "⚠ OVERBILLED"
                } else {
                    "ok"
                };
                let est = if r.is_estimate { " (估算)" } else { "" };
                let dev = r
                    .deviation_pct
                    .map(|d| format!("{d:+.1}%"))
                    .unwrap_or_else(|| "—".to_string());
                println!(
                    "[{}] {} {} model={} trust={} indep={}/{}{} claim={}/{} dev={} {}",
                    r.id,
                    r.timestamp,
                    flag,
                    r.model,
                    r.trust,
                    r.independent_input,
                    r.independent_output,
                    est,
                    r.claimed_input.map(|v| v.to_string()).unwrap_or_default(),
                    r.claimed_output.map(|v| v.to_string()).unwrap_or_default(),
                    dev,
                    r.provider_id,
                );
            }
        }
    }
    Ok(())
}
