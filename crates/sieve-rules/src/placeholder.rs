//! 占位符黑名单：vectorscan 命中后，若候选文本匹配任意 placeholder 正则则拒绝定级 Critical。
//! 关联 PRD §5.1 出站检测 P0 表 + Explore 调研建议。

use regex::Regex;
use std::sync::OnceLock;
use tracing::warn;

static PLACEHOLDER_REGEXES: OnceLock<Vec<Regex>> = OnceLock::new();

/// 全局占位符黑名单（unicode case insensitive）。
///
/// 注意：`regex` crate 不支持反向引用（backreference），"重复同一字符 10+ 次" 的检测
/// 由 `is_all_same_char` 函数单独处理，不走正则。
pub fn placeholder_regexes() -> &'static [Regex] {
    PLACEHOLDER_REGEXES.get_or_init(|| {
        let patterns = [
            r"(?i)your[_-]?api[_-]?key",
            r"(?i)<api[_-]?key>",
            r"(?i)api[_-]?key[_-]?here",
            r"(?i)x{3,}|redacted|\*{3,}|\[redacted\]",
            r"(?i)\bplaceholder\b|\bsample\b|\bdummy\b|\bfake\b|\bexample\b",
            r"sk-ant-api03-[xX]{5,}",
            r"sk-[xX\*]{20,}",
            r"0x0{40}",
        ];
        patterns
            .iter()
            .filter_map(|p| match Regex::new(p) {
                Ok(re) => Some(re),
                Err(e) => {
                    warn!("invalid placeholder regex `{p}`: {e}");
                    None
                }
            })
            .collect()
    })
}

/// 字符串是否全由同一字符组成且长度 >= 11（替代 `^(.)\1{10,}$` 反向引用）。
fn is_all_same_char(s: &str) -> bool {
    if s.len() < 11 {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => chars.all(|c| c == first),
        None => false,
    }
}

/// 候选文本是否匹配任一 placeholder。
pub fn is_placeholder(candidate: &str) -> bool {
    if is_all_same_char(candidate) {
        return true;
    }
    placeholder_regexes().iter().any(|r| r.is_match(candidate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_placeholders() {
        assert!(is_placeholder("YOUR_API_KEY"));
        assert!(is_placeholder("xxx"));
        assert!(is_placeholder("REDACTED"));
        assert!(is_placeholder("[redacted]"));
        assert!(is_placeholder("sk-ant-api03-XXXXXXXX"));
        assert!(is_placeholder("0x0000000000000000000000000000000000000000"));
        assert!(is_placeholder("aaaaaaaaaaaaa"));
    }

    #[test]
    fn does_not_match_real_secrets() {
        assert!(!is_placeholder(
            "sk-ant-api03-real-looking-key-with-mixed-content"
        ));
        assert!(!is_placeholder("AKIAIOSFODNN7EXAMPLE"));
    }
}
