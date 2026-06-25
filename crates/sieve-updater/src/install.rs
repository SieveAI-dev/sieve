//! Atomic rules bundle installation (SPEC-006 §3.3).

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::error::UpdaterError;
use crate::signature::{verify_sha256, verify_signature_with_key};

/// Verifies, decompresses, and atomically installs a rules bundle payload.
///
/// SPEC-006 §3.3 flow:
/// 1. `verify_sha256(payload, expected_sha256)` — content integrity check.
/// 2. `verify_signature_with_key(payload, signature, trusted_key)` — ed25519
///    check against the injected trust root (WARN + skip when `trusted_key`
///    is `None`). Production passes `crate::signature::TRUSTED_PUBKEY`.
/// 3. zstd decompress → `Vec<u8>` (falls back to raw bytes if zstd magic header
///    is absent, so plain-JSON rule files work in testing).
/// 4. Write to `<dest_dir>/.tmp-<version>.json`.
/// 5. Atomic rename → `<dest_dir>/<version>.json`.
/// 6. Update `<dest_dir>/current.json` symlink (Unix) or copy (Windows).
/// 7. Atomic-write `<dest_dir>/latest_version.json` metadata.
///
/// On failure the temporary file is deleted before returning the error so no
/// partial state is left on disk.
///
/// `trusted_key` is the Ed25519 verifying key the `signature` is checked
/// against. It is injected (rather than read directly from [`crate::signature::TRUSTED_PUBKEY`])
/// so callers — production and tests — control the trust root explicitly.
///
/// # Returns
/// The path of the installed `<version>.json` file.
///
/// # Errors
/// - [`UpdaterError::Sha256Mismatch`] — digest mismatch.
/// - [`UpdaterError::Ed25519Failed`] — signature invalid.
/// - [`UpdaterError::DecompressFailed`] — zstd decompression error.
/// - [`UpdaterError::Io`] — filesystem error.
pub async fn install_rules(
    payload: &[u8],
    expected_sha256: &str,
    signature: &str,
    version: &str,
    dest_dir: &Path,
    trusted_key: Option<[u8; 32]>,
) -> Result<PathBuf, UpdaterError> {
    // Step 0: reject a path-unsafe `version` before touching the filesystem.
    // `version` is interpolated into on-disk paths (`<dest_dir>/<version>.json`,
    // `.tmp-<version>.json`, `current.json` → `<version>.json`) and originates
    // from the server-controlled manifest (`manifest.rs` `version: String`, no
    // validator). This is validated independently of signature verification so a
    // malicious / MITM manifest with `version = "../../../tmp/evil"` cannot write
    // outside `dest_dir` even if the trust root were absent. Fail-closed here.
    validate_version_component(version)?;

    // Step 1: sha256 integrity check.
    verify_sha256(payload, expected_sha256)?;

    // Step 2: ed25519 signature check against the injected trust root
    // (WARN + skip when trusted_key = None; production injects TRUSTED_PUBKEY).
    verify_signature_with_key(payload, signature, trusted_key)?;

    // Step 3: zstd decompress (fallback to raw if not a zst stream).
    let decompressed = decompress_zstd(payload)?;

    // Ensure dest_dir exists (mode 0700 on Unix).
    create_dir_secure(dest_dir)?;

    let tmp_path = dest_dir.join(format!(".tmp-{version}.json"));
    let final_path = dest_dir.join(format!("{version}.json"));

    // Step 4: write to temporary file.
    let write_result = write_tmp(&tmp_path, &decompressed).await;
    if let Err(e) = write_result {
        // Best-effort cleanup.
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(e);
    }

    // Step 5: atomic rename tmp → final.
    if let Err(e) = tokio::fs::rename(&tmp_path, &final_path).await {
        let _ = tokio::fs::remove_file(&tmp_path).await;
        return Err(UpdaterError::Io(e));
    }

    // Step 6: update current.json symlink / copy.
    update_current_symlink(dest_dir, version).await?;

    // Step 7: write latest_version.json atomically.
    write_latest_version_json(dest_dir, version, expected_sha256).await?;

    Ok(final_path)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Rejects a `version` that is not a safe single path component.
///
/// Allowed: non-empty, only ASCII `[A-Za-z0-9._-]`, and no `..` substring. This
/// admits every real version form (`2026.05.05.1`, `v1.0`, `0.1.0-alpha`) while
/// rejecting path separators (`/`, `\`), parent refs (`..`), and absolute paths
/// — i.e. anything that could escape `dest_dir` when interpolated into a path.
fn validate_version_component(version: &str) -> Result<(), UpdaterError> {
    let safe = !version.is_empty()
        && !version.contains("..")
        && version
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
    if safe {
        Ok(())
    } else {
        Err(UpdaterError::InvalidVersion(version.to_owned()))
    }
}

/// zstd frame magic number `0xFD2FB528`, stored little-endian on disk as the
/// 4 bytes `0x28 0xB5 0x2F 0xFD` (RFC 8878 §3.1.1). A real zstd stream begins
/// with exactly these bytes.
///
/// NOTE: this constant was previously byte-reversed (`FD 2F B5 28`), so the
/// magic check never matched a real zstd stream and `decompress_zstd` silently
/// fell through to the raw-bytes path — rule bundles were stored **compressed**
/// and would fail to parse. Caught by `tests/updater_e2e.rs` asserting installed
/// content equals the decompressed JSON. Fixed 2026-06-18.
const ZSTD_MAGIC: &[u8] = &[0x28, 0xB5, 0x2F, 0xFD];

/// Decompress `data` with zstd.  If `data` does not begin with the zstd magic
/// header, return `data` as-is (fallback for plain-JSON payloads in tests).
fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>, UpdaterError> {
    if data.len() >= 4 && data[..4] == *ZSTD_MAGIC {
        zstd::decode_all(data).map_err(|e| UpdaterError::DecompressFailed(e.to_string()))
    } else {
        // Not a zstd stream — treat as raw bytes (plain JSON, useful in tests).
        Ok(data.to_vec())
    }
}

/// Write `content` to `path` with mode 0644 (Unix) / default (Windows).
///
/// Uses `tokio::fs::write` for simplicity; the file is created with the
/// default umask on non-Unix platforms.  On Unix we apply 0644 after writing
/// via `std::fs::set_permissions`.
async fn write_tmp(path: &Path, content: &[u8]) -> Result<(), UpdaterError> {
    tokio::fs::write(path, content)
        .await
        .map_err(UpdaterError::Io)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o644);
        std::fs::set_permissions(path, perms).map_err(UpdaterError::Io)?;
    }

    Ok(())
}

