//! 把 `sieve_rules::VectorscanEngine` 适配到 `sieve_core::OutboundEngine` /
//! `sieve_core::InboundEngine` trait。
//!
//! 阶段 1 sieve-core 不依赖 sieve-rules，所以 trait 定义在 sieve-core，
//! 由本 crate 在启动时桥接两边（`.cursorrules §3.3` crate 边界协调）。
//!
//! 关联 ADR-002 / PRD §5.1 / Week 2 出站 / Week 3 入站拦截集成。

use sieve_core::detection::{fingerprint, Action, ContentSource, Detection, Severity};
use sieve_core::error::SieveCoreResult;
use sieve_core::pipeline::inbound::InboundEngine;
use sieve_core::pipeline::outbound::OutboundEngine;
use sieve_core::protocol::unified_message::ContentSpan;
use sieve_core::tool_use_aggregator::CompletedToolCall;
use sieve_rules::engine::{MatchEngine, VectorscanEngine};
use sieve_rules::manifest::{Action as RulesAction, RuleEntry, Severity as RulesSeverity};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// `VectorscanEngine` 包装，实现 `sieve_core::OutboundEngine`。
///
/// 内部持有规则反查表（`rule_id → RuleEntry`），用于从 `MatchHit` 取真实 severity/action。
pub struct OutboundAdapter {
    engine: Arc<VectorscanEngine>,
    /// rule_id → RuleEntry 反查表，用于从 MatchHit 映射元数据。
    rule_lookup: HashMap<String, RuleEntry>,
}

impl OutboundAdapter {
    /// 构造 adapter。
    ///
    /// `rules` 与 `VectorscanEngine::compile` 传入的规则集一致，用于构建反查表。
    pub fn new(engine: Arc<VectorscanEngine>, rules: Vec<RuleEntry>) -> Self {
        let rule_lookup = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
        Self {
            engine,
            rule_lookup,
        }
    }
}

/// 把 `sieve_rules::Severity` 映射为 `sieve_core::Severity`。
fn map_severity(r: RulesSeverity) -> Severity {
    match r {
        RulesSeverity::Low => Severity::Low,
        RulesSeverity::Medium => Severity::Medium,
        RulesSeverity::High => Severity::High,
        RulesSeverity::Critical => Severity::Critical,
    }
}

/// 根据 `RuleEntry.disposition` 和 `RulesAction` 映射为 `sieve_core::Action`。
///
/// v1.4 重构：优先按 `effective_disposition()` 路由，`RulesAction` 作为兜底。
///
/// | Disposition       | Action                                       |
/// |-------------------|----------------------------------------------|
/// | AutoRedact        | `Redact { placeholder }`                     |
/// | GuiPopup          | `HoldForDecision { request_id, timeout_s }`  |
/// | HookTerminal      | `HookMark`                                   |
/// | StatusBar         | `MarkOnly`                                   |
///
/// `timeout_seconds` / `default_on_timeout` 取自 `RuleEntry`，不再硬编码 5。
///
/// 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
fn map_action_by_disposition(
    disposition: sieve_rules::manifest::Disposition,
    _rule_action: RulesAction,
    rule_id: &str,
    timeout_seconds: u32,
) -> Action {
    use sieve_rules::manifest::Disposition;
    match disposition {
        Disposition::AutoRedact => Action::Redact {
            placeholder: format!("[REDACTED:{rule_id}]"),
        },
        Disposition::GuiPopup => Action::HoldForDecision {
            request_id: uuid::Uuid::new_v4(),
            timeout_seconds,
        },
        Disposition::HookTerminal => Action::HookMark,
        Disposition::StatusBar => Action::MarkOnly,
    }
}

