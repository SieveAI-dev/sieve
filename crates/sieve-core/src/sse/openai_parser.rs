//! OpenAI Chat Completions SSE 格式解析器（关联流式解析，Week 6）。
//!
//! ## 格式说明
//!
//! OpenAI SSE 格式仅含 `data:` 行，无 `event:` 头：
//! ```text
//! data: {"id":"chatcmpl-x","object":"chat.completion.chunk","choices":[...]}\n\n
//! data: [DONE]\n\n
//! ```
//!
//! ## 转换规则（SseEvent 映射）
//!
//! | OpenAI 字段 | 产出 `SseEvent` |
//! |------------|----------------|
//! | `delta.content` 非空 | `ContentBlockDelta { delta: TextDelta }` |
//! | `delta.tool_calls[*]` 首次出现（id/name/arguments 任一）| `ContentBlockStart { content_block: ToolUse }` |
//! | `delta.tool_calls[*].function.arguments` 增量 | `ContentBlockDelta { delta: InputJsonDelta }` |
//! | `finish_reason="tool_calls"` | 对所有已开 block 发 `ContentBlockStop`，再发 `MessageStop` |
//! | `finish_reason` 其他非 null 值 | `MessageStop` |
//! | `data: [DONE]` | 流结束信号（不产生 SseEvent） |
//! | `delta` 为空 | 0 个 SseEvent |
//!
//! ## Phase 1 限制
//!
//! - `choices` 数组只处理 `index=0` 的第一条（OpenAI 常用 `n=1`，多候选）
//! - `finish_reason="tool_calls"` 时额外设置 `has_tool_calls=true` 标记，
//!   调用方可通过 [`OpenAiSseParser::has_tool_calls`] 查询

use crate::protocol::openai::{OpenAIStreamingChunk, OpenAIToolCallDelta};
use crate::sse::parser::{SseDelta, SseEvent, SseParse, SseParserError, MAX_SSE_EVENT_BYTES};
use std::collections::HashSet;

// ── [DONE] 标记常量 ───────────────────────────────────────────────────────────

/// OpenAI SSE 流结束标记（`data: [DONE]`）。
const DONE_MARKER: &[u8] = b"[DONE]";

// ── 解析器主体 ────────────────────────────────────────────────────────────────

/// OpenAI Chat Completions SSE 增量解析器（实现 [`SseParse`] trait）。
///
/// 与 [`super::parser::SseParser`]（Anthropic 专用）共享 `SseEvent` 输出类型，
/// 使 pipeline / inbound_filter 无需感知上游协议差异（trait 抽象）。
///
/// ### tool_calls 状态机
///
/// `started_blocks` 记录已发出 `ContentBlockStart` 的 tool_call.index 集合，
/// 保证每个 index 只发一次 Start，且 `finish_reason="tool_calls"` 时发对应的 Stop。
///
/// 典型用法：
/// ```rust
/// use sieve_core::sse::openai_parser::OpenAiSseParser;
/// use sieve_core::sse::parser::SseParse;
///
/// let mut parser = OpenAiSseParser::new();
/// let events = parser.feed(
///     b"data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n"
/// ).unwrap();
/// assert_eq!(events.len(), 1);
/// ```
pub struct OpenAiSseParser {
    buf: Vec<u8>,
    /// `finish_reason="tool_calls"` 出现过时设为 true，供 inbound_filter 走 tool_use 路径。
    has_tool_calls: bool,
    /// 已发出 `ContentBlockStart` 的 tool_call.index 集合，防止重复发 Start。
    ///
    /// 在 finish_reason="tool_calls" 时遍历所有 index 发 ContentBlockStop。
    started_blocks: HashSet<u32>,
}

