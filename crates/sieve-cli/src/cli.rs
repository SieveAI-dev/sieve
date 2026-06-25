//! 命令行解析（clap）。
//!
//! 设计约束：**禁止任何 --disable-critical / --yolo flag**。
//! 安全行为（YOLO mode 拦截 / Critical 不可关）由 sieve-core / sieve-rules 强制，
//! 不暴露给 CLI。
//!
//! Week 5 新增（SPEC-003）：`setup` / `doctor` / `uninstall` 子命令，
//! 仅 macOS Phase 1 支持；非 macOS 编译进友好错误 stub。
//!
//! Week 6 新增（SPEC-004 §2）：`--agent` / `--all-detected` / `--all` 多 agent 参数。
//!
//! 新增：
//! - `decisions`：headless decision CLI（TODO-4）
//! - `audit`：unix-pipeable 审计查询（TODO-5）
//! - `sieve start --no-client-policy`：无 client 在线时的兜底策略

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Sieve LLM 流量代理命令行入口。
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
        /// 禁止添加 --no-dry-run 等关闭安全机制的 flag。
        #[arg(long)]
        dry_run: bool,

        /// 无 client 接 IPC 时的兜底策略（TODO-4）。
        ///
        /// 默认 `auto-block`（保守 fail-closed）；其他选项：
        /// - `auto-warn`：标记 warn 自动放行
        /// - `hold-and-fail-closed`：等待超时后按 default_on_timeout 处置（v1.x 行为）
        #[arg(long, value_enum, default_value_t = NoClientPolicy::AutoBlock)]
        no_client_policy: NoClientPolicy,
    },
    /// 打印版本号并退出。
    Version,
    /// 自动配置 AI agent 环境（仅 macOS Phase 1）。
    ///
    /// 修改 `~/.claude/settings.json`，注册 launchd plist，写审计 setup 日志。
    /// 关联：SPEC-003 §setup / SPEC-004 §2。
    Setup(SetupArgs),
    /// 诊断 Sieve 安装状态（仅 macOS Phase 1）。
    ///
    /// 检查 settings.json / hook / daemon / launchd / canary 共 5 项。
    /// 关联：SPEC-003 §doctor / SPEC-004 §6。
    Doctor(DoctorArgs),
    /// 干净回滚 setup 的所有改动（仅 macOS Phase 1）。
    ///
    /// 从备份目录恢复原文件，卸载 launchd plist。
    /// 关联：SPEC-003 §uninstall / SPEC-004 §2.3。
    Uninstall(UninstallArgs),
    /// 用户规则管理（v2.0）。
    ///
    /// 维护 `~/.sieve/rules/user.toml`：编辑、列出、禁用、启用。
    /// daemon hot-reload 推 Week 6（v2.0 Phase A 仅 ship 文件级操作）。
    Rules(RulesArgs),

    /// 决策面 CLI（headless 工作流，TODO-4）。
    ///
    /// 在 GUI 不在线时通过 CLI 订阅 / 查看 / 解决待决策事件。
    /// CLI 跟 GUI 共用同一组 IPC 方法，不引入特权 endpoint。
    Decisions(DecisionsArgs),

    /// 审计日志查询（unix-pipeable，TODO-5）。
    ///
    /// 直接读 `~/.sieve/audit.db` SQLite，输出 jsonl 格式方便接 jq / fluentd。
    Audit(AuditArgs),

    /// 本地 token 用量与超额计费检测查询（本地用量/计费核算，可选特性）。
    ///
    /// 读 `~/.sieve/usage.db`（严格本地、永不上传），列出独立核算结果与 relay 偏差。
    #[cfg(feature = "usage")]
    Usage(UsageArgs),
}

/// `sieve usage` 参数（本地用量/计费核算，可选特性）。
#[cfg(feature = "usage")]
#[derive(clap::Args, Debug)]
pub struct UsageArgs {
    /// 子命令；省略时等价 `list`。
    #[command(subcommand)]
    pub command: Option<UsageCommand>,
}

