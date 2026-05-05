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
/// - `Default`：推荐平衡配置（默认）
/// - `Relaxed`：降低误报，适合受信任环境
/// - `Custom`：完全自定义（忽略内置默认值）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    Strict,
    #[default]
    Default,
    Relaxed,
    Custom,
}

/// 上游协议（ADR-018 / ADR-026 §决策 4）。
///
/// listener 显式声明协议，daemon 收到请求后按本字段决策路由 + 严格校验请求 path：
/// - `Anthropic` listener 仅接受 `/v1/messages`
/// - `Openai` listener 仅接受 `/v1/chat/completions`
///
/// 错位请求（如 Anthropic listener 收到 `/v1/chat/completions`）→ daemon
/// fail-closed 400 + sieve_blocked event 注入（PRD §9 #3）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    #[default]
    Anthropic,
    Openai,
}

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

    /// 上游协议（默认 `anthropic`，向后兼容旧单上游 schema）。
    #[serde(default)]
    pub protocol: Protocol,
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
}

fn home_path() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn sieve_home() -> PathBuf {
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
    ///   （向后兼容路径，provider_id = `"anthropic"`，protocol = `Anthropic`）
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
            protocol: Protocol::Anthropic,
        }]
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

    /// 解析 `.sieveignore` 路径。显式给定时直接用，否则回退到 `~/.sieve/sieveignore`。
    ///
    /// 若 `HOME` 不可读则 fallback 到 `.sieve/sieveignore`（相对 cwd）并打印 WARN。
    pub fn resolved_sieveignore_path(&self) -> PathBuf {
        if let Some(p) = &self.sieveignore_path {
            return p.clone();
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(".sieve").join("sieveignore");
        }
        tracing::warn!("HOME env var not set; using .sieve/sieveignore relative to cwd");
        PathBuf::from(".sieve").join("sieveignore")
    }

    /// 拼接监听 SocketAddr。
    ///
    /// # Errors
    /// `bind_addr` 或 `port` 无法解析为合法 SocketAddr 时返回错误。
    pub fn listen_addr(&self) -> Result<std::net::SocketAddr> {
        format!("{}:{}", self.bind_addr, self.port)
            .parse()
            .map_err(|e| anyhow!("invalid bind addr/port: {e}"))
    }

    /// 解析审计日志路径。优先级：`audit_db_path` > `log_path` > `~/.sieve/audit.db`。
    ///
    /// # Errors
    /// `$HOME` 不存在或不可识别时返回错误。
    pub fn audit_db_path(&self) -> Result<PathBuf> {
        if let Some(p) = &self.audit_db_path {
            return Ok(p.clone());
        }
        if let Some(p) = &self.log_path {
            return Ok(p.clone());
        }
        let home = std::env::var_os("HOME").ok_or_else(|| {
            anyhow!("HOME env var not set; specify audit_db_path or log_path explicitly")
        })?;
        Ok(PathBuf::from(home).join(".sieve").join("audit.db"))
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
        assert_eq!(ups[0].protocol, Protocol::Anthropic);
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
    fn protocol_default_is_anthropic() {
        // 未指定 protocol 字段时默认 Anthropic（向后兼容）
        let toml_str = r#"
            [[upstream]]
            port = 11454
            url = "https://api.deepseek.com/anthropic"
        "#;
        let c: Config = toml::from_str(toml_str).unwrap();
        let ups = c.resolved_upstreams();
        assert_eq!(ups[0].protocol, Protocol::Anthropic);
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
                },
                UpstreamListener {
                    port: 11454, // duplicate
                    url: "https://api.deepseek.com/anthropic".to_string(),
                    provider_id: "deepseek".to_string(),
                    protocol: Protocol::Anthropic,
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
                },
                UpstreamListener {
                    port: 11454,
                    url: "https://api.deepseek.com/anthropic".to_string(),
                    provider_id: "deepseek".to_string(),
                    protocol: Protocol::Anthropic,
                },
                UpstreamListener {
                    port: 11455,
                    url: "https://api.openai.com".to_string(),
                    provider_id: "openai".to_string(),
                    protocol: Protocol::Openai,
                },
            ],
            ..Config::default()
        };
        assert!(c.check_safety_invariants().is_ok());
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
}
