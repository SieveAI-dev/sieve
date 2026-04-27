//! Fuzz helpers — cargo fuzz 与 AFL++ 共享的 fuzz 函数体。
//!
//! 关联 PRD §9 硬约束 #5：SSE 边界处理 fuzz test 全覆盖。
//!
//! 这些函数不包含具体的 fuzz corpus 逻辑，由 `fuzz/` 子 crate 的 target 调用。
//! 设计为幂等：无论输入如何都不 panic（满足 fuzz 的核心目标）。

use crate::sse::parser::SseParser;
use crate::tool_use_aggregator::Aggregator;

/// SSE Parser fuzz target。
///
/// 覆盖：半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 / 提前断流。
pub fn fuzz_one_sse(data: &[u8]) {
    let mut parser = SseParser::new();
    let _ = parser.push_chunk(data);
    let _ = parser.flush();
}

/// Tool Use Aggregator fuzz target（先 parse 再 aggregate）。
///
/// 覆盖：partial_json 跨 chunk 累积 / malformed JSON 不 panic。
pub fn fuzz_one_tool_use(data: &[u8]) {
    let mut parser = SseParser::new();
    let mut agg = Aggregator::new();
    for event in parser.push_chunk(data) {
        let _ = agg.process(&event);
    }
}

/// 端到端 fuzz target（parser → aggregator，含 flush）。
///
/// 覆盖：完整流式管道，含提前断流场景。
pub fn fuzz_one_pipeline(data: &[u8]) {
    let mut parser = SseParser::new();
    let mut agg = Aggregator::new();
    for event in parser.push_chunk(data) {
        let _ = agg.process(&event);
    }
    for event in parser.flush() {
        let _ = agg.process(&event);
    }
}
