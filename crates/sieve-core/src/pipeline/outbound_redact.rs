//! 出站自动脱敏路径（AutoRedact disposition，OUT-01~05/12）。
//!
//! 提供两套 API：
//! - [`redact_body_bytes`]：在 raw body bytes 中按绝对字节偏移替换（fuzz/单测保留）。
//! - [`redact_segments`]：在解析后的文本段列表中按累计字符偏移替换，
//!   返回替换后的文本段列表，由调用方重新序列化 JSON——这是 daemon AutoRedact 路径
//!   的正确用法（修 #1：AutoRedact 偏移修复）。
//!
//! 关联：PRD v1.4 §6.1（出站 AutoRedact 路径）、ADR-016（二维处置矩阵）。

/// 单个脱敏命中范围（half-open `[start, end)`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactHit {
    /// 命中规则 ID（如 `OUT-01`）。
    pub rule_id: String,
    /// 命中起始字节偏移（含）。
    pub start: usize,
    /// 命中结束字节偏移（不含）。
    pub end: usize,
}

/// [`redact_body_bytes`] 的返回值。
#[derive(Debug)]
pub struct RedactResult {
    /// 脱敏后的 body bytes。
    pub body: Vec<u8>,
    /// 实际发生脱敏的数量（合并后的 span 数）。
    pub redacted_count: usize,
    /// 摘要字符串（如 `"OUT-01, OUT-02"`），用于审计日志。
    pub redacted_summary: String,
}

/// 在 `body` slice 中把 `pos` 向左移动到最近的 UTF-8 字符起始位置。
///
/// UTF-8 continuation byte 以 `10xxxxxx`（`0x80..=0xBF`）开头；
/// 如 body 含非 ASCII 字符（如中文 JSON 字段），正则可能给出 continuation byte 偏移，
/// 此函数保证不截断多字节字符。
pub fn align_to_utf8_char_start(body: &[u8], pos: usize) -> usize {
    if pos >= body.len() {
        return body.len();
    }
    let mut p = pos;
    while p > 0 && (body[p] & 0xC0) == 0x80 {
        p -= 1;
    }
    p
}

