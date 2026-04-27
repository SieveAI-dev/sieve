//! Sieve CLI 入口（关联 PRD §6.1 / ADR-001）。
//!
//! 唯一子命令：`sieve start [--config <path>] [--dry-run]` 启动 daemon；
//! `sieve version` 打印版本号。

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

mod audit;
mod cli;
mod config;
mod daemon;
mod engine_adapter;

use audit::AuditStore;
use cli::{Cli, Command};
use engine_adapter::OutboundAdapter;
use sieve_core::pipeline::outbound::OutboundFilter;
use sieve_rules::engine::VectorscanEngine;
use sieve_rules::loader::load_outbound_rules;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            config: cfg_path,
            dry_run: cli_dry_run,
        } => {
            let mut cfg = config::Config::load(&cfg_path)
                .with_context(|| format!("failed to load config from {}", cfg_path.display()))?;

            // CLI --dry-run 出现（true）时覆盖 config 中的值；
            // 不出现（false）时沿用 config.dry_run（bool OR 语义符合预期：CLI 只能追加 true）。
            if cli_dry_run {
                cfg.dry_run = true;
            }

            cfg.enforce_safety_invariants(); // bind_addr 非 127.0.0.1 → exit(1)

            let audit_path = cfg.audit_db_path()?;
            let _audit = AuditStore::init(&audit_path)
                .with_context(|| format!("init audit store at {}", audit_path.display()))?;

            // 加载出站规则（fail-closed：加载失败直接退出，不 fallback 到无规则模式，ADR-007）
            let rules_path = cfg.resolved_rules_path();
            tracing::info!(path = %rules_path.display(), "loading outbound rules");
            let rules = load_outbound_rules(&rules_path).with_context(|| {
                format!(
                    "failed to load outbound rules from {}; \
                     set rules_path in sieve.toml or ensure the default path exists",
                    rules_path.display()
                )
            })?;
            tracing::info!(count = rules.len(), "outbound rules loaded");

            // 编译 vectorscan db（fail-closed）
            let engine = VectorscanEngine::compile(rules.clone())
                .map_err(|e| anyhow::anyhow!("vectorscan compile: {e}"))?;
            let adapter = OutboundAdapter::new(Arc::new(engine), rules);

            // 加载 .sieveignore
            let sieveignore_path = cfg.resolved_sieveignore_path();
            let sieveignore = load_sieveignore(&sieveignore_path);
            tracing::info!(
                path = %sieveignore_path.display(),
                entries = sieveignore.len(),
                "sieveignore loaded"
            );

            let filter = Arc::new(OutboundFilter::new(
                Arc::new(adapter),
                Arc::new(sieveignore),
            ));

            daemon::run(cfg, filter).await?;
        }
        Command::Version => {
            println!("sieve {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

/// 从文件加载 `.sieveignore` fingerprint 白名单。
///
/// 文件不存在时静默返回空集合（正常状态）；读取失败时打印 WARN 并返回空集合。
/// 每行一个 fingerprint，支持 `#` 注释行和空行。
fn load_sieveignore(path: &Path) -> HashSet<String> {
    if !path.exists() {
        return HashSet::new();
    }
    match std::fs::read_to_string(path) {
        Ok(s) => s
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect(),
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "failed to load .sieveignore; proceeding with empty allowlist"
            );
            HashSet::new()
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_env("SIEVE_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .init();
}
