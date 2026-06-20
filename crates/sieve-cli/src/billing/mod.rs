//! 超额计费检测（ADR-038 / SPEC-010）。
//!
//! 把「中转站不可信」从口号变成可量化戳穿的能力：作为代理，Sieve 双向原始字节都有，
//! 对 `Relay` 上游独立核算 token、与 relay 声明的 `usage` 交叉比对，偏差超容差报警。
//!
//! **设计姿态（ADR-038 裁定）**：
//! - 默认全关（`[billing_check].enabled = false`），不开启零行为变化、零开销。
//! - **本模块纯本地、零网络**——估算用 bundled tiktoken / cl100k proxy，记录落本地
//!   `usage.db`。`count_tokens` 官方直连是独立 opt-in 开关（默认关，本增量未接入），
//!   故「usage 严格本地、永不上传」结构性成立（呼应 SPEC-006）。
//! - 仅 StatusBar 报警，**不阻断流量**（计费监督，非安全拦截，不引入新 Block 路径）。
//!
//! ## 接通状态（JSON 路径已接线，2026-06-20）
//!
//! daemon 在 Anthropic / OpenAI **JSON 响应**路径（`handle_*_json_inbound`）接通观测器：
//! `BillingObserver`（启动一次性构造）经 `proxy_inner`/`proxy_openai` 参数透传 →
//! `build_billing_context`（仅 `Relay` 上游 + `enabled` 时，请求侧独立估算 input）→
//! `spawn_billing_observation`（响应侧 relay `usage` + 独立估算 output 交叉比对 → 写
//! `usage.db` → 超额 `warn`，`sieve usage --overbilled-only` 可查）。
//!
//! ⏭ **剩余**：①SSE 流式路径的 usage 累计观测（`MessageDelta.usage` + completion 累计）
//! ②超额时 StatusBar IPC 弹窗（可复用 `NotifyKind::Generic`）。两者均不影响 JSON 路径检测。

pub mod detector;
pub mod estimator;
pub mod pricing;
pub mod usage_store;

pub use detector::{evaluate, Claimed, Independent, Verdict};
pub use estimator::{Family, TokenEstimator};
pub use usage_store::{UsageRecord, UsageStore};

// 裁决 API 的一部分，由单测 + 剩余 SSE/StatusBar 接线消费（JSON 观测器只 match Overbilled）。
#[allow(unused_imports)]
pub use detector::NotCheckedReason;

use crate::config::Trust;
use std::sync::Arc;

/// daemon 全局超额计费观测器（ADR-038）：绑定一次性构造的估算器 + 本地 usage 存储 + 容差。
///
/// 在 daemon 启动时按 `[billing_check].enabled` 构造一次（`TokenEstimator::new` 加载 BPE
/// 词表开销大），以 `Arc` 透传到响应观测点。**纯本地、零网络**（呼应隐私红线）。
pub struct BillingObserver {
    estimator: TokenEstimator,
    usage: UsageStore,
    tolerance_pct: f64,
}

impl BillingObserver {
    pub fn new(usage: UsageStore, tolerance_pct: f64) -> anyhow::Result<Self> {
        Ok(Self {
            estimator: TokenEstimator::new()?,
            usage,
            tolerance_pct,
        })
    }

    /// 独立估算请求输入文本的 token 数（在请求处理时调用，结果随请求带到响应观测点）。
    pub fn count_input(&self, family: Family, model: &str, messages: &[String]) -> u64 {
        self.estimator.count_input(family, model, messages)
    }

    /// 在响应完成时观测：独立估算 output（completion 全文）→ 与 relay 声明交叉比对 →
    /// 写 `usage.db`（fire-and-forget，调用方 spawn）→ 返回裁决供 StatusBar 报警决策。
    ///
    /// `independent_input` 为请求侧已算好的输入 token 数；`completion` 为补全全文（用于
    /// 独立 output 计数）；`claimed` 为 relay 声明的 `(input, output)`（`None` = 无声明）。
    #[allow(clippy::too_many_arguments)]
    pub async fn observe(
        &self,
        trust: Trust,
        family: Family,
        request_id: Option<String>,
        provider_id: String,
        model: String,
        independent_input: u64,
        completion: &str,
        claimed: Option<(u64, u64)>,
    ) -> Verdict {
        let independent_output = self.estimator.count_output(family, &model, completion);
        let a = assess(
            true,
            self.tolerance_pct,
            trust,
            family,
            request_id,
            provider_id,
            model,
            independent_input,
            independent_output,
            claimed,
        );
        if let Err(e) = self.usage.append(a.record).await {
            tracing::warn!(error = %e, "usage.db append 失败");
        }
        a.verdict
    }
}