/// 把命中范围的字节替换为占位符，返回 [`RedactResult`]。
///
/// # 算法
/// 1. 每个 hit 的 `start`/`end` 先做 UTF-8 字符边界对齐（`align_to_utf8_char_start`）；
/// 2. 按 `start` 升序排序；
/// 3. 合并重叠 / 相邻 span（多个 span 合并时 `rule_id` 取最左命中）；
/// 4. 逐段复制原始字节，用 `[REDACTED:<rule_id>]` 替换各合并 span。
///
/// 如果 `hits` 为空，原样返回 body（`body.to_vec()`，最小拷贝）。
///
/// 关联：ADR-016 §AutoRedact 路径。
pub fn redact_body_bytes(body: &[u8], hits: &[RedactHit]) -> RedactResult {
    if hits.is_empty() {
        return RedactResult {
            body: body.to_vec(),
            redacted_count: 0,
            redacted_summary: String::new(),
        };
    }

    // 1. 对齐 UTF-8 边界
    let mut sorted: Vec<RedactHit> = hits
        .iter()
        .map(|h| RedactHit {
            rule_id: h.rule_id.clone(),
            start: align_to_utf8_char_start(body, h.start.min(body.len())),
            end: align_to_utf8_char_start(body, h.end.min(body.len())),
        })
        .collect();

    // 2. 按 start 升序排序
    sorted.sort_by_key(|h| h.start);

    // 3. 合并重叠 / 相邻 span
    let mut merged: Vec<(usize, usize, String)> = Vec::new();
    for hit in &sorted {
        let start = hit.start;
        let end = hit.end;
        if start >= end {
            // 对齐后 span 变空，跳过
            continue;
        }
        if let Some(last) = merged.last_mut() {
            if start <= last.1 {
                // 重叠或紧邻：扩展结束边界，rule_id 保持第一个
                if end > last.1 {
                    last.1 = end;
                }
            } else {
                merged.push((start, end, hit.rule_id.clone()));
            }
        } else {
            merged.push((start, end, hit.rule_id.clone()));
        }
    }

    let redacted_count = merged.len();
    let redacted_summary = merged
        .iter()
        .map(|(_, _, rule_id)| rule_id.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    // 4. 重组 body
    let mut result: Vec<u8> = Vec::with_capacity(body.len());
    let mut cursor = 0usize;

    for (start, end, rule_id) in &merged {
        if cursor < *start {
            result.extend_from_slice(&body[cursor..*start]);
        }
        let placeholder = format!("[REDACTED:{rule_id}]");
        result.extend_from_slice(placeholder.as_bytes());
        cursor = *end;
    }
    if cursor < body.len() {
        result.extend_from_slice(&body[cursor..]);
    }

    RedactResult {
        body: result,
        redacted_count,
        redacted_summary,
    }
}

/// 文本段级脱敏结果（对应 [`redact_segments`] 的输出）。
#[derive(Debug)]
pub struct SegmentRedactResult {
    /// 脱敏后的文本段列表，顺序与输入 `segments` 一一对应。
    pub texts: Vec<String>,
    /// 实际发生脱敏的总数量（合并后的 span 数，跨所有段）。
    pub redacted_count: usize,
    /// 摘要字符串（如 `"OUT-01, OUT-02"`），用于审计日志。
    pub redacted_summary: String,
}

/// 在解析后的文本段列表中按**累计字符偏移**做脱敏替换。
///
/// # 背景（修 #1：AutoRedact 偏移修复）
///
/// [`Detection.span`] 的 `start`/`end` 是 `extract_text_content()` 返回的
/// **累计文本字符偏移**（即 `body_byte_offset + vectorscan_offset`），
/// 而非 raw JSON body 的字节偏移。直接把这些偏移喂给 [`redact_body_bytes`]
/// 会写错 raw body 的字节范围，无法正确擦除 secret。
///
/// 正确做法：在每个文本段字符串内计算段内偏移后做字符串替换，
/// 然后由调用方把替换后的文本重新填入 JSON 并重新序列化。
///
/// # 参数
/// - `segments`：`(segment_global_start_offset, segment_text)` 列表，
///   顺序与 `AnthropicRequest::extract_text_content()` 返回值一致。
/// - `hits`：要脱敏的命中列表，`start`/`end` 是累计字符偏移（`Detection.span`）。
///
/// # 返回
/// [`SegmentRedactResult`]，其中 `texts` 顺序对应输入 `segments`。
///
/// 关联：PRD v1.4 §6.1（AutoRedact 路径）、ADR-016（二维处置矩阵）。
pub fn redact_segments(segments: &[(usize, String)], hits: &[RedactHit]) -> SegmentRedactResult {
    if hits.is_empty() {
        return SegmentRedactResult {
            texts: segments.iter().map(|(_, t)| t.clone()).collect(),
            redacted_count: 0,
            redacted_summary: String::new(),
        };
    }

    let mut total_redacted = 0usize;
    let mut all_rule_ids: Vec<String> = Vec::new();
    let mut result_texts: Vec<String> = Vec::with_capacity(segments.len());

    for (seg_idx, (seg_start, seg_text)) in segments.iter().enumerate() {
        let seg_end = seg_start + seg_text.len();

        // 过滤出与当前段有交集的 hit（累计偏移范围与段范围重叠）
        let seg_hits: Vec<RedactHit> = hits
            .iter()
            .filter(|h| h.start < seg_end && h.end > *seg_start)
            .map(|h| {
                // 把全局偏移转换为段内字符偏移（clamp 到段边界）
                let local_start = h.start.saturating_sub(*seg_start).min(seg_text.len());
                let local_end = h.end.saturating_sub(*seg_start).min(seg_text.len());
                RedactHit {
                    rule_id: h.rule_id.clone(),
                    start: local_start,
                    end: local_end,
                }
            })
            .collect();

        if seg_hits.is_empty() {
            result_texts.push(seg_text.clone());
            continue;
        }

        // 在 UTF-8 字符串上做 redact（按字节偏移，text 是 UTF-8 已验证）
        let text_bytes = seg_text.as_bytes();
        let redact_result = redact_body_bytes(text_bytes, &seg_hits);

        total_redacted += redact_result.redacted_count;
        if !redact_result.redacted_summary.is_empty() {
            all_rule_ids.push(redact_result.redacted_summary.clone());
        }

        // redact_body_bytes 保证输出有效 UTF-8（placeholder 是 ASCII，原始文本是 UTF-8）
        // Safety: redact_body_bytes 对齐 UTF-8 边界，placeholder 是纯 ASCII
        let new_text = String::from_utf8(redact_result.body).unwrap_or_else(|_| seg_text.clone()); // 极端回退：保留原文
        result_texts.push(new_text);

        // suppress unused variable lint for seg_idx
        let _ = seg_idx;
    }

    SegmentRedactResult {
        texts: result_texts,
        redacted_count: total_redacted,
        redacted_summary: all_rule_ids.join(", "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(rule_id: &str, start: usize, end: usize) -> RedactHit {
        RedactHit {
            rule_id: rule_id.to_string(),
            start,
            end,
        }
    }

    // ── 1. 单 span ───────────────────────────────────────────────────────────

    #[test]
    fn single_span_middle() {
        // "hello secret world"
        //  0     6     12   17
        let body = b"hello secret world";
        let hits = [hit("OUT-01", 6, 12)]; // "secret"
        let r = redact_body_bytes(body, &hits);
        assert_eq!(r.redacted_count, 1);
        assert_eq!(r.redacted_summary, "OUT-01");
        let s = String::from_utf8(r.body).unwrap();
        assert_eq!(s, "hello [REDACTED:OUT-01] world");
    }

    // ── 2. 多 span（不重叠）──────────────────────────────────────────────────

    #[test]
    fn multiple_non_overlapping_spans() {
        // "a secret b key c"
        //  0 2      8 10  13 15
        let body = b"a secret b key c";
        let hits = [hit("OUT-01", 2, 8), hit("OUT-03", 11, 14)];
        let r = redact_body_bytes(body, &hits);
        assert_eq!(r.redacted_count, 2);
        let s = String::from_utf8(r.body).unwrap();
        assert_eq!(s, "a [REDACTED:OUT-01] b [REDACTED:OUT-03] c");
    }

    // ── 3. 重叠 span 合并 ────────────────────────────────────────────────────

    #[test]
    fn overlapping_spans_merged() {
        let body = b"0123456789";
        // [1,6) 和 [4,9) 重叠 → 合并为 [1,9)，rule_id 取第一个 OUT-01
        let hits = [hit("OUT-01", 1, 6), hit("OUT-02", 4, 9)];
        let r = redact_body_bytes(body, &hits);
        assert_eq!(
            r.redacted_count, 1,
            "two overlapping spans must merge into one"
        );
        let s = String::from_utf8(r.body).unwrap();
        assert_eq!(s, "0[REDACTED:OUT-01]9");
    }

    // ── 4. UTF-8 边界对齐 ────────────────────────────────────────────────────

    #[test]
    fn utf8_boundary_alignment() {
        // "ab中cd"：bytes: [a, b, 中(3 bytes), c, d]
        // 偏移：a=0, b=1, 中[0]=2, 中[1]=3, 中[2]=4, c=5, d=6
        let body = "ab中cd".as_bytes();
        // byte 3 和 4 是 '中' 的 continuation byte，align 应向左到 2
        assert_eq!(align_to_utf8_char_start(body, 3), 2);
        assert_eq!(align_to_utf8_char_start(body, 4), 2);
        // byte 5 是 'c'，本身是起始，不需要移动
        assert_eq!(align_to_utf8_char_start(body, 5), 5);
        // 超出 body 长度时返回 body.len()
        assert_eq!(align_to_utf8_char_start(body, 100), body.len());
    }

    #[test]
    fn utf8_body_redact_aligned() {
        // body: "密钥:sk-xxx" — 确保 hit 落在 UTF-8 continuation byte 时不 panic
        let body = "密钥:sk-xxx".as_bytes();
        // '密' 占 3 字节，start=1 是 continuation byte → 对齐后变 start=0
        // end 也对齐；实际替换从字符边界开始
        let hits = [hit("OUT-01", 1, body.len())];
        let r = redact_body_bytes(body, &hits);
        // 不 panic，体内可正常解析
        assert_eq!(r.redacted_count, 1);
    }

    // ── 5. 空 hits ───────────────────────────────────────────────────────────

    #[test]
    fn empty_hits_returns_original() {
        let body = b"no secrets here";
        let r = redact_body_bytes(body, &[]);
        assert_eq!(r.redacted_count, 0);
        assert_eq!(r.body, body);
        assert!(r.redacted_summary.is_empty());
    }

    // ── 额外：span 超出 body 长度 clamp ──────────────────────────────────────

    #[test]
    fn span_clamped_to_body_len() {
        let body = b"hello";
        let hits = [hit("OUT-01", 3, 100)]; // end 超出 body 长度
        let r = redact_body_bytes(body, &hits);
        assert_eq!(r.redacted_count, 1);
        let s = String::from_utf8(r.body).unwrap();
        assert_eq!(s, "hel[REDACTED:OUT-01]");
    }

    // ── redact_segments 测试（修 #1 回归）─────────────────────────────────────

    fn seg(start: usize, text: &str) -> (usize, String) {
        (start, text.to_string())
    }

    /// 单条 OUT-01 命中：segment 内正确替换，原 secret 不存在于结果中。
    #[test]
    fn segments_single_hit_secret_removed() {
        // 段 0：offset=0, text="my sk-ant-api03-secret key"
        // hit 来自 vectorscan: start=3, end=21（相对 text），累计偏移=0+3=3
        let text = "my sk-ant-api03-secret key";
        let segments = vec![seg(0, text)];
        // 累计偏移 start=3, end=21
        let hits = [hit("OUT-01", 3, 21)];
        let r = redact_segments(&segments, &hits);
        assert_eq!(r.redacted_count, 1);
        assert_eq!(r.redacted_summary, "OUT-01");
        assert_eq!(r.texts.len(), 1);
        // 替换后不含原始 secret 片段
        assert!(!r.texts[0].contains("sk-ant-api03-secret"));
        assert!(r.texts[0].contains("[REDACTED:OUT-01]"));
    }

    /// 多条命中（不同段）：各自正确替换。
    #[test]
    fn segments_multiple_hits_different_segments() {
        // 段 0：offset=0, text="secret1 here"（命中 [0,7)）
        // 段 1：offset=12, text="clean text secret2"（命中 [12+10,12+17) = [22,29)）
        let segments = vec![seg(0, "secret1 here"), seg(12, "clean text secret2")];
        let hits = [hit("OUT-01", 0, 7), hit("OUT-02", 22, 29)];
        let r = redact_segments(&segments, &hits);
        assert_eq!(r.redacted_count, 2);
        assert!(!r.texts[0].contains("secret1"));
        assert!(!r.texts[1].contains("secret2"));
        assert!(r.texts[0].contains("[REDACTED:OUT-01]"));
        assert!(r.texts[1].contains("[REDACTED:OUT-02]"));
    }

    /// 中文+emoji UTF-8 命中：UTF-8 边界对齐，不破坏 JSON 结构。
    #[test]
    fn segments_utf8_chinese_emoji_hit() {
        // text: "你好😀sk-ant-secret"
        // "你好😀" = 3+3+4=10 bytes，"sk-ant-secret" 从 byte 10 开始
        let text = "你好😀sk-ant-secret";
        let text_byte_len = text.len();
        let segments = vec![seg(0, text)];
        // 命中整个 sk-ant-secret 部分（byte 10..text_len）
        let hits = [hit("OUT-01", 10, text_byte_len)];
        let r = redact_segments(&segments, &hits);
        assert_eq!(r.redacted_count, 1);
        // 替换后 text 是合法 UTF-8
        assert!(std::str::from_utf8(r.texts[0].as_bytes()).is_ok());
        assert!(!r.texts[0].contains("sk-ant-secret"));
        assert!(r.texts[0].contains("[REDACTED:OUT-01]"));
    }

    /// hit 不与任何段重叠时：原样保留所有段。
    #[test]
    fn segments_hit_outside_all_segments_no_change() {
        let segments = vec![seg(0, "hello world"), seg(20, "foo bar")];
        // hit 在 [50, 60)，不与任何段重叠
        let hits = [hit("OUT-01", 50, 60)];
        let r = redact_segments(&segments, &hits);
        assert_eq!(r.redacted_count, 0);
        assert_eq!(r.texts[0], "hello world");
        assert_eq!(r.texts[1], "foo bar");
    }
}
