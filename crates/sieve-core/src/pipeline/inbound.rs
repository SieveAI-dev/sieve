//! 入站规则匹配节点（Week 3 起实现）。
//!
//! 关联 PRD §5.2 入站检测 P0 表 + UCSB 论文 4 类攻击分类。

use crate::address_guard::{check_substitution, extract_eth_addresses};
use crate::detection::{fingerprint, Action, ContentSource, Detection, Severity};
use crate::error::{SieveCoreError, SieveCoreResult};
use crate::pipeline::streaming::StreamingPipelineNode;
use crate::protocol::unified_message::ContentSpan;
use crate::sse::parser::{SseDelta, SseEvent};
use crate::tool_use_aggregator::CompletedToolCall;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// 入站引擎抽象接口（由 sieve-cli 把 sieve_rules::VectorscanEngine 适配进来）。
///
/// crate 边界：sieve-core 不直接依赖 sieve-rules，通过本 trait 解耦（.cursorrules §3.3）。
pub trait InboundEngine: Send + Sync {
    /// 扫描文本，返回命中的 Detection 列表。
    ///
    /// # Errors
    /// 扫描失败时返回 [`crate::error::SieveCoreError`]。
    fn scan_text(
        &self,
        input: &str,
        source: ContentSource,
        body_offset: usize,
    ) -> SieveCoreResult<Vec<Detection>>;

    /// 检查工具调用，返回命中的 Detection 列表。
    ///
    /// # Errors
    /// 检查失败时返回 [`crate::error::SieveCoreError`]。
    fn check_tool_use(
        &self,
        tool: &CompletedToolCall,
        source: ContentSource,
    ) -> SieveCoreResult<Vec<Detection>>;
}

/// 会话级状态（跨 SSE event 保持）。
#[derive(Default)]
pub struct SessionState {
    /// 当前会话中已见过的 ETH 地址集合（用于 IN-CR-01 地址替换检测）。
    pub addresses_seen: HashSet<String>,
}

/// 入站流式过滤节点，实现 [`StreamingPipelineNode`] trait。
pub struct InboundFilter {
    engine: Arc<dyn InboundEngine>,
    session: Mutex<SessionState>,
    /// `.sieveignore` 加载的 fingerprint 集合（O(1) 查询）。
    sieveignore: Arc<HashSet<String>>,
}

impl InboundFilter {
    /// 新建 InboundFilter。
    pub fn new(engine: Arc<dyn InboundEngine>, sieveignore: Arc<HashSet<String>>) -> Self {
        Self {
            engine,
            session: Mutex::new(SessionState::default()),
            sieveignore,
        }
    }

    /// 把出站 prompt 文本中的 EVM 地址 seed 到会话地址集合。
    ///
    /// 须在入站 SSE 检测（[`StreamingPipelineNode::observe_event`]）开始前调用，
    /// 否则首轮地址替换（prompt 地址 A → 响应地址 B）会漏报 IN-CR-01。
    ///
    /// 关联 PRD §4.2 真实攻击场景 / P0-3 修复。
    ///
    /// # Errors
    /// session mutex 中毒时返回 [`SieveCoreError`]。
    pub fn seed_known_addresses_from_text(&self, text: &str) -> SieveCoreResult<()> {
        let mut session = self
            .session
            .lock()
            .map_err(|_| SieveCoreError::Forwarder("session mutex poisoned".into()))?;
        for addr in extract_eth_addresses(text) {
            session.addresses_seen.insert(addr);
        }
        Ok(())
    }

    /// 过滤掉 sieveignore 中已知的 fingerprint。
    ///
    /// PRD §9 #3 #8：Critical severity 永远不被过滤——
    /// `.sieveignore` 白名单仅对 High / Medium / Low 有效。
    fn filter_sieveignore(&self, dets: Vec<Detection>) -> Vec<Detection> {
        dets.into_iter()
            .filter(|d| {
                d.severity == Severity::Critical || !self.sieveignore.contains(&d.fingerprint)
            })
            .collect()
    }
}

impl StreamingPipelineNode for InboundFilter {
    fn name(&self) -> &str {
        "inbound-filter"
    }

