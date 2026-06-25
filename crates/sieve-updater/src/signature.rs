//! Signature and digest verification for the rules update channel.

use sha2::{Digest, Sha256};

use crate::error::UpdaterError;

/// Ed25519 trusted public key (32 bytes) for rules-package verification.
///
/// This is the long-lived distribution trust root: the matching private key
/// signs every rules package and clients verify against this key fail-closed.
///
/// When `None`, `verify_signature` **skips** the check and emits a warning
/// (alpha builds without the `ga_keys` feature). It never silently passes —
/// the warning is mandatory.
pub const TRUSTED_PUBKEY: Option<[u8; 32]> = Some([
    0x1a, 0x27, 0x99, 0xc7, 0x65, 0x2e, 0x2e, 0x17, 0x2a, 0x46, 0xdc, 0xe2, 0xa8, 0x29, 0x34, 0x59,
    0xe1, 0xce, 0x65, 0x1e, 0xfd, 0x5a, 0x80, 0xa6, 0x3a, 0xd1, 0x8d, 0xc0, 0x20, 0xd7, 0x4c, 0x04,
]);

/// GA 编译期密钥 gate。
///
/// 启用 `ga_keys` feature（GA release build）时，若 [`TRUSTED_PUBKEY`] 仍为占位
/// `None`，则**编译失败**——阻止 fail-open 的规则签名校验进入 GA 二进制，兑现
/// `SECURITY.md` 验签承诺。alpha build（默认无此 feature）行为完全不变
/// （`verify_signature` 仍 skip+warn）。
#[cfg(feature = "ga_keys")]
const _: () = assert!(
    TRUSTED_PUBKEY.is_some(),
    "ga_keys feature enabled but TRUSTED_PUBKEY is None — GA build must embed a real Ed25519 verifying key"
);

/// Verifies an Ed25519 signature over `data`.
///
/// Trust-root behaviour:
/// - `TRUSTED_PUBKEY = None` → skip + warn (never silent pass).
/// - Public key present → use `ed25519-dalek` to verify.
///
/// `sig_str` is a lowercase hex-encoded 64-byte Ed25519 signature.
///
/// # Errors
/// Returns [`UpdaterError::Ed25519Failed`] if verification fails.
pub fn verify_signature(data: &[u8], sig_str: &str) -> Result<(), UpdaterError> {
    verify_signature_with_key(data, sig_str, TRUSTED_PUBKEY)
}

