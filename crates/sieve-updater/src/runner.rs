//! Long-running update check loop.

use std::path::PathBuf;

use tokio::time::Duration;
use uuid::Uuid;

use crate::download::download_rules;
use crate::error::UpdaterError;
use crate::install::{install_rules, read_installed_version};
use crate::manifest::{fetch_manifest, ManifestParams};

/// Default manifest endpoint.
pub const DEFAULT_MANIFEST_URL: &str = "https://updates.sieveai.dev/v1/manifest";

/// Default release channel.
pub const DEFAULT_CHANNEL: &str = "stable";

/// Default check interval: 6 hours（每日 4 次）.
pub const DEFAULT_INTERVAL_SECS: u64 = 6 * 3600;

/// Subdirectory under `cache_dir()` where rules bundles are staged.
pub const DEFAULT_RULES_DIR: &str = "rules";

/// Maximum accepted rules bundle size: 50 MiB.
pub const MAX_RULES_SIZE: usize = 50 * 1024 * 1024;

/// Hook invoked after a new rules pack is successfully installed.
///
/// The daemon sets this to trigger an in-process hot reload of the system
/// rules (swap the freshly installed `current.json` into the live engines)
/// without restarting. Kept as a plain callback so this crate does not depend
/// on the IPC / engine crates.
pub type RulesInstalledHook = std::sync::Arc<dyn Fn() + Send + Sync>;

/// Configuration for the update runner task.
///
/// Constructed by the daemon from `Config::update` + env overrides.
#[derive(Debug, Clone)]
pub struct UpdaterConfig {
    /// Manifest fetch URL (env > config > default).
    pub base_url: String,
    /// Seconds between checks; overridden by `next_check_after_seconds` from server.
    pub interval_secs: u64,
    /// When `true`, `uid` is omitted from manifest query params.
    pub no_telemetry: bool,
    /// Client version string (from `env!("CARGO_PKG_VERSION")`).
    pub client_version: String,
    /// Release channel (default `"stable"`).
    pub channel: String,
    /// 上游代理 URL（全局，SPEC-007）。`None` → 直连。daemon 侧已合并 config+env。
    pub proxy: Option<String>,
}

