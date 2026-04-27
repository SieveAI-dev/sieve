//! 规则匹配引擎（占位，Week 2 起用 vectorscan 实现）。

use crate::error::SieveRulesResult;

/// 一次匹配的结果（Week 2 起扩展）。
#[derive(Debug, Clone)]
pub struct MatchHit {
    /// 命中的规则 ID（如 OUT-01）。
    pub rule_id: String,
    /// 命中位置在输入字节流的起始偏移（闭区间）。
    pub start: usize,
    /// 命中位置的结束偏移（开区间）。
    pub end: usize,
}

/// 多模式匹配引擎 trait。Week 2 起由 VectorscanEngine 实现。
pub trait MatchEngine: Send + Sync {
    /// 对输入字节流执行多模式匹配，返回所有命中。
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>>;
}
