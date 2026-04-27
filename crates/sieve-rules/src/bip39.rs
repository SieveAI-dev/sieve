//! BIP39 SHA-256 checksum 验证（关联 PRD §9 #4 差异化点）。
//!
//! 仅验证 checksum，不验证词列表完整性——调用方应已用 wordlist 过滤过未知词。
//!
//! 算法见 BIP-0039 spec(<https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki>):
//! - 12/15/18/21/24 词对应 128/160/192/224/256 bit entropy + 4/5/6/7/8 bit checksum
//! - checksum = SHA-256(entropy)[..cs_len_bits]

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// 验证 BIP39 助记词的 SHA-256 checksum 位是否正确。
///
/// 仅验证 checksum，不验证词列表完整性——调用方应已用 wordlist 过滤过未知词。
pub fn verify_checksum(words: &[&str], wordlist_index: &HashMap<&'static str, usize>) -> bool {
    let n = words.len();
    if ![12, 15, 18, 21, 24].contains(&n) {
        return false;
    }

    let mut bits: Vec<u8> = Vec::with_capacity(n * 11);
    for w in words {
        let idx = match wordlist_index.get(w) {
            Some(&i) => i,
            None => return false,
        };
        for i in (0..11).rev() {
            bits.push(((idx >> i) & 1) as u8);
        }
    }

    let cs_len = n * 11 / 33;
    let ent_bits = n * 11 - cs_len;
    let ent_bytes = ent_bits / 8;
    let mut entropy = vec![0u8; ent_bytes];
    for i in 0..ent_bits {
        entropy[i / 8] |= bits[i] << (7 - (i % 8));
    }

    let hash = Sha256::digest(&entropy);
    let expected: Vec<u8> = (0..cs_len)
        .map(|i| (hash[i / 8] >> (7 - (i % 8))) & 1)
        .collect();
    bits[ent_bits..] == expected[..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wordlist::wordlist_index;

    /// BIP39 spec 测试向量 (https://github.com/trezor/python-mnemonic/blob/master/vectors.json)
    /// entropy = "00000000000000000000000000000000"
    /// mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
    #[test]
    fn known_valid_12_words() {
        let words: Vec<&str> = ["abandon"; 11]
            .iter()
            .chain(["about"].iter())
            .copied()
            .collect();
        assert!(verify_checksum(&words, wordlist_index()));
    }

    /// 把最后一个词从 "about" 改为 "ability" 应当 checksum 失败
    #[test]
    fn invalid_checksum_12_words() {
        let mut words: Vec<&str> = vec!["abandon"; 11];
        words.push("ability");
        assert!(!verify_checksum(&words, wordlist_index()));
    }

    /// 24 词全 0 entropy: "abandon" × 23 + "art"
    /// 用 spec 给的 vector
    #[test]
    fn known_valid_24_words() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
        let words: Vec<&str> = mnemonic.split_whitespace().collect();
        assert!(verify_checksum(&words, wordlist_index()));
    }

    /// 错误词数（11 / 13 / 25）拒绝
    #[test]
    fn wrong_word_count_rejected() {
        let words11: Vec<&str> = vec!["abandon"; 11];
        let words13: Vec<&str> = vec!["abandon"; 13];
        assert!(!verify_checksum(&words11, wordlist_index()));
        assert!(!verify_checksum(&words13, wordlist_index()));
    }

    /// 未知词（不在 wordlist）拒绝
    #[test]
    fn unknown_word_rejected() {
        let words: Vec<&str> = vec!["abandon"; 11].into_iter().chain(["bogus"]).collect();
        assert!(!verify_checksum(&words, wordlist_index()));
    }
}