/// `sieve usage` 子命令。
#[cfg(feature = "usage")]
#[derive(Debug, Subcommand)]
pub enum UsageCommand {
    /// 列出最近的用量记录（默认 20 条）。
    List {
        /// 显示最后 N 条（默认 20）。
        #[arg(long, default_value_t = 20)]
        limit: u32,

        /// 只显示检出超额（overbilled）的记录。
        #[arg(long)]
        overbilled_only: bool,

        /// 输出格式（jsonl 默认）。
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,
    },
}

/// `sieve rules` 参数。
#[derive(clap::Args, Debug)]
pub struct RulesArgs {
    /// 子命令。
    #[command(subcommand)]
    pub command: RulesCommand,
}

/// `sieve rules` 子命令枚举（Phase A MVP 4 个）。
#[derive(Debug, Subcommand)]
pub enum RulesCommand {
    /// 调用 `$EDITOR`（fallback vim/nano）编辑 user.toml；
    /// 关闭后 lint + atomic backup + rename + IPC reload（reload TBD Week 6）。
    Edit,
    /// 列出 user.toml 中所有规则 + 系统规则数量摘要。
    List,
    /// 禁用指定规则（在 user.toml 中加 `enabled = false`，不删除）。
    Disable {
        /// 要禁用的规则 ID。
        id: String,
    },
    /// 启用指定规则（反向操作）。
    Enable {
        /// 要启用的规则 ID。
        id: String,
    },
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
    /// Codex CLI（OpenAI；PreToolUse hook 注册在 ~/.codex/hooks.json）。
    Codex,
}

impl std::fmt::Display for AgentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentKind::Claude => write!(f, "claude"),
            AgentKind::Openclaw => write!(f, "openclaw"),
            AgentKind::Hermes => write!(f, "hermes"),
            AgentKind::Codex => write!(f, "codex"),
        }
    }
}

/// `sieve setup` 参数（SPEC-003 §setup / SPEC-004 §2.1）。
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

/// `sieve uninstall` 参数（SPEC-003 §uninstall / SPEC-004 §2.3）。
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

// ── 新增类型 ─────────────────────────────────────────────────────────

/// 无 client 接 IPC 时的兜底策略（TODO-4）。
///
/// daemon 在 IPC server 没有 client 连接时（或 decision request 超时无人响应）按此策略走。
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum NoClientPolicy {
    /// 无 client 在线时直接 block（最保守，fail-closed；默认值）。
    AutoBlock,
    /// 无 client 在线时标记 warn 放行（低风险 headless 场景）。
    AutoWarn,
    /// 等待超时后按 default_on_timeout 处置（等价于 v1.x 行为）。
    HoldAndFailClosed,
}

/// `sieve decisions` 参数（TODO-4）。
#[derive(clap::Args, Debug)]
pub struct DecisionsArgs {
    /// 子命令。
    #[command(subcommand)]
    pub command: DecisionsCommand,
}

/// `sieve decisions` 子命令枚举。
#[derive(Debug, Subcommand)]
pub enum DecisionsCommand {
    /// 流式订阅 pending decision 事件（每行一个 JSON object，jsonl 格式）。
    Watch {
        /// 输出 jsonl 格式（默认开启）。
        #[arg(long)]
        format_jsonl: bool,

        /// 按 severity 过滤（critical / high / medium / low）。
        #[arg(long, value_enum)]
        severity: Option<Severity>,

        /// 按 listener 上游 provider-id 过滤。
        #[arg(long)]
        provider_id: Option<String>,
    },

    /// 查询单个 pending decision 的完整上下文。
    Show {
        /// Decision request UUID。
        id: String,
    },

