//! 更新通道共享 TLS client 构造（manifest fetch + rules download 复用）。
//!
//! **安全不变量**：GA/release 二进制**无条件** `https_only()`，
//! 唯一允许的出站是 `updates.sieveai.dev` / `cdn.sieveai.dev` 的 TLS。
//!
//! 为支持 hermetic 集成测试（把 updater 指向 localhost mock 而不做 TLS 握手），
//! **仅 debug 构建**在 `SIEVE_UPDATE_ALLOW_HTTP` 置位时改用 `https_or_http()` 允许明文 HTTP。
//! 该放宽分支 `#[cfg(debug_assertions)]`，在 release/GA 构建里**编译期消除**——
//! 生产二进制根本不存在降级到明文的代码路径。

use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

use crate::error::UpdaterError;

/// 更新通道 HTTP(S) client 类型别名（webpki 根 + ProxyConnector，SPEC-007）。
pub(crate) type UpdateClient = Client<
    hyper_rustls::HttpsConnector<sieve_core::forwarder::ProxyConnector>,
    http_body_util::Full<bytes::Bytes>,
>;

/// 是否允许明文 HTTP 出站。release/GA 恒 `false`（编译期），仅 debug 读测试 env。
#[cfg(debug_assertions)]
fn allow_plain_http() -> bool {
    std::env::var("SIEVE_UPDATE_ALLOW_HTTP")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
}

#[cfg(not(debug_assertions))]
#[inline]
fn allow_plain_http() -> bool {
    false
}

/// 构造更新通道 client（webpki 根 + https_only + ProxyConnector）。
///
/// 见模块文档：debug 构建 + `SIEVE_UPDATE_ALLOW_HTTP` 时放宽为 https_or_http（仅测试用）。
pub(crate) fn build_update_client(
    proxy: &sieve_core::forwarder::ProxyConfig,
) -> Result<UpdateClient, UpdaterError> {
    let schemes = hyper_rustls::HttpsConnectorBuilder::new().with_webpki_roots();
    // `https_only()` 与 `https_or_http()` 在 hyper-rustls 0.27 返回同一 typestate，可分支。
    let protocols = if allow_plain_http() {
        schemes.https_or_http()
    } else {
        schemes.https_only()
    };
    let tls = protocols
        .enable_http1()
        .wrap_connector(sieve_core::forwarder::ProxyConnector::new(proxy.clone()));
    Ok(Client::builder(TokioExecutor::new()).build(tls))
}
