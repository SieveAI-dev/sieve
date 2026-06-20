//! 独立 token 计数（ADR-038 决策 2 / SPEC-010 §4）。
//!
//! **永远优先权威信源，只在对抗不可信 relay 时才自己算；自己算也不手搓 tokenizer。**
//!
//! - **OpenAI**：用 `tiktoken-rs`（GPT-4o 及更新 `o200k_base`，老模型 `cl100k_base`），
//!   接近精确。
//! - **Anthropic**：Claude **无公开 tokenizer**，用 `cl100k_base` BPE 作**近似 proxy**
//!   （量级与 Claude 真实分词可比，足够抓「乘 1.5」量级虚报，**非精确账单**）。结果
//!   标 `is_estimate = true`。本模块**纯本地计算、零网络**——`count_tokens` 官方直连
//!   是独立 opt-in 开关（默认关，本增量不接入），故「usage 永不上传」结构性成立。

use anyhow::{Context, Result};
use tiktoken_rs::{cl100k_base, o200k_base, CoreBPE};

/// 上游协议族（决定用哪个 tokenizer）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    OpenAi,
    Anthropic,
}

/// OpenAI chat 每条消息的固定框架开销（ChatML：`<|im_start|>role\n...<|im_end|>`）。
/// 经验值 3 token/消息 + 3 token 回复引导。用于让输入计数接近计费口径。
const OPENAI_PER_MESSAGE_TOKENS: u64 = 3;
const OPENAI_REPLY_PRIMING_TOKENS: u64 = 3;

/// token 估算器，持有两套 BPE（构造一次、全程复用，避免每请求重建）。
pub struct TokenEstimator {
    o200k: CoreBPE,
    cl100k: CoreBPE,
}

impl TokenEstimator {
    /// 构造估算器（加载 bundled BPE 词表，开销较大，应只做一次）。
    pub fn new() -> Result<Self> {
        Ok(Self {
            o200k: o200k_base().context("load o200k_base tokenizer")?,
            cl100k: cl100k_base().context("load cl100k_base tokenizer")?,
        })
    }

    /// 选择 OpenAI model 对应的 BPE：GPT-4o / o1 / o3 / o4 / gpt-4.1 / gpt-5 系用
    /// `o200k_base`，其余老模型用 `cl100k_base`。
    fn openai_bpe(&self, model: &str) -> &CoreBPE {
        if uses_o200k(model) {
            &self.o200k
        } else {
            &self.cl100k
        }
    }

    /// 计数一段纯文本的 token 数。Anthropic 走 `cl100k_base` proxy（近似）。
    pub fn count_text(&self, family: Family, model: &str, text: &str) -> u64 {
        let bpe = match family {
            Family::OpenAi => self.openai_bpe(model),
            // Claude 无公开 tokenizer，cl100k_base 作近似 proxy（标 is_estimate）。
            Family::Anthropic => &self.cl100k,
        };
        bpe.encode_ordinary(text).len() as u64
    }

    /// 计数多条消息的输入 token（含 OpenAI per-message 框架开销）。
    ///
    /// `messages` 为按对话顺序展平的文本段。Anthropic 不加 ChatML 框架开销
    /// （其计费口径不同，proxy 已是近似），仅累加文本计数。
    pub fn count_input(&self, family: Family, model: &str, messages: &[String]) -> u64 {
        let text_tokens: u64 = messages
            .iter()
            .map(|m| self.count_text(family, model, m))
            .sum();
        match family {
            Family::OpenAi => {
                text_tokens
                    + OPENAI_PER_MESSAGE_TOKENS * messages.len() as u64
                    + OPENAI_REPLY_PRIMING_TOKENS
            }
            Family::Anthropic => text_tokens,
        }
    }

    /// 计数输出（completion）文本的 token 数。
    pub fn count_output(&self, family: Family, model: &str, completion: &str) -> u64 {
        self.count_text(family, model, completion)
    }

    /// 该 family 的计数是否为近似估算（Anthropic = true，OpenAI = false）。
    pub fn is_estimate(family: Family) -> bool {
        matches!(family, Family::Anthropic)
    }
}

/// 判断 OpenAI model 是否使用 `o200k_base`（GPT-4o 及更新系）。
fn uses_o200k(model: &str) -> bool {
    let m = model.to_ascii_lowercase();
    const O200K_MARKERS: &[&str] = &[
        "gpt-4o",
        "gpt-4.1",
        "gpt-5",
        "o1",
        "o3",
        "o4",
        "chatgpt-4o",
        "gpt-image",
    ];
    O200K_MARKERS.iter().any(|marker| m.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_model_routing_picks_o200k_for_new_models() {
        assert!(uses_o200k("gpt-4o-2024-08-06"));
        assert!(uses_o200k("o1-mini"));
        assert!(uses_o200k("gpt-4.1"));
        assert!(!uses_o200k("gpt-3.5-turbo"));
        assert!(!uses_o200k("text-davinci-003"));
    }

    #[test]
    fn counts_are_deterministic_and_nonzero() {
        let est = TokenEstimator::new().expect("build estimator");
        let text = "The quick brown fox jumps over the lazy dog.";
        let a = est.count_text(Family::OpenAi, "gpt-4o", text);
        let b = est.count_text(Family::OpenAi, "gpt-4o", text);
        assert_eq!(a, b, "同一输入 token 计数必须确定");
        assert!(a > 0);
    }

    #[test]
    fn anthropic_is_marked_estimate_openai_is_not() {
        assert!(TokenEstimator::is_estimate(Family::Anthropic));
        assert!(!TokenEstimator::is_estimate(Family::OpenAi));
    }

    #[test]
    fn openai_input_includes_framing_overhead() {
        let est = TokenEstimator::new().expect("build estimator");
        let msgs = vec!["hello".to_string(), "world".to_string()];
        let with_overhead = est.count_input(Family::OpenAi, "gpt-4o", &msgs);
        let raw: u64 = msgs
            .iter()
            .map(|m| est.count_text(Family::OpenAi, "gpt-4o", m))
            .sum();
        // 2 条消息 → +3*2 框架 +3 引导 = +9
        assert_eq!(with_overhead, raw + 9);
    }

    #[test]
    fn anthropic_input_has_no_chatml_overhead() {
        let est = TokenEstimator::new().expect("build estimator");
        let msgs = vec!["hello".to_string()];
        let input = est.count_input(Family::Anthropic, "claude-sonnet-4", &msgs);
        let raw = est.count_text(Family::Anthropic, "claude-sonnet-4", "hello");
        assert_eq!(input, raw);
    }
}
