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
//! - daemon 启动时（或收到请求时）调 `load()` 加载路由表
//! - OpenClaw 在请求中注入 `X-Sieve-Provider: <id>` header（setup 配置时同步写入）
//! - daemon 按 header 值调 `lookup_by_provider_id()` 选上游；未命中时用 `Config.upstream_url` 兜底
//!
//! ## 关联
//!
//! - known-issues-v1.4.md F-1（现已修复）
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
    /// 目前由 daemon 在请求处理时调用（Phase 1 后期接入）。
    /// 当前仅 setup 写入，daemon 尚未读取，故此方法暂时被标为 dead_code。
    ///
    /// # Errors
    ///
    /// 文件存在但内容不是合法 JSON 时返回 `Err`。
    #[allow(dead_code)] // TODO(daemon-routing): daemon Forwarder 接入后删除此注解
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

    /// 按 provider id 查找对应的上游 URL。
    ///
    /// 返回 `None` 时调用方应使用 `Config.upstream_url` 兜底。
    /// daemon Forwarder 在收到含 `X-Sieve-Provider` header 的请求时调用此方法。
    #[allow(dead_code)] // TODO(daemon-routing): daemon Forwarder 接入后删除此注解
    pub fn lookup_by_provider_id(&self, provider_id: &str) -> Option<&str> {
        self.routes.get(provider_id).map(|s| s.as_str())
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
    #[allow(dead_code)] // TODO(daemon-routing): daemon Forwarder 接入后删除此注解
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.routes.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}
