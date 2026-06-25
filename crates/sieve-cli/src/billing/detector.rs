//! 超额计费异常检测（SPEC-010 §6）。
//!
//! **目标不是逐 token 精确对账，而是超额计费异常检测**：乘 1.5 = 多报 50%，远高于
//! 任何 tokenizer 噪声（±5~10%），藏不住。只需够「抓系统性虚报」。
//!
//! 信任分级：
//! - `Official`（官方直连）：`usage` 权威，**不核算**。
//! - `Relay`（中转）：`usage` 视为未经验证声明，独立核算 + 交叉比对，偏差超容差报警。
//!
//! 只对**正偏差**（relay 声明 > 独立计数）报警——relay 少报不构成对用户的超额收费。

use crate::config::Trust;

/// 独立核算的 token 数（本地计算结果）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Independent {
    pub input: u64,
    pub output: u64,
    /// 是否为近似估算（Anthropic = true）。影响报警措辞，不影响是否触发。
    pub is_estimate: bool,
}

impl Independent {
    pub fn total(&self) -> u64 {
        self.input + self.output
    }
}

/// relay 在响应体 `usage` 字段中声明的 token 数（未经验证）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Claimed {
    pub input: u64,
    pub output: u64,
}

impl Claimed {
    pub fn total(&self) -> u64 {
        self.input + self.output
    }
}

/// 未核算的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotCheckedReason {
    /// `[billing_check].enabled = false`。
    Disabled,
    /// 官方直连（`usage` 权威，无需核算）。
    OfficialAuthoritative,
    /// 无 relay usage 声明（如请求被拦截、未发到上游）。
    NoUsageClaim,
}

/// 检测裁决。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verdict {
    /// 未核算（见原因）。
    NotChecked(NotCheckedReason),
    /// 已核算，偏差在容差内。`deviation_pct` 为有符号百分比（正=relay 多报）。
    Ok { deviation_pct: f64 },
    /// 检出超额计费：relay 声明显著高于独立计数。
    Overbilled {
        deviation_pct: f64,
        independent_total: u64,
        claimed_total: u64,
        /// 独立计数是否为近似估算（Anthropic）。estimate 时为「疑似」，measured 时为「确认」。
        is_estimate: bool,
    },
}

impl Verdict {
    /// 是否检出超额（供调用方决定是否报警 / 写记录）。
    /// JSON 观测器直接 `match Overbilled{..}` 取字段；本便捷方法由单测 + 剩余
    /// StatusBar 接线消费。
    #[allow(dead_code)]
    pub fn is_overbilled(&self) -> bool {
        matches!(self, Verdict::Overbilled { .. })
    }

    /// 用于 usage.db 落库的稳定标签。
    pub fn label(&self) -> &'static str {
        match self {
            Verdict::NotChecked(NotCheckedReason::Disabled) => "not_checked_disabled",
            Verdict::NotChecked(NotCheckedReason::OfficialAuthoritative) => "not_checked_official",
            Verdict::NotChecked(NotCheckedReason::NoUsageClaim) => "not_checked_no_claim",
            Verdict::Ok { .. } => "ok",
            Verdict::Overbilled { .. } => "overbilled",
        }
    }
}

/// 有符号偏差百分比：`(claimed - independent) / independent * 100`。
/// 正值 = relay 多报。`independent == 0` 时：claimed 也为 0 返回 0；否则返回 +∞
/// （凭空声明 token，必报）。
fn signed_deviation_pct(independent: u64, claimed: u64) -> f64 {
    if independent == 0 {
        return if claimed == 0 { 0.0 } else { f64::INFINITY };
    }
    (claimed as f64 - independent as f64) / independent as f64 * 100.0
}

