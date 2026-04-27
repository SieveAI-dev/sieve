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
    /// 启动透传 daemon（Week 1：纯字节透传，无规则匹配）。
    Start {
        /// config.toml 路径；文件不存在时使用内置默认值。
        #[arg(short, long, default_value = "sieve.toml")]
        config: PathBuf,
    },
    /// 打印版本号并退出。
    Version,
}
