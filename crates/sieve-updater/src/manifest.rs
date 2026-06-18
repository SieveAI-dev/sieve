//! Manifest schema + HTTP fetch (ADR-030 §3 服务端响应格式 + §5.4).

use http::Uri;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::UpdaterError;

// ── Schema ───────────────────────────────────────────────────────────────────

/// Top-level manifest returned by `updates.sieveai.dev/v1/manifest`.
///
/// ADR-030 §3: the server may omit unknown future fields; unknown fields
/// are silently ignored here (`#[serde(deny_unknown_fields)]` is intentionally
/// NOT used so that forward-compatible additions don't break older clients).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    /// Schema version (ADR-030 §3).
    pub schema: u32,

    /// Rules bundle update info.
    pub rules: Option<RulesInfo>,

    /// Client update info.
    pub client: Option<ClientInfo>,

    /// Seconds until the client should re-check.
    ///
    /// When present, overrides `UpdaterConfig::interval_secs`.
    pub next_check_after_seconds: Option<u64>,
}

/// Information about the available rules bundle update (ADR-030 §3).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RulesInfo {
    /// Semantic version string (e.g. `"2025-05-01.1"`).
    pub version: String,
    /// Download URL for the rules bundle.
    pub url: String,
    /// Hex-encoded SHA-256 digest of the bundle bytes.
    pub sha256: String,
    /// Size in bytes.
    pub size: u64,
    /// Hex-encoded Ed25519 signature over the bundle bytes.
    pub signature: String,
}

/// Information about the available client (daemon) update (ADR-030 §3).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientInfo {
    /// Latest available version string.
    pub latest: String,
    /// Minimum supported version (older clients should warn loudly).
    pub min_supported: String,
    /// Optional human-readable deprecation notice.
    pub deprecation_notice: Option<String>,
}

// ── Fetch parameters ─────────────────────────────────────────────────────────

/// Query parameters sent with every manifest request (ADR-030 §3).
#[derive(Debug, Clone)]
pub struct ManifestParams {
    /// Current client version string (e.g. `"0.1.0-alpha"`).
    pub v: String,
    /// Target OS (`macos`, `linux`, `windows`, …).
    pub os: String,
    /// CPU architecture (`x86_64`, `aarch64`, …).
    pub arch: String,
    /// Install UUID — omitted when `None` (SIEVE_NO_TELEMETRY).
    pub uid: Option<Uuid>,
    /// Release channel (e.g. `"stable"`).
    pub ch: String,
}

// ── HTTP client ──────────────────────────────────────────────────────────────

/// Fetches the update manifest from `url` with the given parameters.
///
/// ADR-030 §5.4:
/// - TLS 1.2+ enforced by hyper-rustls (webpki roots, https_only).
/// - No cookies, no Authorization header.
/// - User-Agent: `sieve-updater/<v>`.
///
/// # Errors
/// Returns [`UpdaterError::Http`] on transport/status failures,
/// [`UpdaterError::SerdeJson`] on parse failure.
pub async fn fetch_manifest(
    url: &str,
    params: ManifestParams,
    proxy: &sieve_core::forwarder::ProxyConfig,
) -> Result<Manifest, UpdaterError> {
    let query = build_query(&params);
    let full_url = format!("{url}?{query}");
    let uri: Uri = full_url
        .parse()
        .map_err(|e| UpdaterError::Http(format!("invalid manifest URL: {e}")))?;

    let client = crate::tls::build_update_client(proxy)?;
    let req = http::Request::builder()
        .method("GET")
        .uri(uri)
        .header("User-Agent", format!("sieve-updater/{}", params.v))
        .header("Accept", "application/json")
        .body(http_body_util::Full::new(bytes::Bytes::new()))
        .map_err(|e| UpdaterError::Http(format!("build request: {e}")))?;

    let resp = client
        .request(req)
        .await
        .map_err(|e| UpdaterError::Http(format!("transport error: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(UpdaterError::Http(format!(
            "manifest server returned HTTP {status}"
        )));
    }

    let body_bytes = resp
        .into_body()
        .collect()
        .await
        .map_err(|e| UpdaterError::Http(format!("read body: {e}")))?
        .to_bytes();

    let manifest: Manifest = serde_json::from_slice(&body_bytes)?;
    Ok(manifest)
}

