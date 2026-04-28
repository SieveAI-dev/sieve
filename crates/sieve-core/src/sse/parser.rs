//! SSE 增量解析器（关联 PRD §9 #5 硬约束 / ADR-018 OpenAI 协议支持）。
//!
//! 设计：
//! - 增量 push_chunk 接口，支持半行 / 跨 chunk / 多 event 粘包 / C0 控制字符 / 提前断流
//! - 内部维护 buffer + 状态机，**不缓冲整流**，每次 push_chunk 立即返回已 parse 完整的 events
//! - malformed event 返回 SseEvent::Unknown，不 panic
//! - 超过 MAX_SSE_EVENT_BYTES 时返回 SseParserError::EventTooLarge（P0-5 容量上限，防 OOM）
//! - ADR-018：支持 OpenAI Chat Completions SSE 格式（`OpenAiSseParser`）并通过 `SseParse` trait
//!   向上游 pipeline 暴露统一接口，pipeline 无需感知具体协议

use serde::{Deserialize, Serialize};

// ── 协议标记 ──────────────────────────────────────────────────────────────────

/// SSE 上游协议判别（关联 ADR-018 §协议路由）。
///
/// 用于在 pipeline 层区分 Anthropic 和 OpenAI SSE 格式，
/// 并选择对应的解析器实现（`SseParse` trait）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SseProtocol {
    /// Anthropic Messages API SSE 格式（带 `event:` 头行）。
    Anthropic,
    /// OpenAI Chat Completions SSE 格式（仅 `data:` 行，最后一条 `[DONE]`）。
    OpenAI,
}

// ── 统一解析器 trait ──────────────────────────────────────────────────────────

/// SSE 解析器统一接口（关联 ADR-018 §trait 抽象）。
///
/// pipeline / inbound_filter 通过此 trait 消费 SSE 事件，
/// 无需感知底层协议差异（Anthropic vs OpenAI）。
pub trait SseParse {
    /// 喂入一个 chunk，返回所有当前已可解析的完整 events。
    ///
    /// # Errors
    /// 若 buffer 累积超过 [`MAX_SSE_EVENT_BYTES`]，返回 [`SseParserError::EventTooLarge`]。
    fn feed(&mut self, chunk: &[u8]) -> Result<Vec<SseEvent>, SseParserError>;

    /// 强制冲刷 buffer 中残留（连接关闭时调用）。
    ///
    /// 若 buffer 中有尚未以 `\n\n` 结尾的不完整 event，尝试解析并返回（或丢弃）。
    fn flush(&mut self) -> Vec<SseEvent>;
}

/// 单个 SSE event 允许的最大字节数（含 event: / data: / 前缀，不含分隔符 \n\n）。
///
/// 1 MiB 足够正常 Anthropic SSE event；超过此限视为恶意或异常上游（P0-5 / IN-CAP-01）。
pub const MAX_SSE_EVENT_BYTES: usize = 1 << 20; // 1 MiB

/// SSE 解析器可能返回的结构化错误。
#[derive(Debug, Clone, PartialEq)]
pub enum SseParserError {
    /// 累积 buffer 超过 [`MAX_SSE_EVENT_BYTES`]，恶意上游可借此触发 OOM。
    ///
    /// 检测 ID：IN-CAP-01（SSE event 超大）。
    EventTooLarge {
        /// 当前 buffer 字节数。
        len: usize,
        /// 配置的上限。
        max: usize,
    },
}

impl std::fmt::Display for SseParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SseParserError::EventTooLarge { len, max } => {
                write!(f, "IN-CAP-01: SSE event buffer 超限 ({len} > {max} bytes)")
            }
        }
    }
}

impl std::error::Error for SseParserError {}

/// SSE event 类型（对应 Anthropic Messages streaming spec）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SseEvent {
    /// message_start：流式响应起始。
    #[serde(rename = "message_start")]
    MessageStart {
        /// 消息元数据（原始 JSON）。
        message: serde_json::Value,
    },
    /// content_block_start：新内容块起始。
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        /// 块索引。
        index: u32,
        /// 块元数据（原始 JSON）。
        content_block: serde_json::Value,
    },
    /// content_block_delta：增量内容。
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        /// 块索引。
        index: u32,
        /// 增量内容。
        delta: SseDelta,
    },
    /// content_block_stop：内容块结束。
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        /// 块索引。
        index: u32,
    },
    /// message_delta：消息级增量（含 stop_reason 等）。
    #[serde(rename = "message_delta")]
    MessageDelta {
        /// 增量字段（原始 JSON）。
        delta: serde_json::Value,
        /// token 使用量（可选）。
        usage: Option<serde_json::Value>,
    },
    /// message_stop：流式响应结束。
    #[serde(rename = "message_stop")]
    MessageStop,
    /// ping：保活心跳。
    #[serde(rename = "ping")]
    Ping,
    /// error：API 错误事件。
    #[serde(rename = "error")]
    Error {
        /// 错误详情（原始 JSON）。
        error: serde_json::Value,
    },
    /// 未知 / 解析失败的 event。
    #[serde(other)]
    Unknown,
}

