//! 配置加载（关联 docs/design/data-model.md §配置）。
//!
//! Phase 1 字段：`upstream_url` / `port` / `bind_addr` / `log_path` /
//! `tls_verify_upstream`。
//! Week 2 新增：`rules_path` / `sieveignore_path` / `dry_run`。
//! Week 3 新增：`inbound_rules_path`（入站规则路径）。
//! Week 5 新增：`ipc_socket_path` / `pending_dir` / `decisions_dir` /
//!              `preset` / `launchd_plist_path` / `gui_socket_enabled` /
//!              `audit_db_path`（SPEC-003 / data-model.md §5）。
//! `#[serde(deny_unknown_fields)]` 确保配置文件中的危险字段（如
//! `disable_critical`）被强制拒绝，不会静默忽略。

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 检测预设级别（SPEC-003 / data-model.md §5）。
///
/// - `Strict`：所有规则最高灵敏度
/// - `Standard`：推荐平衡配置（默认；v1 旧值 `default` 在 v2 重命名，SPEC-005 §5.6）
/// - `Relaxed`：降低误报，适合受信任环境
/// - `Custom`：完全自定义（忽略内置默认值）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    Strict,
    /// 推荐平衡配置（默认）。SPEC-005 §5.6：v1 旧值 `default` 在 v2 重命名为 `standard`；
    /// `alias = "default"` 兼容旧 sieve.toml，wire 输出统一 `standard`（与 GUI Preset enum 对齐）。
    #[default]
    #[serde(alias = "default")]
    Standard,
    Relaxed,
    Custom,
}

/// 上游协议（ADR-018 / ADR-026 §决策 4）。
///
/// `Auto`（默认，未显式声明）：daemon 按请求 path 自适应路由，**不做协议错位拒绝**。
/// legacy `upstream_url` 与省略 `protocol` 字段的 `[[upstream]]` 均映射为此态，保留 v1.x
/// 单 upstream 双协议能力（ADR-026 §决策 1 向后兼容 + PRD §9 #16/#9 双协议硬约束）。
///
/// `Anthropic` / `Openai`（显式声明）：daemon 严格校验请求 path——`Anthropic` listener 仅
/// 接受 `/v1/messages`，`Openai` listener 仅接受 `/v1/chat/completions`。错位请求（如
/// Anthropic listener 收到 `/v1/chat/completions`）→ daemon fail-closed 400 + sieve_blocked
/// event 注入（PRD §9 #3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    /// 协议未显式声明——按请求 path 自适应，不强制错位（向后兼容默认值）。
    #[default]
    Auto,
    Anthropic,
    Openai,
}

/// 配置化 endpoint 路由规则（A3）：单条 `path → provider` 映射。
///
/// `[[upstream.routes]]` 表项。`provider` 只接受 `anthropic` / `openai`（`auto` 无意义，
/// 由 [`Config::enforce_safety_invariants`] 拒绝）。让「OpenAI 兼容但路径不同的中转站」
/// 只需加一行配置即可路由到对应 codec，不改 daemon 代码（参见 `resolve_endpoint_route`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RouteRule {
    /// 请求 path 精确匹配（如 `/custom/v1/chat`）。
    pub path: String,
    /// 命中后路由到的出站 provider（`anthropic` / `openai`）。
    pub provider: Protocol,
}

/// 上游信任级别（ADR-038 超额计费检测）。
///
/// - `Official`：官方直连（`api.anthropic.com` / `api.openai.com`）。relay 无法插手，
///   上游回报的 `usage` **权威**，直接采纳，不必独立核算。
/// - `Relay`：经第三方中转站。`usage` 是 relay 完全控制的响应体里的一段 JSON，视为
///   **未经验证的声明**，开启 `[billing_check]` 时须独立核算 + 交叉比对。
///
/// 每个 [`UpstreamListener`] 未显式声明 `trust` 时由
/// [`UpstreamListener::resolved_trust`] 按 URL host 派生；**无法判定时保守归为
/// `Relay`**（把可信当不可信只多算一次，把不可信当可信会漏掉欺诈）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trust {
    Official,
    Relay,
}

/// 内置官方 host 白名单（ADR-038 决策 1，「可配置扩展」MVP 硬编码两 host）。
/// daemon trust 透传接通前仅被 `resolved_trust` + 单测消费，保留 allow 直到 Inc2 落地。
#[allow(dead_code)]
const OFFICIAL_HOSTS: &[&str] = &["api.anthropic.com", "api.openai.com"];

/// 单 listener 上游配置（ADR-026 §决策 1）。
///
/// 一个 sieve daemon 可同时绑定多个 port，每个 port 对应一个上游 LLM endpoint。
/// 哑 client（Claude Code、Codex CLI、Cursor 等仅认 single base_url 的 agent）
/// 无法注入路由 header，靠不同 port 区分上游。
///
/// `bind_addr` 不在本结构内——所有 listener 共享 [`Config::bind_addr`]，
/// 强制 `127.0.0.1`（PRD §9 #2 完全本地）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpstreamListener {
    /// 监听端口。同一 daemon 内必须唯一（端口冲突在
    /// [`Config::enforce_safety_invariants`] 中检查）。
    pub port: u16,

    /// 真实上游 URL（含 path 前缀，如 `https://api.deepseek.com/anthropic`）。
    /// path 前缀的拼接由 [`sieve_core::Forwarder`] 处理（ADR-026 TODO-1）。
    pub url: String,

    /// 上游身份标识，用于审计 / 日志 / IPC 事件标注。
    /// 留空时由 [`UpstreamListener::resolved_provider_id`] 从 URL host 派生。
    #[serde(default)]
    pub provider_id: String,

    /// 上游协议（默认 `auto`：按 path 自适应、不强制错位，向后兼容；ADR-026）。
    #[serde(default)]
    pub protocol: Protocol,

    /// 配置化 endpoint 路由表（A3）：把非标准请求 path 显式映射到出站 provider。
    /// 空表（默认）时按「标准路径 → provider，其余 → listener `protocol`」解析，行为不变。
    /// 接「协议兼容但路径不同的中转站」只需加一行 `{ path = "/自定义/路径", provider = "openai" }`，
    /// 零代码改动。
    #[serde(default)]
    pub routes: Vec<RouteRule>,

    /// 上游信任级别（ADR-038）。`None`（未显式声明）时由
    /// [`UpstreamListener::resolved_trust`] 按 URL host 派生：官方 host → `Official`，
    /// 其余（含无法解析）→ `Relay`（保守 fail-closed）。仅 `Relay` 上游在开启
    /// `[billing_check]` 时参与独立 token 核算。
    #[serde(default)]
    pub trust: Option<Trust>,

    /// 该 listener 专属上游代理 URL（覆盖全局 [`Config::proxy`]）。
    /// 形如 `socks5://127.0.0.1:6153` / `http://127.0.0.1:6152`（SPEC-007）。
    #[serde(default)]
    pub proxy: Option<String>,

    /// 显式直连，无视全局 proxy 与 env（优先级最高）。
    #[serde(default)]
    pub no_proxy: bool,
}