impl OpenAiSseParser {
    /// 新建解析器。
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(4096),
            has_tool_calls: false,
            started_blocks: HashSet::new(),
        }
    }

    /// 当前流是否含 tool_calls 类响应（`finish_reason="tool_calls"` 时为 `true`）。
    ///
    /// 供 inbound_filter 判断走 tool_use 拦截路径（finish_reason 处理）。
    pub fn has_tool_calls(&self) -> bool {
        self.has_tool_calls
    }

    /// 将一个完整的 `data:` payload（已去掉 `data:` 前缀和首尾空白）转换为 0~N 个 SseEvent。
    ///
    /// - `[DONE]` → 空列表（流结束，不产生 event）
    /// - 空 delta → 空列表
    /// - 只处理 `choices[0]`（Phase 1 限制）
    fn convert_data_line(&mut self, payload: &str) -> Vec<SseEvent> {
        // [DONE] 标记：流结束，不产生 SseEvent
        let trimmed = payload.trim();
        if trimmed.as_bytes() == DONE_MARKER {
            return Vec::new();
        }

        let chunk: OpenAIStreamingChunk = match serde_json::from_str(trimmed) {
            Ok(c) => c,
            // malformed JSON → 产生 0 个 event，不 panic（同 Anthropic 解析器 Unknown 策略）
            Err(_) => return Vec::new(),
        };

        let mut events = Vec::new();

        // include_usage 时末尾 chunk（choices 为空）携带 usage → 归一化为
        // MessageDelta，与 Anthropic SSE 对齐，供超额计费观测消费（否则 OpenAI SSE 拿不到
        // relay 声明的 usage，四路径覆盖留缺口）。
        if let Some(usage) = chunk.usage {
            events.push(SseEvent::MessageDelta {
                delta: serde_json::json!({}),
                usage: Some(usage),
            });
        }

        // Phase 1：只处理 choices[0]；usage-only chunk（choices 为空）返回上面已累计的 events。
        let choice = match chunk.choices.into_iter().next() {
            Some(c) => c,
            None => return events,
        };

        // finish_reason 处理
        // 注意：先处理 tool_calls delta（包含 Start/Delta），再发 Stop + MessageStop，
        // 保证 Aggregator 先收到 Start/Delta 才收到 Stop。
        let finish_reason = choice.finish_reason.clone();

        let delta = choice.delta;

        // delta.content 非空 → TextDelta
        if let Some(text) = delta.content {
            if !text.is_empty() {
                events.push(SseEvent::ContentBlockDelta {
                    index: 0,
                    delta: SseDelta::TextDelta { text },
                });
            }
        }

        // delta.tool_calls → ContentBlockStart（首次）+ InputJsonDelta（arguments 片段）
        if let Some(tool_calls) = delta.tool_calls {
            for tc in tool_calls {
                let tc_index = tc.index;

                // arguments 片段（先取，用于「是否开 block」判定 + 后续 InputJsonDelta）
                let partial_json = extract_arguments(&tc);
                let has_args = partial_json
                    .as_deref()
                    .map(|s| !s.is_empty())
                    .unwrap_or(false);

                // 首次出现此 index：只要带 id / name / arguments 任一即发 ContentBlockStart。
                //
                // 安全修复（四路由对等）：此前仅在「有 id 或 name」时才开 block。但 OpenAI
                // 流式下 tool_call 首帧可能只带 arguments 续传（多 tool_call 后续工具、中转站
                // 重组分片、或恶意上游故意构造）。若不开 block，aggregator 无对应条目 →
                // 后续 InputJsonDelta 被静默丢弃 → finish 时该 index 不在 started_blocks
                // 不发 ContentBlockStop → 永不产出 CompletedToolCall → on_tool_use_complete
                // 不触发 → Critical tool_use 检测（IN-CR-02/03/04/05）被整段绕过。开 block 后
                // finish 必发 Stop：partial_json 解析成功走检测、失败走 aggregator fail-closed。
                if !self.started_blocks.contains(&tc_index)
                    && (tc.id.is_some()
                        || tc.function.as_ref().and_then(|f| f.name.as_ref()).is_some()
                        || has_args)
                {
                    let id = tc.id.as_deref().unwrap_or("").to_owned();
                    let name = tc
                        .function
                        .as_ref()
                        .and_then(|f| f.name.as_deref())
                        .unwrap_or("")
                        .to_owned();
                    events.push(SseEvent::ContentBlockStart {
                        index: tc_index,
                        content_block: serde_json::json!({
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": {}
                        }),
                    });
                    self.started_blocks.insert(tc_index);
                }

                // arguments 片段 → InputJsonDelta
                if let Some(partial_json) = partial_json {
                    if !partial_json.is_empty() {
                        events.push(SseEvent::ContentBlockDelta {
                            // 用 tool_call index 做 block index，便于 aggregator 跨 chunk 对齐
                            index: tc_index,
                            delta: SseDelta::InputJsonDelta { partial_json },
                        });
                    }
                }
            }
        }

        // finish_reason 非 null → 可能需要发 ContentBlockStop（tool_calls 场景）+ MessageStop
        if let Some(ref reason) = finish_reason {
            if reason == "tool_calls" {
                self.has_tool_calls = true;
                // 对所有已开 block 发 ContentBlockStop（按 index 升序，保证确定性）
                let mut indices: Vec<u32> = self.started_blocks.iter().copied().collect();
                indices.sort_unstable();
                for idx in indices {
                    events.push(SseEvent::ContentBlockStop { index: idx });
                }
            }
            events.push(SseEvent::MessageStop);
        }

        events
    }
}

