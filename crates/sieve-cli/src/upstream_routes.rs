//! OpenAI 兼容上游路由表（F-1 修复）。
//!
//! ## 问题
//!
//! setup --agent openclaw 把所有 provider 的 baseUrl 改为 Sieve 代理地址后，
//! daemon 需要知道"把 OpenClaw 转来的请求发往哪个真实上游"。
//! daemon 的 `Config.upstream_url` 只有一个，无法区分多个 provider。
//!
//! ## 设计
//!
//! - setup 时读取 OpenClaw config 中各 provider 的原始 baseUrl，
//!   写到 `~/.sieve/upstream-routes.json`
//! - JSON schema：`{"provider_id": "https://original.upstream.url", ...}`
//! - daemon 启动时调 `load()` 加载路由表，对每个 provider 预构建 Forwarder（连接池）
//! - OpenClaw 在请求中注入 `X-Sieve-Provider: <id>` header（setup 配置时同步写入）
//! - daemon 按 header 值查 provider_forwarders map 选上游；未命中时用 `Config.upstream_url` 兜底
//! - 转发前移除 `X-Sieve-Provider` header（内部路由标记，不透传给真实上游）
//!
//! ## 关联
//!
//! - known-issues-v1.4.md R11-#1（现已修复）
//! - SPEC-004 §4.2

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 上游路由表（provider id → upstream base URL）。
///
/// JSON Lines schema:
/// ```json
/// {
///   "anthropic": "https://api.anthropic.com",
///   "openai": "https://api.openai.com",
///   "openrouter": "https://openrouter.ai/api"
/// }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpstreamRoutes {
    #[serde(flatten)]
    routes: HashMap<String, String>,
}

impl UpstreamRoutes {
    /// 从 JSON 文件加载路由表。文件不存在时返回空路由表。
    ///
    /// daemon 启动时调用，按 X-Sieve-Provider header 为 OpenClaw 请求路由上游。
    ///
    /// # Errors
    ///
    /// 文件存在但内容不是合法 JSON 时返回 `Err`。
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("读取 {} 失败", path.display()))?;
        let routes: Self = serde_json::from_str(&raw)
            .with_context(|| format!("解析 {} 失败（须为有效 JSON 对象）", path.display()))?;
        Ok(routes)
    }

    /// 保存路由表到 JSON 文件（父目录必须已存在）。
    ///
    /// # Errors
    ///
    /// 文件写入失败时返回 `Err`。
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.routes)
            .context("序列化 upstream-routes.json 失败")?;
        std::fs::write(path, json.as_bytes())
            .with_context(|| format!("写入 {} 失败", path.display()))?;
        Ok(())
    }

    /// 插入或更新一条路由（provider id → upstream URL）。
    pub fn insert(&mut self, provider_id: impl Into<String>, upstream_url: impl Into<String>) {
        self.routes.insert(provider_id.into(), upstream_url.into());
    }

    /// 当前路由条数。
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// 路由表是否为空。
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// 遍历所有 (provider_id, upstream_url) 对。
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.routes.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    #[test]
    fn insert_and_iter_returns_entries() {
        let mut r = UpstreamRoutes::default();
        r.insert("openai", "https://api.openai.com");
        r.insert("openrouter", "https://openrouter.ai/api");
        assert_eq!(r.len(), 2);
        let mut pairs: Vec<(String, String)> = r
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        pairs.sort();
        assert_eq!(
            pairs[0],
            ("openai".to_owned(), "https://api.openai.com".to_owned())
        );
        assert_eq!(
            pairs[1],
            (
                "openrouter".to_owned(),
                "https://openrouter.ai/api".to_owned()
            )
        );
    }

    #[test]
    fn load_nonexistent_returns_empty() {
        let path = std::path::Path::new("/tmp/upstream-routes-does-not-exist-sieve-test.json");
        let r = UpstreamRoutes::load(path).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn load_valid_json_roundtrip() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            tmp,
            r#"{{"openai": "https://api.openai.com", "deepseek": "https://api.deepseek.com"}}"#
        )
        .unwrap();
        let r = UpstreamRoutes::load(tmp.path()).unwrap();
        assert_eq!(r.len(), 2);
        let urls: Vec<_> = r.iter().map(|(_, v)| v).collect();
        assert!(
            urls.contains(&"https://api.openai.com"),
            "should contain openai url"
        );
        assert!(
            urls.contains(&"https://api.deepseek.com"),
            "should contain deepseek url"
        );
    }

    #[test]
    fn load_invalid_json_returns_err() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        writeln!(tmp, "this is not json {{").unwrap();
        let result = UpstreamRoutes::load(tmp.path());
        assert!(result.is_err(), "invalid JSON should return Err");
    }
}