fn build_query(p: &ManifestParams) -> String {
    let mut parts = vec![
        format!("v={}", url_encode(&p.v)),
        format!("os={}", url_encode(&p.os)),
        format!("arch={}", url_encode(&p.arch)),
        format!("ch={}", url_encode(&p.ch)),
    ];
    if let Some(uid) = &p.uid {
        parts.push(format!("uid={uid}"));
    }
    parts.join("&")
}

fn url_encode(s: &str) -> String {
    // Minimal percent-encoding for query values (replace space/+/& etc.).
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full_manifest() {
        let json = r#"{
            "schema": 1,
            "rules": {
                "version": "2025-05-01.1",
                "url": "https://cdn.sieveai.dev/rules/2025-05-01.1.tar.zst",
                "sha256": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                "size": 102400,
                "signature": "deadbeef"
            },
            "client": {
                "latest": "0.2.0",
                "min_supported": "0.1.0",
                "deprecation_notice": null
            },
            "next_check_after_seconds": 21600
        }"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.schema, 1);
        let rules = m.rules.unwrap();
        assert_eq!(rules.version, "2025-05-01.1");
        assert_eq!(rules.size, 102400);
        let client = m.client.unwrap();
        assert_eq!(client.latest, "0.2.0");
        assert_eq!(client.min_supported, "0.1.0");
        assert!(client.deprecation_notice.is_none());
        assert_eq!(m.next_check_after_seconds, Some(21600));
    }

    #[test]
    fn deserialize_minimal_manifest() {
        let json = r#"{"schema": 1}"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.schema, 1);
        assert!(m.rules.is_none());
        assert!(m.client.is_none());
        assert!(m.next_check_after_seconds.is_none());
    }

    #[test]
    fn deserialize_missing_required_field_fails() {
        // rules.sha256 is required — but RulesInfo is nested and optional.
        // An empty rules object with missing required fields should fail.
        let json = r#"{
            "schema": 1,
            "rules": {
                "version": "2025-05-01.1",
                "url": "https://cdn.sieveai.dev/rules/latest.tar.zst"
            }
        }"#;
        // `sha256`, `size`, `signature` are missing → deserialization should fail.
        let result: Result<Manifest, _> = serde_json::from_str(json);
        assert!(
            result.is_err(),
            "missing required fields in rules must fail"
        );
    }

    #[test]
    fn unknown_top_level_fields_are_ignored() {
        // Forward-compatible: new server fields must not break old clients.
        let json = r#"{"schema": 1, "future_field": "ignored"}"#;
        let m: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.schema, 1);
    }

    #[test]
    fn build_query_omits_uid_when_none() {
        let p = ManifestParams {
            v: "0.1.0".to_string(),
            os: "macos".to_string(),
            arch: "aarch64".to_string(),
            uid: None,
            ch: "stable".to_string(),
        };
        let q = build_query(&p);
        assert!(!q.contains("uid"), "uid must not appear when None: {q}");
        assert!(q.contains("v=0.1.0"));
        assert!(q.contains("os=macos"));
        assert!(q.contains("arch=aarch64"));
        assert!(q.contains("ch=stable"));
    }

    #[test]
    fn build_query_includes_uid_when_some() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let p = ManifestParams {
            v: "0.1.0".to_string(),
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            uid: Some(id),
            ch: "stable".to_string(),
        };
        let q = build_query(&p);
        assert!(
            q.contains("uid=550e8400-e29b-41d4-a716-446655440000"),
            "uid must appear when Some: {q}"
        );
    }
}