impl Default for OpenAiSseParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SseParse for OpenAiSseParser {
    /// 喂入一个 chunk，返回所有当前已可解析的完整 events。
    ///
    /// # Errors
    /// 若 buffer 累积超过 [`MAX_SSE_EVENT_BYTES`]，返回 [`SseParserError::EventTooLarge`]。
    fn feed(&mut self, chunk: &[u8]) -> Result<Vec<SseEvent>, SseParserError> {
        self.buf.extend_from_slice(chunk);

        // P0-5 容量上限（与 Anthropic 解析器相同上限）
        if self.buf.len() > MAX_SSE_EVENT_BYTES {
            return Err(SseParserError::EventTooLarge {
                len: self.buf.len(),
                max: MAX_SSE_EVENT_BYTES,
            });
        }

        let mut events = Vec::new();

        // OpenAI SSE event 以 \n\n 分隔（复用 find_event_end 逻辑）
        while let Some((event_end, sep_end)) = find_event_end(&self.buf) {
            let event_bytes = self.buf[..event_end].to_vec();
            self.buf.drain(..sep_end);
            events.extend(self.parse_openai_event(&event_bytes));
        }

        Ok(events)
    }

    /// 强制冲刷 buffer 中残留（连接关闭时调用）。
    ///
    /// 若 buffer 含完整 `data:` 行（仅缺末尾 `\n\n`），尝试解析并产生对应 SseEvent。
    /// 解析失败时丢弃 + warn（fail-safe；流已断，不能再 fail-closed 关流）。
    ///
    /// 参考 Anthropic [`super::parser::SseParser::flush`] 的残留事件处理策略（提前断流）。
    fn flush(&mut self) -> Vec<SseEvent> {
        let remaining = std::mem::take(&mut self.buf);
        if remaining.is_empty() {
            return Vec::new();
        }

        // 尝试将残留内容当作完整 event 解析（复用 parse_openai_event 路径）
        let events = self.parse_openai_event(&remaining);
        if events.is_empty() {
            // 真正的半行或解析失败：warn 后丢弃
            tracing::warn!(
                bytes = remaining.len(),
                "OpenAI SSE flush: 残留 {} 字节无法解析，丢弃（提前断流）",
                remaining.len()
            );
        }
        events
    }
}

// ── 内部辅助函数 ──────────────────────────────────────────────────────────────

/// 从单个 event 字节块中提取所有 OpenAI data 行并转换为 SseEvent 列表。
///
/// OpenAI SSE 无 `event:` 头，仅有 `data:` 行（格式差异）。
impl OpenAiSseParser {
    fn parse_openai_event(&mut self, bytes: &[u8]) -> Vec<SseEvent> {
        // C0 控制字符清洗（与 Anthropic 解析器保持一致）
        let cleaned: Vec<u8> = bytes
            .iter()
            .map(|&b| {
                if b < 0x20 && b != b'\t' && b != b'\n' && b != b'\r' {
                    b' '
                } else {
                    b
                }
            })
            .collect();

        let s = match std::str::from_utf8(&cleaned) {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let mut all_events = Vec::new();

        for line in s.lines() {
            if line.starts_with(':') || line.is_empty() {
                continue;
            }
            let payload = if let Some(p) = line.strip_prefix("data: ") {
                p
            } else if let Some(p) = line.strip_prefix("data:") {
                p
            } else {
                // 非 data: 行（OpenAI SSE 应无 event: 行，忽略其他行）
                continue;
            };

            all_events.extend(self.convert_data_line(payload));
        }

        all_events
    }
}

/// 提取 [`OpenAIToolCallDelta`] 中的 arguments 片段（None 表示此 chunk 无 arguments）。
fn extract_arguments(tc: &OpenAIToolCallDelta) -> Option<String> {
    tc.function
        .as_ref()
        .and_then(|f| f.arguments.as_ref())
        .cloned()
}

/// 找到 SSE event 边界（`\n\n` 或 `\r\n\r\n`），返回 `(event_end, separator_end)` 偏移。
///
/// 与 `parser.rs` 中的同名函数相同逻辑，此处单独复制避免跨模块暴露私有函数。
fn find_event_end(buf: &[u8]) -> Option<(usize, usize)> {
    let len = buf.len();
    let mut i = 0;
    while i < len {
        if i + 3 < len
            && buf[i] == b'\r'
            && buf[i + 1] == b'\n'
            && buf[i + 2] == b'\r'
            && buf[i + 3] == b'\n'
        {
            return Some((i, i + 4));
        }
        if i + 1 < len && buf[i] == b'\n' && buf[i + 1] == b'\n' {
            return Some((i, i + 2));
        }
        i += 1;
    }
    None
}

// ── 单元测试（13 个，覆盖任务书全部 case）────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sse::parser::{SseDelta, SseEvent};