/// 一次完整核算的输入：独立计数 + relay 声明 + 上下文。
pub struct Assessment {
    pub verdict: Verdict,
    pub record: UsageRecord,
}

/// 每请求的超额计费观测上下文（ADR-038）。在请求处理时构造（含已算好的 input token
/// 数），随请求带到响应观测点完成核算。仅 `Relay` 上游 + `enabled` 时为 `Some`。
pub struct BillingContext {
    pub observer: Arc<BillingObserver>,
    pub trust: Trust,
    pub provider_id: String,
    pub request_id: String,
    pub model: String,
    pub family: Family,
    pub independent_input: u64,
}

/// 高层便捷入口：给定独立计数与 relay 声明，跑检测并组装可落库记录。
///
/// 纯函数（不写盘、不联网），便于在 daemon 观测器与测试中复用。
#[allow(clippy::too_many_arguments)]
pub fn assess(
    enabled: bool,
    tolerance_pct: f64,
    trust: Trust,
    family: Family,
    request_id: Option<String>,
    provider_id: String,
    model: String,
    independent_input: u64,
    independent_output: u64,
    claimed: Option<(u64, u64)>,
) -> Assessment {
    let is_estimate = TokenEstimator::is_estimate(family);
    let independent = Independent {
        input: independent_input,
        output: independent_output,
        is_estimate,
    };
    let claimed_struct = claimed.map(|(input, output)| Claimed { input, output });
    let verdict = evaluate(
        enabled,
        trust,
        &independent,
        claimed_struct.as_ref(),
        tolerance_pct,
    );
    let record = UsageRecord::from_verdict(
        request_id,
        provider_id,
        model,
        trust,
        is_estimate,
        independent_input,
        independent_output,
        claimed,
        &verdict,
    );
    Assessment { verdict, record }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assess_relay_inflation_end_to_end() {
        // relay 把 Anthropic usage 整体乘 1.5 → 检出 overbilled + 记录标 relay/estimate。
        let a = assess(
            true,
            15.0,
            Trust::Relay,
            Family::Anthropic,
            Some("req_x".into()),
            "shady-relay".into(),
            "claude-sonnet-4".into(),
            1000,
            2000,
            Some((1500, 3000)),
        );
        assert!(a.verdict.is_overbilled());
        assert_eq!(a.record.verdict, "overbilled");
        assert_eq!(a.record.trust, "relay");
        assert!(a.record.is_estimate);
        // 价表存在 → 成本被算出，且 claimed 成本 > expected 成本。
        let expected = a.record.expected_cost_usd.unwrap();
        let claimed = a.record.claimed_cost_usd.unwrap();
        assert!(claimed > expected, "虚报应使 claimed 成本高于应收");
    }

    #[test]
    fn assess_official_records_but_not_checked() {
        let a = assess(
            true,
            15.0,
            Trust::Official,
            Family::OpenAi,
            None,
            "openai".into(),
            "gpt-4o".into(),
            1000,
            500,
            Some((9999, 9999)),
        );
        assert_eq!(
            a.verdict,
            Verdict::NotChecked(NotCheckedReason::OfficialAuthoritative)
        );
        assert_eq!(a.record.verdict, "not_checked_official");
        assert!(!a.record.is_estimate, "OpenAI 计数非估算");
        assert!(a.record.deviation_pct.is_none(), "未核算不应有偏差值");
    }

    #[test]
    fn assess_disabled_is_zero_behavior() {
        let a = assess(
            false,
            15.0,
            Trust::Relay,
            Family::OpenAi,
            None,
            "relay".into(),
            "gpt-4o".into(),
            10,
            10,
            Some((9999, 9999)),
        );
        assert_eq!(a.verdict, Verdict::NotChecked(NotCheckedReason::Disabled));
    }
}