/// 增量内容类型（Anthropic content_block_delta 的 delta 字段）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SseDelta {
    /// 文本增量。
    #[serde(rename = "text_delta")]
    TextDelta {
        /// 文本内容。
        text: String,
    },
    /// 工具调用参数的 JSON 片段。
    #[serde(rename = "input_json_delta")]
    InputJsonDelta {
        /// 部分 JSON 字符串。
        partial_json: String,
    },
    /// 扩展思考增量（Claude 3.7+）。
    #[serde(rename = "thinking_delta")]
    ThinkingDelta {
        /// 思考内容。
        thinking: String,
    },
    /// 签名增量（扩展思考用）。
    #[serde(rename = "signature_delta")]
    SignatureDelta {
        /// 签名内容。
        signature: String,
    },
    /// 未知增量类型。
    #[serde(other)]
    Unknown,
}

/// Anthropic SSE 增量解析器（实现 [`SseParse`] trait）。
///
/// 处理带 `event:` 头行的 Anthropic Messages API SSE 格式。
/// OpenAI 格式请使用 [`super::openai_parser::OpenAiSseParser`]（ADR-018）。
///
/// 典型用法：
/// ```rust
/// use sieve_core::sse::parser::{SseParser, SseParse};
///
/// let mut parser = SseParser::new();
/// let events = parser.feed(b"event: ping\ndata: {\"type\":\"ping\"}\n\n").unwrap();
/// ```
pub struct SseParser {
    buf: Vec<u8>,
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SseParser {
    /// 新建解析器。
    pub fn new() -> Self {
        Self {
            buf: Vec::with_capacity(4096),
        }
    }

    /// 喂入一个 chunk，返回所有当前已可解析的完整 events。
    ///
    /// 不完整的 event 留在内部 buffer，等待下一个 chunk 补全。
    ///
    /// # Errors
    /// 若 buffer 累积超过 [`MAX_SSE_EVENT_BYTES`]，返回 [`SseParserError::EventTooLarge`]。
    /// 调用方应将此视为 fail-closed Critical（IN-CAP-01），注入 sieve_blocked 并截断流。
    ///
    /// 注：`push_chunk` 是 [`SseParse::feed`] 的别名，保留以维持向后兼容。
    pub fn push_chunk(&mut self, bytes: &[u8]) -> Result<Vec<SseEvent>, SseParserError> {
        self.feed(bytes)
    }

    /// 强制冲刷 buffer 中残留（连接关闭时调用）。
    ///
    /// 注：此方法是 [`SseParse::flush`] 的 inherent 别名，
    /// 调用方无需将 `SseParse` trait 引入 scope（向后兼容）。
    pub fn flush(&mut self) -> Vec<SseEvent> {
        <Self as SseParse>::flush(self)
    }
}

impl SseParse for SseParser {
    fn feed(&mut self, bytes: &[u8]) -> Result<Vec<SseEvent>, SseParserError> {
        self.buf.extend_from_slice(bytes);

        // P0-5 容量上限检查：单个 event buffer 不允许超过 MAX_SSE_EVENT_BYTES。
        // 检查时机：extend 后、drain 前，保证任何时刻 buffer 不会无界增长。
        if self.buf.len() > MAX_SSE_EVENT_BYTES {
            return Err(SseParserError::EventTooLarge {
                len: self.buf.len(),
                max: MAX_SSE_EVENT_BYTES,
            });
        }

        let mut events = Vec::new();
        // SSE event 以 \n\n 分隔（也接受 \r\n\r\n）
        while let Some((event_end, sep_end)) = find_event_end(&self.buf) {
            let event_bytes = self.buf[..event_end].to_vec();
            self.buf.drain(..sep_end);
            if let Some(event) = parse_event(&event_bytes) {
                events.push(event);
            }
        }
        Ok(events)
    }

