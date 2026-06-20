//! 本地 token 用量记录存储（ADR-038 决策 4 / SPEC-010 §7）。
//!
//! **隐私红线**：token 用量正是 README 发誓「从不上传」的那个 usage record。
//! 本 store **严格本地**（`~/.sieve/usage.db`，0600，append-only），**无任何出站路径**
//! ——本模块只做本地 SQLite 写入，不含任何 HTTP client。即便聚合分析也禁止上传
//! （呼应 [SPEC-006 §9.1](../../specs/SPEC-006-update-and-telemetry.md) 禁传表）。
//!
//! 结构上独立于 `audit.db`（token 用量是新数据域），但经 `request_id` 与审计 events
//! 关联，不污染审计模型。镜像 `AuditStore` 的 append-only + spawn_blocking + Mutex 范式。

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::detector::Verdict;

/// 当前 usage.db schema 版本。
const CURRENT_SCHEMA_VERSION: u32 = 1;

const CREATE_TABLE_DDL: &str = r#"
CREATE TABLE IF NOT EXISTS usage_records (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp           TEXT    NOT NULL,
    request_id          TEXT,
    provider_id         TEXT    NOT NULL DEFAULT 'unknown',
    model               TEXT    NOT NULL DEFAULT 'unknown',
    trust               TEXT    NOT NULL,        -- official | relay
    independent_input   INTEGER NOT NULL,
    independent_output  INTEGER NOT NULL,
    claimed_input       INTEGER,                 -- NULL 若无 relay 声明
    claimed_output      INTEGER,
    deviation_pct       REAL,                    -- 有符号；NULL 若未核算
    is_estimate         INTEGER NOT NULL,        -- 0/1：Anthropic 近似为 1
    verdict             TEXT    NOT NULL,        -- detector::Verdict::label()
    expected_cost_usd   REAL,                    -- 独立计数 × 官方单价；NULL 若价表缺失
    claimed_cost_usd    REAL                     -- relay 声明 × 官方单价；NULL 同上
);
"#;

/// append-only：拒绝任何 UPDATE / DELETE（与 audit.db 同款不变量）。
const APPEND_ONLY_TRIGGERS_DDL: &str = r#"
CREATE TRIGGER IF NOT EXISTS usage_no_update BEFORE UPDATE ON usage_records
BEGIN SELECT RAISE(ABORT, 'usage_records is append-only'); END;
CREATE TRIGGER IF NOT EXISTS usage_no_delete BEFORE DELETE ON usage_records
BEGIN SELECT RAISE(ABORT, 'usage_records is append-only'); END;
"#;

const INSERT_SQL: &str = r#"
INSERT INTO usage_records (
    timestamp, request_id, provider_id, model, trust,
    independent_input, independent_output, claimed_input, claimed_output,
    deviation_pct, is_estimate, verdict, expected_cost_usd, claimed_cost_usd
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
"#;

/// 一条待落库的用量记录。
#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub request_id: Option<String>,
    pub provider_id: String,
    pub model: String,
    /// "official" | "relay"
    pub trust: String,
    pub independent_input: u64,
    pub independent_output: u64,
    pub claimed_input: Option<u64>,
    pub claimed_output: Option<u64>,
    pub deviation_pct: Option<f64>,
    pub is_estimate: bool,
    pub verdict: String,
    pub expected_cost_usd: Option<f64>,
    pub claimed_cost_usd: Option<f64>,
}

/// 本地 usage 存储句柄（SQLite append-only，严格本地）。
pub struct UsageStore {
    conn: Arc<Mutex<Connection>>,
}

