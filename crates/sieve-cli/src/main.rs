//! Sieve CLI 入口（关联 PRD §6.1 / ADR-001）。
//!
//! 唯一子命令：`sieve start --config <path>` 启动透传 daemon；
//! `sieve version` 打印版本号。

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use clap::Parser;

mod audit;
mod cli;
mod config;
mod daemon;

use audit::AuditStore;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    match cli.command {
        Command::Start { config: cfg_path } => {
            let cfg = config::Config::load(&cfg_path)
                .with_context(|| format!("failed to load config from {}", cfg_path.display()))?;
            cfg.enforce_safety_invariants(); // bind_addr 非 127.0.0.1 → exit(1)
            let audit_path = cfg.audit_db_path()?;
            let _audit = AuditStore::init(&audit_path)
                .with_context(|| format!("init audit store at {}", audit_path.display()))?;
            daemon::run(cfg).await?;
        }
        Command::Version => {
            println!("sieve {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_env("SIEVE_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .init();
}