/// 可注入信任公钥的验签实现（生产经 [`verify_signature`] 固定用 [`TRUSTED_PUBKEY`]）。
///
/// 抽出 `trusted_pubkey` 参数让调用方（[`crate::install::install_rules`]）显式注入信任根，
/// 生产固定传 [`TRUSTED_PUBKEY`]（已嵌入真公钥，验签 fail-closed 生效），测试以确定性测试
/// 密钥注入 `Some(测试公钥)` 覆盖 fail-closed 分支。生产语义不变。
pub(crate) fn verify_signature_with_key(
    data: &[u8],
    sig_str: &str,
    trusted_pubkey: Option<[u8; 32]>,
) -> Result<(), UpdaterError> {
    match trusted_pubkey {
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
/// Called after downloading any bundle before applying it.
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

    /// When the trusted key is `None`, verification must return Ok but emit a
    /// tracing warning — not a silent pass. Production now pins a real
    /// `TRUSTED_PUBKEY`, so this skip path is exercised by passing `None`
    /// explicitly rather than relying on the embedded constant.
    ///
    /// The warn output is not asserted (would need a tracing subscriber
    /// capture); the Ok return is.
    #[test]
    fn signature_pubkey_none_skips_and_returns_ok() {
        let result = verify_signature_with_key(b"any data", "deadbeef", None);
        assert!(
            result.is_ok(),
            "None pubkey must return Ok (with warn): {result:?}"
        );
    }

    /// 信任根已嵌入真公钥——`TRUSTED_PUBKEY = Some(真根)`，验签 fail-closed
    /// 生效（不再是占位 `None` 的 skip+warn）。守护：改回 `None` 会让此测试红；
    /// `ga_keys` 启用时文件顶部 const 断言在编译期同样保证非 `None`。
    #[test]
    fn trust_root_is_embedded() {
        assert!(
            TRUSTED_PUBKEY.is_some(),
            "distribution trust root must embed a real Ed25519 verifying key, not None"
        );
    }

    // ── Some(真公钥) fail-closed 分支回归 ──────────────────────────────
    //
    // 生产 TRUSTED_PUBKEY 已是 Some(真根)，Some 分支即生产路径。这组单测用确定性
    // 测试密钥（from_bytes，不依赖 rng）注入 verify_signature_with_key 的 Some 分支，
    // 守护「真公钥下验签必须 fail-closed」。

    /// 真公钥 + 正确签名 → 通过。
    #[test]
    fn signature_valid_with_real_key_passes() {
        use ed25519_dalek::{Signer, SigningKey};
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let pubkey = signing.verifying_key().to_bytes();
        let data = b"rule bundle payload";
        let sig_hex = hex::encode(signing.sign(data).to_bytes());
        verify_signature_with_key(data, &sig_hex, Some(pubkey))
            .expect("valid signature with real key must pass");
    }

    /// 真公钥 + 篡改数据 → fail-closed 拒绝（核心安全契约）。
    #[test]
    fn signature_tampered_data_rejected() {
        use ed25519_dalek::{Signer, SigningKey};
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let pubkey = signing.verifying_key().to_bytes();
        let sig_hex = hex::encode(signing.sign(b"original payload").to_bytes());
        let err = verify_signature_with_key(b"tampered payload", &sig_hex, Some(pubkey))
            .expect_err("tampered data must be rejected (fail-closed)");
        assert!(
            matches!(err, UpdaterError::Ed25519Failed(_)),
            "got: {err:?}"
        );
    }

    /// 用错误公钥验真签名 → 拒绝（防止换公钥绕过）。
    #[test]
    fn signature_wrong_key_rejected() {
        use ed25519_dalek::{Signer, SigningKey};
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let other_pubkey = SigningKey::from_bytes(&[9u8; 32])
            .verifying_key()
            .to_bytes();
        let data = b"rule bundle payload";
        let sig_hex = hex::encode(signing.sign(data).to_bytes());
        let err = verify_signature_with_key(data, &sig_hex, Some(other_pubkey))
            .expect_err("signature under different key must be rejected");
        assert!(
            matches!(err, UpdaterError::Ed25519Failed(_)),
            "got: {err:?}"
        );
    }

    /// 真公钥 + 非法 hex 签名 → 拒绝。
    #[test]
    fn signature_bad_hex_rejected_with_real_key() {
        use ed25519_dalek::SigningKey;
        let pubkey = SigningKey::from_bytes(&[7u8; 32])
            .verifying_key()
            .to_bytes();
        let err = verify_signature_with_key(b"data", "not-valid-hex!!", Some(pubkey))
            .expect_err("invalid signature hex must be rejected");
        assert!(
            matches!(err, UpdaterError::Ed25519Failed(_)),
            "got: {err:?}"
        );
    }

    /// 真公钥 + 非 64 字节签名 → 拒绝。
    #[test]
    fn signature_wrong_length_rejected_with_real_key() {
        use ed25519_dalek::SigningKey;
        let pubkey = SigningKey::from_bytes(&[7u8; 32])
            .verifying_key()
            .to_bytes();
        let err = verify_signature_with_key(b"data", "deadbeef", Some(pubkey))
            .expect_err("non-64-byte signature must be rejected");
        assert!(
            matches!(err, UpdaterError::Ed25519Failed(_)),
            "got: {err:?}"
        );
    }
}
