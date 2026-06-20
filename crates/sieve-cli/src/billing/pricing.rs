//! 内置官方公开单价表（ADR-038 决策 2 / SPEC-010 §6 计费比对）。
//!
//! 单价用于把 token 数换算成「应收成本（USD）」做计费比对与本地展示。
//! **超额检测本身基于 token 偏差**（更直接、不依赖价表时效）；价表仅用于
//! 成本展示与记录。官方调价后需更新本表；缺失 model 时回退 `None`（只记 token
//! 偏差，不算成本）。单价为 USD / 百万 token（input, output），2026 量级近似值。

/// 某 model 的每百万 token 单价（USD）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPrice {
    /// 输入（prompt）单价，USD / 1M tokens。
    pub input_per_mtok: f64,
    /// 输出（completion）单价，USD / 1M tokens。
    pub output_per_mtok: f64,
}

impl ModelPrice {
    /// 按 token 数计算成本（USD）。
    pub fn cost_usd(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        (input_tokens as f64 / 1_000_000.0) * self.input_per_mtok
            + (output_tokens as f64 / 1_000_000.0) * self.output_per_mtok
    }
}

/// 价表项：`(匹配子串, 单价)`。按从**具体到通用**的顺序排列，命中即返回，
/// 确保 `claude-3-5-haiku` 先命中 `haiku` 而非通用 `claude`。
const PRICE_TABLE: &[(&str, ModelPrice)] = &[
    // ── Anthropic ──
    (
        "haiku",
        ModelPrice {
            input_per_mtok: 0.80,
            output_per_mtok: 4.0,
        },
    ),
    (
        "sonnet",
        ModelPrice {
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
        },
    ),
    (
        "opus",
        ModelPrice {
            input_per_mtok: 15.0,
            output_per_mtok: 75.0,
        },
    ),
    // ── OpenAI ──
    (
        "gpt-4o-mini",
        ModelPrice {
            input_per_mtok: 0.15,
            output_per_mtok: 0.60,
        },
    ),
    (
        "gpt-4o",
        ModelPrice {
            input_per_mtok: 2.5,
            output_per_mtok: 10.0,
        },
    ),
    (
        "o1-mini",
        ModelPrice {
            input_per_mtok: 1.1,
            output_per_mtok: 4.4,
        },
    ),
    (
        "o1",
        ModelPrice {
            input_per_mtok: 15.0,
            output_per_mtok: 60.0,
        },
    ),
];

/// 查内置价表（大小写不敏感，子串匹配，具体优先）。缺失返回 `None`。
pub fn lookup(model: &str) -> Option<ModelPrice> {
    let m = model.to_ascii_lowercase();
    PRICE_TABLE
        .iter()
        .find(|(needle, _)| m.contains(needle))
        .map(|(_, price)| *price)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specific_model_matches_before_generic() {
        // haiku 在 opus/sonnet 前命中，不被通用项误伤。
        let p = lookup("claude-3-5-haiku-20241022").unwrap();
        assert_eq!(p.input_per_mtok, 0.80);
        let p = lookup("claude-opus-4-20250101").unwrap();
        assert_eq!(p.input_per_mtok, 15.0);
        let p = lookup("gpt-4o-mini").unwrap();
        assert_eq!(p.input_per_mtok, 0.15);
        let p = lookup("gpt-4o-2024-08-06").unwrap();
        assert_eq!(p.input_per_mtok, 2.5);
    }

    #[test]
    fn unknown_model_returns_none() {
        assert!(lookup("some-future-model-x").is_none());
    }

    #[test]
    fn cost_computation() {
        let p = ModelPrice {
            input_per_mtok: 3.0,
            output_per_mtok: 15.0,
        };
        // 1M input + 1M output = 3 + 15 = 18 USD
        assert!((p.cost_usd(1_000_000, 1_000_000) - 18.0).abs() < 1e-9);
    }
}
