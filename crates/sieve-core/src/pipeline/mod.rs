//! Pipeline 节点（架构图 ②⑦）：Week 2 起填充实现。

pub mod inbound;
pub mod outbound;

use crate::error::SieveCoreResult;
use crate::protocol::unified_message::UnifiedMessage;

/// Pipeline 节点 trait（占位，Week 2 起 OutboundFilter / InboundFilter / ToolUseAggregator 实现）。
///
/// 关联架构图节点 ②（出站过滤）和节点 ⑦（入站过滤）。
pub trait PipelineNode: Send + Sync {
    /// 节点名（用于审计日志，需稳定不变）。
    fn name(&self) -> &str;

    /// 处理一个 UnifiedMessage（Week 2 起返回 Detection 列表）。
    ///
    /// # Errors
    /// 处理失败时返回对应 [`crate::error::SieveCoreError`]。
    fn process(&self, msg: &mut UnifiedMessage) -> SieveCoreResult<()>;
}