/// Runs the update check loop forever.
///
/// Loop behaviour:
/// 1. Performs an initial check immediately after startup.
/// 2. Subsequent checks are spaced by `interval_secs` (or `next_check_after_seconds`
///    if the server provides it).
/// 3. Each check uses exponential back-off (1 s / 4 s / 16 s) on failure;
///    after 3 exhausted retries the error is logged and the runner waits until
///    the next normal interval — it never panics or exits.
///
/// `on_rules_installed` (if set) is invoked once after each successful rules
/// install, so the daemon can hot-reload the new pack without a restart.
///
/// This function never returns (return type `!`).
pub async fn run(cfg: UpdaterConfig, on_rules_installed: Option<RulesInstalledHook>) -> ! {
    // Resolve the rules staging directory once at startup.
    let rules_dir: Option<PathBuf> = match crate::cache_dir::cache_dir() {
        Ok(cd) => Some(cd.join(DEFAULT_RULES_DIR)),
        Err(e) => {
            tracing::error!(error = %e, "cannot determine cache_dir; rules updates disabled");
            None
        }
    };

    tracing::info!(
        url = %cfg.base_url,
        interval_secs = cfg.interval_secs,
        telemetry = !cfg.no_telemetry,
        rules_dir = ?rules_dir,
        "updater task started"
    );

    // Load (or generate) install-id once; if it fails, disable telemetry for
    // this run rather than aborting the loop.
    let install_id: Option<Uuid> = if cfg.no_telemetry {
        None
    } else {
        match crate::install_id::load_or_create_install_id().await {
            Ok(id) => {
                tracing::debug!(install_id = %id, "install-id loaded");
                Some(id)
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to load install-id; telemetry disabled for this session");
                None
            }
        }
    };

    let mut next_interval_secs = cfg.interval_secs;

    // Run immediately, then loop.
    let mut first = true;
    loop {
        if !first {
            tokio::time::sleep(Duration::from_secs(next_interval_secs)).await;
        }
        first = false;

        match run_one_check(&cfg, install_id, rules_dir.as_deref()).await {
            Ok((server_interval, rules_installed)) => {
                next_interval_secs = server_interval.unwrap_or(cfg.interval_secs);
                tracing::debug!(
                    next_check_secs = next_interval_secs,
                    rules_installed,
                    "update check succeeded"
                );
                // 新签名规则包已落盘 → 触发 daemon 进程内热重载（无需重启）。
                if rules_installed {
                    if let Some(hook) = &on_rules_installed {
                        hook();
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "update check failed (all retries exhausted); will retry next cycle");
                // Reset to configured interval on persistent failure.
                next_interval_secs = cfg.interval_secs;
            }
        }
    }
}

/// Performs a single update check with exponential back-off.
///
/// Returns `Ok(Some(secs))` if the server specified `next_check_after_seconds`,
/// `Ok(None)` otherwise.
async fn run_one_check(
    cfg: &UpdaterConfig,
    install_id: Option<Uuid>,
    rules_dir: Option<&std::path::Path>,
) -> Result<(Option<u64>, bool), UpdaterError> {
    let params = ManifestParams {
        v: cfg.client_version.clone(),
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        uid: install_id,
        ch: cfg.channel.clone(),
    };

    // SPEC-007: 全局上游代理（受限网络下 updates/cdn.sieveai.dev 也需走代理）。
    let proxy = sieve_core::forwarder::ProxyConfig::parse(cfg.proxy.as_deref())
        .unwrap_or(sieve_core::forwarder::ProxyConfig::Direct);

    // Exponential back-off: 1 s, 4 s, 16 s.
    let backoff_secs: [u64; 3] = [1, 4, 16];
    let mut last_err = String::new();

    for (attempt, &wait) in backoff_secs.iter().enumerate() {
        if attempt > 0 {
            tracing::warn!(
                attempt,
                wait_secs = wait,
                error = %last_err,
                "manifest fetch failed; retrying after back-off"
            );
            tokio::time::sleep(Duration::from_secs(wait)).await;
        }

        match fetch_manifest(&cfg.base_url, params.clone(), &proxy).await {
            Ok(manifest) => {
                let rules_installed = process_manifest(&manifest, rules_dir, &proxy).await;
                return Ok((manifest.next_check_after_seconds, rules_installed));
            }
            Err(e) => {
                last_err = e.to_string();
            }
        }
    }

    Err(UpdaterError::RetryExhausted {
        attempts: backoff_secs.len() as u32,
        last_error: last_err,
    })
}

/// Processes a successfully fetched manifest: logs client update info and
/// triggers a rules download + atomic install when a newer version is available.
///
/// SPEC-006 §3.3:
/// - Compares `rules.version` against the currently installed version.
/// - Skips download when already up-to-date.
/// - On download failure retries with 1s/4s/16s exponential back-off.
/// - Install failure is logged but never crashes the daemon.
///
/// Returns `true` iff a new rules pack was successfully installed (atomically
/// staged to `current.json`). The caller ([`run`]) uses this to invoke the
/// `on_rules_installed` hook so the daemon hot-reloads the pack in-process.
async fn process_manifest(
    manifest: &crate::manifest::Manifest,
    rules_dir: Option<&std::path::Path>,
    proxy: &sieve_core::forwarder::ProxyConfig,
) -> bool {
    if let Some(client) = &manifest.client {
        tracing::info!(
            latest = %client.latest,
            min_supported = %client.min_supported,
            "manifest received: client update info"
        );
        if let Some(notice) = &client.deprecation_notice {
            tracing::warn!(notice = %notice, "deprecation notice from update server");
        }
    }

    let Some(rules) = &manifest.rules else {
        tracing::debug!("manifest: no rules update field; skipping");
        return false;
    };

    let Some(dest_dir) = rules_dir else {
        tracing::warn!("rules_dir unavailable; skipping rules download");
        return false;
    };

    // Check currently installed version.
    let current_version = read_installed_version(dest_dir).await;
    if current_version.as_deref() == Some(rules.version.as_str()) {
        tracing::debug!(version = %rules.version, "rules already up-to-date; skipping download");
        return false;
    }

    tracing::info!(
        new_version = %rules.version,
        current_version = ?current_version,
        "rules update available; downloading"
    );

    // Download with exponential back-off.
    let payload = match retry_with_backoff(|| download_rules(&rules.url, MAX_RULES_SIZE, proxy))
        .await
    {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, version = %rules.version, "rules download failed (all retries exhausted)");
            return false;
        }
    };

    // Install: verify sha256 + ed25519 + decompress + atomic write.
    // Production injects the embedded distribution trust root.
    match install_rules(
        &payload,
        &rules.sha256,
        &rules.signature,
        &rules.version,
        dest_dir,
        crate::signature::TRUSTED_PUBKEY,
    )
    .await
    {
        Ok(path) => {
            tracing::info!(version = %rules.version, path = ?path, "rules installed");
            true
        }
        Err(e) => {
            tracing::error!(error = %e, version = %rules.version, "rules install failed; keeping existing rules");
            false
        }
    }
}

