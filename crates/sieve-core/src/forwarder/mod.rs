//! 上游转发（Anthropic 透传，见架构图节点 ③）。
//!
//! TLS：rustls 0.23 + aws-lc-rs provider + webpki-roots（reproducible build 友好）。
//! ALPN：h2 + http/1.1 都协商，Anthropic 默认走 h2。
//!
//! 关联 reproducible build（webpki-roots 避免系统证书依赖）。

use crate::error::{SieveCoreError, SieveCoreResult};
use http_body_util::BodyExt;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::sync::OnceLock;

mod proxy;
pub use proxy::{MaybeProxyStream, ProxyConfig, ProxyConnector, ProxyStream};

type BoxBody = http_body_util::combinators::BoxBody<bytes::Bytes, hyper::Error>;

/// 全局 crypto provider 安装标志（aws-lc-rs，与 hyper-rustls feature 对齐）。
///
/// OnceLock 保证多线程下只安装一次，后续调用幂等。
static CRYPTO_PROVIDER: OnceLock<()> = OnceLock::new();

fn install_crypto_provider() {
    CRYPTO_PROVIDER.get_or_init(|| {
        // aws-lc-rs 是 hyper-rustls "aws-lc-rs" feature 的默认 provider。
        // 失败说明调用方已安装了其他 provider，不强制覆盖。
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

/// 上游转发器（全局复用，内置连接池）。
pub struct Forwarder {
    client: Client<hyper_rustls::HttpsConnector<proxy::ProxyConnector>, BoxBody>,
    upstream_host: String,
    upstream_scheme: String,
    /// 上游 URL 中的 path 前缀（已 trim 末尾 `/`）。
    ///
    /// 例：`https://api.anthropic.com` → `""`；
    /// `https://api.deepseek.com/anthropic` → `"/anthropic"`；
    /// `https://api.deepseek.com/anthropic/` → `"/anthropic"`。
    /// `rewrite_uri` 时拼接到 client 请求 path 之前。
    upstream_path_prefix: String,
}

impl Forwarder {
    /// 新建 Forwarder。
    ///
    /// # Arguments
    /// * `upstream_url` - 形如 `https://api.anthropic.com` 或
    ///   `https://api.deepseek.com/anthropic`（含 path 前缀的中转站）。
    ///
    /// # Errors
    /// URI 格式非法或 TLS 配置失败时返回 [`SieveCoreError::Forwarder`] /
    /// [`SieveCoreError::TlsConfig`]。
    pub fn new(upstream_url: &str, proxy: ProxyConfig) -> SieveCoreResult<Self> {
        install_crypto_provider();

        let url = http::Uri::try_from(upstream_url)
            .map_err(|e| SieveCoreError::Forwarder(format!("invalid upstream uri: {e}")))?;
        let scheme = url
            .scheme_str()
            .ok_or_else(|| SieveCoreError::Forwarder("upstream uri missing scheme".into()))?
            .to_owned();
        let host = url
            .authority()
            .ok_or_else(|| SieveCoreError::Forwarder("upstream uri missing authority".into()))?
            .to_string();
        // 抽取 path 前缀。trim_end_matches('/') 同时处理：
        //   - `https://api.anthropic.com`（path = "/" → "")
        //   - `https://api.deepseek.com/anthropic/`（trailing slash → "/anthropic"）
        // 拼接时与 client 请求 path（必带前导 `/`）天然不会双斜杠。
        let path_prefix = url.path().trim_end_matches('/').to_owned();

        // webpki-roots：编译期内嵌根证书，不依赖系统证书 store。
        // reproducible build 友好。
        let root_store =
            rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let tls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = HttpsConnectorBuilder::new()
            .with_tls_config(tls_config)
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .wrap_connector(proxy::ProxyConnector::new(proxy));

        let client = Client::builder(TokioExecutor::new()).build::<_, BoxBody>(connector);

        Ok(Self {
            client,
            upstream_host: host,
            upstream_scheme: scheme,
            upstream_path_prefix: path_prefix,
        })
    }

    /// 把客户端请求透传到上游，返回上游响应（body 流式）。
    ///
    /// 调用方负责先用 [`Forwarder::rewrite_uri`] 重写 URI，
    /// 并设置正确的 Host header。
    ///
    /// # Errors
    /// 上游连接或请求失败时返回 [`SieveCoreError::Forwarder`]。
    pub async fn forward(
        &self,
        req: http::Request<BoxBody>,
    ) -> SieveCoreResult<http::Response<hyper::body::Incoming>> {
        self.client
            .request(req)
            .await
            .map_err(|e| SieveCoreError::Forwarder(format!("upstream request failed: {e}")))
    }

    /// 重写客户端请求 URI 到上游：scheme + authority + 上游 path 前缀 + 请求 path/query。
    ///
    /// 上游 URL 含 path 前缀时（如 `https://api.deepseek.com/anthropic`），
    /// 前缀会被拼接到 client 请求 path 之前——修复 v1.x 丢弃前缀的 bug。
    ///
    /// # Errors
    /// URI 重组失败时返回 [`SieveCoreError::Forwarder`]。
    pub fn rewrite_uri(&self, original: &http::Uri) -> SieveCoreResult<http::Uri> {
        let path_and_query = original.path_and_query().map(|p| p.as_str()).unwrap_or("/");
        let new_uri = format!(
            "{}://{}{}{}",
            self.upstream_scheme, self.upstream_host, self.upstream_path_prefix, path_and_query
        );
        http::Uri::try_from(new_uri)
            .map_err(|e| SieveCoreError::Forwarder(format!("uri rewrite failed: {e}")))
    }

    /// 上游 host（用于 Host header）。
    pub fn upstream_host(&self) -> &str {
        &self.upstream_host
    }
}

/// 把任意 `Body` 包成 `BoxBody<Bytes, hyper::Error>`。
///
/// 统一 body 类型以便 [`Forwarder`] 发送。
pub fn box_body<B>(body: B) -> BoxBody
where
    B: http_body::Body<Data = bytes::Bytes, Error = hyper::Error> + Send + Sync + 'static,
{
    BodyExt::boxed(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwarder_new_parses_https_uri() {
        let f = Forwarder::new("https://api.anthropic.com", ProxyConfig::Direct).unwrap();
        assert_eq!(f.upstream_host(), "api.anthropic.com");
    }

    #[test]
    fn forwarder_new_invalid_uri_returns_error() {
        let result = Forwarder::new("not a uri !!!", ProxyConfig::Direct);
        assert!(result.is_err());
    }

    #[test]
    fn rewrite_uri_keeps_path_and_query() {
        let f = Forwarder::new("https://api.anthropic.com", ProxyConfig::Direct).unwrap();
        let original: http::Uri = "/v1/messages?beta=1".parse().unwrap();
        let new = f.rewrite_uri(&original).unwrap();
        assert_eq!(
            new.to_string(),
            "https://api.anthropic.com/v1/messages?beta=1"
        );
    }

    #[test]
    fn rewrite_uri_root_path() {
        let f = Forwarder::new("https://api.anthropic.com", ProxyConfig::Direct).unwrap();
        let original: http::Uri = "/".parse().unwrap();
        let new = f.rewrite_uri(&original).unwrap();
        assert_eq!(new.to_string(), "https://api.anthropic.com/");
    }

    // ── upstream path prefix 修复 ──────────────────────────────

    #[test]
    fn rewrite_uri_with_path_prefix() {
        // DeepSeek Anthropic 兼容入口：含 /anthropic 前缀
        let f = Forwarder::new("https://api.deepseek.com/anthropic", ProxyConfig::Direct).unwrap();
        let original: http::Uri = "/v1/messages".parse().unwrap();
        let new = f.rewrite_uri(&original).unwrap();
        assert_eq!(
            new.to_string(),
            "https://api.deepseek.com/anthropic/v1/messages"
        );
    }

    #[test]
    fn rewrite_uri_path_prefix_with_query() {
        let f = Forwarder::new("https://api.deepseek.com/anthropic", ProxyConfig::Direct).unwrap();
        let original: http::Uri = "/v1/messages?beta=1".parse().unwrap();
        let new = f.rewrite_uri(&original).unwrap();
        assert_eq!(
            new.to_string(),
            "https://api.deepseek.com/anthropic/v1/messages?beta=1"
        );
    }

    #[test]
    fn rewrite_uri_path_prefix_trailing_slash_normalized() {
        // 用户配置末尾带 `/`，结果应与不带 `/` 一致（无双斜杠）
        let f = Forwarder::new("https://api.deepseek.com/anthropic/", ProxyConfig::Direct).unwrap();
        let original: http::Uri = "/v1/messages".parse().unwrap();
        let new = f.rewrite_uri(&original).unwrap();
        assert_eq!(
            new.to_string(),
            "https://api.deepseek.com/anthropic/v1/messages"
        );
    }

    #[test]
    fn rewrite_uri_multi_segment_path_prefix() {
        // 多层前缀也支持（某些中转站会用 /api/v2 这种结构）
        let f = Forwarder::new("https://relay.example.com/api/v2", ProxyConfig::Direct).unwrap();
        let original: http::Uri = "/v1/messages".parse().unwrap();
        let new = f.rewrite_uri(&original).unwrap();
        assert_eq!(
            new.to_string(),
            "https://relay.example.com/api/v2/v1/messages"
        );
    }

    #[test]
    fn upstream_host_excludes_path_prefix() {
        // 关键不变量：upstream_host() 必须只返回 authority（用于 HTTP Host header），
        // 绝不能含 path 前缀，否则上游会拒绝请求。
        let f = Forwarder::new("https://api.deepseek.com/anthropic", ProxyConfig::Direct).unwrap();
        assert_eq!(f.upstream_host(), "api.deepseek.com");
    }
}
