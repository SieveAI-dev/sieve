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
use engine_adapter::{InboundAdapter, OutboundAdapter};
use sieve_core::pipeline::outbound::OutboundFilter;
use sieve_rules::engine::VectorscanEngine;
use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};

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

            // 编译出站 vectorscan db（fail-closed）
            let engine = VectorscanEngine::compile(rules.clone())
                .map_err(|e| anyhow::anyhow!("vectorscan compile: {e}"))?;
            let adapter = OutboundAdapter::new(Arc::new(engine), rules);

            // 加载 .sieveignore（出站 + 入站共用同一份）
            let sieveignore_path = cfg.resolved_sieveignore_path();
            let sieveignore = load_sieveignore(&sieveignore_path);
            tracing::info!(
                path = %sieveignore_path.display(),
                entries = sieveignore.len(),
                "sieveignore loaded"
            );
            let sieveignore_arc = Arc::new(sieveignore);

            let filter = Arc::new(OutboundFilter::new(
                Arc::new(adapter),
                Arc::clone(&sieveignore_arc),
            ));

            // 加载入站规则（fail-closed，ADR-007）
            let inbound_rules_path = cfg.resolved_inbound_rules_path();
            tracing::info!(path = %inbound_rules_path.display(), "loading inbound rules");
            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
                format!(
                    "failed to load inbound rules from {}; \
                         set inbound_rules_path in sieve.toml or ensure the default path exists",
                    inbound_rules_path.display()
                )
            })?;

            // 占位规则（pattern == "__ADDRESS_GUARD_PLACEHOLDER__"）不传 vectorscan 编译
            let (placeholder_rules, vectorscan_rules): (Vec<_>, Vec<_>) = inbound_rules_raw
                .iter()
                .cloned()
                .partition(|r| r.pattern == "__ADDRESS_GUARD_PLACEHOLDER__");
            tracing::info!(
                count = vectorscan_rules.len(),
                placeholders = placeholder_rules.len(),
                "inbound rules partitioned"
            );

            // 编译入站 vectorscan db（独立实例，fail-closed）
            let inbound_engine_vs = VectorscanEngine::compile(vectorscan_rules)
                .map_err(|e| anyhow::anyhow!("inbound vectorscan compile: {e}"))?;
            // InboundAdapter 持有全量 rule_lookup（含 placeholder，用于反查元数据）
            let inbound_adapter =
                InboundAdapter::new(Arc::new(inbound_engine_vs), inbound_rules_raw);

            // YOLO mode 运行时审计（防御性双保险）
            audit_yolo_disabled(&cfg)?;

            daemon::run(
                cfg,
                filter,
                Arc::new(inbound_adapter),
                Arc::clone(&sieveignore_arc),
            )
            .await?;
        }
        Command::Version => {
            println!("sieve {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

/// 防御性检查：确认配置中无任何试图禁用 Critical 检测的字段。
///
/// Phase 1 实现：`Config` 已用 `#[serde(deny_unknown_fields)]` 在反序列化时拒绝
/// 所有未知字段（含 `disable_critical` / `yolo` / `bypass` 等），此函数作为
/// 运行时第二道防线，仅记录审计日志。
///
/// # Errors
/// 当前实现不返回错误；签名保留 `Result<()>` 便于 Week 4 扩展检查逻辑。
fn audit_yolo_disabled(cfg: &config::Config) -> Result<()> {
    // dry_run 模式下 fail-closed 规则仍强制 Block（ADR-007 §2）
    if cfg.dry_run {
        tracing::warn!(
            "dry_run=true: non-fail-closed Critical detections will only be logged, \
             NOT blocked. Fail-closed rules (IN-CR-01/02/05/IN-GEN-01/03/OUT-01~12) \
             remain enforced regardless."
        );
    }
    tracing::info!("YOLO mode audit: passed (no critical-disable fields detected)");
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