    /// 解决一个 pending decision（批准 / 拒绝 / warn）。
    Resolve {
        /// Decision request UUID。
        id: String,

        /// 批准（Allow）。与 --block / --warn 互斥。
        #[arg(long, conflicts_with_all = &["block", "warn"])]
        approve: bool,

        /// 拒绝（Block / Deny）。与 --approve / --warn 互斥。
        #[arg(long, conflicts_with_all = &["approve", "warn"])]
        block: bool,

        /// 标记 warn 放行。与 --approve / --block 互斥。
        #[arg(long, conflicts_with_all = &["approve", "block"])]
        warn: bool,

        /// 决策理由（可选，写入 audit）。
        #[arg(long)]
        reason: Option<String>,
    },
}

/// severity 枚举（共用于 decisions watch + audit query）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

/// 输出格式枚举（共用于 audit tail / query）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Jsonl,
    Pretty,
}

/// `sieve audit` 参数（TODO-5）。
#[derive(clap::Args, Debug)]
pub struct AuditArgs {
    /// 子命令。
    #[command(subcommand)]
    pub command: AuditCommand,
}

/// `sieve audit` 子命令枚举。
#[derive(Debug, Subcommand)]
pub enum AuditCommand {
    /// 显示最后 N 条审计事件，支持 --follow 流式跟踪。
    Tail {
        /// 流式跟踪新事件（500ms 轮询 SQLite）。
        #[arg(short = 'f', long)]
        follow: bool,

        /// 输出格式（jsonl 默认）。
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,

        /// 显示最后 N 条（默认 20）。
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },

    /// 按条件查询审计事件。
    Query {
        /// 时间范围（如 "1h" / "24h" / "7d"）。
        #[arg(long)]
        since: Option<String>,

        /// 按 severity 过滤。
        #[arg(long, value_enum)]
        severity: Option<Severity>,

        /// 按 rule_id 过滤。
        #[arg(long)]
        rule_id: Option<String>,

        /// 按 listener 上游 provider_id 过滤（v3 schema 新列）。
        #[arg(long)]
        provider_id: Option<String>,

        /// 输出格式（jsonl 默认）。
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,
    },

    /// 显示单条审计事件完整内容。
    Show {
        /// 审计事件 id（INTEGER PRIMARY KEY）。
        id: i64,
    },

    /// 生成 full 档加密审计的 age 密钥对（加密审计档案，可选特性）。
    ///
    /// 公钥（recipient）打印到 stdout，由用户粘贴进 `config.toml [audit].recipient`；
    /// 口令保护后的私钥写 0600 文件，**用户须移出本机**（密码管理器/离线介质）。
    /// 口令经环境变量 `SIEVE_AUDIT_PASSPHRASE` 提供（不回显）。
    /// **口令丢失 = 归档永久不可读（by design）。**
    #[cfg(feature = "audit-crypto")]
    Keygen {
        /// 口令保护的私钥输出路径（默认 `~/.sieve/audit-identity.age`）。
        #[arg(long)]
        out: Option<PathBuf>,

        /// 覆盖已存在的私钥文件（默认拒绝，防误删旧归档的解密能力）。
        #[arg(long)]
        force: bool,
    },

    /// 轮换 full 档密钥对（生成新对；**旧归档段仍需对应的旧私钥解密**）。
    #[cfg(feature = "audit-crypto")]
    RotateKey {
        /// 新私钥输出路径（默认 `~/.sieve/audit-identity-rotated.age`）。
        #[arg(long)]
        out: Option<PathBuf>,
    },

    /// 解密并校验 full 档归档段（审计用，**应在离线/另一台机器执行**）。
    ///
    /// 用口令（`SIEVE_AUDIT_PASSPHRASE`）解锁私钥，逐条校验哈希链 + 解密，输出脱敏后内容。
    #[cfg(feature = "audit-crypto")]
    Decrypt {
        /// 口令保护的私钥文件路径（keygen 产出）。
        #[arg(long)]
        identity: PathBuf,

        /// 要解密的归档段文件（`archive-*.jsonl`）。
        segment: PathBuf,
    },
}
