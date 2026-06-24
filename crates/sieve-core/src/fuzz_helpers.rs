//! Fuzz helpers — cargo fuzz 与 AFL++ 共享的 fuzz 函数体。
//!
//! 关联硬约束：SSE 边界处理 fuzz test 全覆盖。
//! 新增 OpenAI SSE parser fuzz target（`fuzz_one_sse_openai`）。
//!
//! 这些函数不包含具体的 fuzz corpus 逻辑，由 `fuzz/` 子 crate 的 target 调用。
//! 设计为幂等：无论输入如何都不 panic（满足 fuzz 的核心目标）。

use crate::sse::openai_parser::OpenAiSseParser;
use crate::sse::parser::{SseParse, SseParser};
use crate::tool_use_aggregator::Aggregator;

/// Anthropic SSE Parser fuzz target。
///
/// 覆盖：半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 / 提前断流。
/// 容量超限时返回 Err，忽略即可（fuzz 目标是不 panic）。
pub fn fuzz_one_sse(data: &[u8]) {
    let mut parser = SseParser::new();
    let _ = parser.feed(data);
    let _ = parser.flush();
}

/// OpenAI SSE Parser fuzz target（关联 fuzz 覆盖）。
///
/// 覆盖：半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 /
/// 提前断流 / [DONE] 标记 / finish_reason 变体 / 空 delta / tool_calls delta。
/// 容量超限时返回 Err，忽略即可（fuzz 目标是不 panic）。
pub fn fuzz_one_sse_openai(data: &[u8]) {
    let mut parser = OpenAiSseParser::new();
    let _ = parser.feed(data);
    let _ = parser.flush();
    // 读取但不使用 has_tool_calls，确保该路径被 fuzz 覆盖
    let _ = parser.has_tool_calls();
}

/// Tool Use Aggregator fuzz target（先 parse 再 aggregate）。
///
/// 覆盖：partial_json 跨 chunk 累积 / malformed JSON 不 panic。
/// 容量超限时返回 Err，忽略即可（fuzz 目标是不 panic）。
pub fn fuzz_one_tool_use(data: &[u8]) {
    let mut parser = SseParser::new();
    let mut agg = Aggregator::new();
    if let Ok(events) = parser.feed(data) {
        for event in events {
            let _ = agg.process(&event);
        }
    }
}

/// 端到端 fuzz target（parser → aggregator，含 flush）。
///
/// 覆盖：完整流式管道，含提前断流场景。
/// 容量超限时返回 Err，忽略即可（fuzz 目标是不 panic）。
pub fn fuzz_one_pipeline(data: &[u8]) {
    let mut parser = SseParser::new();
    let mut agg = Aggregator::new();
    if let Ok(events) = parser.feed(data) {
        for event in events {
            let _ = agg.process(&event);
        }
    }
    for event in parser.flush() {
        let _ = agg.process(&event);
    }
}
