//! Ed25519 规则包签名验证（Week 1 占位，Week 5 起做实际下发验证）。

use crate::error::{SieveRulesError, SieveRulesResult};

/// Ed25519 验签器。
pub struct Verifier {
    /// 已加载的公钥（Week 1 占位，Week 5 起从 config 加载）。
    public_key: Option<ed25519_dalek::VerifyingKey>,
}

impl Verifier {
    /// 新建空验签器（无公钥）。
    pub fn empty() -> Self {
        Self { public_key: None }
    }

    /// 用 32 字节公钥构造。
    pub fn from_public_key_bytes(bytes: &[u8; 32]) -> SieveRulesResult<Self> {
        let key = ed25519_dalek::VerifyingKey::from_bytes(bytes)
            .map_err(|e| SieveRulesError::Signature(e.to_string()))?;
        Ok(Self {
            public_key: Some(key),
        })
    }

    /// 验证 64 字节签名。
    pub fn verify(&self, message: &[u8], signature_bytes: &[u8; 64]) -> SieveRulesResult<()> {
        use ed25519_dalek::Verifier as _;

        let key = self
            .public_key
            .as_ref()
            .ok_or_else(|| SieveRulesError::Signature("no public key loaded".into()))?;
        let sig = ed25519_dalek::Signature::from_bytes(signature_bytes);
        key.verify(message, &sig)
            .map_err(|e| SieveRulesError::Signature(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_verifier_rejects() {
        let v = Verifier::empty();
        let result = v.verify(b"msg", &[0u8; 64]);
        assert!(result.is_err());
    }
}