impl UsageStore {
    /// 初始化 usage 存储：打开 SQLite，建表（幂等），装 append-only 触发器，
    /// 收紧文件权限到 0600（隐私红线）。
    pub fn init(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建 usage 目录 {} 失败", parent.display()))?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("打开 usage 数据库 {} 失败", path.display()))?;
        conn.execute_batch(CREATE_TABLE_DDL)
            .context("创建 usage_records 表失败")?;
        conn.execute_batch(&format!("PRAGMA user_version = {CURRENT_SCHEMA_VERSION};"))?;
        conn.execute_batch(APPEND_ONLY_TRIGGERS_DDL)
            .context("安装 usage append-only 触发器失败")?;

        restrict_permissions(path);

        tracing::debug!(
            path = %path.display(),
            "usage store initialized (SQLite, schema v{CURRENT_SCHEMA_VERSION}, local-only)"
        );
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 异步写入一条用量记录（spawn_blocking + Mutex 串行化，fire-and-forget 友好）。
    pub async fn append(&self, rec: UsageRecord) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let guard = conn
                .lock()
                .map_err(|e| anyhow!("usage mutex poisoned: {e}"))?;
            let timestamp = Utc::now().to_rfc3339();
            guard.execute(
                INSERT_SQL,
                params![
                    timestamp,
                    rec.request_id,
                    rec.provider_id,
                    rec.model,
                    rec.trust,
                    rec.independent_input,
                    rec.independent_output,
                    rec.claimed_input,
                    rec.claimed_output,
                    rec.deviation_pct,
                    rec.is_estimate as i64,
                    rec.verdict,
                    rec.expected_cost_usd,
                    rec.claimed_cost_usd,
                ],
            )?;
            Ok::<(), anyhow::Error>(())
        })
        .await
        .context("spawn_blocking failed")??;
        Ok(())
    }
}

impl UsageRecord {
    /// 把检测结果折叠成一条落库记录（不含网络，纯数据组装）。
    ///
    /// `is_estimate` 由调用点的协议族决定（Anthropic = true），独立于裁决类型，
    /// 故 `NotChecked` 记录也能正确标注估算性质。
    #[allow(clippy::too_many_arguments)]
    pub fn from_verdict(
        request_id: Option<String>,
        provider_id: String,
        model: String,
        trust: crate::config::Trust,
        is_estimate: bool,
        independent_input: u64,
        independent_output: u64,
        claimed: Option<(u64, u64)>,
        verdict: &Verdict,
    ) -> Self {
        let price = super::pricing::lookup(&model);
        let expected_cost_usd = price.map(|p| p.cost_usd(independent_input, independent_output));
        let claimed_cost_usd = match (price, claimed) {
            (Some(p), Some((ci, co))) => Some(p.cost_usd(ci, co)),
            _ => None,
        };
        let deviation_pct = match verdict {
            Verdict::Ok { deviation_pct } => Some(*deviation_pct),
            Verdict::Overbilled { deviation_pct, .. } => Some(*deviation_pct),
            Verdict::NotChecked(_) => None,
        };
        Self {
            request_id,
            provider_id,
            model,
            trust: match trust {
                crate::config::Trust::Official => "official".to_string(),
                crate::config::Trust::Relay => "relay".to_string(),
            },
            independent_input,
            independent_output,
            claimed_input: claimed.map(|(i, _)| i),
            claimed_output: claimed.map(|(_, o)| o),
            deviation_pct,
            is_estimate,
            verdict: verdict.label().to_string(),
            expected_cost_usd,
            claimed_cost_usd,
        }
    }
}

/// 收紧文件权限到 0600（Unix）。非 Unix 平台 no-op。
#[cfg(unix)]
fn restrict_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o600);
        let _ = std::fs::set_permissions(path, perms);
    }
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn append_and_append_only_invariant() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("usage.db");
        let store = UsageStore::init(&path).expect("init usage store");

        let rec = UsageRecord {
            request_id: Some("req_1".into()),
            provider_id: "some-relay".into(),
            model: "claude-sonnet-4".into(),
            trust: "relay".into(),
            independent_input: 1000,
            independent_output: 2000,
            claimed_input: Some(1500),
            claimed_output: Some(3000),
            deviation_pct: Some(50.0),
            is_estimate: true,
            verdict: "overbilled".into(),
            expected_cost_usd: Some(0.033),
            claimed_cost_usd: Some(0.0495),
        };
        store.append(rec).await.expect("append");

        // append-only：直接 UPDATE / DELETE 必须被触发器拒绝。
        let conn = Connection::open(&path).unwrap();
        assert!(
            conn.execute("UPDATE usage_records SET model = 'x'", [])
                .is_err(),
            "UPDATE 必须被 append-only 触发器拒绝"
        );
        assert!(
            conn.execute("DELETE FROM usage_records", []).is_err(),
            "DELETE 必须被 append-only 触发器拒绝"
        );
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM usage_records", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn usage_db_is_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let path = dir.path().join("usage.db");
        let _store = UsageStore::init(&path).expect("init");
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "usage.db 必须 0600（隐私红线）");
    }
}