impl UpstreamListener {
    /// 规范化 provider_id：显式给定时直接用，否则从 URL host 派生（去掉 port 后缀）。
    ///
    /// daemon.rs 在 Stage B/C 接通 multi-listener 后会消费此方法，暂时仅在
    /// 单元测试中使用——保留 `#[allow(dead_code)]` 直到 Stage B 落地。
    #[allow(dead_code)]
    pub fn resolved_provider_id(&self) -> String {
        if !self.provider_id.is_empty() {
            return self.provider_id.clone();
        }
        match http::Uri::try_from(&self.url) {
            Ok(uri) => uri
                .authority()
                .map(|a| a.host().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            Err(_) => "unknown".to_string(),
        }
    }

    /// 规范化信任级别（ADR-038）：显式 `trust` 优先；否则按 URL host 派生——
    /// 命中 [`OFFICIAL_HOSTS`] → [`Trust::Official`]，其余或无法解析 → [`Trust::Relay`]
    /// （保守 fail-closed：宁可对官方多算一次，绝不把 relay 误当可信而漏掉欺诈）。
    ///
    /// daemon trust 透传接通前仅被单测消费，保留 `#[allow(dead_code)]` 直到 Inc2 落地
    /// （与 [`UpstreamListener::resolved_provider_id`] 同款 Stage 渐进式接通）。
    #[allow(dead_code)]
    pub fn resolved_trust(&self) -> Trust {
        if let Some(t) = self.trust {
            return t;
        }
        match http::Uri::try_from(&self.url) {
            Ok(uri) => match uri.authority().map(|a| a.host().to_string()) {
                Some(host) if OFFICIAL_HOSTS.contains(&host.as_str()) => Trust::Official,
                _ => Trust::Relay,
            },
            Err(_) => Trust::Relay,
        }
    }
}

/// Update / telemetry-beacon configuration (ADR-030 §5).
///
/// Corresponds to the `[update]` section in `sieve.toml`.
/// All fields have defaults so the section is entirely optional
/// (existing configs without `[update]` continue to work).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct UpdateConfig {
    /// Enable the update check + telemetry beacon (default `true`).
    /// Can also be disabled via `SIEVE_NO_UPDATE` env var.
    pub enabled: bool,
    /// Enable install-ID telemetry in manifest requests (default `true`).
    /// Can also be disabled via `SIEVE_NO_TELEMETRY` env var.
    pub telemetry: bool,
    /// Override manifest URL (default `None` → uses sieve-updater built-in default).
    /// Can also be overridden via `SIEVE_UPDATE_URL` env var.
    pub url: Option<String>,
    /// How often to check for updates, in hours (default 6).
    pub check_interval_hours: u64,
    /// Release channel to report in manifest requests (default `"stable"`).
    pub channel: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            telemetry: true,
            url: None,
            check_interval_hours: 6,
            channel: "stable".to_string(),
        }
    }
}

/// 审计日志档位（ADR-037 决策 1）。
///
/// - `Off`：什么都不留。
/// - `Metadata`（默认）：现状——只写 `audit.db` 元数据（fingerprint + 最小元信息），
///   **零行为变化**。命名刻意避开既有术语 `decisions`（`~/.sieve/decisions/` 灰名单 +
///   `sieve decisions` CLI），见 ADR-037。
/// - `Full`（opt-in）：在 `Metadata` 基础上额外把**脱敏后**内容加密归档（write-only
///   logging + 哈希链 + 保留期）。默认关闭。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuditLevel {
    Off,
    #[default]
    Metadata,
    Full,
}

/// `full` 档归档段切分粒度（ADR-037 决策 5：保留期按段清理）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveRotation {
    #[default]
    Daily,
    Weekly,
    Monthly,
}

/// 加密审计日志配置（ADR-037 / SPEC-009）。对应 `[audit]` 段。
///
/// 整段可选；省略时 `level = metadata`（= 现状，零行为变化）。`full` 档的密钥
/// 管理见 `sieve audit keygen`，daemon **只持公钥 `recipient`**，无解密能力。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct AuditConfig {
    /// 日志档位（默认 `metadata`）。
    pub level: AuditLevel,
    /// `full` 档加密归档的 age recipient 公钥（`age1...`）。
    /// `level = full` 时**必填**（[`Config::check_safety_invariants`] fail-fast 校验）；
    /// daemon 只持此公钥，私钥离线（ADR-037 决策 3 write-only logging）。
    pub recipient: Option<String>,
    /// 归档段目录（`None` → `~/.sieve/audit-archive/`）。
    pub archive_dir: Option<PathBuf>,
    /// 保留期天数；超期段整段删除（ADR-037 决策 5）。`0` = 永久保留。
    pub retention_days: u32,
    /// 是否对归档记录加哈希链防篡改（ADR-037 决策 4，默认开）。
    pub hash_chain: bool,
    /// 归档段切分粒度（默认 `daily`）。
    pub rotation: ArchiveRotation,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            level: AuditLevel::Metadata,
            recipient: None,
            archive_dir: None,
            retention_days: 30,
            hash_chain: true,
            rotation: ArchiveRotation::Daily,
        }
    }
}

/// 超额计费检测配置（ADR-038 / SPEC-010）。对应 `[billing_check]` 段。
///
/// 整段可选；**默认全关**（`enabled = false`），不开启则零行为变化、零新增出站、
/// 零计算开销。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BillingCheckConfig {
    /// 是否启用超额计费检测（默认 `false`）。仅对 `Relay` 上游生效。
    pub enabled: bool,
    /// 偏差容差百分比（默认 `15.0`）。独立计数与 relay 声明偏差超此值则报警。
    /// 远高于 tokenizer 噪声 ±5~10%、远低于欺诈量级 +50%（ADR-038 决策 2）。
    pub tolerance_pct: f64,
    /// 是否允许调官方 `count_tokens` 直连（**默认 `false`**，ADR-038 决策 5 路径 C）。
    /// 唯一可能触发 Sieve 主动出站的开关；仅用户显式开启才生效，且 UI 须显著警示
    /// 「会向官方 endpoint 发起一次主动出站」。默认姿态下 PRD §9 #2 一字不破。
    pub count_tokens_optin: bool,
}

impl Default for BillingCheckConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tolerance_pct: 15.0,
            count_tokens_optin: false,
        }
    }
}

