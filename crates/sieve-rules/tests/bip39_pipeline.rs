//! BIP39 pipeline 测试（Sieve 差异化点）。
//!
//! 验证：
//! - 标准助记词 verify_checksum 通过
//! - checksum 错误的 24 词不通过
//! - candidate_bip39_windows 正确提取窗口
//! - 仅词表命中但 checksum 失败的窗口不产生 Critical

use sieve_rules::bip39::{candidate_bip39_windows, verify_checksum};
use sieve_rules::wordlist::wordlist_index;

// ---------------------------------------------------------------------------
// verify_checksum 直接单元测试（confirm existing behavior still works）
// ---------------------------------------------------------------------------

/// 标准 12 词助记词（entropy = 全零 128 bit）必须通过。
///
/// BIP39 spec 测试向量: abandon×11 + about
#[test]
fn bip39_valid_12_words_passes_checksum() {
    let words: Vec<&str> = std::iter::repeat_n("abandon", 11)
        .chain(std::iter::once("about"))
        .collect();
    assert!(
        verify_checksum(&words, wordlist_index()),
        "标准 12 词助记词应通过 SHA-256 checksum"
    );
}

/// 标准 24 词助记词（abandon×23 + art）必须通过。
#[test]
fn bip39_valid_24_words_passes_checksum() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon \
                    abandon abandon abandon abandon abandon abandon abandon abandon \
                    abandon abandon abandon abandon abandon abandon abandon art";
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    assert_eq!(words.len(), 24);
    assert!(
        verify_checksum(&words, wordlist_index()),
        "标准 24 词助记词应通过 SHA-256 checksum"
    );
}

/// 24 词全在词表但最后一词换成 checksum 错误的词（zoo），不得通过。
///
/// 差异化点：仅词表命中不足以定级 Critical。
#[test]
fn bip39_24_words_wrong_checksum_fails() {
    // abandon×23 + zoo（zoo 在词表中，但 checksum 不对）
    let mut words: Vec<&str> = std::iter::repeat_n("abandon", 23).collect();
    words.push("zoo");
    // 先确认 zoo 在词表中（否则测试前提不成立）
    assert!(
        wordlist_index().contains_key("zoo"),
        "zoo 应在 BIP39 词表中"
    );
    // checksum 应当失败
    assert!(
        !verify_checksum(&words, wordlist_index()),
        "abandon×23 + zoo 的 checksum 应失败（差异化点）"
    );
}

/// 词数不合法（11 词）拒绝。
#[test]
fn bip39_wrong_word_count_rejected() {
    let words: Vec<&str> = std::iter::repeat_n("abandon", 11).collect();
    assert!(
        !verify_checksum(&words, wordlist_index()),
        "11 词应被拒绝（不是合法 BIP39 词数）"
    );
}

// ---------------------------------------------------------------------------
// candidate_bip39_windows 测试
// ---------------------------------------------------------------------------

/// 12 个全在词表的 token 应产生一个候选窗口。
#[test]
fn candidate_windows_12_words_produces_one_candidate() {
    let tokens: Vec<&str> = std::iter::repeat_n("abandon", 11)
        .chain(std::iter::once("about"))
        .collect();
    let wl = wordlist_index();
    let windows = candidate_bip39_windows(&tokens, wl);
    // 12 词 run，只有一个 12-词窗口
    assert_eq!(windows.len(), 1, "12 个词应产生 1 个窗口");
    assert_eq!(windows[0].len(), 12);
}

/// 混入非词表词时，只有连续在词表的段落被提取。
#[test]
fn candidate_windows_skips_non_wordlist_tokens() {
    let wl = wordlist_index();
    // 12 个 abandon，然后一个非词表词，然后 12 个 about
    let mut tokens: Vec<&str> = std::iter::repeat_n("abandon", 12).collect();
    tokens.push("NOTINWL");
    tokens.extend(std::iter::repeat_n("about", 12));

    let windows = candidate_bip39_windows(&tokens, wl);
    // 两段各自 12 词，各产生 1 个窗口 = 2 个窗口
    assert_eq!(windows.len(), 2, "两段独立 run 应各产生 1 个窗口");
}

/// 24 词 run 应产生多个窗口（24 词的滑窗 + 12 词的滑窗等）。
#[test]
fn candidate_windows_24_words_produces_multiple() {
    let tokens: Vec<&str> = std::iter::repeat_n("abandon", 23).chain(["art"]).collect();
    let wl = wordlist_index();
    let windows = candidate_bip39_windows(&tokens, wl);
    // 24 词 run 应有 12/15/18/21/24 各对应的滑窗
    // 12: 24-12+1=13, 15: 10, 18: 7, 21: 4, 24: 1 => 35 个
    assert_eq!(windows.len(), 35, "24 词 run 应产生 35 个候选窗口");
}

/// 空输入不产生候选。
#[test]
fn candidate_windows_empty_tokens() {
    let tokens: Vec<&str> = vec![];
    let wl = wordlist_index();
    let windows = candidate_bip39_windows(&tokens, wl);
    assert!(windows.is_empty());
}

/// 所有 token 均不在词表时不产生候选。
#[test]
fn candidate_windows_all_non_wordlist() {
    let tokens: Vec<&str> = vec!["foo", "bar", "baz"];
    let wl = wordlist_index();
    let windows = candidate_bip39_windows(&tokens, wl);
    assert!(windows.is_empty());
}

// ---------------------------------------------------------------------------
// 端到端：valid 助记词通过 candidate + verify 链路
// ---------------------------------------------------------------------------

/// 将有效 12 词助记词嵌入文本中，经 candidate_bip39_windows → verify_checksum 链路命中。
#[test]
fn bip39_pipeline_valid_mnemonic_detected() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon \
                    abandon abandon abandon about";
    let tokens: Vec<&str> = mnemonic.split_whitespace().collect();
    let wl = wordlist_index();
    let candidates = candidate_bip39_windows(&tokens, wl);
    let any_valid = candidates.iter().any(|w| verify_checksum(w, wl));
    assert!(any_valid, "有效 12 词助记词应通过 candidate + verify 链路");
}

/// abandon×23 + zoo（全在词表但 checksum 错）经链路不产生 Critical。
///
/// 差异化点：仅词表命中不足以定级 Critical。
#[test]
fn bip39_pipeline_wrong_checksum_not_detected() {
    let mut tokens: Vec<&str> = std::iter::repeat_n("abandon", 23).collect();
    tokens.push("zoo");
    let wl = wordlist_index();
    let candidates = candidate_bip39_windows(&tokens, wl);
    // 有候选（全在词表），但没有一个通过 checksum
    assert!(!candidates.is_empty(), "应有候选（所有词在词表）");
    let any_valid = candidates.iter().any(|w| verify_checksum(w, wl));
    assert!(!any_valid, "abandon×23+zoo 全部候选 checksum 应失败");
}
