//! 命令行解析（clap）。
//!
//! 设计约束（ADR-007）：**禁止任何 --disable-critical / --yolo flag**。
//! 安全行为（YOLO mode 拦截 / Critical 不可关）由 sieve-core / sieve-rules 强制，
//! 不暴露给 CLI。
//!
//! Week 5 新增（ADR-015 / SPEC-003）：`setup` / `doctor` / `uninstall` 子命令，
//! 仅 macOS Phase 1 支持；非 macOS 编译进友好错误 stub。

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
    /// 自动配置 Claude Code 环境（仅 macOS Phase 1）。
    ///
    /// 修改 `~/.claude/settings.json`，注册 launchd plist，写审计 setup 日志。
    /// 关联：ADR-015 / SPEC-003 §setup。
    Setup(SetupArgs),
    /// 诊断 Sieve 安装状态（仅 macOS Phase 1）。
    ///
    /// 检查 settings.json / hook / daemon / launchd / canary 共 5 项。
    /// 关联：ADR-015 / SPEC-003 §doctor。
    Doctor,
    /// 干净回滚 setup 的所有改动（仅 macOS Phase 1）。
    ///
    /// 从备份目录恢复原文件，卸载 launchd plist。
    /// 关联：ADR-015 / SPEC-003 §uninstall。
    Uninstall(UninstallArgs),
}

/// `sieve setup` 参数（ADR-015 / SPEC-003 §setup）。
#[derive(clap::Args, Debug)]
pub struct SetupArgs {
    /// 不实际改文件，仅打印 diff（dry-run 模式）。
    #[arg(long)]
    pub dry_run: bool,
    /// 不询问确认，直接执行（CI / 自动化用；仍打印 diff）。
    #[arg(long)]
    pub yes: bool,
}

/// `sieve uninstall` 参数（ADR-015 / SPEC-003 §uninstall）。
#[derive(clap::Args, Debug)]
pub struct UninstallArgs {
    /// 不实际改文件，仅打印将恢复的内容。
    #[arg(long)]
    pub dry_run: bool,
    /// 不询问确认，直接执行。
    #[arg(long)]
    pub yes: bool,
}
