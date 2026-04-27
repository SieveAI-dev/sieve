//! 把 `sieve_rules::VectorscanEngine` 适配到 `sieve_core::OutboundEngine` trait。
//!
//! 阶段 1 sieve-core 不依赖 sieve-rules，所以 `OutboundEngine` trait 定义在 sieve-core，
//! 由本 crate 在启动时桥接两边（`.cursorrules §3.3` crate 边界协调）。
//!
//! 关联 ADR-002 / PRD §5.1 / Week 2 出站拦截集成。

use sieve_core::detection::{fingerprint, Action, ContentSource, Detection, Severity};
use sieve_core::error::SieveCoreResult;
use sieve_core::pipeline::outbound::OutboundEngine;
use sieve_core::protocol::unified_message::ContentSpan;
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

/// 把 `sieve_rules::Action` 映射为 `sieve_core::Action`。
///
/// `Warn` → `WarnConfirm { countdown_secs: 5 }`：Week 4 实际接入弹窗，此处占位。
fn map_action(r: RulesAction) -> Action {
    match r {
        RulesAction::Block => Action::Block,
        RulesAction::Warn => Action::WarnConfirm { countdown_secs: 5 },
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

impl OutboundEngine for OutboundAdapter {
    /// 扫描文本，返回已过滤（placeholder + per-rule allowlist）的命中列表。
    ///
    /// - `body_byte_offset`：该文本段在原始请求 body 中的绝对起始偏移，
    ///   用于生成 `Detection.span`（精确字节区间，half-open [start, end)）。
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

            // placeholder / per-rule allowlist 过滤
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
            let action = rule.map(|r| map_action(r.action)).unwrap_or(Action::Block);
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
    fn map_action_warn_becomes_warn_confirm() {
        let a = map_action(RulesAction::Warn);
        assert!(matches!(a, Action::WarnConfirm { countdown_secs: 5 }));
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
}
