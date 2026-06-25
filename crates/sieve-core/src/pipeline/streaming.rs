//! 入站流式 Pipeline 节点 trait（关联 Pipeline 节点 ⑦）。
//!
//! Week 3 起 InboundFilter 实现本 trait；sieve-cli 通过 Arc<dyn StreamingPipelineNode>
//! 将其注入到流式代理处理循环中。

use crate::detection::Detection;
use crate::error::SieveCoreResult;
use crate::sse::parser::SseEvent;
use crate::tool_use_aggregator::CompletedToolCall;

/// 入站流式 Pipeline 节点接口。
///
/// 每个 SSE event 到达后调用 [`observe_event`]；
/// Tool Use 聚合完成后调用 [`on_tool_use_complete`]；
/// 流结束时调用 [`on_message_stop`]。
///
/// 所有方法返回 [`Vec<Detection>`]，空列表表示无命中。
pub trait StreamingPipelineNode: Send + Sync {
    /// 节点名（用于审计日志，需稳定不变）。
    fn name(&self) -> &str;

    /// 观察一个 SSE event，返回命中列表。
    ///
    /// 实现者应保持幂等性（同一 event 不应被重复处理）。
    ///
    /// # Errors
    /// 处理失败时返回 [`crate::error::SieveCoreError`]。
    fn observe_event(&mut self, event: &SseEvent) -> SieveCoreResult<Vec<Detection>>;

    /// 工具调用聚合完成回调，返回命中列表。
    ///
    /// 在 [`crate::tool_use_aggregator::Aggregator::process`] 返回
    /// `Some(CompletedToolCall)` 后由调用方触发。
    ///
    /// # Errors
    /// 处理失败时返回 [`crate::error::SieveCoreError`]。
    fn on_tool_use_complete(&mut self, tool: &CompletedToolCall)
        -> SieveCoreResult<Vec<Detection>>;

    /// 流结束回调（message_stop event 后调用），返回命中列表。
    ///
    /// 实现者可在此做会话级聚合检测（如 BIP39 助记词拼接检测）。
    ///
    /// # Errors
    /// 处理失败时返回 [`crate::error::SieveCoreError`]。
    fn on_message_stop(&mut self) -> SieveCoreResult<Vec<Detection>>;
}
