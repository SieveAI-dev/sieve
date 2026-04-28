//! 命令行解析（clap）。
//!
//! 设计约束（ADR-007）：**禁止任何 --disable-critical / --yolo flag**。
//! 安全行为（YOLO mode 拦截 / Critical 不可关）由 sieve-core / sieve-rules 强制，
//! 不暴露给 CLI。
//!
//! Week 5 新增（ADR-015 / SPEC-003）：`setup` / `doctor` / `uninstall` 子命令，
//! 仅 macOS Phase 1 支持；非 macOS 编译进友好错误 stub。
//!
//! Week 6 新增（SPEC-004 §2）：`--agent` / `--all-detected` / `--all` 多 agent 参数。

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
    /// 自动配置 AI agent 环境（仅 macOS Phase 1）。
    ///
    /// 修改 `~/.claude/settings.json`，注册 launchd plist，写审计 setup 日志。
    /// 关联：ADR-015 / SPEC-003 §setup / SPEC-004 §2。
    Setup(SetupArgs),
    /// 诊断 Sieve 安装状态（仅 macOS Phase 1）。
    ///
    /// 检查 settings.json / hook / daemon / launchd / canary 共 5 项。
    /// 关联：ADR-015 / SPEC-003 §doctor / SPEC-004 §6。
    Doctor(DoctorArgs),
    /// 干净回滚 setup 的所有改动（仅 macOS Phase 1）。
    ///
    /// 从备份目录恢复原文件，卸载 launchd plist。
    /// 关联：ADR-015 / SPEC-003 §uninstall / SPEC-004 §2.3。
    Uninstall(UninstallArgs),
}

/// 支持的 AI agent 类型（SPEC-004 §2.1）。
///
/// 传入未知值时 clap 自动报错并列出有效值（exit 2）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum AgentKind {
    /// Claude Code（Anthropic Messages API）。
    Claude,
    /// OpenClaw（OpenAI Chat Completions 协议；TBD-01 实测后完善配置注入）。
    Openclaw,
    /// Hermes（OpenAI Chat Completions 协议；TBD-02 实测后完善配置注入）。
    Hermes,
}

impl std::fmt::Display for AgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentKind::Claude => write!(f, "claude"),
            AgentKind::Openclaw => write!(f, "openclaw"),
            AgentKind::Hermes => write!(f, "hermes"),
        }
    }
}

/// `sieve setup` 参数（ADR-015 / SPEC-003 §setup / SPEC-004 §2.1）。
#[derive(clap::Args, Debug)]
pub struct SetupArgs {
    /// 指定要配置的 agent（可重复；默认 = claude）。
    ///
    /// 例：`--agent claude --agent openclaw`。
    /// 与 `--all-detected` 互斥。
    #[arg(long, value_enum, conflicts_with = "all_detected")]
    pub agent: Vec<AgentKind>,

    /// 自动检测系统已安装的所有 agent，逐个 dry-run + 用户确认（SPEC-004 §3）。
    ///
    /// 与 `--agent` 互斥。
    #[arg(long, conflicts_with = "agent")]
    pub all_detected: bool,

    /// 不实际改文件，仅打印 diff（dry-run 模式）。
    #[arg(long)]
    pub dry_run: bool,
    /// 不询问确认，直接执行（CI / 自动化用；仍打印 diff）。
    #[arg(long)]
    pub yes: bool,
}

/// `sieve doctor` 参数（SPEC-004 §2.2）。
#[derive(clap::Args, Debug, Default)]
pub struct DoctorArgs {
    /// 只检查指定 agent。不传则检查所有已通过 setup 配置的 agent。
    ///
    /// 与 `--all` 互斥。
    #[arg(long, value_enum, conflicts_with = "all")]
    pub agent: Option<AgentKind>,

    /// 检查所有 agent（等价于不传参数的默认行为，显式声明用于脚本清晰度）。
    ///
    /// 与 `--agent` 互斥。
    #[arg(long, conflicts_with = "agent")]
    pub all: bool,
}

/// `sieve uninstall` 参数（ADR-015 / SPEC-003 §uninstall / SPEC-004 §2.3）。
#[derive(clap::Args, Debug)]
pub struct UninstallArgs {
    /// 只回滚指定 agent 的改动。与 `--all` 互斥。
    ///
    /// 不传 `--agent` 且不传 `--all` 时：输出提示并 exit 2（SPEC-004 §2.3）。
    #[arg(long, value_enum, conflicts_with = "all")]
    pub agent: Option<AgentKind>,

    /// 移除所有 agent 适配（按 setup.log 逆序全部回滚）。与 `--agent` 互斥。
    #[arg(long, conflicts_with = "agent")]
    pub all: bool,

    /// 不实际改文件，仅打印将恢复的内容。
    #[arg(long)]
    pub dry_run: bool,
    /// 不询问确认，直接执行。
    #[arg(long)]
    pub yes: bool,
}
