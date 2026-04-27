//! 上游转发（Anthropic 透传，见架构图节点 ③）。
//!
//! TLS：rustls 0.23 + aws-lc-rs provider + webpki-roots（reproducible build 友好）。
//! ALPN：h2 + http/1.1 都协商，Anthropic 默认走 h2。
//!
//! 关联 PRD §6.1 + ADR-006（reproducible build，webpki-roots 避免系统证书依赖）。

use crate::error::{SieveCoreError, SieveCoreResult};
use http_body_util::BodyExt;
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::sync::OnceLock;

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
    client: Client<
        hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
        BoxBody,
    >,
    upstream_host: String,
    upstream_scheme: String,
}

impl Forwarder {
    /// 新建 Forwarder。
    ///
    /// # Arguments
    /// * `upstream_host_with_scheme` - 形如 `https://api.anthropic.com`。
    ///
    /// # Errors
    /// URI 格式非法或 TLS 配置失败时返回 [`SieveCoreError::Forwarder`] /
    /// [`SieveCoreError::TlsConfig`]。
    pub fn new(upstream_host_with_scheme: &str) -> SieveCoreResult<Self> {
        install_crypto_provider();

        let url = http::Uri::try_from(upstream_host_with_scheme)
            .map_err(|e| SieveCoreError::Forwarder(format!("invalid upstream uri: {e}")))?;
        let scheme = url
            .scheme_str()
            .ok_or_else(|| SieveCoreError::Forwarder("upstream uri missing scheme".into()))?
            .to_owned();
        let host = url
            .authority()
            .ok_or_else(|| SieveCoreError::Forwarder("upstream uri missing authority".into()))?
            .to_string();

        // webpki-roots：编译期内嵌根证书，不依赖系统证书 store。
        // reproducible build 友好，见 ADR-006。
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
            .build();

        let client = Client::builder(TokioExecutor::new()).build::<_, BoxBody>(connector);

        Ok(Self {
            client,
            upstream_host: host,
            upstream_scheme: scheme,
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

    /// 重写客户端请求 URI 到上游（scheme + authority，保留 path + query）。
    ///
    /// # Errors
    /// URI 重组失败时返回 [`SieveCoreError::Forwarder`]。
    pub fn rewrite_uri(&self, original: &http::Uri) -> SieveCoreResult<http::Uri> {
        let path_and_query = original
            .path_and_query()
            .map(|p| p.as_str())
            .unwrap_or("/");
        let new_uri = format!(
            "{}://{}{}",
            self.upstream_scheme, self.upstream_host, path_and_query
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
        let f = Forwarder::new("https://api.anthropic.com").unwrap();
        assert_eq!(f.upstream_host(), "api.anthropic.com");
    }

    #[test]
    fn forwarder_new_invalid_uri_returns_error() {
        let result = Forwarder::new("not a uri !!!");
        assert!(result.is_err());
    }

    #[test]
    fn rewrite_uri_keeps_path_and_query() {
        let f = Forwarder::new("https://api.anthropic.com").unwrap();
        let original: http::Uri = "/v1/messages?beta=1".parse().unwrap();
        let new = f.rewrite_uri(&original).unwrap();
        assert_eq!(
            new.to_string(),
            "https://api.anthropic.com/v1/messages?beta=1"
        );
    }

    #[test]
    fn rewrite_uri_root_path() {
        let f = Forwarder::new("https://api.anthropic.com").unwrap();
        let original: http::Uri = "/".parse().unwrap();
        let new = f.rewrite_uri(&original).unwrap();
        assert_eq!(new.to_string(), "https://api.anthropic.com/");
    }
}
