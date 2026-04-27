//! IN-CR-01 地址替换检测（strsim Levenshtein，关联 UCSB 论文 attack 1）。
//!
//! 算法：对比入站响应文本中出现的 ETH 地址与会话中已知地址集合，
//! 若存在 Levenshtein 距离 ∈ [1, 3] 且长度相同的近似地址，判定为地址替换攻击。

use std::collections::HashSet;

/// 提取文本中所有 ETH 地址（`0x[a-fA-F0-9]{40}`），统一转为小写。
///
/// # Examples
/// ```
/// use sieve_core::address_guard::extract_eth_addresses;
///
/// let addrs = extract_eth_addresses("send to 0xABCDEF1234567890ABCDEF1234567890ABCDEF12");
/// assert_eq!(addrs.len(), 1);
/// ```
pub fn extract_eth_addresses(text: &str) -> Vec<String> {
    // 手动扫描，避免 regex crate 在 hot path 每次编译（实际上 regex 有内部缓存，但显式更清晰）
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut result = Vec::new();
    let mut i = 0;

    while i + 2 <= len {
        // 找 0x 前缀
        if bytes[i] == b'0' && i + 1 < len && bytes[i + 1] == b'x' {
            // 检查是否以单词边界开始（前一个字符不是 hex 字符）
            let word_start = i == 0 || !is_hex_char(bytes[i - 1]);
            if word_start {
                // 数 hex 字符
                let addr_start = i;
                let hex_start = i + 2;
                let mut j = hex_start;
                while j < len && is_hex_char(bytes[j]) {
                    j += 1;
                }
                // ETH 地址：0x + 40 hex chars，且后面不跟 hex（单词边界）
                let hex_len = j - hex_start;
                let word_end = j >= len || !is_hex_char(bytes[j]);
                if hex_len == 40 && word_end {
                    let addr = text[addr_start..j].to_lowercase();
                    result.push(addr);
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }

    result
}

fn is_hex_char(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b)
}

/// 检查 candidate 是否是 seen 中某地址的"近似但不相同"替换。
///
/// 阈值：Levenshtein distance ∈ [1, 3] 且 `candidate.len() == seen_addr.len()`（ETH 地址固定长度）。
///
/// 返回原始（被仿冒）地址，如发现替换；否则返回 `None`。
///
/// # Examples
/// ```
/// use std::collections::HashSet;
/// use sieve_core::address_guard::check_substitution;
///
/// let mut seen = HashSet::new();
/// seen.insert("0xabcdef1234567890abcdef1234567890abcdef12".to_string());
/// let result = check_substitution(&seen, "0xabcdef1234567890abcdef1234567890abcdef13");
/// assert!(result.is_some());
/// ```
pub fn check_substitution(seen: &HashSet<String>, candidate: &str) -> Option<String> {
    let cand_lower = candidate.to_lowercase();
    // 完全相同 → 不算替换
    if seen.contains(&cand_lower) {
        return None;
    }
    for s in seen {
        // 长度必须相同（ETH 地址固定 42 字符）
        if s.len() != cand_lower.len() {
            continue;
        }
        let d = strsim::levenshtein(s, &cand_lower);
        if (1..=3).contains(&d) {
            return Some(s.clone());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_eth() {
        let text = "send to 0xABCDEF1234567890ABCDEF1234567890ABCDEF12 please";
        let addrs = extract_eth_addresses(text);
        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0], "0xabcdef1234567890abcdef1234567890abcdef12");
    }

    #[test]
    fn extract_multiple_addresses() {
        let text = "from 0x1111111111111111111111111111111111111111 to 0x2222222222222222222222222222222222222222";
        let addrs = extract_eth_addresses(text);
        assert_eq!(addrs.len(), 2);
    }

    #[test]
    fn extract_no_match_short_hex() {
        // 0x + 39 hex chars → 不是有效 ETH 地址
        let text = "0xabc123";
        let addrs = extract_eth_addresses(text);
        assert!(addrs.is_empty());
    }

    #[test]
    fn detects_one_char_substitution() {
        let mut seen = HashSet::new();
        seen.insert("0xabcdef1234567890abcdef1234567890abcdef12".to_string());
        let candidate = "0xabcdef1234567890abcdef1234567890abcdef13"; // 末位 2→3
        assert!(check_substitution(&seen, candidate).is_some());
    }

    #[test]
    fn ignores_identical_address() {
        let mut seen = HashSet::new();
        seen.insert("0xabcdef1234567890abcdef1234567890abcdef12".to_string());
        let candidate = "0xabcdef1234567890abcdef1234567890abcdef12";
        assert!(check_substitution(&seen, candidate).is_none());
    }

    #[test]
    fn ignores_completely_different_address() {
        let mut seen = HashSet::new();
        seen.insert("0xabcdef1234567890abcdef1234567890abcdef12".to_string());
        let candidate = "0x0000000000000000000000000000000000000000"; // distance 大
        assert!(check_substitution(&seen, candidate).is_none());
    }

    #[test]
    fn ignores_different_length() {
        let mut seen = HashSet::new();
        seen.insert("0xabc".to_string());
        let candidate = "0xabcd"; // 长度不同
        assert!(check_substitution(&seen, candidate).is_none());
    }

    #[test]
    fn case_insensitive_comparison() {
        let mut seen = HashSet::new();
        seen.insert("0xabcdef1234567890abcdef1234567890abcdef12".to_string());
        // 完全相同但大写 → 不算替换（同一地址）
        let candidate = "0xABCDEF1234567890ABCDEF1234567890ABCDEF12";
        assert!(check_substitution(&seen, candidate).is_none());
    }
}