/// Update `<dest_dir>/current.json` to point at `<version>.json`.
///
/// On Unix: unlink old symlink (if any), create new symlink.
/// On Windows: copy file (symlinks require elevated permissions).
async fn update_current_symlink(dest_dir: &Path, version: &str) -> Result<(), UpdaterError> {
    let current = dest_dir.join("current.json");
    let target_name = format!("{version}.json");

    #[cfg(unix)]
    {
        // Remove stale symlink / regular file.
        if current.exists() || current.symlink_metadata().is_ok() {
            tokio::fs::remove_file(&current)
                .await
                .map_err(UpdaterError::Io)?;
        }
        tokio::fs::symlink(&target_name, &current)
            .await
            .map_err(UpdaterError::Io)?;
    }

    #[cfg(windows)]
    {
        let source = dest_dir.join(&target_name);
        // Try symlink first; fall back to copy on permission error.
        let sym_result = tokio::fs::symlink_file(&source, &current).await;
        if sym_result.is_err() {
            tokio::fs::copy(&source, &current)
                .await
                .map_err(UpdaterError::Io)?;
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        // Unknown platform: skip symlink.
        let _ = version;
    }

    Ok(())
}

/// Atomically write `<dest_dir>/latest_version.json`.
async fn write_latest_version_json(
    dest_dir: &Path,
    version: &str,
    sha256: &str,
) -> Result<(), UpdaterError> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let unix_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let meta = json!({
        "version": version,
        "installed_at": unix_ts,
        "sha256": sha256,
    });
    let meta_bytes = serde_json::to_vec_pretty(&meta).map_err(UpdaterError::SerdeJson)?;

    let tmp_meta = dest_dir.join(".tmp-latest_version.json");
    let final_meta = dest_dir.join("latest_version.json");

    write_tmp(&tmp_meta, &meta_bytes).await?;
    tokio::fs::rename(&tmp_meta, &final_meta)
        .await
        .map_err(UpdaterError::Io)?;

    Ok(())
}

/// Create `dir` with mode 0700 on Unix, default on other platforms.
fn create_dir_secure(dir: &Path) -> Result<(), UpdaterError> {
    if dir.exists() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(dir)
            .map_err(UpdaterError::Io)?;
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(dir).map_err(UpdaterError::Io)?;
    }
    Ok(())
}

// ── Helpers exposed for runner ────────────────────────────────────────────────