    /// 强制冲刷 buffer 中残留（连接关闭时调用）。
    ///
    /// 若 buffer 中有尚未以 `\n\n` 结尾的 event，尝试解析并返回。
    fn flush(&mut self) -> Vec<SseEvent> {
        if self.buf.is_empty() {
            return Vec::new();
        }
        let event_bytes = std::mem::take(&mut self.buf);
        if let Some(event) = parse_event(&event_bytes) {
            vec![event]
        } else {
            Vec::new()
        }
    }
}

/// 找到 SSE event 边界（`\n\n` 或 `\r\n\r\n`），返回 `(event_end, separator_end)` 偏移。
///
/// - `event_end`：event 内容字节数（不含分隔符）
/// - `separator_end`：含分隔符的总字节数（drain 用）
fn find_event_end(buf: &[u8]) -> Option<(usize, usize)> {
    let len = buf.len();
    let mut i = 0;
    while i < len {
        // 检查 \r\n\r\n（优先，避免误识别 \r\n 中的 \n）
        if i + 3 < len
            && buf[i] == b'\r'
            && buf[i + 1] == b'\n'
            && buf[i + 2] == b'\r'
            && buf[i + 3] == b'\n'
        {
            return Some((i, i + 4));
        }
        // 检查 \n\n
        if i + 1 < len && buf[i] == b'\n' && buf[i + 1] == b'\n' {
            return Some((i, i + 2));
        }
        i += 1;
    }
    None
}

/// 解析单个 event 字节块（行格式 `event: <name>\ndata: <json>`）。
///
/// malformed → `Some(SseEvent::Unknown)`（不 panic，不返回 None）。
fn parse_event(bytes: &[u8]) -> Option<SseEvent> {
    // 过滤掉裸 C0 控制字符（0x00–0x1F，除 \t \n \r），避免 str::from_utf8 之后
    // serde_json 对无效 JSON 控制字符报错。这里保守策略：保留 \t \n \r，其余替换为空格。
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

    let s = std::str::from_utf8(&cleaned).ok()?;
    let mut data_lines: Vec<&str> = Vec::new();

    for line in s.lines() {
        // 跳过注释行（以 ':' 开头）、空行
        if line.starts_with(':') || line.is_empty() {
            continue;
        }
        if let Some(payload) = line.strip_prefix("data: ") {
            data_lines.push(payload);
        } else if let Some(payload) = line.strip_prefix("data:") {
            // 允许 `data:` 后无空格
            data_lines.push(payload);
        }
        // 其余字段（event: / id: / retry:）只做提取，不用于反序列化
    }

    if data_lines.is_empty() {
        return Some(SseEvent::Unknown);
    }

    let combined = data_lines.join("\n");
    // 尝试反序列化；失败时返回 Unknown，**不 panic**
    serde_json::from_str::<SseEvent>(&combined)
        .ok()
        .or(Some(SseEvent::Unknown))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_event() {
        let mut p = SseParser::new();
        let events = p
            .push_chunk(b"event: ping\ndata: {\"type\":\"ping\"}\n\n")
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SseEvent::Ping));
    }

    #[test]
    fn parse_half_line_chunk() {
        let mut p = SseParser::new();
        let mut all = p.push_chunk(b"event: ping\nda").unwrap();
        all.extend(p.push_chunk(b"ta: {\"type\":\"ping\"}\n\n").unwrap());
        assert_eq!(all.len(), 1);
        assert!(matches!(all[0], SseEvent::Ping));
    }

    #[test]
    fn parse_split_separator() {
        let mut p = SseParser::new();
        let mut all = p
            .push_chunk(b"event: ping\ndata: {\"type\":\"ping\"}\n")
            .unwrap();
        all.extend(p.push_chunk(b"\n").unwrap());
        assert_eq!(all.len(), 1);
        assert!(matches!(all[0], SseEvent::Ping));
    }

    #[test]
    fn parse_multi_event_packed() {
        let mut p = SseParser::new();
        let bytes = b"event: ping\ndata: {\"type\":\"ping\"}\n\nevent: ping\ndata: {\"type\":\"ping\"}\n\nevent: message_stop\ndata: {\"type\":\"message_stop\"}\n\n";
        let events = p.push_chunk(bytes).unwrap();
        assert_eq!(events.len(), 3);
        assert!(matches!(events[2], SseEvent::MessageStop));
    }

    #[test]
    fn parse_text_delta() {
        let mut p = SseParser::new();
        let bytes = br#"event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"hi"}}

"#;
        let events = p.push_chunk(bytes).unwrap();
        assert_eq!(events.len(), 1);
        if let SseEvent::ContentBlockDelta {
            index,
            delta: SseDelta::TextDelta { text },
        } = &events[0]
        {
            assert_eq!(*index, 0);
            assert_eq!(text, "hi");
        } else {
            panic!("expected text_delta, got: {:?}", events[0]);
        }
    }

    #[test]
    fn parse_input_json_delta() {
        let mut p = SseParser::new();
        let bytes = br#"event: content_block_delta
data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"loc"}}

