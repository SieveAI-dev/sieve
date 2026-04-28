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

/// Sieve 顶层配置。
///
/// 对应 `sieve.toml`（ADR-003 / data-model.md §配置）。
/// 文件不存在时 [`Config::load`] 返回 [`Config::default`]。
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// 上游 LLM API 端点（默认 `https://api.anthropic.com`）。
    #[serde(default = "default_upstream")]
    pub upstream_url: String,

    /// 本地代理监听端口（默认 11453，PRD §6.1）。
    #[serde(default = "default_port")]
    pub port: u16,

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

    /// 强制安全不变量：`bind_addr` 必须是 `127.0.0.1`，否则打印 FATAL 并 `exit(1)`。
    ///
    /// 关联 ADR-003 / PRD §9 #2 / data-model.md §配置。
    /// 不提供 fallback，不 warn 后继续：非 loopback 绑定是配置错误，
    /// 悄悄启动会暴露代理到局域网，违反"完全本地"承诺。
    pub fn enforce_safety_invariants(&self) {
        if self.bind_addr != "127.0.0.1" {
            eprintln!(
                "FATAL: bind_addr must be 127.0.0.1 (got {:?}). \
                 Sieve refuses to bind on a non-loopback address. See ADR-003.",
                self.bind_addr
            );
            std::process::exit(1);
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
}