/// Sieve 顶层配置。
///
/// 对应 `sieve.toml`（ADR-003 / data-model.md §配置）。
/// 文件不存在时 [`Config::load`] 返回 [`Config::default`]。
///
/// **多 listener 配置（ADR-026）**：
/// - 推荐用 `[[upstream]]` 数组列出每个 listener（含 port / url / provider_id / protocol）
/// - 旧字段 `upstream_url` + `port` 仍可用，等价于单元素 upstreams
///   （[`Config::resolved_upstreams`] 自动映射）
/// - 同时给两套配置时 `upstreams` 优先，旧字段被忽略并打 WARN
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// 上游 LLM API 端点（**已废弃**，保留向后兼容）。
    ///
    /// 仅当 `upstreams` 为空时生效，等价于 `[[upstream]] { port, url = upstream_url,
    /// provider_id = "anthropic", protocol = anthropic }`。
    /// 推荐用 `upstreams` 数组配置 multi-listener（ADR-026）。
    #[serde(default = "default_upstream")]
    pub upstream_url: String,

    /// 本地代理监听端口（**已废弃**，保留向后兼容；见 `upstream_url`）。
    /// 默认 11453（PRD §6.1）。
    #[serde(default = "default_port")]
    pub port: u16,

    /// 多 listener 上游配置（ADR-026）。
    ///
    /// 非空时优先于 `upstream_url` + `port`。每个 listener 独立 bind 一个端口，
    /// 各自连接不同的真实上游。哑 client（Claude Code 等）通过指向不同 port
    /// 切换上游，无须注入路由 header。
    ///
    /// `bind_addr` 不在本字段内——所有 listener 共享 [`Config::bind_addr`]
    /// （强制 `127.0.0.1`）。
    #[serde(default, rename = "upstream")]
    pub upstreams: Vec<UpstreamListener>,

    /// 全局兜底上游代理 URL（可选）。每个 upstream 未设 proxy 且未 no_proxy 时继承。
    /// 也可由 env `ALL_PROXY` / `HTTPS_PROXY` 兜底（config 优先）。SPEC-007。
    #[serde(default)]
    pub proxy: Option<String>,

    /// 监听地址。**强制 `127.0.0.1`**（ADR-003 / PRD §9 #2 完全本地）。
    /// 任何其他值都会触发 [`Config::enforce_safety_invariants`] 中的 exit(1)。
    #[serde(default = "default_bind_addr")]
    pub bind_addr: String,

    /// 审计日志路径（SQLite），`None` 时由 daemon 决定默认路径。
    #[serde(default)]
    pub log_path: Option<PathBuf>,

    /// 是否校验上游 TLS 证书（默认 `true`；测试可关，会打印 WARN）。
    #[serde(default = "default_tls_verify")]
    pub tls_verify_upstream: bool,

    /// 出站规则 toml 路径（Week 2，默认 `crates/sieve-rules/rules/outbound.toml`）。
    #[serde(default)]
    pub rules_path: Option<PathBuf>,

    /// `.sieveignore` 路径（默认 `~/.sieve/sieveignore`）。
    #[serde(default)]
    pub sieveignore_path: Option<PathBuf>,

    /// 仅记录命中，不实际拦截（dry-run 模式，默认 `false`）。
    ///
    /// `true` 时 [`Config::enforce_safety_invariants`] 会打印 WARN。
    /// CLI `--dry-run` flag 出现时会覆盖此值为 `true`（见 cli.rs）。
    #[serde(default)]
    pub dry_run: bool,

    /// 入站规则 toml 路径（Week 3，默认 `crates/sieve-rules/rules/inbound.toml`）。
    #[serde(default)]
    pub inbound_rules_path: Option<PathBuf>,

    // ── Week 5 新字段（SPEC-003 / data-model.md §5）────────────────────────
    // Week 6+ 会在 daemon 启动时读取这些字段；当前仅反序列化使用，暂时 allow dead_code。
    /// Unix socket 路径（GUI / sieve-hook 连接用，默认 `~/.sieve/ipc.sock`）。
    #[serde(default = "default_ipc_socket")]
    #[allow(dead_code)]
    pub ipc_socket_path: PathBuf,

    /// 待决策文件目录（默认 `~/.sieve/pending/`）。
    #[serde(default = "default_pending_dir")]
    #[allow(dead_code)]
    pub pending_dir: PathBuf,

    /// 决策文件目录（默认 `~/.sieve/decisions/`）。
    #[serde(default = "default_decisions_dir")]
    #[allow(dead_code)]
    pub decisions_dir: PathBuf,

    /// 检测预设级别（默认 `Default`）。
    #[serde(default)]
    #[allow(dead_code)]
    pub preset: Preset,

    /// launchd plist 路径（macOS，默认 `~/Library/LaunchAgents/com.sieve.daemon.plist`）。
    #[serde(default = "default_launchd_plist")]
    #[allow(dead_code)]
    pub launchd_plist_path: PathBuf,

    /// 是否启用 GUI Unix socket（默认 `false`；Week 6+ 启用）。
    #[serde(default = "default_gui_socket_enabled")]
    #[allow(dead_code)]
    pub gui_socket_enabled: bool,

    /// SQLite 审计数据库路径（Week 5；`None` 时沿用 `log_path` 或 `~/.sieve/audit.db`）。
    #[serde(default)]
    pub audit_db_path: Option<PathBuf>,

    /// Update check + telemetry beacon settings (ADR-030 §5).
    ///
    /// The `[update]` section is optional; omitting it preserves all defaults
    /// (`enabled = true`, `telemetry = true`, `check_interval_hours = 6`,
    /// `channel = "stable"`).
    #[serde(default)]
    pub update: UpdateConfig,

    /// 加密审计日志配置（ADR-037）。`[audit]` 段可选；省略时 `level = metadata`
    /// （现状，零行为变化）。
    #[serde(default)]
    pub audit: AuditConfig,

    /// 超额计费检测配置（ADR-038）。`[billing_check]` 段可选；默认全关。
    #[serde(default)]
    pub billing_check: BillingCheckConfig,
}

fn home_path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// 优先级：`$SIEVE_HOME` env var > `$HOME/.sieve`。与 `sieve_ipc::paths::sieve_home`
/// 对齐，使 daemon / IPC / 审计 DB / sieveignore 共享同一根目录。
fn sieve_home() -> PathBuf {
    if let Some(p) = std::env::var_os("SIEVE_HOME") {
        return PathBuf::from(p);
    }
    home_path().join(".sieve")
}

fn default_ipc_socket() -> PathBuf {
    sieve_home().join("ipc.sock")
}

fn default_pending_dir() -> PathBuf {
    sieve_home().join("pending")
}

fn default_decisions_dir() -> PathBuf {
    sieve_home().join("decisions")
}

fn default_launchd_plist() -> PathBuf {
    home_path()
        .join("Library")
        .join("LaunchAgents")
        .join("com.sieve.daemon.plist")
}

fn default_gui_socket_enabled() -> bool {
    false
}

fn default_upstream() -> String {
    "https://api.anthropic.com".to_string()
}