    // 构造 OpenAI streaming chunk JSON（只含 delta.content）
    fn chunk_content(content: &str, finish: Option<&str>) -> String {
        let finish_str = match finish {
            Some(r) => format!("\"{}\"", r),
            None => "null".to_owned(),
        };
        format!(
            r#"{{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{{"index":0,"delta":{{"content":"{}"}},"finish_reason":{}}}]}}"#,
            content, finish_str
        )
    }

    // 构造 OpenAI streaming chunk JSON（只含 delta.tool_calls）
    fn chunk_tool(tc_index: u32, args_frag: &str) -> String {
        format!(
            r#"{{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{{"index":0,"delta":{{"tool_calls":[{{"index":{},"function":{{"arguments":"{}"}}}}]}},"finish_reason":null}}]}}"#,
            tc_index, args_frag
        )
    }

    fn make_data(json: &str) -> Vec<u8> {
        format!("data: {}\n\n", json).into_bytes()
    }

    // ─── include_usage 末尾 chunk（choices 空）→ MessageDelta 带 usage ──
    #[test]
    fn openai_usage_only_chunk_emits_message_delta() {
        let mut p = OpenAiSseParser::new();
        let json = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4o","choices":[],"usage":{"prompt_tokens":12,"completion_tokens":34,"total_tokens":46}}"#;
        let events = p.feed(&make_data(json)).unwrap();
        let usage = events
            .iter()
            .find_map(|e| match e {
                SseEvent::MessageDelta { usage: Some(u), .. } => Some(u.clone()),
                _ => None,
            })
            .expect("usage-only chunk 应发 MessageDelta 带 usage");
        assert_eq!(
            usage.get("prompt_tokens").and_then(|v| v.as_u64()),
            Some(12)
        );
        assert_eq!(
            usage.get("completion_tokens").and_then(|v| v.as_u64()),
            Some(34)
        );
    }

    // ─── Test 1: minimal 单条 data 含 delta.content="hi" ────────────────────
    #[test]
    fn openai_minimal_content_delta() {
        let mut p = OpenAiSseParser::new();
        let events = p.feed(&make_data(&chunk_content("hi", None))).unwrap();
        assert_eq!(events.len(), 1);
        if let SseEvent::ContentBlockDelta {
            index,
            delta: SseDelta::TextDelta { text },
        } = &events[0]
        {
            assert_eq!(*index, 0);
            assert_eq!(text, "hi");
        } else {
            panic!("expected TextDelta, got: {:?}", events[0]);
        }
    }

    // ─── Test 2: 多 chunk 生成 "hello world" ─────────────────────────────────
    #[test]
    fn openai_multi_chunk_text() {
        let mut p = OpenAiSseParser::new();
        let mut all = p.feed(&make_data(&chunk_content("hello", None))).unwrap();
        all.extend(p.feed(&make_data(&chunk_content(" world", None))).unwrap());
        assert_eq!(all.len(), 2);
        if let SseEvent::ContentBlockDelta {
            delta: SseDelta::TextDelta { text },
            ..
        } = &all[0]
        {
            assert_eq!(text, "hello");
        } else {
            panic!("unexpected: {:?}", all[0]);
        }
        if let SseEvent::ContentBlockDelta {
            delta: SseDelta::TextDelta { text },
            ..
        } = &all[1]
        {
            assert_eq!(text, " world");
        } else {
            panic!("unexpected: {:?}", all[1]);
        }
    }