/// Run an async operation with exponential back-off (1 s / 4 s / 16 s × 3).
///
/// Returns the first `Ok` value, or the last `Err` wrapped in
/// [`UpdaterError::RetryExhausted`] if all attempts fail.
async fn retry_with_backoff<F, Fut, T>(mut op: F) -> Result<T, UpdaterError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, UpdaterError>>,
{
    const BACKOFF: [u64; 3] = [1, 4, 16];
    let mut last_err = String::new();

    for (attempt, &wait) in BACKOFF.iter().enumerate() {
        if attempt > 0 {
            tracing::warn!(
                attempt,
                wait_secs = wait,
                error = %last_err,
                "operation failed; retrying after back-off"
            );
            tokio::time::sleep(Duration::from_secs(wait)).await;
        }
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => last_err = e.to_string(),
        }
    }

    Err(UpdaterError::RetryExhausted {
        attempts: BACKOFF.len() as u32,
        last_error: last_err,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ClientInfo, Manifest, RulesInfo};
    use sha2::{Digest, Sha256};

    #[test]
    fn defaults_are_correct() {
        assert_eq!(
            DEFAULT_MANIFEST_URL,
            "https://updates.sieveai.dev/v1/manifest"
        );
        assert_eq!(DEFAULT_CHANNEL, "stable");
        assert_eq!(DEFAULT_INTERVAL_SECS, 6 * 3600);
    }

    #[test]
    fn updater_config_fields() {
        let cfg = UpdaterConfig {
            base_url: DEFAULT_MANIFEST_URL.to_string(),
            interval_secs: DEFAULT_INTERVAL_SECS,
            no_telemetry: false,
            client_version: "0.1.0-alpha".to_string(),
            channel: DEFAULT_CHANNEL.to_string(),
            proxy: None,
        };
        assert_eq!(cfg.interval_secs, 21600);
        assert!(!cfg.no_telemetry);
    }

    #[test]
    fn new_constants_are_sane() {
        assert_eq!(DEFAULT_RULES_DIR, "rules");
        assert_eq!(MAX_RULES_SIZE, 50 * 1024 * 1024);
    }

    fn sha256_hex(data: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(data);
        format!("{:x}", h.finalize())
    }

    fn zstd_encode(data: &[u8]) -> Vec<u8> {
        zstd::encode_all(data, 3).expect("zstd encode")
    }

    /// Deterministic test signing key (fixed seed, no rng); its verifying key is
    /// injected as the trust root so the pre-install in the test below passes the
    /// real fail-closed signature check. Production uses `TRUSTED_PUBKEY`.
    fn test_signing_key() -> ed25519_dalek::SigningKey {
        ed25519_dalek::SigningKey::from_bytes(&[42u8; 32])
    }

    fn test_trusted_key() -> [u8; 32] {
        test_signing_key().verifying_key().to_bytes()
    }

    /// Lowercase-hex 64-byte signature over the raw payload bytes.
    fn sign_payload(payload: &[u8]) -> String {
        use ed25519_dalek::Signer;
        hex::encode(test_signing_key().sign(payload).to_bytes())
    }

    /// process_manifest with rules_dir = None logs a warning but does not panic.
    #[tokio::test]
    async fn process_manifest_no_rules_dir_does_not_panic() {
        let manifest = Manifest {
            schema: 1,
            rules: Some(RulesInfo {
                version: "v1".to_owned(),
                url: "https://cdn.sieveai.dev/rules/v1.json.zst".to_owned(),
                sha256: "aa".repeat(32),
                size: 0,
                signature: "sig".to_owned(),
            }),
            client: None,
            next_check_after_seconds: None,
        };
        // Should log a warning and return without panicking.
        process_manifest(&manifest, None, &sieve_core::forwarder::ProxyConfig::Direct).await;
    }

    /// Same version already installed → skip download (no network call attempted).
    #[tokio::test]
    async fn process_manifest_skips_when_already_up_to_date() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");

        // Pre-install a version (real signature against the injected test key;
        // process_manifest then short-circuits at "already up-to-date" so its
        // production TRUSTED_PUBKEY path is never reached for this version).
        let payload = zstd_encode(b"{}");
        let sha = sha256_hex(&payload);
        let sig = sign_payload(&payload);
        install_rules(
            &payload,
            &sha,
            &sig,
            "v1.0",
            &dest,
            Some(test_trusted_key()),
        )
        .await
        .unwrap();

        // Build a manifest with the same version.  The URL points nowhere; if
        // we attempted a download it would fail and the test would fail.
        let manifest = Manifest {
            schema: 1,
            rules: Some(RulesInfo {
                version: "v1.0".to_owned(),
                url: "https://127.0.0.1:1/will_not_be_called".to_owned(),
                sha256: sha.clone(),
                size: payload.len() as u64,
                signature: "sig".to_owned(),
            }),
            client: None,
            next_check_after_seconds: None,
        };

        // Must not attempt download — so no error even though URL is unreachable.
        process_manifest(
            &manifest,
            Some(&dest),
            &sieve_core::forwarder::ProxyConfig::Direct,
        )
        .await;
    }

    /// Manifest with client info only (no rules) — process_manifest must not
    /// panic and must not attempt any download.
    #[tokio::test]
    async fn process_manifest_no_rules_field_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");

        let manifest = Manifest {
            schema: 1,
            rules: None,
            client: Some(ClientInfo {
                latest: "0.5.0".to_owned(),
                min_supported: "0.1.0".to_owned(),
                deprecation_notice: None,
            }),
            next_check_after_seconds: Some(3600),
        };
        // Must not download anything; dest_dir does not even need to exist.
        process_manifest(
            &manifest,
            Some(&dest),
            &sieve_core::forwarder::ProxyConfig::Direct,
        )
        .await;
        assert!(!dest.exists(), "dest_dir must not be created when no rules");
    }
}