    fn observe_event(&mut self, event: &SseEvent) -> SieveCoreResult<Vec<Detection>> {
        let mut hits = Vec::new();

        if let SseEvent::ContentBlockDelta {
            delta: SseDelta::TextDelta { text },
            ..
        } = event
        {
            // 1. 文本扫描（IN-GEN-* 通用规则 + 危险命令检测）
            hits.extend(
                self.engine
                    .scan_text(text, ContentSource::InboundAssistantText, 0)?,
            );

            // 2. IN-CR-01 地址替换检测
            let addrs = extract_eth_addresses(text);
            let mut session = self
                .session
                .lock()
                .map_err(|_| SieveCoreError::Forwarder("session mutex poisoned".into()))?;

            for addr in addrs {
                if let Some(orig) = check_substitution(&session.addresses_seen, &addr) {
                    let fp = fingerprint("IN-CR-01", &format!("{orig}->{addr}"));
                    hits.push(Detection {
                        id: Uuid::new_v4(),
                        rule_id: "IN-CR-01".into(),
                        severity: Severity::Critical,
                        action: Action::Block,
                        source: ContentSource::InboundAssistantText,
                        span: ContentSpan {
                            start: 0,
                            end: addr.len(),
                        },
                        evidence_truncated: format!("{orig}->{addr}"),
                        fingerprint: fp,
                    });
                }
                session.addresses_seen.insert(addr);
            }
        }

        Ok(self.filter_sieveignore(hits))
    }

    fn on_tool_use_complete(
        &mut self,
        tool: &CompletedToolCall,
    ) -> SieveCoreResult<Vec<Detection>> {
        let hits = self
            .engine
            .check_tool_use(tool, ContentSource::InboundToolUseInput)?;
        Ok(self.filter_sieveignore(hits))
    }

    fn on_message_stop(&mut self) -> SieveCoreResult<Vec<Detection>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::{fingerprint, Action, ContentSource, Detection, Severity};
    use crate::protocol::unified_message::ContentSpan;
    use uuid::Uuid;

    /// Mock InboundEngine：
    /// - 文本含 "rm -rf" → 返回 IN-CR-02 命中
    /// - 工具名含 "signTransaction" → 返回 IN-CR-05 命中
    struct MockEngine;

    impl InboundEngine for MockEngine {
        fn scan_text(
            &self,
            input: &str,
            source: ContentSource,
            _body_offset: usize,
        ) -> SieveCoreResult<Vec<Detection>> {
            if input.contains("rm -rf") {
                Ok(vec![Detection {
                    id: Uuid::new_v4(),
                    rule_id: "IN-CR-02".into(),
                    severity: Severity::Critical,
                    action: Action::Block,
                    source,
                    span: ContentSpan { start: 0, end: 5 },
                    evidence_truncated: "**".into(),
                    fingerprint: fingerprint("IN-CR-02", "rm -rf"),
                }])
            } else if input.contains("suspicious_high") {
                // High severity detection，用于验证 sieveignore 可以合法压制非 Critical
                Ok(vec![Detection {
                    id: Uuid::new_v4(),
                    rule_id: "IN-GEN-01".into(),
                    severity: Severity::High,
                    action: Action::HookMark,
                    source,
                    span: ContentSpan { start: 0, end: 15 },
                    evidence_truncated: "suspicious_high".into(),
                    fingerprint: fingerprint("IN-GEN-01", "suspicious_high"),
                }])
            } else {
                Ok(vec![])
            }
        }

        fn check_tool_use(
            &self,
            tool: &CompletedToolCall,
            source: ContentSource,
        ) -> SieveCoreResult<Vec<Detection>> {
            if tool.name.contains("signTransaction") {
                Ok(vec![Detection {
                    id: Uuid::new_v4(),
                    rule_id: "IN-CR-05".into(),
                    severity: Severity::Critical,
                    action: Action::Block,
                    source,
                    span: ContentSpan {
                        start: 0,
                        end: tool.name.len(),
                    },
                    evidence_truncated: tool.name.clone(),
                    fingerprint: fingerprint("IN-CR-05", &tool.name),
                }])
            } else {
                Ok(vec![])
            }
        }
    }

