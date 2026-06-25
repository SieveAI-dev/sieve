//! OpenAI SSE Parser fuzz target（SSE 边界 fuzz 全覆盖硬约束）。
//!
//! 覆盖：半行 chunk / 跨 chunk 分隔符 / C0 控制字符 / 多 event 粘包 /
//! 提前断流 / [DONE] 标记 / finish_reason 变体 / 空 delta / tool_calls delta。
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    sieve_core::fuzz_helpers::fuzz_one_sse_openai(data);
});