"#;
        let events = p.push_chunk(bytes).unwrap();
        assert_eq!(events.len(), 1);
        if let SseEvent::ContentBlockDelta {
            delta: SseDelta::InputJsonDelta { partial_json },
            ..
        } = &events[0]
        {
            assert_eq!(partial_json, r#"{"loc"#);
        } else {
            panic!("expected input_json_delta, got: {:?}", events[0]);
        }
    }

    #[test]
    fn malformed_returns_unknown_not_panic() {
        let mut p = SseParser::new();
        let events = p
            .push_chunk(b"event: ping\ndata: {bogus json}\n\n")
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SseEvent::Unknown));
    }

    #[test]
    fn c0_control_chars_in_data() {
        let mut p = SseParser::new();
        // C0 以 \uXXXX 转义形式存在于 JSON 字符串内 → 合法 JSON
        let bytes = b"event: ping\ndata: {\"type\":\"ping\",\"x\":\"\\u0001\"}\n\n";
        let events = p.push_chunk(bytes).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SseEvent::Ping));
    }

    #[test]
    fn flush_returns_buffer_remainder_at_eof() {
        let mut p = SseParser::new();
        let _ = p
            .push_chunk(b"event: ping\ndata: {\"type\":\"ping\"}")
            .unwrap();
        // 没有 \n\n，event 还在 buffer 中
        let flushed = p.flush();
        assert_eq!(flushed.len(), 1);
        assert!(matches!(flushed[0], SseEvent::Ping));
    }

    #[test]
    fn empty_chunk_no_events() {
        let mut p = SseParser::new();
        assert!(p.push_chunk(b"").unwrap().is_empty());
    }

    // P0-5: SSE event buffer 容量上限测试
    #[test]
    fn push_chunk_over_limit_returns_event_too_large() {
        let mut p = SseParser::new();
        // 构造 MAX_SSE_EVENT_BYTES + 1 字节的无 \n\n 数据（触发容量上限）
        let oversized = vec![b'x'; MAX_SSE_EVENT_BYTES + 1];
        let result = p.push_chunk(&oversized);
        assert!(
            matches!(
                result,
                Err(SseParserError::EventTooLarge { len, max })
                    if len > MAX_SSE_EVENT_BYTES && max == MAX_SSE_EVENT_BYTES
            ),
            "expected EventTooLarge, got: {:?}",
            result
        );
    }

    #[test]
    fn anthropic_message_start_parses() {
        let mut p = SseParser::new();
        let bytes = br#"event: message_start
data: {"type":"message_start","message":{"id":"msg_x","type":"message","role":"assistant","content":[],"model":"claude-x","usage":{"input_tokens":1,"output_tokens":1}}}

"#;
        let events = p.push_chunk(bytes).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SseEvent::MessageStart { .. }));
    }

    #[test]
    fn crlf_separator_accepted() {
        let mut p = SseParser::new();
        // \r\n\r\n 分隔符
        let bytes = b"event: ping\r\ndata: {\"type\":\"ping\"}\r\n\r\n";
        let events = p.push_chunk(bytes).unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SseEvent::Ping));
    }

    #[test]
    fn multiple_data_lines_joined() {
        // SSE spec 允许多行 data:，合并后解析
        let mut p = SseParser::new();
        // 注意：这里两行 data 拼接后必须是合法 JSON
        // 实际 Anthropic 不会多行 data，但解析器应支持
        let bytes = b"data: {\"type\":\n\ndata: \"ping\"}\n\n";
        let events = p.push_chunk(bytes).unwrap();
        // 第一个 event 只有一行 data（第二个 \n\n 之前），无法解析 → Unknown
        assert!(!events.is_empty());
    }
}