/// 旧接口：仅用 `RulesAction` 映射（兜底，无 disposition 信息时使用）。
///
/// `Warn` → `HookMark`（v1.4 后 Warn 一律走 HookTerminal 路径）。
///
/// 注：修 #2 后生产路径不再调用此函数（disposition 优先），
/// 保留用于单元测试验证 Warn → HookMark 的语义不变。
#[allow(dead_code)]
fn map_action(r: RulesAction) -> Action {
    match r {
        RulesAction::Block => Action::Block,
        RulesAction::Warn => Action::HookMark,
        RulesAction::Mark => Action::MarkOnly,
        RulesAction::Allow => Action::SilentLog,
    }
}

/// 截断并脱敏证据片段（用于 `Detection.evidence_truncated`）。
///
/// 超过 8 字符时，保留前 4 + `***` + 后 4，防止原始密钥写入审计日志。
fn redact_evidence(matched: &str) -> String {
    let chars: Vec<char> = matched.chars().collect();
    let len = chars.len();
    if len <= 8 {
        "*".repeat(len)
    } else {
        let head: String = chars[..4].iter().collect();
        let tail: String = chars[len - 4..].iter().collect();
        format!("{head}***{tail}")
    }
}

/// `VectorscanEngine` 包装，实现 `sieve_core::InboundEngine`。
///
/// 与 [`OutboundAdapter`] 共用辅助函数（`map_severity` / `map_action` / `redact_evidence`），
/// 额外在工具调用检查中调用 `sieve_rules::critical_lock::enforce_action` 保证 fail-closed。
pub struct InboundAdapter {
    engine: Arc<VectorscanEngine>,
    /// rule_id → RuleEntry 反查表。
    rule_lookup: HashMap<String, RuleEntry>,
}

impl InboundAdapter {
    /// 构造 adapter。
    pub fn new(engine: Arc<VectorscanEngine>, rules: Vec<RuleEntry>) -> Self {
        let rule_lookup = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
        Self {
            engine,
            rule_lookup,
        }
    }
}

impl InboundEngine for InboundAdapter {
    fn scan_text(
        &self,
        input: &str,
        source: ContentSource,
        body_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>> {
        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
        })?;

        let mut detections = Vec::new();
        for hit in hits {
            let rule = self.rule_lookup.get(&hit.rule_id);

            let evidence_start = hit.start.min(input.len());
            let evidence_end = hit.end.min(input.len());
            let matched_text = &input[evidence_start..evidence_end];

            if let Some(r) = rule {
                if self.engine.is_excluded(matched_text, r) {
                    continue;
                }
            }

            let severity = rule
                .map(|r| map_severity(r.severity))
                .unwrap_or(Severity::Critical);

            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复，入站侧）。
            //
            // 规则显式写了 disposition 时直接路由；
            // disposition=None 且 fail-closed 时才强制 Block。
            // 这确保 IN-CR-02（hook_terminal）/ IN-CR-05（gui_popup）即使在 fail-closed
            // 名单里也能走正确的 HookMark / HoldForDecision 路径（不被截成 Block）。
            //
            // 关联：ADR-016（二维处置矩阵）、ADR-014（双层防御）、PRD v1.4 §5.4。
            let action = if let Some(r) = rule {
                if let Some(disp) = r.disposition {
                    // 显式 disposition：直接路由，不经过 enforce_action
                    let timeout = r.timeout_seconds.unwrap_or(60);
                    map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
                } else {
                    // 无显式 disposition：走旧路径（enforce_action → Block or action）
                    let enforced =
                        sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
                    if enforced == RulesAction::Block {
                        Action::Block
                    } else {
                        let disp = r.effective_disposition();
                        let timeout = r.timeout_seconds.unwrap_or(60);
                        map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
                    }
                }
            } else {
                // 规则表中找不到：fail-closed Block
                Action::Block
            };

            let evidence_truncated = redact_evidence(matched_text);
            let fp = fingerprint(&hit.rule_id, matched_text);

            detections.push(Detection {
                id: Uuid::new_v4(),
                rule_id: hit.rule_id.clone(),
                severity,
                action,
                source,
                span: ContentSpan {
                    start: body_offset + hit.start,
                    end: body_offset + hit.end,
                },
                evidence_truncated,
                fingerprint: fp,
            });
        }
        Ok(detections)
    }

    fn check_tool_use(
        &self,
        tool: &CompletedToolCall,
        source: ContentSource,
    ) -> SieveCoreResult<Vec<Detection>> {
        let mut hits = Vec::new();
        // 1. 工具名扫描（IN-CR-05 签名工具）
        hits.extend(self.scan_text(&tool.name, source, 0)?);
        // 2. 工具输入序列化扫描（IN-CR-02 危险 shell 等）
        if let Ok(input_str) = serde_json::to_string(&tool.input) {
            hits.extend(self.scan_text(&input_str, source, 0)?);
        }
        Ok(hits)
    }
}