    // ─── Test 3: tool_call arguments 增量（两个 chunk 拼接）──────────────────
    #[test]
    fn openai_tool_call_arguments_incremental() {
        let mut p = OpenAiSseParser::new();
        let c1 = chunk_tool(0, r#"{\"a"#);
        let c2 = chunk_tool(0, r#":1}"#);
        let mut all = p.feed(&make_data(&c1)).unwrap();
        all.extend(p.feed(&make_data(&c2)).unwrap());
        // 两个 chunk 各产生 1 个 InputJsonDelta
        let json_deltas: Vec<_> = all
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SseEvent::ContentBlockDelta {
                        delta: SseDelta::InputJsonDelta { .. },
                        ..
                    }
                )
            })
            .collect();
        assert_eq!(json_deltas.len(), 2);
    }

    // ─── Test 4: [DONE] 识别为流结束，不产生 event ───────────────────────────
    #[test]
    fn openai_done_produces_no_event() {
        let mut p = OpenAiSseParser::new();
        let events = p.feed(b"data: [DONE]\n\n").unwrap();
        assert!(events.is_empty(), "expected empty, got: {:?}", events);
    }

    // ─── Test 5: finish_reason="stop" 产生 MessageStop ───────────────────────
    #[test]
    fn openai_finish_reason_stop_produces_message_stop() {
        let mut p = OpenAiSseParser::new();
        // finish_reason="stop" 时 delta.content 通常为空，但仍测试 MessageStop
        let json = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#;
        let events = p.feed(&make_data(json)).unwrap();
        assert!(
            events.contains(&SseEvent::MessageStop),
            "expected MessageStop, got: {:?}",
            events
        );
        assert!(!p.has_tool_calls());
    }

    // ─── Test 6: finish_reason="tool_calls" 产生 MessageStop + has_tool_calls ─
    #[test]
    fn openai_finish_reason_tool_calls() {
        let mut p = OpenAiSseParser::new();
        let json = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}"#;
        let events = p.feed(&make_data(json)).unwrap();
        assert!(
            events.contains(&SseEvent::MessageStop),
            "expected MessageStop, got: {:?}",
            events
        );
        assert!(p.has_tool_calls(), "expected has_tool_calls=true");
    }

    // ─── Test 7: 半行 chunk（无 \n\n）→ 不产生 event ─────────────────────────
    #[test]
    fn openai_half_line_chunk_no_event() {
        let mut p = OpenAiSseParser::new();
        // 故意不附 \n\n，event 留在 buffer
        let events = p
            .feed(b"data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\"")
            .unwrap();
        assert!(events.is_empty(), "expected empty, got: {:?}", events);
    }

    // ─── Test 8: 跨 chunk 分隔符（\n 然后 \n）────────────────────────────────
    #[test]
    fn openai_cross_chunk_separator() {
        let mut p = OpenAiSseParser::new();
        let json = chunk_content("x", None);
        let full = format!("data: {}\n", json);
        let mut events = p.feed(full.as_bytes()).unwrap();
        // 第一个 chunk 只有一个 \n，不完整
        assert!(events.is_empty());
        events.extend(p.feed(b"\n").unwrap());
        // 第二个 chunk 补全 \n\n，现在可以解析
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            SseEvent::ContentBlockDelta {
                delta: SseDelta::TextDelta { .. },
                ..
            }
        ));
    }

    // ─── Test 9: C0 控制字符被安全处理（不 panic）───────────────────────────
    #[test]
    fn openai_c0_control_chars_safe() {
        let mut p = OpenAiSseParser::new();
        // 在 data 行中注入 \x01 等 C0 字符，解析器应不 panic，结果不需要有效 event
        let raw = b"data: \x01{\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\n";
        let result = p.feed(raw);
        // 不 panic，不 Err（C0 替换为空格后 JSON 解析可能失败，但不 panic）
        assert!(result.is_ok());
    }

    // ─── Test 10: 空 delta → 0 个 SseEvent ──────────────────────────────────
    #[test]
    fn openai_empty_delta_no_event() {
        let mut p = OpenAiSseParser::new();
        let json = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":null}]}"#;
        let events = p.feed(&make_data(json)).unwrap();
        assert!(events.is_empty(), "expected empty, got: {:?}", events);
    }

    // ─── Test 11: 多 event 粘包（3 个 data 行连续）───────────────────────────
    #[test]
    fn openai_multi_event_packed() {
        let mut p = OpenAiSseParser::new();
        let c1 = chunk_content("a", None);
        let c2 = chunk_content("b", None);
        let c3 = chunk_content("c", None);
        let packed = format!("data: {}\n\ndata: {}\n\ndata: {}\n\n", c1, c2, c3);
        let events = p.feed(packed.as_bytes()).unwrap();
        let text_deltas: Vec<_> = events
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SseEvent::ContentBlockDelta {
                        delta: SseDelta::TextDelta { .. },
                        ..
                    }
                )
            })
            .collect();
        assert_eq!(text_deltas.len(), 3);
    }

    // ─── Test 12: 提前断流（不完整 data 行）→ flush 丢弃半行，不 panic ────────
    #[test]
    fn openai_premature_eof_flush_safe() {
        let mut p = OpenAiSseParser::new();
        // 喂入半行，不带 \n\n
        let _ = p.feed(b"data: {\"id\":\"x\",\"incomplete\"").unwrap();
        // flush 应安全丢弃，不 panic
        let flushed = p.flush();
        assert!(
            flushed.is_empty(),
            "expected empty on flush, got: {:?}",
            flushed
        );
    }

    // ─── Test R6-#3a: 完整 OpenAI tool_call 流 → Aggregator 输出 CompletedToolCall ─
    #[test]
    fn openai_tool_call_e2e_aggregator() {
        use crate::tool_use_aggregator::Aggregator;

        let mut p = OpenAiSseParser::new();
        let mut agg = Aggregator::new();

        // Chunk 1: 首个 delta，含 id + function.name（首次出现 index=0，应发 ContentBlockStart）
        let chunk1 = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{"role":"assistant","tool_calls":[{"index":0,"id":"call_001","type":"function","function":{"name":"bash","arguments":""}}]},"finish_reason":null}]}"#;
        // Chunk 2: arguments 第一片
        let chunk2 = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"cmd\":"}}]},"finish_reason":null}]}"#;
        // Chunk 3: arguments 第二片
        let chunk3 = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"ls\"}"}}]},"finish_reason":null}]}"#;
        // Chunk 4: finish_reason="tool_calls"，应发 ContentBlockStop + MessageStop
        let chunk4 = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}"#;

        let mut all_events = Vec::new();
        for chunk in [chunk1, chunk2, chunk3, chunk4] {
            all_events.extend(p.feed(&make_data(chunk)).unwrap());
        }

        assert!(
            p.has_tool_calls(),
            "has_tool_calls should be true after finish_reason=tool_calls"
        );

        // 验证事件序列含 ContentBlockStart, ContentBlockDelta, ContentBlockStop, MessageStop
        let has_start = all_events
            .iter()
            .any(|e| matches!(e, SseEvent::ContentBlockStart { index: 0, .. }));
        let has_delta = all_events.iter().any(|e| {
            matches!(
                e,
                SseEvent::ContentBlockDelta {
                    index: 0,
                    delta: SseDelta::InputJsonDelta { .. },
                    ..
                }
            )
        });
        let has_stop = all_events
            .iter()
            .any(|e| matches!(e, SseEvent::ContentBlockStop { index: 0 }));
        let has_msg_stop = all_events
            .iter()
            .any(|e| matches!(e, SseEvent::MessageStop));

        assert!(
            has_start,
            "missing ContentBlockStart in events: {all_events:?}"
        );
        assert!(
            has_delta,
            "missing ContentBlockDelta(InputJsonDelta) in events: {all_events:?}"
        );
        assert!(
            has_stop,
            "missing ContentBlockStop in events: {all_events:?}"
        );
        assert!(
            has_msg_stop,
            "missing MessageStop in events: {all_events:?}"
        );

        // Aggregator end-to-end：喂入所有事件，应产出 1 个 CompletedToolCall
        let mut completed = Vec::new();
        for event in &all_events {
            if let Ok(Some(tool)) = agg.process(event) {
                completed.push(tool);
            }
        }
        assert_eq!(
            completed.len(),
            1,
            "expected 1 CompletedToolCall, got {}: {all_events:?}",
            completed.len()
        );
        assert_eq!(completed[0].id, "call_001");
        assert_eq!(completed[0].name, "bash");
        // 拼接后的 arguments: {"cmd":"ls"}
        assert_eq!(
            completed[0].input.get("cmd").and_then(|v| v.as_str()),
            Some("ls")
        );
    }

    // ─── 安全回归：tool_call 首帧无 id/name（仅 arguments）仍须被聚合，不得绕过检测 ───
    // 恶意 / 中转站上游可构造首帧只含 arguments 的 tool_call SSE。修复前 parser 仅在
    // 「有 id 或 name」时开 block → aggregator 静默丢弃后续 delta → finish 不发 Stop →
    // 无 CompletedToolCall → on_tool_use_complete 不触发 → Critical tool_use 检测被整段
    // 绕过。本测试锁死：危险 tool_call 必产出 CompletedToolCall 交检测。修复前 completed=0。
    #[test]
    fn openai_tool_call_first_chunk_without_id_still_aggregated() {
        use crate::tool_use_aggregator::Aggregator;

        let mut p = OpenAiSseParser::new();
        let mut agg = Aggregator::new();

        // 首帧只带 arguments（无 id、无 function.name）—— 危险命令首片
        let chunk1 = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"command\":\"rm -rf"}}]},"finish_reason":null}]}"#;
        let chunk2 = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"function":{"arguments":" /\"}"}}]},"finish_reason":null}]}"#;
        let chunk3 = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}"#;

        let mut all_events = Vec::new();
        for chunk in [chunk1, chunk2, chunk3] {
            all_events.extend(p.feed(&make_data(chunk)).unwrap());
        }

        // 修复关键：即便首帧无 id/name，也必须开 block 并在 finish 发 Stop。
        assert!(
            all_events
                .iter()
                .any(|e| matches!(e, SseEvent::ContentBlockStart { index: 0, .. })),
            "首帧无 id/name 也必须发 ContentBlockStart，否则 tool_call 绕过检测: {all_events:?}"
        );
        assert!(
            all_events
                .iter()
                .any(|e| matches!(e, SseEvent::ContentBlockStop { index: 0 })),
            "finish 必须发 ContentBlockStop: {all_events:?}"
        );

        // Aggregator 必须产出 CompletedToolCall（交 on_tool_use_complete 做 Critical 检测）
        let mut completed = Vec::new();
        for event in &all_events {
            if let Ok(Some(tool)) = agg.process(event) {
                completed.push(tool);
            }
        }
        assert_eq!(
            completed.len(),
            1,
            "首帧无 id/name 的 tool_call 必须产出 CompletedToolCall（修复前为 0，被绕过）: {all_events:?}"
        );
        // id/name 缺失用空串占位，但危险 arguments 完整保留供检测扫描
        assert_eq!(completed[0].id, "");
        assert_eq!(completed[0].name, "");
        assert_eq!(
            completed[0].input.get("command").and_then(|v| v.as_str()),
            Some("rm -rf /")
        );
    }

    // ─── Test R6-#3b: ContentBlockStart 对同一 index 只发一次 ──────────────────
    #[test]
    fn openai_tool_call_start_emitted_only_once_per_index() {
        let mut p = OpenAiSseParser::new();

        // 两个 chunk 都含同一 index=0 的 id+name，Start 只应发一次
        let chunk1 = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_abc","type":"function","function":{"name":"sign","arguments":""}}]},"finish_reason":null}]}"#;
        let chunk2 = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_abc","type":"function","function":{"name":"sign","arguments":"{}"}}]},"finish_reason":null}]}"#;

        let mut events = p.feed(&make_data(chunk1)).unwrap();
        events.extend(p.feed(&make_data(chunk2)).unwrap());

        let start_count = events
            .iter()
            .filter(|e| matches!(e, SseEvent::ContentBlockStart { index: 0, .. }))
            .count();
        assert_eq!(
            start_count, 1,
            "ContentBlockStart for index=0 should appear exactly once, got {start_count}: {events:?}"
        );
    }

    // ─── Test R7-#1a: flush 残留 data 行（缺末尾 \n\n） → 产生 TextDelta ────────
    #[test]
    fn flush_residual_data_produces_text_delta() {
        let mut p = OpenAiSseParser::new();
        // 喂入完整 JSON 但不带 \n\n，event 留在 buffer
        let json = chunk_content("hello", None);
        let raw = format!("data: {}", json);
        let _ = p.feed(raw.as_bytes()).unwrap();
        // flush 应解析残留，产生 1 个 ContentBlockDelta TextDelta("hello")
        let events = p.flush();
        assert_eq!(
            events.len(),
            1,
            "expected 1 event from flush, got: {events:?}"
        );
        if let SseEvent::ContentBlockDelta {
            index,
            delta: SseDelta::TextDelta { text },
        } = &events[0]
        {
            assert_eq!(*index, 0);
            assert_eq!(text, "hello");
        } else {
            panic!("expected TextDelta, got: {:?}", events[0]);
        }
    }

    // ─── Test R7-#1b: flush 残留 tool_calls 首次出现 → ContentBlockStart + InputJsonDelta ─
    #[test]
    fn flush_residual_tool_calls_start_and_delta() {
        let mut p = OpenAiSseParser::new();
        // 含 id+name 的首次 tool_calls delta，缺末尾 \n\n
        let json = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_999","type":"function","function":{"name":"deploy","arguments":"{}"}}]},"finish_reason":null}]}"#;
        let raw = format!("data: {}", json);
        let _ = p.feed(raw.as_bytes()).unwrap();
        let events = p.flush();
        // 应产生 ContentBlockStart（首次 index=0）+ ContentBlockDelta InputJsonDelta
        let has_start = events
            .iter()
            .any(|e| matches!(e, SseEvent::ContentBlockStart { index: 0, .. }));
        let has_delta = events.iter().any(|e| {
            matches!(
                e,
                SseEvent::ContentBlockDelta {
                    index: 0,
                    delta: SseDelta::InputJsonDelta { .. },
                    ..
                }
            )
        });
        assert!(
            has_start,
            "expected ContentBlockStart from flush, got: {events:?}"
        );
        assert!(
            has_delta,
            "expected InputJsonDelta from flush, got: {events:?}"
        );
    }

    // ─── Test R7-#1c: flush 含 finish_reason="tool_calls" → ContentBlockStop + MessageStop ─
    #[test]
    fn flush_finish_reason_tool_calls_produces_stop_events() {
        let mut p = OpenAiSseParser::new();
        // 先通过正常 feed 建立一个已开的 block（有 \n\n 的完整 chunk）
        let start_chunk = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call_abc","type":"function","function":{"name":"sign","arguments":""}}]},"finish_reason":null}]}"#;
        let _ = p.feed(&make_data(start_chunk)).unwrap();

        // finish_reason chunk 不带 \n\n，留在 buffer
        let finish_json = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4","choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]}"#;
        let raw = format!("data: {}", finish_json);
        let _ = p.feed(raw.as_bytes()).unwrap();

        let events = p.flush();
        let has_stop = events
            .iter()
            .any(|e| matches!(e, SseEvent::ContentBlockStop { index: 0 }));
        let has_msg_stop = events.iter().any(|e| matches!(e, SseEvent::MessageStop));
        assert!(
            has_stop,
            "expected ContentBlockStop from flush, got: {events:?}"
        );
        assert!(
            has_msg_stop,
            "expected MessageStop from flush, got: {events:?}"
        );
        assert!(
            p.has_tool_calls(),
            "expected has_tool_calls=true after flush"
        );
    }

    // ─── Test R7-#1d: flush 解析失败（非法 JSON）→ 不 panic，返回空 events ─────
    #[test]
    fn flush_invalid_json_no_panic_returns_empty() {
        let mut p = OpenAiSseParser::new();
        // 喂入无效 JSON，不带 \n\n
        let _ = p.feed(b"data: notvalidjson").unwrap();
        // flush 应不 panic，返回空列表（解析失败丢弃）
        let events = p.flush();
        assert!(
            events.is_empty(),
            "expected empty on invalid JSON flush, got: {events:?}"
        );
    }

    // ─── Test 13: 混合协议——Anthropic parser 不解析 OpenAI 格式（反之亦然）──
    #[test]
    fn protocol_isolation_anthropic_vs_openai() {
        use crate::sse::parser::SseParser;

        // OpenAI 格式（仅 data:，无 event: 行）喂给 Anthropic parser → Unknown
        let mut anthropic = SseParser::new();
        let openai_chunk = chunk_content("hi", None);
        let events = anthropic.push_chunk(&make_data(&openai_chunk)).unwrap();
        // Anthropic parser 无法识别 OpenAI chunk 结构（没有 "type" 字段） → Unknown
        assert!(
            events.iter().all(|e| matches!(e, SseEvent::Unknown)),
            "Anthropic parser should return Unknown for OpenAI chunks, got: {:?}",
            events
        );

        // Anthropic 格式（含 event: ping）喂给 OpenAI parser → 0 个 event（无 data: 可解析）或丢弃
        let mut openai_p = OpenAiSseParser::new();
        let anthropic_chunk = b"event: ping\ndata: {\"type\":\"ping\"}\n\n";
        let events2 = openai_p.feed(anthropic_chunk).unwrap();
        // OpenAI parser 处理此 chunk 时遇到 data: 行，尝试解析 {"type":"ping"} 为 OpenAIStreamingChunk
        // 但缺少 id/object/created/model 字段，JSON 解析失败 → 0 个 event
        assert!(
            events2.is_empty(),
            "OpenAI parser should produce 0 events for Anthropic SSE, got: {:?}",
            events2
        );
    }
}
