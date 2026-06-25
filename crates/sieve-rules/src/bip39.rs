//! BIP39 SHA-256 checksum 验证（Sieve 差异化点）。
//!
//! 仅验证 checksum，不验证词列表完整性——调用方应已用 wordlist 过滤过未知词。
//!
//! 算法见 BIP-0039 spec(<https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki>):
//! - 12/15/18/21/24 词对应 128/160/192/224/256 bit entropy + 4/5/6/7/8 bit checksum
//! - checksum = SHA-256(entropy)[..cs_len_bits]

use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// 从文本中提取所有 **全部在 BIP39 词表中** 的连续 12/15/18/21/24 词窗口。
///
/// 算法：
/// 1. 按空白分词；
/// 2. 找出所有连续 token 中每个都在 wordlist 的最长前缀 runs；
/// 3. 对每个 run，用滑窗按合法词数（12/15/18/21/24）提取候选。
///
/// 返回每个候选窗口的 token Vec；调用方再对每个 Vec 调 `verify_checksum`。
///
/// 差异化点：只有全部在词表 **且** checksum 通过的才定级 Critical。
pub fn candidate_bip39_windows<'a>(
    tokens: &'a [&'a str],
    wordlist_index: &HashMap<&'static str, usize>,
) -> Vec<Vec<&'a str>> {
    const VALID_COUNTS: [usize; 5] = [12, 15, 18, 21, 24];

    // 标记每个 token 是否在词表中
    let in_wl: Vec<bool> = tokens
        .iter()
        .map(|t| wordlist_index.contains_key(t))
        .collect();

    let n = tokens.len();
    let mut results = Vec::new();

    // 对每个起始位置，找连续在词表的 run 长度
    let mut i = 0;
    while i < n {
        if !in_wl[i] {
            i += 1;
            continue;
        }
        // 计算从 i 开始的连续词表词 run 长度
        let mut run_len = 0;
        while i + run_len < n && in_wl[i + run_len] {
            run_len += 1;
        }
        // 在此 run 内滑窗提取所有合法词数窗口
        for &count in &VALID_COUNTS {
            if run_len >= count {
                for start in 0..=(run_len - count) {
                    results.push(tokens[i + start..i + start + count].to_vec());
                }
            }
        }
        i += run_len;
    }

    results
}

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
