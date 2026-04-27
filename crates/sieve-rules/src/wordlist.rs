//! BIP39 词表：静态嵌入 + O(1) 词→索引查找。
//!
//! 词表来源：bitcoin/bips BIP-0039 English Wordlist（MIT License）。

use std::collections::HashMap;
use std::sync::OnceLock;

/// BIP39 英文词表原始文本（编译期嵌入）。
///
/// 通过 `include_str!` 在编译期嵌入，零运行时 IO。
static WORDLIST_RAW: &str = include_str!("../wordlist/english.txt");

static WORDLIST_INDEX: OnceLock<HashMap<&'static str, usize>> = OnceLock::new();

/// 返回词 → 索引的 HashMap（懒初始化，线程安全）。
///
/// 索引为 0-based，与 BIP39 bit encoding 对应（idx 对应 11-bit value）。
/// 注释行（`#` 开头）和空行自动跳过。
pub fn wordlist_index() -> &'static HashMap<&'static str, usize> {
    WORDLIST_INDEX.get_or_init(|| {
        WORDLIST_RAW
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .enumerate()
            .map(|(i, w)| (w, i))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wordlist_has_2048_entries() {
        assert_eq!(wordlist_index().len(), 2048);
    }

    #[test]
    fn first_word_abandon_at_index_0() {
        assert_eq!(wordlist_index().get("abandon"), Some(&0));
    }

    #[test]
    fn last_word_zoo_at_index_2047() {
        assert_eq!(wordlist_index().get("zoo"), Some(&2047));
    }

    #[test]
    fn about_at_index_3() {
        // BIP39 spec: abandon=0, ability=1, able=2, about=3
        assert_eq!(wordlist_index().get("about"), Some(&3));
    }
}
