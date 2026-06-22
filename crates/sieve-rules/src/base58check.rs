//! Base58Check 校验和验证（关联 ADR-042：OUT-12 Bitcoin WIF / OUT-13 BIP-32 扩展私钥）。
//!
//! 与 BIP39 checksum 同一设计哲学（PRD §9 #4）：仅校验和通过的候选才动作，
//! 把「看起来像私钥」收窄到「数学上是合法 Base58Check 序列化」，大幅降误报。
//!
//! Base58Check 编码：payload 尾接 `SHA256(SHA256(payload))[..4]` 校验和，整体 Base58 编码。
//! 本模块纯本地算术，无网络 IO（sieve-rules 禁网络，ADR-024）。

use sha2::{Digest, Sha256};

/// Bitcoin Base58 字母表（去除易混字符 `0` `O` `I` `l`）。
const BASE58_ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// 解码 Base58 字符串为字节（big-endian）。含非 Base58 字符则返回 `None`。
fn base58_decode(s: &str) -> Option<Vec<u8>> {
    if s.is_empty() {
        return None;
    }
    // big-num 累加：result = result * 58 + digit，存为 big-endian 字节。
    let mut bytes: Vec<u8> = Vec::new();
    for ch in s.bytes() {
        let val = BASE58_ALPHABET.iter().position(|&a| a == ch)? as u32;
        let mut carry = val;
        for b in bytes.iter_mut().rev() {
            carry += (*b as u32) * 58;
            *b = (carry & 0xff) as u8;
            carry >>= 8;
        }
        while carry > 0 {
            bytes.insert(0, (carry & 0xff) as u8);
            carry >>= 8;
        }
    }
    // 前导 '1'（Base58 的 0）映射为前导 0 字节。
    for ch in s.bytes() {
        if ch == b'1' {
            bytes.insert(0, 0);
        } else {
            break;
        }
    }
    Some(bytes)
}

/// 验证候选串是否为合法 Base58Check 编码
/// （尾 4 字节校验和 == `SHA256(SHA256(payload))[..4]`）。
///
/// 关联 ADR-042：OUT-12（WIF）/ OUT-13（BIP-32 xprv）的 second-pass 校验和门。
/// 返回 `true` 仅当：Base58 解码成功、解码长度 ≥ 5、尾 4 字节校验和匹配。
pub fn verify_base58check(candidate: &str) -> bool {
    let decoded = match base58_decode(candidate) {
        Some(v) => v,
        None => return false,
    };
    if decoded.len() < 5 {
        return false;
    }
    let split = decoded.len() - 4;
    let (payload, checksum) = decoded.split_at(split);
    let hash1 = Sha256::digest(payload);
    let hash2 = Sha256::digest(hash1);
    hash2[..4] == checksum[..]
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试向量来自公开标准：Bitcoin wiki「Wallet import format」+ BIP-32 spec test vector 1。
    const VALID_WIF_UNCOMPRESSED: &str = "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ";
    const VALID_WIF_COMPRESSED: &str = "KwdMAjGmerYanjeui5SHS7JkmpZvVipYvB2LJGU1ZxJwYvP98617";
    const VALID_XPRV: &str = "xprv9s21ZrQH143K3QTDL4LXw2F7HEK3wJUD2nW2nRk4stbPy6cq3jPPqjiChkVvvNKmPGJxWUtg6LnF5kejMRNNU3TGtRBeJgk33yuGBxrMPHi";

    #[test]
    fn valid_wif_uncompressed_passes() {
        assert!(verify_base58check(VALID_WIF_UNCOMPRESSED));
    }

    #[test]
    fn valid_wif_compressed_passes() {
        assert!(verify_base58check(VALID_WIF_COMPRESSED));
    }

    #[test]
    fn valid_xprv_passes() {
        assert!(verify_base58check(VALID_XPRV));
    }

    #[test]
    fn tampered_wif_fails_checksum() {
        // 末字符 'J' → 'K'，破坏校验和
        let mut s = VALID_WIF_UNCOMPRESSED.to_string();
        s.pop();
        s.push('K');
        assert!(!verify_base58check(&s));
    }

    #[test]
    fn tampered_xprv_fails_checksum() {
        let mut s = VALID_XPRV.to_string();
        s.pop();
        s.push('a');
        assert!(!verify_base58check(&s));
    }

    #[test]
    fn truncated_fails() {
        assert!(!verify_base58check(&VALID_XPRV[..VALID_XPRV.len() - 1]));
    }

    #[test]
    fn non_base58_char_fails() {
        // 末位换成 '0'（不在 Base58 字母表）
        assert!(!verify_base58check(
            "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyT0"
        ));
    }

    #[test]
    fn empty_fails() {
        assert!(!verify_base58check(""));
    }

    #[test]
    fn short_string_fails() {
        // 合法 Base58 但解码后不足 5 字节
        assert!(!verify_base58check("111"));
    }
}
