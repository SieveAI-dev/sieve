//! Signature and digest verification (ADR-030 §5.5 + §待决项 #2).

use sha2::{Digest, Sha256};

use crate::error::UpdaterError;

/// Ed25519 trusted public key (32 bytes).
///
/// ADR-030 §待决项 #2: the signing key is not yet finalised. This constant
/// is intentionally `None` until the ADR-006 follow-up establishes the
/// sigstore / trusted-key distribution mechanism.
///
/// When `None`, `verify_signature` **skips** the check and emits a warning.
/// It never silently passes — the warning is mandatory.
///
/// TODO(ADR-006 follow-up + ADR-030 §待决项 #2): replace with the real 32-byte
/// ed25519 verifying key once the signing infrastructure is set up.
pub const TRUSTED_PUBKEY: Option<[u8; 32]> = None;

/// ADR-034 GA 编译期密钥 gate。
///
/// 启用 `ga_keys` feature（GA release build）时，若 [`TRUSTED_PUBKEY`] 仍为占位
/// `None`，则**编译失败**——阻止 fail-open 的规则签名校验进入 GA 二进制，兑现
/// `SECURITY.md` 验签承诺。alpha build（默认无此 feature）行为完全不变
/// （`verify_signature` 仍 skip+warn）。
#[cfg(feature = "ga_keys")]
const _: () = assert!(
    TRUSTED_PUBKEY.is_some(),
    "ga_keys feature enabled but TRUSTED_PUBKEY is None — GA build must embed a real Ed25519 verifying key (ADR-034)"
);

/// Verifies an Ed25519 signature over `data`.
///
/// ADR-030 §5.5:
/// - `TRUSTED_PUBKEY = None` → skip + warn (never silent pass).
/// - Public key present → use `ed25519-dalek` to verify.
///
/// `sig_str` is a lowercase hex-encoded 64-byte Ed25519 signature.
///
/// # Errors
/// Returns [`UpdaterError::Ed25519Failed`] if verification fails.
pub fn verify_signature(data: &[u8], sig_str: &str) -> Result<(), UpdaterError> {
    match TRUSTED_PUBKEY {
        None => {
            tracing::warn!(
                "ed25519 trusted pubkey not configured, skipping signature verification"
            );
            Ok(())
        }
        Some(pubkey_bytes) => {
            use ed25519_dalek::{Signature, VerifyingKey};

            let pubkey = VerifyingKey::from_bytes(&pubkey_bytes)
                .map_err(|e| UpdaterError::Ed25519Failed(format!("invalid public key: {e}")))?;

            let sig_bytes = hex::decode(sig_str)
                .map_err(|e| UpdaterError::Ed25519Failed(format!("invalid signature hex: {e}")))?;
            if sig_bytes.len() != 64 {
                return Err(UpdaterError::Ed25519Failed(format!(
                    "signature must be 64 bytes, got {}",
                    sig_bytes.len()
                )));
            }
            let sig_arr: [u8; 64] = sig_bytes.try_into().map_err(|_| {
                UpdaterError::Ed25519Failed("signature byte conversion failed".to_owned())
            })?;
            let signature = Signature::from_bytes(&sig_arr);

            use ed25519_dalek::Verifier;
            pubkey
                .verify(data, &signature)
                .map_err(|e| UpdaterError::Ed25519Failed(format!("verification failed: {e}")))
        }
    }
}

/// Verifies that `data` has the expected SHA-256 digest.
///
/// ADR-030 §5.5: called after downloading any bundle before applying it.
///
/// `expected_hex` is a lowercase 64-character hex string.
///
/// # Errors
/// Returns [`UpdaterError::Sha256Mismatch`] if the digest does not match.
pub fn verify_sha256(data: &[u8], expected_hex: &str) -> Result<(), UpdaterError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual = format!("{:x}", hasher.finalize());
    if actual.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(UpdaterError::Sha256Mismatch {
            expected: expected_hex.to_owned(),
            actual,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_correct_digest_passes() {
        // echo -n "hello" | sha256sum → 2cf24dba…
        let data = b"hello";
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        verify_sha256(data, expected).expect("correct sha256 must pass");
    }

    #[test]
    fn sha256_wrong_digest_fails() {
        let data = b"hello";
        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        let err = verify_sha256(data, wrong).expect_err("wrong sha256 must fail");
        assert!(
            matches!(err, UpdaterError::Sha256Mismatch { .. }),
            "expected Sha256Mismatch, got: {err:?}"
        );
    }

    /// When TRUSTED_PUBKEY is None, verify_signature must return Ok but emit
    /// a tracing warning — this is not silent pass.
    ///
    /// The tracing warn output is not asserted here (would require a tracing
    /// subscriber subscriber capture), but the Ok return is verified.
    #[test]
    fn signature_pubkey_none_skips_and_returns_ok() {
        // TRUSTED_PUBKEY is None at compile time.
        assert!(
            TRUSTED_PUBKEY.is_none(),
            "test assumes TRUSTED_PUBKEY = None"
        );
        let result = verify_signature(b"any data", "deadbeef");
        assert!(
            result.is_ok(),
            "None pubkey must return Ok (with warn): {result:?}"
        );
    }

    /// ADR-034: 默认/alpha build（无 `ga_keys` feature）下占位公钥保持 `None`，
    /// fail-open skip+warn 契约不变。`ga_keys` 启用时由本文件顶部 const 断言在
    /// 编译期阻止 `None`（无法运行时测试），故此处仅守护默认契约。
    #[test]
    fn ga_keys_gate_inactive_in_default_build() {
        #[cfg(not(feature = "ga_keys"))]
        assert!(
            TRUSTED_PUBKEY.is_none(),
            "default/alpha build must keep placeholder TRUSTED_PUBKEY = None"
        );
    }
}