/// 核心裁决。纯函数、无 IO、无网络——可完整单测。
///
/// - `enabled`：`[billing_check].enabled`
/// - `trust`：上游信任级（仅 `Relay` 核算）
/// - `independent`：本地独立计数
/// - `claimed`：relay 声明的 usage（`None` = 无声明，如被拦请求）
/// - `tolerance_pct`：容差（默认 15%）
pub fn evaluate(
    enabled: bool,
    trust: Trust,
    independent: &Independent,
    claimed: Option<&Claimed>,
    tolerance_pct: f64,
) -> Verdict {
    if !enabled {
        return Verdict::NotChecked(NotCheckedReason::Disabled);
    }
    if trust == Trust::Official {
        return Verdict::NotChecked(NotCheckedReason::OfficialAuthoritative);
    }
    let claimed = match claimed {
        Some(c) => c,
        None => return Verdict::NotChecked(NotCheckedReason::NoUsageClaim),
    };

    let independent_total = independent.total();
    let claimed_total = claimed.total();
    let deviation_pct = signed_deviation_pct(independent_total, claimed_total);

    // 只对正偏差（多报）报警；少报不构成对用户的超额收费。
    if deviation_pct > tolerance_pct {
        Verdict::Overbilled {
            deviation_pct,
            independent_total,
            claimed_total,
            is_estimate: independent.is_estimate,
        }
    } else {
        Verdict::Ok { deviation_pct }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 15.0;

    fn indep(input: u64, output: u64, is_estimate: bool) -> Independent {
        Independent {
            input,
            output,
            is_estimate,
        }
    }

    // ── 红线回归测试：relay 虚报必须被检出（Phase 2 必带）─────────

    #[test]
    fn relay_inflating_by_1_5x_is_detected() {
        // 独立计数 input=1000 output=2000（total 3000）；relay 整体乘 1.5（total 4500）。
        let independent = indep(1000, 2000, false);
        let claimed = Claimed {
            input: 1500,
            output: 3000,
        };
        let v = evaluate(true, Trust::Relay, &independent, Some(&claimed), TOL);
        match v {
            Verdict::Overbilled {
                deviation_pct,
                independent_total,
                claimed_total,
                ..
            } => {
                assert_eq!(independent_total, 3000);
                assert_eq!(claimed_total, 4500);
                assert!(
                    (deviation_pct - 50.0).abs() < 1e-6,
                    "乘 1.5 应得 +50% 偏差，实际 {deviation_pct}"
                );
            }
            other => panic!("relay 乘 1.5 必须被检出为 Overbilled，实际 {other:?}"),
        }
    }

    #[test]
    fn relay_adding_constant_above_tolerance_is_detected() {
        // 独立 1000，relay 加常数 +500（+50%）。
        let independent = indep(1000, 0, false);
        let claimed = Claimed {
            input: 1500,
            output: 0,
        };
        let v = evaluate(true, Trust::Relay, &independent, Some(&claimed), TOL);
        assert!(v.is_overbilled(), "加常数虚报应被检出，实际 {v:?}");
    }

    #[test]
    fn anthropic_estimate_still_catches_gross_inflation_and_marks_estimate() {
        // Anthropic 近似估算下，乘 1.5 仍远超容差，被检出且标 is_estimate。
        let independent = indep(800, 1200, true);
        let claimed = Claimed {
            input: 1200,
            output: 1800,
        };
        let v = evaluate(true, Trust::Relay, &independent, Some(&claimed), TOL);
        match v {
            Verdict::Overbilled { is_estimate, .. } => assert!(is_estimate),
            other => panic!("估算下乘 1.5 仍应检出，实际 {other:?}"),
        }
    }

    // ── 不误报 ──────────────────────────────────────────────────────────────

    #[test]
    fn honest_relay_within_tolerance_is_ok() {
        // tokenizer 噪声级偏差（+8%），不报警。
        let independent = indep(1000, 0, false);
        let claimed = Claimed {
            input: 1080,
            output: 0,
        };
        let v = evaluate(true, Trust::Relay, &independent, Some(&claimed), TOL);
        assert!(
            matches!(v, Verdict::Ok { .. }),
            "容差内偏差不应报警，实际 {v:?}"
        );
    }

    #[test]
    fn relay_underreporting_is_not_flagged() {
        // relay 少报（-30%）不构成超额收费，不报警。
        let independent = indep(1000, 0, false);
        let claimed = Claimed {
            input: 700,
            output: 0,
        };
        let v = evaluate(true, Trust::Relay, &independent, Some(&claimed), TOL);
        assert!(matches!(v, Verdict::Ok { .. }), "少报不应触发，实际 {v:?}");
    }

    #[test]
    fn official_trust_is_never_checked_even_if_usage_differs() {
        // 官方直连：即便声明与独立计数差很多，也采纳 usage 权威、不核算。
        let independent = indep(1000, 0, false);
        let claimed = Claimed {
            input: 5000,
            output: 0,
        };
        let v = evaluate(true, Trust::Official, &independent, Some(&claimed), TOL);
        assert_eq!(
            v,
            Verdict::NotChecked(NotCheckedReason::OfficialAuthoritative)
        );
    }

    #[test]
    fn disabled_is_never_checked() {
        let independent = indep(1000, 0, false);
        let claimed = Claimed {
            input: 5000,
            output: 0,
        };
        let v = evaluate(false, Trust::Relay, &independent, Some(&claimed), TOL);
        assert_eq!(v, Verdict::NotChecked(NotCheckedReason::Disabled));
    }

    #[test]
    fn blocked_request_without_usage_claim_is_not_checked() {
        // 被拦请求未发到上游 → 无 usage 声明，可接受的小缺口。
        let independent = indep(1000, 0, false);
        let v = evaluate(true, Trust::Relay, &independent, None, TOL);
        assert_eq!(v, Verdict::NotChecked(NotCheckedReason::NoUsageClaim));
    }

    #[test]
    fn claimed_tokens_from_zero_independent_is_infinite_deviation() {
        // relay 凭空声明 token（独立计数为 0）→ +∞ 偏差，必报。
        let independent = indep(0, 0, false);
        let claimed = Claimed {
            input: 500,
            output: 0,
        };
        let v = evaluate(true, Trust::Relay, &independent, Some(&claimed), TOL);
        assert!(v.is_overbilled(), "凭空声明 token 必报，实际 {v:?}");
    }
}