fn default_port() -> u16 {
    11453
}

fn default_bind_addr() -> String {
    "127.0.0.1".to_string()
}

fn default_tls_verify() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            upstream_url: default_upstream(),
            port: default_port(),
            upstreams: Vec::new(),
            proxy: None,
            bind_addr: default_bind_addr(),
            log_path: None,
            tls_verify_upstream: default_tls_verify(),
            rules_path: None,
            sieveignore_path: None,
            dry_run: false,
            inbound_rules_path: None,
            ipc_socket_path: default_ipc_socket(),
            pending_dir: default_pending_dir(),
            decisions_dir: default_decisions_dir(),
            preset: Preset::default(),
            launchd_plist_path: default_launchd_plist(),
            gui_socket_enabled: default_gui_socket_enabled(),
            audit_db_path: None,
            update: UpdateConfig::default(),
            audit: AuditConfig::default(),
            billing_check: BillingCheckConfig::default(),
        }
    }
}

impl Config {
    /// 从 TOML 文件加载配置；文件不存在时返回 [`Config::default`]。
    ///
    /// # Errors
    /// 文件存在但读取或解析失败时返回错误。
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            tracing::warn!(path = %path.display(), "config file not found, using defaults");
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("read config {}", path.display()))?;
        let cfg: Self =
            toml::from_str(&s).with_context(|| format!("parse config {}", path.display()))?;
        Ok(cfg)
    }

    /// 检查安全不变量（纯函数，可单测）：返回首个违规的 FATAL 描述，无违规返回 `Ok(())`。
    ///
    /// 关联 ADR-003 / PRD §9 #2 / ADR-026 §决策 1。供 [`Self::enforce_safety_invariants`]
    /// 包装成 exit-on-fail 行为；测试代码直接调本方法验证检测逻辑。
    pub fn check_safety_invariants(&self) -> Result<(), String> {
        if self.bind_addr != "127.0.0.1" {
            return Err(format!(
                "bind_addr must be 127.0.0.1 (got {:?}). \
                 Sieve refuses to bind on a non-loopback address. See ADR-003.",
                self.bind_addr
            ));
        }

        // ADR-026: 端口冲突检查（针对 multi-listener 配置）。
        let upstreams = self.resolved_upstreams();
        let mut seen_ports = std::collections::HashSet::new();
        for u in &upstreams {
            if !seen_ports.insert(u.port) {
                return Err(format!(
                    "duplicate listener port {} in `upstreams`. \
                     Each listener must bind a unique port. See ADR-026.",
                    u.port
                ));
            }
            // A3: routes 表的 provider 必须是确定 provider（anthropic/openai）；
            // auto 无法决定 codec，配了等于无效路由——fail-fast 拒绝而非静默忽略。
            for r in &u.routes {
                if r.provider == Protocol::Auto {
                    return Err(format!(
                        "listener port {} route path {:?} has provider = \"auto\"; \
                         routes must name a concrete provider (anthropic/openai). See A3.",
                        u.port, r.path
                    ));
                }
            }
        }

        // ADR-037: `full` 档必须配置 age recipient 公钥，否则归档无处加密。
        // daemon 只持公钥（write-only logging），缺它则无法初始化 ArchiveWriter。
        if self.audit.level == AuditLevel::Full {
            match self.audit.recipient.as_deref() {
                None | Some("") => {
                    return Err(
                        "audit.level = \"full\" requires `audit.recipient` (an age public \
                         key, e.g. age1...). Run `sieve audit keygen` first. See ADR-037."
                            .to_string(),
                    );
                }
                Some(r) if !r.starts_with("age1") => {
                    return Err(format!(
                        "audit.recipient must be an age public key starting with `age1` \
                         (got {r:?}). See ADR-037."
                    ));
                }
                _ => {}
            }
        }

        // ADR-038: 容差必须是 (0, 100] 的正数百分比，否则比对无意义。
        if self.billing_check.enabled {
            let tol = self.billing_check.tolerance_pct;
            if !(tol > 0.0 && tol <= 100.0) {
                return Err(format!(
                    "billing_check.tolerance_pct must be in (0, 100] (got {tol}). See ADR-038."
                ));
            }
        }

        Ok(())
    }

    /// 强制安全不变量：违规时打印 FATAL 并 `exit(1)`，正常时打印降级 WARN。
    ///
    /// 关联 ADR-003 / PRD §9 #2 / data-model.md §配置 / ADR-026 §决策 1。
    /// 配置错误悄悄启动会暴露代理到局域网或半启动状态，违反"完全本地"承诺。
    pub fn enforce_safety_invariants(&self) {
        if let Err(msg) = self.check_safety_invariants() {
            eprintln!("FATAL: {msg}");
            std::process::exit(1);
        }

        // 同时给 upstreams + 旧字段（非默认值）→ WARN（不强制 error，
        // 旧字段已被 resolved_upstreams 忽略，仅提示用户清理）。
        if !self.upstreams.is_empty() {
            let upstream_explicit = self.upstream_url != default_upstream();
            let port_explicit = self.port != default_port();
            if upstream_explicit || port_explicit {
                tracing::warn!(
                    "config: both `upstreams` and legacy `upstream_url`/`port` are set; \
                     `upstreams` takes precedence, legacy fields ignored. \
                     Remove the legacy fields to silence this warning."
                );
            }
        }

        if !self.tls_verify_upstream {
            tracing::warn!(
                "tls_verify_upstream=false: upstream TLS certificate NOT verified. \
                 Only use in controlled test environments."
            );
        }

        if self.dry_run {
            tracing::warn!("dry_run mode: detections logged but not blocked");
        }
    }

    /// 返回规范化的 listener 列表（ADR-026）。
    ///
    /// - `upstreams` 非空时直接克隆返回
    /// - `upstreams` 为空时从旧 `upstream_url` + `port` 字段映射成单元素 vec
    ///   （向后兼容路径，provider_id = `"anthropic"`，protocol = `Auto`：按请求
    ///   path 自适应，不强制协议错位，保留 v1.x 单 upstream 双协议能力）
    ///
    /// daemon 启动后所有 listener 创建逻辑应走此方法，不直接读
    /// [`Config::upstream_url`] / [`Config::port`]。
    pub fn resolved_upstreams(&self) -> Vec<UpstreamListener> {
        if !self.upstreams.is_empty() {
            return self.upstreams.clone();
        }
        vec![UpstreamListener {
            port: self.port,
            url: self.upstream_url.clone(),
            provider_id: "anthropic".to_string(),
            // legacy 单 upstream 未声明协议 → Auto：按 path 自适应，不强制错位
            // （ADR-026 §决策 1 向后兼容 + PRD §9 #16/#9 双协议）。
            protocol: Protocol::Auto,
            // legacy 单 upstream 无配置化路由表（A3）。
            routes: Vec::new(),
            // trust 留 None → resolved_trust() 按实际 url host 派生（默认 upstream_url
            // 指向 api.anthropic.com → Official；用户改成 relay 则派生 Relay，ADR-038）。
            trust: None,
            proxy: None,
            no_proxy: false,
        }]
    }

    /// 计算某 upstream 的有效代理 URL（SPEC-007 §2 优先级链）。
    /// no_proxy > upstream.proxy > 全局 proxy > env(HTTPS_PROXY/ALL_PROXY) > 直连(None)。
    pub fn effective_proxy(&self, up: &UpstreamListener) -> Option<String> {
        if up.no_proxy {
            return None;
        }
        if let Some(p) = up.proxy.as_ref().filter(|s| !s.is_empty()) {
            return Some(p.clone());
        }
        self.global_proxy()
    }

    /// 全局代理：`config.proxy` > env(`HTTPS_PROXY`/`ALL_PROXY`)。
    /// 供 header-routing 等无 upstream 上下文复用（SPEC-007）。
    pub fn global_proxy(&self) -> Option<String> {
        if let Some(p) = self.proxy.as_ref().filter(|s| !s.is_empty()) {
            return Some(p.clone());
        }
        // env 兜底：HTTPS_PROXY 优先于 ALL_PROXY（scheme-specific 优先）。
        for key in ["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"] {
            if let Some(v) = std::env::var_os(key) {
                let v = v.to_string_lossy().trim().to_string();
                if !v.is_empty() {
                    return Some(v);
                }
            }
        }
        None
    }

    /// 解析出站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/outbound.toml`（相对 cwd）。
    pub fn resolved_rules_path(&self) -> PathBuf {
        if let Some(p) = &self.rules_path {
            return p.clone();
        }
        PathBuf::from("crates/sieve-rules/rules/outbound.toml")
    }

    /// 解析入站规则路径。显式给定时直接用，否则回退到 `crates/sieve-rules/rules/inbound.toml`（相对 cwd）。
    pub fn resolved_inbound_rules_path(&self) -> PathBuf {
        if let Some(p) = &self.inbound_rules_path {
            return p.clone();
        }
        PathBuf::from("crates/sieve-rules/rules/inbound.toml")
    }

    /// 解析签名规则包路径。
    ///
    /// 签名规则包经更新通道下发，由 `sieve-updater` 验签后安装到缓存目录的
    /// `current.json`（见 `sieve-updater::install::install_rules`）。
    /// daemon 启动优先从此包加载系统规则；包不存在时降级 dev TOML，
    /// 再降级空集 fail-safe（引擎可独立构建运行供审计）。
    ///
    /// 优先级：`SIEVE_RULES_PACK` env 覆盖（测试 / 自定义部署）> updater 缓存目录的 `current.json`。
    /// 返回 `None` 仅当缓存目录无法确定（HOME 等环境缺失，极少见），调用方按无包处理。
    pub fn resolved_rules_pack_path(&self) -> Option<PathBuf> {
        if let Some(v) = std::env::var_os("SIEVE_RULES_PACK") {
            let v = PathBuf::from(v);
            if !v.as_os_str().is_empty() {
                return Some(v);
            }
        }
        sieve_updater::cache_dir::cache_dir()
            .ok()
            .map(|d| d.join("current.json"))
    }

    /// 解析 `.sieveignore` 路径。显式给定时直接用，否则 `<sieve_home>/sieveignore`
    /// （`SIEVE_HOME` env var > `$HOME/.sieve`）。
    pub fn resolved_sieveignore_path(&self) -> PathBuf {
        if let Some(p) = &self.sieveignore_path {
            return p.clone();
        }
        if std::env::var_os("SIEVE_HOME").is_none() && std::env::var_os("HOME").is_none() {
            tracing::warn!(
                "HOME / SIEVE_HOME 均未设置；fallback 到 .sieve/sieveignore（相对 cwd）"
            );
        }
        sieve_home().join("sieveignore")
    }

    /// 拼接监听 SocketAddr（**已废弃**，仅保留兼容性 + 测试使用）。
    ///
    /// ADR-026 后 daemon 走 [`Config::resolved_upstreams`] 多 listener 路径，
    /// 不再读 single `port` 字段。本方法保留供旧 sieve.toml 反序列化校验测试使用。
    ///
    /// # Errors
    /// `bind_addr` 或 `port` 无法解析为合法 SocketAddr 时返回错误。
    #[allow(dead_code)]
    pub fn listen_addr(&self) -> Result<std::net::SocketAddr> {
        format!("{}:{}", self.bind_addr, self.port)
            .parse()
            .map_err(|e| anyhow!("invalid bind addr/port: {e}"))
    }

    /// 解析审计日志路径。优先级：`audit_db_path` > `log_path` >
    /// `<sieve_home>/audit.db`（`SIEVE_HOME` env var > `$HOME/.sieve`）。
    ///
    /// # Errors
    /// `$SIEVE_HOME` 与 `$HOME` 均未设置时返回错误。
    pub fn audit_db_path(&self) -> Result<PathBuf> {
        if let Some(p) = &self.audit_db_path {
            return Ok(p.clone());
        }
        if let Some(p) = &self.log_path {
            return Ok(p.clone());
        }
        if std::env::var_os("SIEVE_HOME").is_none() && std::env::var_os("HOME").is_none() {
            return Err(anyhow!(
                "neither SIEVE_HOME nor HOME set; specify audit_db_path or log_path explicitly"
            ));
        }
        Ok(sieve_home().join("audit.db"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let c = Config::default();
        assert_eq!(c.bind_addr, "127.0.0.1");
        assert_eq!(c.port, 11453);
        assert_eq!(c.upstream_url, "https://api.anthropic.com");
        assert!(c.tls_verify_upstream);
        assert!(c.log_path.is_none());
    }

    #[test]
    fn listen_addr_parses() {
        let c = Config::default();
        let addr = c.listen_addr().unwrap();
        assert_eq!(addr.port(), 11453);
        assert!(addr.ip().is_loopback());
    }

    #[test]
    fn parse_minimal_toml() {
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            port = 11453
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(c.bind_addr, "127.0.0.1");
        assert!(c.tls_verify_upstream);
    }

    #[test]
    fn parse_full_toml() {
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            port = 12000
            bind_addr = "127.0.0.1"
            tls_verify_upstream = false
            log_path = "/tmp/audit.db"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(c.port, 12000);
        assert!(!c.tls_verify_upstream);
        assert_eq!(c.log_path.unwrap().to_str().unwrap(), "/tmp/audit.db");
    }

    #[test]
    fn unknown_field_rejected() {
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            disable_critical = true
        "#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(
            result.is_err(),
            "must reject unknown fields (deny_unknown_fields)"
        );
    }

    /// 特性门回归守护：`usage` / `audit-crypto` 可选特性关闭时（默认构建），含
    /// `[billing_check]` 段与 `audit.level = "full"` 的 config 仍必须能反序列化 +
    /// 通过安全校验（config 结构体始终编译，仅功能代码按 feature 分支）。
    ///
    /// 防回归：若误把 `BillingCheckConfig` / `AuditConfig` 整体 gate 掉，含这两段的
    /// 用户 config 会因未知字段反序列化失败——本测试在两种 feature 下都会跑到，守护它。
    #[test]
    fn optional_feature_config_sections_always_deserialize() {
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            port = 12001

            [billing_check]
            enabled = true
            tolerance_pct = 12.5

            [audit]
            level = "full"
            recipient = "age1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsexample"
        "#;
        let c: Config = toml::from_str(toml_str).expect(
            "含 [billing_check] + audit.level=full 的 config 必须能反序列化（结构体不 gate）",
        );
        assert!(c.billing_check.enabled);
        assert_eq!(c.audit.level, AuditLevel::Full);
        // 配了合法 age recipient 的 full 档 config 必须通过安全校验（加载成功，
        // 运行期由 build_archive_writer 在特性关时优雅降级，不在此拒绝）。
        c.check_safety_invariants()
            .expect("配了 age recipient 的 full 档 config 必须通过安全校验");
    }

    #[test]
    fn parse_dry_run_and_rules_path() {
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            port = 11453
            dry_run = true
            rules_path = "/x.toml"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        assert!(c.dry_run, "dry_run should be true");
        assert_eq!(c.rules_path.as_ref().unwrap().to_str().unwrap(), "/x.toml");
    }

    #[test]
    fn resolved_rules_path_explicit() {
        let c = Config {
            rules_path: Some(PathBuf::from("/custom/rules.toml")),
            ..Config::default()
        };
        assert_eq!(c.resolved_rules_path(), PathBuf::from("/custom/rules.toml"));
    }

    #[test]
    fn resolved_rules_path_fallback() {
        let c = Config::default();
        let p = c.resolved_rules_path();
        assert!(
            p.ends_with("outbound.toml"),
            "fallback should end with outbound.toml, got {:?}",
            p
        );
    }

    #[test]
    fn resolved_sieveignore_path_explicit() {
        let c = Config {
            sieveignore_path: Some(PathBuf::from("/my/.sieveignore")),
            ..Config::default()
        };
        assert_eq!(
            c.resolved_sieveignore_path(),
            PathBuf::from("/my/.sieveignore")
        );
    }

    // ── R2-#6 audit_db_path 优先级链测试 ────────────────────────────────────

    #[test]
    fn audit_db_path_explicit_field_wins() {
        // audit_db_path 字段优先于 log_path 和默认值
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            port = 11453
            audit_db_path = "/custom/audit.db"
            log_path = "/old/log.db"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        let path = c.audit_db_path().unwrap();
        assert_eq!(
            path,
            PathBuf::from("/custom/audit.db"),
            "audit_db_path 字段应优先于 log_path"
        );
    }

    #[test]
    fn audit_db_path_falls_back_to_log_path() {
        // 没有 audit_db_path 时应回退到 log_path
        let toml_str = r#"
            upstream_url = "https://api.anthropic.com"
            port = 11453
            log_path = "/old/log.db"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        let path = c.audit_db_path().unwrap();
        assert_eq!(path, PathBuf::from("/old/log.db"), "应回退到 log_path");
    }

    #[test]
    fn audit_db_path_falls_back_to_default() {
        // 两个字段都没有时，应回退到 ~/.sieve/audit.db
        // 假设 HOME 已设置（CI 环境通常有）
        if std::env::var_os("HOME").is_none() {
            return; // HOME 未设置时跳过
        }
        let c = Config::default();
        let path = c.audit_db_path().unwrap();
        assert!(
            path.ends_with(".sieve/audit.db"),
            "默认路径应以 .sieve/audit.db 结尾，实际: {path:?}"
        );
    }

    // ── ADR-026 multi-listener schema 测试 ──────────────────────────────

    #[test]
    fn legacy_schema_maps_to_single_upstream() {
        // 旧 sieve.toml 仅含 upstream_url + port → resolved_upstreams 返回单元素
        let c = Config {
            upstream_url: "https://api.anthropic.com".to_string(),
            port: 11453,
            ..Config::default()
        };
        let ups = c.resolved_upstreams();
        assert_eq!(ups.len(), 1);
        assert_eq!(ups[0].port, 11453);
        assert_eq!(ups[0].url, "https://api.anthropic.com");
        assert_eq!(ups[0].provider_id, "anthropic");
        // legacy 未声明协议 → Auto（按 path 自适应；显式声明才强制错位）
        assert_eq!(ups[0].protocol, Protocol::Auto);
    }

    #[test]
    fn upstream_proxy_overrides_global() {
        let toml_str = r#"
            proxy = "socks5://127.0.0.1:1"
            [[upstream]]
            port = 11453
            url = "https://api.anthropic.com"
            proxy = "http://127.0.0.1:2"
            [[upstream]]
            port = 11454
            url = "http://127.0.0.1:8080"
            no_proxy = true
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        let ups = c.resolved_upstreams();
        // upstream[0] 自身 proxy 覆盖全局
        assert_eq!(
            c.effective_proxy(&ups[0]),
            Some("http://127.0.0.1:2".to_string())
        );
        // upstream[1] no_proxy → 直连，无视全局
        assert_eq!(c.effective_proxy(&ups[1]), None);
    }

    #[test]
    fn global_proxy_applies_when_upstream_unset() {
        let toml_str = r#"
            proxy = "socks5://127.0.0.1:6153"
            [[upstream]]
            port = 11453
            url = "https://api.anthropic.com"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        let ups = c.resolved_upstreams();
        assert_eq!(
            c.effective_proxy(&ups[0]),
            Some("socks5://127.0.0.1:6153".to_string())
        );
    }

    #[test]
    fn parse_multi_upstream_toml() {
        let toml_str = r#"
            [[upstream]]
            port = 11453
            url = "https://api.anthropic.com"
            provider_id = "anthropic"
            protocol = "anthropic"

            [[upstream]]
            port = 11454
            url = "https://api.deepseek.com/anthropic"
            provider_id = "deepseek"
            protocol = "anthropic"

            [[upstream]]
            port = 11455
            url = "https://api.openai.com"
            provider_id = "openai"
            protocol = "openai"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        let ups = c.resolved_upstreams();
        assert_eq!(ups.len(), 3);
        assert_eq!(ups[0].port, 11453);
        assert_eq!(ups[1].port, 11454);
        assert_eq!(ups[1].url, "https://api.deepseek.com/anthropic");
        assert_eq!(ups[2].provider_id, "openai");
        assert_eq!(ups[2].protocol, Protocol::Openai);
    }

    #[test]
    fn upstreams_takes_precedence_over_legacy_fields() {
        // 同时给 upstreams + 旧字段时，新字段优先；旧字段被忽略
        let toml_str = r#"
            upstream_url = "https://legacy.example.com"
            port = 9999

            [[upstream]]
            port = 11454
            url = "https://api.deepseek.com/anthropic"
            provider_id = "deepseek"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        let ups = c.resolved_upstreams();
        assert_eq!(ups.len(), 1, "upstreams 非空时应忽略旧字段");
        assert_eq!(ups[0].port, 11454);
        assert_eq!(ups[0].url, "https://api.deepseek.com/anthropic");
    }

    #[test]
    fn protocol_default_is_auto() {
        // 未指定 protocol 字段时默认 Auto：按请求 path 自适应，不强制协议错位
        // （ADR-026 §决策 1 向后兼容 + PRD §9 #16/#9 双协议）。
        let toml_str = r#"
            [[upstream]]
            port = 11454
            url = "https://api.deepseek.com/anthropic"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        let ups = c.resolved_upstreams();
        assert_eq!(ups[0].protocol, Protocol::Auto);
        // provider_id 留空时由 resolved_provider_id 派生
        assert!(ups[0].provider_id.is_empty());
        assert_eq!(ups[0].resolved_provider_id(), "api.deepseek.com");
    }

    #[test]
    fn resolved_provider_id_explicit_wins_over_url_host() {
        let u = UpstreamListener {
            port: 11454,
            url: "https://api.deepseek.com/anthropic".to_string(),
            provider_id: "my-deepseek-relay".to_string(),
            protocol: Protocol::Anthropic,
            routes: Vec::new(),
            trust: None,
            proxy: None,
            no_proxy: false,
        };
        assert_eq!(u.resolved_provider_id(), "my-deepseek-relay");
    }

    #[test]
    fn resolved_provider_id_falls_back_to_url_host() {
        let u = UpstreamListener {
            port: 11454,
            url: "https://api.deepseek.com/anthropic".to_string(),
            provider_id: String::new(),
            protocol: Protocol::Anthropic,
            routes: Vec::new(),
            trust: None,
            proxy: None,
            no_proxy: false,
        };
        assert_eq!(u.resolved_provider_id(), "api.deepseek.com");
    }

    #[test]
    fn resolved_provider_id_handles_invalid_url() {
        let u = UpstreamListener {
            port: 11454,
            url: "not a uri !!!".to_string(),
            provider_id: String::new(),
            protocol: Protocol::Anthropic,
            routes: Vec::new(),
            trust: None,
            proxy: None,
            no_proxy: false,
        };
        // 不应 panic，给出 fallback 标识
        assert_eq!(u.resolved_provider_id(), "unknown");
    }

    #[test]
    fn check_invariants_detects_port_conflict() {
        let c = Config {
            upstreams: vec![
                UpstreamListener {
                    port: 11454,
                    url: "https://api.anthropic.com".to_string(),
                    provider_id: "anthropic".to_string(),
                    protocol: Protocol::Anthropic,
                    routes: Vec::new(),
                    trust: None,
                    proxy: None,
                    no_proxy: false,
                },
                UpstreamListener {
                    port: 11454, // duplicate
                    url: "https://api.deepseek.com/anthropic".to_string(),
                    provider_id: "deepseek".to_string(),
                    protocol: Protocol::Anthropic,
                    routes: Vec::new(),
                    trust: None,
                    proxy: None,
                    no_proxy: false,
                },
            ],
            ..Config::default()
        };
        let err = c.check_safety_invariants().unwrap_err();
        assert!(
            err.contains("duplicate listener port 11454"),
            "应检测端口冲突，实际: {err}"
        );
    }

    #[test]
    fn check_invariants_rejects_non_loopback_bind() {
        let c = Config {
            bind_addr: "0.0.0.0".to_string(),
            ..Config::default()
        };
        let err = c.check_safety_invariants().unwrap_err();
        assert!(
            err.contains("bind_addr must be 127.0.0.1"),
            "应拒绝非 loopback 绑定，实际: {err}"
        );
    }

    #[test]
    fn check_invariants_passes_for_default_config() {
        let c = Config::default();
        assert!(c.check_safety_invariants().is_ok());
    }

    #[test]
    fn check_invariants_passes_for_valid_multi_listener() {
        let c = Config {
            upstreams: vec![
                UpstreamListener {
                    port: 11453,
                    url: "https://api.anthropic.com".to_string(),
                    provider_id: "anthropic".to_string(),
                    protocol: Protocol::Anthropic,
                    routes: Vec::new(),
                    trust: None,
                    proxy: None,
                    no_proxy: false,
                },
                UpstreamListener {
                    port: 11454,
                    url: "https://api.deepseek.com/anthropic".to_string(),
                    provider_id: "deepseek".to_string(),
                    protocol: Protocol::Anthropic,
                    routes: Vec::new(),
                    trust: None,
                    proxy: None,
                    no_proxy: false,
                },
                UpstreamListener {
                    port: 11455,
                    url: "https://api.openai.com".to_string(),
                    provider_id: "openai".to_string(),
                    protocol: Protocol::Openai,
                    routes: Vec::new(),
                    trust: None,
                    proxy: None,
                    no_proxy: false,
                },
            ],
            ..Config::default()
        };
        assert!(c.check_safety_invariants().is_ok());
    }

    #[test]
    fn check_invariants_rejects_route_with_auto_provider() {
        let c = Config {
            upstreams: vec![UpstreamListener {
                port: 11454,
                url: "https://relay.example.com".to_string(),
                provider_id: "relay".to_string(),
                protocol: Protocol::Auto,
                routes: vec![RouteRule {
                    path: "/custom/chat".to_string(),
                    provider: Protocol::Auto, // 非法：route 必须指定确定 provider
                }],
                trust: None,
                proxy: None,
                no_proxy: false,
            }],
            ..Config::default()
        };
        let err = c.check_safety_invariants().unwrap_err();
        assert!(
            err.contains("provider = \"auto\""),
            "route 配 provider=auto 应被拒绝，实际: {err}"
        );
    }

    #[test]
    fn check_invariants_accepts_concrete_route_provider() {
        let c = Config {
            upstreams: vec![UpstreamListener {
                port: 11454,
                url: "https://relay.example.com".to_string(),
                provider_id: "relay".to_string(),
                protocol: Protocol::Auto,
                routes: vec![RouteRule {
                    path: "/custom/chat".to_string(),
                    provider: Protocol::Openai,
                }],
                trust: None,
                proxy: None,
                no_proxy: false,
            }],
            ..Config::default()
        };
        assert!(
            c.check_safety_invariants().is_ok(),
            "确定 provider 的 route 应通过安全校验"
        );
    }

    #[test]
    fn upstream_listener_rejects_unknown_field() {
        // UpstreamListener 也带 deny_unknown_fields，错配字段被拒
        let toml_str = r#"
            [[upstream]]
            port = 11454
            url = "https://api.deepseek.com/anthropic"
            disable_critical = true
        "#;
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_err(), "UpstreamListener 应拒绝未知字段");
    }

    // ── ADR-038 信任分级（Trust）────────────────────────────────────────────

    fn listener(url: &str, trust: Option<Trust>) -> UpstreamListener {
        UpstreamListener {
            port: 11453,
            url: url.to_string(),
            provider_id: String::new(),
            protocol: Protocol::Auto,
            routes: Vec::new(),
            trust,
            proxy: None,
            no_proxy: false,
        }
    }

    #[test]
    fn resolved_trust_official_hosts_derive_official() {
        assert_eq!(
            listener("https://api.anthropic.com", None).resolved_trust(),
            Trust::Official
        );
        assert_eq!(
            listener("https://api.openai.com/v1", None).resolved_trust(),
            Trust::Official
        );
    }

    #[test]
    fn resolved_trust_relay_host_derives_relay() {
        assert_eq!(
            listener("https://some-relay.example.com/anthropic", None).resolved_trust(),
            Trust::Relay
        );
    }

    #[test]
    fn resolved_trust_unparseable_url_is_relay_fail_closed() {
        // 无法解析 host → 保守归为 Relay（绝不把不可信误当可信）。
        assert_eq!(
            listener("not a uri !!!", None).resolved_trust(),
            Trust::Relay
        );
    }

    #[test]
    fn resolved_trust_explicit_overrides_host_derivation() {
        // 显式声明优先：即便 host 是官方，显式标 relay 也按 relay（反之亦然）。
        assert_eq!(
            listener("https://api.anthropic.com", Some(Trust::Relay)).resolved_trust(),
            Trust::Relay
        );
        assert_eq!(
            listener("https://shady-relay.io", Some(Trust::Official)).resolved_trust(),
            Trust::Official
        );
    }

    #[test]
    fn legacy_upstream_maps_to_official_via_host() {
        // 旧字段默认 upstream_url = api.anthropic.com → resolved_trust 派生 Official。
        let c = Config::default();
        let ups = c.resolved_upstreams();
        assert_eq!(ups.len(), 1);
        assert_eq!(ups[0].resolved_trust(), Trust::Official);
    }

    // ── ADR-037 / ADR-038 config schema 默认值与 fail-fast ──────────────────

    #[test]
    fn audit_and_billing_defaults_are_off_or_status_quo() {
        let c = Config::default();
        // ADR-037：默认 metadata（= 现状，零行为变化），full 默认关。
        assert_eq!(c.audit.level, AuditLevel::Metadata);
        assert_eq!(c.audit.retention_days, 30);
        assert!(c.audit.hash_chain);
        assert_eq!(c.audit.rotation, ArchiveRotation::Daily);
        assert!(c.audit.recipient.is_none());
        // ADR-038：默认全关。
        assert!(!c.billing_check.enabled);
        assert!(!c.billing_check.count_tokens_optin);
        assert_eq!(c.billing_check.tolerance_pct, 15.0);
    }

    #[test]
    fn audit_full_without_recipient_is_rejected() {
        let c = Config {
            audit: AuditConfig {
                level: AuditLevel::Full,
                recipient: None,
                ..AuditConfig::default()
            },
            ..Config::default()
        };
        let err = c.check_safety_invariants().unwrap_err();
        assert!(
            err.contains("audit.recipient"),
            "full 档缺 recipient 应被拒，实际: {err}"
        );
    }

    #[test]
    fn audit_full_with_non_age_recipient_is_rejected() {
        let c = Config {
            audit: AuditConfig {
                level: AuditLevel::Full,
                recipient: Some("ssh-ed25519 AAAA...".to_string()),
                ..AuditConfig::default()
            },
            ..Config::default()
        };
        let err = c.check_safety_invariants().unwrap_err();
        assert!(err.contains("age1"), "非 age 公钥应被拒，实际: {err}");
    }

    #[test]
    fn audit_full_with_valid_recipient_passes() {
        let c = Config {
            audit: AuditConfig {
                level: AuditLevel::Full,
                recipient: Some(
                    "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p".to_string(),
                ),
                ..AuditConfig::default()
            },
            ..Config::default()
        };
        assert!(c.check_safety_invariants().is_ok());
    }

    #[test]
    fn billing_tolerance_out_of_range_is_rejected() {
        for bad in [0.0, -5.0, 150.0] {
            let c = Config {
                billing_check: BillingCheckConfig {
                    enabled: true,
                    tolerance_pct: bad,
                    ..BillingCheckConfig::default()
                },
                ..Config::default()
            };
            let err = c.check_safety_invariants().unwrap_err();
            assert!(
                err.contains("tolerance_pct"),
                "容差 {bad} 越界应被拒，实际: {err}"
            );
        }
    }

    #[test]
    fn billing_disabled_skips_tolerance_validation() {
        // 未启用时即便容差越界也不报错（零开销，不校验）。
        let c = Config {
            billing_check: BillingCheckConfig {
                enabled: false,
                tolerance_pct: 0.0,
                ..BillingCheckConfig::default()
            },
            ..Config::default()
        };
        assert!(c.check_safety_invariants().is_ok());
    }

    #[test]
    fn audit_and_billing_sections_parse_from_toml() {
        let toml_str = r#"
            [audit]
            level = "full"
            recipient = "age1ql3z7hjy54pw3hyww5ayyfg7zqgvc7w3j2elw8zmrj2kg5sfn9aqmcac8p"
            retention_days = 7
            hash_chain = true
            rotation = "weekly"

            [billing_check]
            enabled = true
            tolerance_pct = 20.0
            count_tokens_optin = true

            [[upstream]]
            port = 11454
            url = "https://some-relay.example.com/anthropic"
            trust = "relay"
        "#;
        let c: Config = toml::from_str(toml_str).expect("应能解析 [audit]/[billing_check]/trust");
        assert_eq!(c.audit.level, AuditLevel::Full);
        assert_eq!(c.audit.rotation, ArchiveRotation::Weekly);
        assert_eq!(c.audit.retention_days, 7);
        assert!(c.billing_check.enabled);
        assert_eq!(c.billing_check.tolerance_pct, 20.0);
        assert!(c.billing_check.count_tokens_optin);
        assert_eq!(c.upstreams[0].trust, Some(Trust::Relay));
        assert_eq!(c.upstreams[0].resolved_trust(), Trust::Relay);
    }

    #[test]
    fn audit_rejects_unknown_field() {
        // [audit] 带 deny_unknown_fields。
        let toml_str = r#"
            [audit]
            level = "metadata"
            disable_encryption = true
        "#;
        assert!(
            toml::from_str::<Config>(toml_str).is_err(),
            "[audit] 应拒绝未知字段"
        );
    }
}
