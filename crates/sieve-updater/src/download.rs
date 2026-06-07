//! Rules bundle download (ADR-030 §5, SPEC-006 §3.3).

use http::Uri;
use http_body_util::BodyExt;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

use crate::error::UpdaterError;

/// Default maximum payload size for a downloaded rules bundle (50 MiB).
///
/// ADR-030: rule bundles are expected to be small; reject anything larger to
/// limit memory use and guard against hostile CDN responses.
pub const DEFAULT_MAX_RULES_SIZE: usize = 50 * 1024 * 1024;

fn build_tls_client(
    proxy: &sieve_core::forwarder::ProxyConfig,
) -> Result<
    Client<
        hyper_rustls::HttpsConnector<sieve_core::forwarder::ProxyConnector>,
        http_body_util::Full<bytes::Bytes>,
    >,
    UpdaterError,
> {
    let tls = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .wrap_connector(sieve_core::forwarder::ProxyConnector::new(proxy.clone()));
    let client = Client::builder(TokioExecutor::new()).build(tls);
    Ok(client)
}

/// Downloads a rules bundle from `url` and returns the raw bytes.
///
/// ADR-030 §5 / SPEC-006 §3.3:
/// - TLS 1.2+ enforced by hyper-rustls.
/// - User-Agent: `sieve-updater/<CARGO_PKG_VERSION>`.
/// - Returns at most `max_size` bytes; exceeding it yields
///   [`UpdaterError::ResponseTooLarge`].
/// - HTTP non-200 → [`UpdaterError::Http`].
/// - Does **not** retry; call site (runner) owns retry logic.
///
/// # Errors
/// - [`UpdaterError::Http`] on transport or status errors.
/// - [`UpdaterError::ResponseTooLarge`] if the body exceeds `max_size`.
pub async fn download_rules(
    url: &str,
    max_size: usize,
    proxy: &sieve_core::forwarder::ProxyConfig,
) -> Result<Vec<u8>, UpdaterError> {
    let uri: Uri = url
        .parse()
        .map_err(|e| UpdaterError::Http(format!("invalid rules URL: {e}")))?;

    let client = build_tls_client(proxy)?;
    let version = env!("CARGO_PKG_VERSION");
    let req = http::Request::builder()
        .method("GET")
        .uri(uri)
        .header("User-Agent", format!("sieve-updater/{version}"))
        .header("Accept", "application/octet-stream")
        .body(http_body_util::Full::new(bytes::Bytes::new()))
        .map_err(|e| UpdaterError::Http(format!("build request: {e}")))?;

    let resp = client
        .request(req)
        .await
        .map_err(|e| UpdaterError::Http(format!("transport error: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(UpdaterError::Http(format!(
            "rules server returned HTTP {status}"
        )));
    }

    // Collect body frames with size guard.
    let mut collected: Vec<u8> = Vec::new();
    let mut body = resp.into_body();
    while let Some(frame) = body
        .frame()
        .await
        .transpose()
        .map_err(|e| UpdaterError::Http(format!("read body: {e}")))?
    {
        if let Ok(data) = frame.into_data() {
            let new_size = collected.len() + data.len();
            if new_size > max_size {
                return Err(UpdaterError::ResponseTooLarge {
                    size: new_size,
                    max: max_size,
                });
            }
            collected.extend_from_slice(&data);
        }
    }

    Ok(collected)
}

#[cfg(test)]
mod tests {
    use super::*;

    // `download_rules` enforces https_only via hyper-rustls, so real network
    // tests would require a TLS server.  The full download path is exercised
    // in runner integration tests (process_manifest + install flow).
    // Here we test the logic that is independently testable without a server.

    /// `ResponseTooLarge` error variant is constructed correctly.
    #[test]
    fn response_too_large_variant_fields() {
        let err = UpdaterError::ResponseTooLarge {
            size: 2048,
            max: 1024,
        };
        let msg = err.to_string();
        assert!(
            msg.contains("2048"),
            "error message must include size: {msg}"
        );
        assert!(
            msg.contains("1024"),
            "error message must include max: {msg}"
        );
    }

    /// Invalid URL returns Http error without panicking.
    #[tokio::test]
    async fn invalid_url_returns_http_error() {
        let err = download_rules(
            "not a valid url !!!",
            1024,
            &sieve_core::forwarder::ProxyConfig::Direct,
        )
        .await
            .expect_err("invalid URL must fail");
        assert!(
            matches!(err, UpdaterError::Http(_)),
            "expected Http error, got: {err:?}"
        );
    }

    /// DEFAULT_MAX_RULES_SIZE is exactly 50 MiB.
    #[test]
    fn default_max_rules_size_is_50_mib() {
        assert_eq!(DEFAULT_MAX_RULES_SIZE, 50 * 1024 * 1024);
    }
}