impl OutboundEngine for OutboundAdapter {
    /// 扫描文本，返回已过滤（per-rule allowlist）的命中列表，并执行 BIP39 second-pass。
    ///
    /// - `body_byte_offset`：该文本段在原始请求 body 中的绝对起始偏移，
    ///   用于生成 `Detection.span`（精确字节区间，half-open [start, end)）。
    ///
    /// BIP39 second-pass（PRD §9 #4）：vectorscan 之后独立扫描。
    /// 先提取全部在词表的连续词窗口，再做 SHA-256 checksum 验证，
    /// **仅 checksum 通过才生成 Critical Detection**。
    /// 词表命中但 checksum 失败的窗口**不得**定级 Critical（差异化要求）。
    fn scan_text(
        &self,
        input: &str,
        source: ContentSource,
        body_byte_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>> {
        let hits = self.engine.scan(input.as_bytes()).map_err(|e| {
            sieve_core::error::SieveCoreError::Forwarder(format!("vectorscan scan: {e}"))
        })?;

        let mut detections = Vec::new();
        for hit in hits {
            let rule = self.rule_lookup.get(&hit.rule_id);

            // per-rule allowlist 过滤
            let evidence_start = hit.start.min(input.len());
            let evidence_end = hit.end.min(input.len());
            let matched_text = &input[evidence_start..evidence_end];

            if let Some(r) = rule {
                if self.engine.is_excluded(matched_text, r) {
                    continue;
                }
            }

            let severity = rule
                .map(|r| map_severity(r.severity))
                .unwrap_or(Severity::Critical);
            // v1.4：disposition 优先于 enforce_action（修 #2：路由短路修复）。
            //
            // 规则显式写了 disposition 时，**直接按 disposition 路由**——
            // 这确保 OUT-01（auto_redact）即使在 fail-closed 名单里也走 Redact 而非 Block。
            // 只有 disposition=None（旧规则 / 无显式配置）且 fail-closed 时，才走 Block。
            //
            // 关联：ADR-016（二维处置矩阵）、PRD v1.4 §5.4。
            let action = rule
                .map(|r| {
                    if let Some(disp) = r.disposition {
                        // 显式 disposition：直接路由，不经过 enforce_action
                        let timeout = r.timeout_seconds.unwrap_or(60);
                        map_action_by_disposition(disp, r.action, &hit.rule_id, timeout)
                    } else {
                        // 无显式 disposition：走旧路径（enforce_action → Block or action）
                        let enforced =
                            sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
                        if enforced == RulesAction::Block {
                            Action::Block
                        } else {
                            let disp = r.effective_disposition();
                            let timeout = r.timeout_seconds.unwrap_or(60);
                            map_action_by_disposition(disp, enforced, &hit.rule_id, timeout)
                        }
                    }
                })
                .unwrap_or(Action::Block);
            let evidence_truncated = redact_evidence(matched_text);
            let fp = fingerprint(&hit.rule_id, matched_text);

            detections.push(Detection {
                id: Uuid::new_v4(),
                rule_id: hit.rule_id.clone(),
                severity,
                action,
                source,
                span: ContentSpan {
                    start: body_byte_offset + hit.start,
                    end: body_byte_offset + hit.end,
                },
                evidence_truncated,
                fingerprint: fp,
            });
        }

        // BIP39 second-pass（关联 PRD §9 #4 差异化点）
        // vectorscan 不覆盖 BIP39，此处独立扫描：
        // 1. 按空白分词，提取全在词表的连续窗口
        // 2. 对每个窗口做 SHA-256 checksum 验证
        // 3. 仅 checksum 通过的窗口定级 Critical（OUT-09）
        let wl = sieve_rules::wordlist::wordlist_index();
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let candidates = sieve_rules::bip39::candidate_bip39_windows(&tokens, wl);
        for window in candidates {
            if sieve_rules::bip39::verify_checksum(&window, wl) {
                let window_text = window.join(" ");
                let evidence_truncated = redact_evidence(&window_text);
                let fp = fingerprint("OUT-09", &window_text);
                detections.push(Detection {
                    id: Uuid::new_v4(),
                    rule_id: "OUT-09".to_string(),
                    severity: Severity::Critical,
                    action: Action::Block,
                    source,
                    // span 为整个输入范围的近似（无精确字节偏移）
                    span: ContentSpan {
                        start: body_byte_offset,
                        end: body_byte_offset + input.len(),
                    },
                    evidence_truncated,
                    fingerprint: fp,
                });
                // 同一文本只需报一次（找到一个有效助记词即触发拦截）
                break;
            }
        }

        Ok(detections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sieve_rules::engine::VectorscanEngine;
    use sieve_rules::manifest::{Action as RulesAction, RuleEntry, Severity as RulesSeverity};

    fn make_rule(
        id: &str,
        pattern: &str,
        severity: RulesSeverity,
        action: RulesAction,
    ) -> RuleEntry {
        RuleEntry {
            id: id.into(),
            description: id.into(),
            pattern: pattern.into(),
            severity,
            action,
            entropy_min: None,
            keywords: vec![],
            allowlist_regexes: vec![],
            allowlist_stopwords: vec![],
            disposition: None,
            timeout_seconds: None,
            default_on_timeout: sieve_rules::manifest::DefaultOnTimeout::Block,
        }
    }

    #[test]
    fn scan_detects_pattern() {
        let rules = vec![make_rule(
            "OUT-TEST",
            r"secret",
            RulesSeverity::Critical,
            RulesAction::Block,
        )];
        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);
        let hits = adapter
            .scan_text("my secret key", ContentSource::OutboundUserText, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "OUT-TEST");
        assert_eq!(hits[0].severity, Severity::Critical);
        assert!(matches!(hits[0].action, Action::Block));
    }

    #[test]
    fn scan_no_match_returns_empty() {
        let rules = vec![make_rule(
            "OUT-TEST",
            r"secret",
            RulesSeverity::High,
            RulesAction::Warn,
        )];
        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);
        let hits = adapter
            .scan_text(
                "nothing suspicious here",
                ContentSource::OutboundUserText,
                0,
            )
            .unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn map_action_warn_becomes_hook_mark() {
        // v1.4：Warn 一律走 HookTerminal 路径（HookMark action）
        let a = map_action(RulesAction::Warn);
        assert!(matches!(a, Action::HookMark));
    }

    #[test]
    fn redact_evidence_short() {
        let r = redact_evidence("abc");
        assert_eq!(r, "***");
    }

    #[test]
    fn redact_evidence_long() {
        let r = redact_evidence("1234567890abcdef");
        assert!(r.starts_with("1234"));
        assert!(r.ends_with("cdef"));
        assert!(r.contains("***"));
    }

    #[test]
    fn span_offset_applied() {
        let rules = vec![make_rule(
            "OUT-OFF",
            r"hello",
            RulesSeverity::Low,
            RulesAction::Mark,
        )];
        let engine = VectorscanEngine::compile(rules.clone()).unwrap();
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);
        // offset=100, text starts at byte 0 within "say hello", pattern at 4..9
        let hits = adapter
            .scan_text("say hello", ContentSource::OutboundSystemText, 100)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].span.start, 104); // 100 + 4
        assert_eq!(hits[0].span.end, 109); // 100 + 9
    }

    // ── 修 #2 回归：disposition 优先于 enforce_action ──────────────────────────

    /// disposition=auto_redact 即使 action=block（fail-closed 名单）也走 Redact 路径。
    ///
    /// 修 #2（路由短路修复）：OUT-01 等 AutoRedact 规则在 fail-closed 名单里，
    /// 旧代码 enforce_action 会把 action 强制变 Block，跳过 disposition 路由。
    /// 修复后：显式 disposition 优先，OUT-01 必须走 Action::Redact 而非 Action::Block。
    #[test]
    fn disposition_auto_redact_beats_enforce_action() {
        let mut rule = make_rule(
            "OUT-01", // 在 fail-closed 名单里
            r"sk-ant",
            RulesSeverity::Critical,
            RulesAction::Block,
        );
        rule.disposition = Some(sieve_rules::manifest::Disposition::AutoRedact);

        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
        let adapter = OutboundAdapter::new(Arc::new(engine), vec![rule]);

        let hits = adapter
            .scan_text("my sk-ant-key here", ContentSource::OutboundUserText, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "OUT-01");
        // 关键断言：应该是 Redact，不是 Block
        assert!(
            matches!(hits[0].action, Action::Redact { .. }),
            "disposition=auto_redact 应走 Redact 路径，实际: {:?}",
            hits[0].action
        );
    }

    /// disposition=hook_terminal 即使在 fail-closed 名单里也走 HookMark 路径。
    ///
    /// 修 #2 回归：IN-CR-02 等 HookTerminal 规则不应被 enforce_action 截成 Block。
    #[test]
    fn disposition_hook_terminal_beats_enforce_action() {
        let mut rule = make_rule(
            "IN-CR-02", // 在 fail-closed 名单里
            r"rm -rf",
            RulesSeverity::Critical,
            RulesAction::Block,
        );
        rule.disposition = Some(sieve_rules::manifest::Disposition::HookTerminal);

        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
        let adapter = InboundAdapter::new(Arc::new(engine), vec![rule]);

        let hits = adapter
            .scan_text("run: rm -rf /tmp", ContentSource::InboundAssistantText, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "IN-CR-02");
        // 关键断言：应该是 HookMark，不是 Block
        assert!(
            matches!(hits[0].action, Action::HookMark),
            "disposition=hook_terminal 应走 HookMark 路径，实际: {:?}",
            hits[0].action
        );
    }

    /// disposition=gui_popup 即使在 fail-closed 名单里也走 HoldForDecision 路径。
    #[test]
    fn disposition_gui_popup_beats_enforce_action() {
        let mut rule = make_rule(
            "IN-CR-05-EVM", // 在 fail-closed 名单里
            r"eth_signTypedData",
            RulesSeverity::Critical,
            RulesAction::Block,
        );
        rule.disposition = Some(sieve_rules::manifest::Disposition::GuiPopup);
        rule.timeout_seconds = Some(60);

        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
        let adapter = InboundAdapter::new(Arc::new(engine), vec![rule]);

        let hits = adapter
            .scan_text(
                "call eth_signTypedData method",
                ContentSource::InboundAssistantText,
                0,
            )
            .unwrap();
        assert_eq!(hits.len(), 1);
        // 关键断言：应该是 HoldForDecision，不是 Block
        assert!(
            matches!(hits[0].action, Action::HoldForDecision { .. }),
            "disposition=gui_popup 应走 HoldForDecision 路径，实际: {:?}",
            hits[0].action
        );
    }
}
