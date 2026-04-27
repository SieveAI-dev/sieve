//! 配置加载（关联 docs/design/data-model.md §配置）。
//!
//! Phase 1 字段：`upstream_url` / `port` / `bind_addr` / `log_path` /
//! `tls_verify_upstream`。
//! `#[serde(deny_unknown_fields)]` 确保配置文件中的危险字段（如
//! `disable_critical`）被强制拒绝，不会静默忽略。

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

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
        let cfg: Self = toml::from_str(&s)
            .with_context(|| format!("parse config {}", path.display()))?;
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

    /// 解析审计日志路径。`log_path` 显式给定时直接用,否则回退到 `~/.sieve/audit.db`。
    ///
    /// # Errors
    /// `$HOME` 不存在或不可识别时返回错误。
    pub fn audit_db_path(&self) -> Result<PathBuf> {
        if let Some(p) = &self.log_path {
            return Ok(p.clone());
        }
        let home = std::env::var_os("HOME")
            .ok_or_else(|| anyhow!("HOME env var not set; specify log_path explicitly"))?;
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
        assert!(result.is_err(), "must reject unknown fields (deny_unknown_fields)");
    }
}