    #[test]
    fn dangerous_shell_in_text_detected() {
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        let evt = SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::TextDelta {
                text: "run rm -rf /".into(),
            },
        };
        let hits = f.observe_event(&evt).unwrap();
        assert!(!hits.is_empty());
        assert_eq!(hits[0].rule_id, "IN-CR-02");
    }

    #[test]
    fn signing_tool_call_detected() {
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        let tool = CompletedToolCall {
            id: "x".into(),
            name: "eth_signTransaction".into(),
            input: serde_json::json!({}),
        };
        let hits = f.on_tool_use_complete(&tool).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "IN-CR-05");
    }

    #[test]
    fn address_substitution_detected_across_events() {
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        // 第一个 event：植入原始地址
        let _ = f
            .observe_event(&SseEvent::ContentBlockDelta {
                index: 0,
                delta: SseDelta::TextDelta {
                    text: "send 0xabcdef1234567890abcdef1234567890abcdef12 here".into(),
                },
            })
            .unwrap();
        // 第二个 event：出现近似（末位 2→3）地址
        let hits = f
            .observe_event(&SseEvent::ContentBlockDelta {
                index: 0,
                delta: SseDelta::TextDelta {
                    text: "actually 0xabcdef1234567890abcdef1234567890abcdef13 here".into(),
                },
            })
            .unwrap();
        assert!(hits.iter().any(|d| d.rule_id == "IN-CR-01"));
    }

    /// sieveignore 可以合法压制 High / Medium 等非 Critical detection。
    /// Critical 不在此测试验证范围——见 sieveignore_does_not_suppress_critical。
    #[test]
    fn sieveignore_filters_non_critical_fingerprint() {
        let fp = fingerprint("IN-GEN-01", "suspicious_high");
        let mut ignore = HashSet::new();
        ignore.insert(fp);
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(ignore));
        let evt = SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::TextDelta {
                text: "suspicious_high pattern here".into(),
            },
        };
        let hits = f.observe_event(&evt).unwrap();
        assert!(
            hits.is_empty(),
            "sieveignore should suppress High/non-Critical detection"
        );
    }

    #[test]
    fn non_text_delta_event_returns_no_hits() {
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        // MessageStop 不产生命中
        let hits = f.observe_event(&SseEvent::MessageStop).unwrap();
        assert!(hits.is_empty());
    }

    /// seed_known_addresses_from_text 预注入 prompt 地址，首轮地址替换可被 IN-CR-01 检测。
    ///
    /// 关联 P0-3 / PRD §4.2：prompt 地址 A + SSE 仅出现地址 B → 命中。
    #[test]
    fn seed_from_prompt_enables_first_round_address_substitution_detection() {
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(HashSet::new()));
        // 模拟 outbound prompt seed：提前把地址 A 注入 session
        f.seed_known_addresses_from_text(
            "please send to 0xabcdef1234567890abcdef1234567890abcdef12 from wallet",
        )
        .unwrap();
        // SSE 响应只出现近似地址 B（末字符 2→3），未在 SSE 中出现原始地址 A
        let hits = f
            .observe_event(&SseEvent::ContentBlockDelta {
                index: 0,
                delta: SseDelta::TextDelta {
                    text: "send to 0xabcdef1234567890abcdef1234567890abcdef13 now".into(),
                },
            })
            .unwrap();
        assert!(
            hits.iter().any(|d| d.rule_id == "IN-CR-01"),
            "should detect IN-CR-01 when address was seeded from prompt"
        );
    }

    /// PRD §9 #3 #8：Critical detection 不得被 .sieveignore 压制。
    /// 验证 IN-CR-02（危险 shell）和 IN-CR-05（签名工具调用）在加入 sieveignore 后仍然命中。
    #[test]
    fn sieveignore_does_not_suppress_critical() {
        // 构造同时包含 IN-CR-02 和 IN-CR-05 fingerprint 的 sieveignore
        let fp_cr02 = fingerprint("IN-CR-02", "rm -rf");
        let fp_cr05 = fingerprint("IN-CR-05", "eth_signTransaction");
        let mut ignore = HashSet::new();
        ignore.insert(fp_cr02);
        ignore.insert(fp_cr05);

        // 验证文本扫描 Critical（IN-CR-02）不被压制
        let mut f = InboundFilter::new(Arc::new(MockEngine), Arc::new(ignore.clone()));
        let evt = SseEvent::ContentBlockDelta {
            index: 0,
            delta: SseDelta::TextDelta {
                text: "run rm -rf /".into(),
            },
        };
        let hits = f.observe_event(&evt).unwrap();
        assert!(
            !hits.is_empty(),
            "Critical IN-CR-02 must not be suppressed by sieveignore"
        );
        assert_eq!(hits[0].rule_id, "IN-CR-02");
        assert_eq!(hits[0].severity, Severity::Critical);

        // 验证工具调用 Critical（IN-CR-05）不被压制
        let mut f2 = InboundFilter::new(Arc::new(MockEngine), Arc::new(ignore));
        let tool = CompletedToolCall {
            id: "x".into(),
            name: "eth_signTransaction".into(),
            input: serde_json::json!({}),
        };
        let hits2 = f2.on_tool_use_complete(&tool).unwrap();
        assert!(
            !hits2.is_empty(),
            "Critical IN-CR-05 must not be suppressed by sieveignore"
        );
        assert_eq!(hits2[0].rule_id, "IN-CR-05");
        assert_eq!(hits2[0].severity, Severity::Critical);
    }
}