/// Read the currently installed version from `<dest_dir>/latest_version.json`.
///
/// Returns `None` if the file does not exist or cannot be parsed.
pub async fn read_installed_version(dest_dir: &Path) -> Option<String> {
    let path = dest_dir.join("latest_version.json");
    let data = tokio::fs::read(&path).await.ok()?;
    let v: serde_json::Value = serde_json::from_slice(&data).ok()?;
    v["version"].as_str().map(|s| s.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use sha2::{Digest, Sha256};

    fn sha256_hex(data: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(data);
        format!("{:x}", h.finalize())
    }

    fn zstd_encode(data: &[u8]) -> Vec<u8> {
        zstd::encode_all(data, 3).expect("zstd encode")
    }

    /// Deterministic test signing key (fixed seed, no rng) so signatures are
    /// reproducible. Production never uses this; tests inject its verifying key
    /// as the trust root to exercise the real fail-closed verification path.
    fn test_signing_key() -> SigningKey {
        SigningKey::from_bytes(&[42u8; 32])
    }

    /// Returns the test trust root (verifying key bytes) to pass as
    /// `install_rules`' `trusted_key`.
    fn test_trusted_key() -> [u8; 32] {
        test_signing_key().verifying_key().to_bytes()
    }

    /// Signs `payload` with the deterministic test key, returning the
    /// lowercase-hex 64-byte signature `install_rules` expects.
    ///
    /// The signed bytes are exactly `payload` — the same bytes
    /// `install_rules` passes to `verify_signature_with_key` (the raw,
    /// pre-decompression download bytes).
    fn sign_payload(payload: &[u8]) -> String {
        hex::encode(test_signing_key().sign(payload).to_bytes())
    }

    /// Happy-path: zstd payload + correct sha256 + pubkey None (skip sig).
    #[tokio::test]
    async fn happy_path_installs_and_creates_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");

        let raw_json = br#"{"rules": []}"#;
        let payload = zstd_encode(raw_json);
        let sha = sha256_hex(&payload);
        // Sign the raw (pre-decompression) payload — exactly what install_rules verifies.
        let sig = sign_payload(&payload);

        let installed = install_rules(
            &payload,
            &sha,
            &sig,
            "2026.05.05.1",
            &dest,
            Some(test_trusted_key()),
        )
        .await
        .expect("install must succeed");

        assert!(installed.exists(), "final file must exist");
        assert_eq!(
            installed,
            dest.join("2026.05.05.1.json"),
            "filename must match version"
        );

        // 回归（2026-06-18 ZSTD_MAGIC 字节序 bug）：安装的文件必须是**解压后**的原始
        // JSON，而非 zstd 压缩字节。此前 magic 反序致 decompress_zstd 走 raw fallback，
        // 把压缩流原样写盘，sieve-rules 加载必失败。
        assert_eq!(
            std::fs::read(&installed).unwrap(),
            raw_json,
            "installed content must be the DECOMPRESSED rule JSON, not the zstd payload"
        );

        // current.json symlink must point at 2026.05.05.1.json.
        let current = dest.join("current.json");
        assert!(
            current.exists() || current.symlink_metadata().is_ok(),
            "current.json must exist"
        );

        // latest_version.json must be present and contain correct version.
        let lv: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dest.join("latest_version.json")).unwrap())
                .unwrap();
        assert_eq!(lv["version"], "2026.05.05.1");
        assert_eq!(lv["sha256"], sha);
    }

    /// sha256 mismatch → error + no tmp file left.
    #[tokio::test]
    async fn sha256_mismatch_returns_error_and_cleans_up() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");

        let payload = zstd_encode(b"some content");
        let wrong_sha = "0".repeat(64);

        // sha256 is checked before signature, so this fails at the digest step;
        // the signature value is irrelevant (verification is never reached).
        let err = install_rules(
            &payload,
            &wrong_sha,
            "sig",
            "v1",
            &dest,
            Some(test_trusted_key()),
        )
        .await
        .expect_err("must fail on sha256 mismatch");
        assert!(
            matches!(err, UpdaterError::Sha256Mismatch { .. }),
            "wrong variant: {err:?}"
        );

        // No tmp file should remain.
        assert!(
            !dest.join(".tmp-v1.json").exists(),
            "tmp file must be cleaned up"
        );
    }

    /// Non-zstd bytes (plain JSON fallback) should install successfully.
    #[tokio::test]
    async fn plain_json_fallback_installs() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");

        let payload = br#"{"rules":[]}"#.to_vec();
        let sha = sha256_hex(&payload);
        let sig = sign_payload(&payload);

        install_rules(
            &payload,
            &sha,
            &sig,
            "plain.1",
            &dest,
            Some(test_trusted_key()),
        )
        .await
        .expect("plain JSON fallback must succeed");

        assert!(dest.join("plain.1.json").exists());
    }

    /// Deliberately corrupt zstd data → DecompressFailed.
    #[tokio::test]
    async fn bad_zstd_returns_decompress_failed() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");

        // Starts with zstd magic but rest is garbage.
        let mut payload = ZSTD_MAGIC.to_vec();
        payload.extend_from_slice(&[0xFF; 64]);
        let sha = sha256_hex(&payload);
        // Valid signature so verification passes and the flow reaches the
        // decompress step, which is where this test expects the failure.
        let sig = sign_payload(&payload);

        let err = install_rules(
            &payload,
            &sha,
            &sig,
            "v-bad",
            &dest,
            Some(test_trusted_key()),
        )
        .await
        .expect_err("must fail on bad zstd");
        assert!(
            matches!(err, UpdaterError::DecompressFailed(_)),
            "wrong variant: {err:?}"
        );
    }

    /// Second install of newer version updates symlink, old file preserved.
    #[tokio::test]
    async fn second_install_updates_symlink_and_preserves_old_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");

        let mk = |content: &[u8]| -> (Vec<u8>, String, String) {
            let p = zstd_encode(content);
            let s = sha256_hex(&p);
            let sig = sign_payload(&p);
            (p, s, sig)
        };

        let (p1, s1, sig1) = mk(b"version1");
        install_rules(&p1, &s1, &sig1, "v1.0", &dest, Some(test_trusted_key()))
            .await
            .expect("first install");

        let (p2, s2, sig2) = mk(b"version2");
        install_rules(&p2, &s2, &sig2, "v2.0", &dest, Some(test_trusted_key()))
            .await
            .expect("second install");

        // Old file must still exist (for rollback).
        assert!(dest.join("v1.0.json").exists(), "v1.0.json must be kept");
        // New file exists.
        assert!(dest.join("v2.0.json").exists(), "v2.0.json must exist");

        // latest_version.json must reflect v2.0.
        let lv: serde_json::Value =
            serde_json::from_slice(&std::fs::read(dest.join("latest_version.json")).unwrap())
                .unwrap();
        assert_eq!(lv["version"], "v2.0");
    }

    /// Path-traversal / path-unsafe `version` must be rejected fail-closed
    /// before any filesystem work (server-controlled field; sig is fail-open).
    #[tokio::test]
    async fn rejects_path_unsafe_version() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");
        let payload = zstd_encode(b"{}");
        let sha = sha256_hex(&payload);

        for bad in [
            "../../../tmp/evil",
            "..",
            "a/b",
            "a\\b",
            "",
            "x/../y",
            "/abs",
            "..foo",
        ] {
            // version is validated first (before sha256 and signature), so this
            // fails at step 0; the signature value is never reached.
            let err = install_rules(&payload, &sha, "sig", bad, &dest, Some(test_trusted_key()))
                .await
                .expect_err("path-unsafe version must be rejected");
            assert!(
                matches!(err, UpdaterError::InvalidVersion(_)),
                "version {bad:?} must be InvalidVersion, got {err:?}"
            );
        }

        // A rejected install must not have created dest_dir or written anything.
        assert!(
            !dest.exists(),
            "dest_dir must not be created for a rejected version"
        );
    }

    /// Real-world version forms must remain valid (guard must not over-reject).
    #[test]
    fn validate_version_accepts_real_versions() {
        for ok in [
            "2026.05.05.1",
            "v1.0",
            "0.1.0-alpha",
            "v2.0",
            "plain.1",
            "2026.01.01.1",
        ] {
            validate_version_component(ok)
                .unwrap_or_else(|e| panic!("{ok:?} should be valid: {e:?}"));
        }
    }

    /// `read_installed_version` returns None when no metadata file exists.
    #[tokio::test]
    async fn read_installed_version_none_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");
        assert!(read_installed_version(&dest).await.is_none());
    }

    /// `read_installed_version` returns the correct version after install.
    #[tokio::test]
    async fn read_installed_version_returns_correct_version() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("rules");

        let payload = zstd_encode(b"{}");
        let sha = sha256_hex(&payload);
        let sig = sign_payload(&payload);
        install_rules(
            &payload,
            &sha,
            &sig,
            "2026.01.01.1",
            &dest,
            Some(test_trusted_key()),
        )
        .await
        .unwrap();

        let ver = read_installed_version(&dest).await;
        assert_eq!(ver.as_deref(), Some("2026.01.01.1"));
    }
}
