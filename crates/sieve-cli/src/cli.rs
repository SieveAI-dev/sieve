//! 命令行解析（clap）。
//!
//! 设计约束（ADR-007）：**禁止任何 --disable-critical / --yolo flag**。
//! 安全行为（YOLO mode 拦截 / Critical 不可关）由 sieve-core / sieve-rules 强制，
//! 不暴露给 CLI。

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Sieve LLM 流量代理命令行入口（PRD §6.1）。
#[derive(Debug, Parser)]
#[command(name = "sieve", version, about = "Sieve LLM traffic proxy")]
pub struct Cli {
    /// 子命令。
    #[command(subcommand)]
    pub command: Command,
}

/// 顶层子命令枚举。
#[derive(Debug, Subcommand)]
pub enum Command {
    /// 启动 daemon（Week 2：出站规则拦截 + 透传）。
    Start {
        /// config.toml 路径；文件不存在时使用内置默认值。
        #[arg(short, long, default_value = "sieve.toml")]
        config: PathBuf,

        /// 仅记录命中，不实际拦截（覆盖 config.dry_run 为 true）。
        ///
        /// flag 出现即为 true；不出现时沿用 config.toml 中的 dry_run 值。
        /// 禁止添加 --no-dry-run 等关闭安全机制的 flag（ADR-007）。
        #[arg(long)]
        dry_run: bool,
    },
    /// 打印版本号并退出。
    Version,
}
