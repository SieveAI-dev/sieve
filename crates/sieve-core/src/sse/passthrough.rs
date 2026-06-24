//! SSE 字节级透传（Week 1）。
//!
//! Week 1 不解析 SSE，直接把上游 response body 作为客户端 response body 返回。
//! 关键约束：任何缓冲都会破坏 chunk 边界，Week 1 不允许聚合。
//!
//! 由于 hyper 的 `Incoming` body 已实现流式 `poll_frame`，
//! 直接把上游 `Response::into_body()` 包装成响应 body 即可零成本透传。
//!
//! 此模块当前为占位 + 未来 Parser 接入点；具体透传由 sieve-cli daemon 完成。
//! Week 3 起替换为完整 SSE parser（含 fuzz test，见 .cursorrules §二 #5）。

use crate::error::SieveCoreResult;

/// 透传上下文（占位，Week 3 起加入 ParserState）。
pub struct PassthroughContext;

impl PassthroughContext {
    /// 新建透传上下文。
    pub fn new() -> Self {
        Self
    }

    /// 观察经过的 SSE chunk（Week 1 不做任何转换，返回 Ok(())）。
    ///
    /// Week 3 起此处插入 SSE parser，产出 [`crate::protocol::unified_message::UnifiedMessage`]。
    pub fn observe(&mut self, _chunk: &[u8]) -> SieveCoreResult<()> {
        Ok(())
    }
}

impl Default for PassthroughContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_empty_chunk_ok() {
        let mut ctx = PassthroughContext::new();
        assert!(ctx.observe(&[]).is_ok());
    }

    #[test]
    fn observe_arbitrary_bytes_ok() {
        let mut ctx = PassthroughContext::new();
        let chunk = b"data: {\"type\":\"content_block_delta\"}\n\n";
        assert!(ctx.observe(chunk).is_ok());
    }
}
