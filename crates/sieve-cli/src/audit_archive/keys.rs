//! `full` 档密钥生命周期（ADR-037 决策 3/5/6 / SPEC-009 §4）。
//!
//! write-only logging：daemon **只持公钥 recipient**（加密能力），**私钥 identity 离线**
//! 用口令（age 原生 scrypt）保护。本模块负责：①生成密钥对 ②口令保护/解锁私钥。
//!
//! **口令丢失 = 归档永久不可读（by design）**——见 [`generate`] 的警示约定。

use age::secrecy::{ExposeSecret, SecretString};
use age::x25519::{Identity, Recipient};
use anyhow::{anyhow, Context, Result};
use std::str::FromStr;

/// 一对新生成的密钥：公钥（写 config）+ 口令保护后的私钥密文（写离线文件）。
pub struct GeneratedKeyPair {
    /// age recipient 公钥字符串（`age1...`）——写入 `config.toml [audit].recipient`。
    pub recipient: String,
    /// 口令（scrypt）保护后的 identity 私钥密文——写 0600 文件，用户移出本机。
    pub protected_identity: Vec<u8>,
}

/// 生成 X25519 密钥对，并用口令（scrypt）保护私钥。
///
/// daemon 永不接触私钥——只拿 `recipient`。`protected_identity` 是私钥 bech32 串
/// 经 age scrypt 加密的密文；**口令一旦丢失，私钥无法解锁，归档永久不可读**。
pub fn generate(passphrase: &SecretString) -> Result<GeneratedKeyPair> {
    if passphrase.expose_secret().is_empty() {
        return Err(anyhow!("passphrase must not be empty"));
    }
    let identity = Identity::generate();
    let recipient = identity.to_public().to_string();

    // identity 的 bech32 私钥串（"AGE-SECRET-KEY-1..."），用 scrypt 口令加密保护。
    let secret_str = identity.to_string();
    let protected_identity = age::encrypt(
        &age::scrypt::Recipient::new(passphrase.clone()),
        secret_str.expose_secret().as_bytes(),
    )
    .context("scrypt-encrypt identity for at-rest protection")?;

    Ok(GeneratedKeyPair {
        recipient,
        protected_identity,
    })
}

/// 用口令解锁离线私钥密文，还原 [`Identity`]（仅审计解密时调用，daemon 不调）。
///
/// 口令错误 → 返回错误（无法区分「口令错」与「文件损坏」，均不可读，by design）。
pub fn unlock_identity(protected_identity: &[u8], passphrase: &SecretString) -> Result<Identity> {
    let secret_bytes = age::decrypt(
        &age::scrypt::Identity::new(passphrase.clone()),
        protected_identity,
    )
    .map_err(|e| anyhow!("无法解锁私钥（口令错误或文件损坏，归档不可读）: {e}"))?;
    let secret_str = String::from_utf8(secret_bytes).context("identity 私钥不是合法 UTF-8")?;
    Identity::from_str(secret_str.trim()).map_err(|e| anyhow!("解锁后的 identity 解析失败: {e}"))
}

/// 校验一个字符串是合法 age recipient 公钥（config fail-fast 之外的二次确认）。
pub fn parse_recipient(s: &str) -> Result<Recipient> {
    Recipient::from_str(s.trim()).map_err(|e| anyhow!("非法 age recipient 公钥 {s:?}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_then_unlock_roundtrip() {
        let pass = SecretString::from("correct horse battery staple".to_owned());
        let kp = generate(&pass).expect("generate keypair");
        assert!(kp.recipient.starts_with("age1"), "recipient 应为 age1 公钥");
        // 公钥可解析。
        parse_recipient(&kp.recipient).expect("recipient parses");
        // 正确口令可解锁。
        let id = unlock_identity(&kp.protected_identity, &pass).expect("unlock");
        // 解锁出的 identity 公钥 == 原 recipient（同一密钥对）。
        assert_eq!(id.to_public().to_string(), kp.recipient);
    }

    #[test]
    fn wrong_passphrase_cannot_unlock() {
        let pass = SecretString::from("right-pass".to_owned());
        let kp = generate(&pass).expect("generate");
        let wrong = SecretString::from("wrong-pass".to_owned());
        assert!(
            unlock_identity(&kp.protected_identity, &wrong).is_err(),
            "错误口令必须无法解锁（口令丢失=永久不可读 by design）"
        );
    }

    #[test]
    fn protected_identity_does_not_leak_secret_bytes() {
        // 离线私钥密文中不得出现明文 bech32 私钥前缀。
        let pass = SecretString::from("p".to_owned());
        let kp = generate(&pass).expect("generate");
        let haystack = String::from_utf8_lossy(&kp.protected_identity);
        assert!(
            !haystack.contains("AGE-SECRET-KEY-1"),
            "口令保护后的私钥密文不得含明文私钥"
        );
    }

    #[test]
    fn empty_passphrase_rejected() {
        let pass = SecretString::from(String::new());
        assert!(generate(&pass).is_err());
    }
}
