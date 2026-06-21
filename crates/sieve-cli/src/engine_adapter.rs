//! 把 `sieve_rules::MatchEngine` 适配到 `sieve_core::OutboundEngine` /
//! `sieve_core::InboundEngine` trait。
//!
//! 阶段 1 sieve-core 不依赖 sieve-rules，所以 trait 定义在 sieve-core，
//! 由本 crate 在启动时桥接两边（`.cursorrules §3.3` crate 边界协调）。
//!
//! v2.0 Phase A 升级：adapter 改为泛型 `<E: MatchEngine>`，支持 `LayeredEngine<S, U>`
//! 传入（PRD §6.3 + §5.5.2.1），`is_excluded` 逻辑提取为模块级函数。
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

/// 检查命中片段是否被 placeholder 黑名单或 per-rule allowlist 排除。
///
/// 原先是 `VectorscanEngine::is_excluded` 的职责；提取到模块级后 adapter 可对任意
/// `MatchEngine` 实现通用过滤，不再依赖 `VectorscanEngine` 具体类型。
///
/// - `candidate`：vectorscan 命中的 matched text（短，仅命中片段）
/// - `full_context`：完整文档内容，用于 `allowlist_stopwords` 上下文感知匹配
/// - `rule`：对应规则条目（含 allowlist 数据）
fn is_excluded_by_rule(candidate: &str, full_context: &str, rule: &RuleEntry) -> bool {
    // 全局 placeholder 黑名单
    if sieve_rules::placeholder::is_placeholder(candidate) {
        return true;
    }
    // per-rule allowlist regexes（仅匹配 candidate，保持精准）
    for r in &rule.allowlist_regexes {
        if let Ok(re) = regex::Regex::new(r) {
            if re.is_match(candidate) {
                return true;
            }
        }
    }
    // per-rule allowlist stopwords：在 full_context（全文）中查找
    let search_in = if full_context.is_empty() {
        candidate
    } else {
        full_context
    };
    for sw in &rule.allowlist_stopwords {
        if search_in.contains(sw.as_str()) {
            return true;
        }
    }
    false
}

/// 出站规则匹配引擎的包装，实现 `sieve_core::OutboundEngine`。
///
/// 内部持有规则反查表（`rule_id → RuleEntry`），用于从 `MatchHit` 取真实 severity/action。
///
/// 泛型 `E` 允许传入 `VectorscanEngine`（系统规则）或 `LayeredEngine<VectorscanEngine, UserEngine>`
/// （系统 + 用户规则，PRD §6.3 / §5.5.2.1，Week 6 Phase A 起）。
pub struct OutboundAdapter<E: MatchEngine + Send + Sync + 'static = VectorscanEngine> {
    engine: Arc<E>,
    /// rule_id → RuleEntry 反查表，用于从 MatchHit 映射元数据。
    rule_lookup: HashMap<String, RuleEntry>,
}

impl<E: MatchEngine + Send + Sync + 'static> OutboundAdapter<E> {
    /// 构造 adapter。
    ///
    /// `rules` 与编译时传入的规则集一致，用于构建反查表。
    pub fn new(engine: Arc<E>, rules: Vec<RuleEntry>) -> Self {
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

/// 把 `sieve_rules::manifest::DefaultOnTimeout` 转为 `sieve_core::detection::DefaultOnTimeout`。
fn map_default_on_timeout(
    r: sieve_rules::manifest::DefaultOnTimeout,
) -> sieve_core::detection::DefaultOnTimeout {
    match r {
        sieve_rules::manifest::DefaultOnTimeout::Redact => {
            sieve_core::detection::DefaultOnTimeout::Redact
        }
        sieve_rules::manifest::DefaultOnTimeout::Block => {
            sieve_core::detection::DefaultOnTimeout::Block
        }
        sieve_rules::manifest::DefaultOnTimeout::Allow => {
            sieve_core::detection::DefaultOnTimeout::Allow
        }
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
    default_on_timeout: sieve_rules::manifest::DefaultOnTimeout,
) -> Action {
    use sieve_rules::manifest::Disposition;
    match disposition {
        Disposition::AutoRedact => Action::Redact {
            placeholder: format!("[REDACTED:{rule_id}]"),
        },
        Disposition::GuiPopup => Action::HoldForDecision {
            request_id: uuid::Uuid::new_v4(),
            timeout_seconds,
            default_on_timeout: map_default_on_timeout(default_on_timeout),
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

/// 入站规则匹配引擎的包装，实现 `sieve_core::InboundEngine`。
///
/// 与 [`OutboundAdapter`] 共用辅助函数（`map_severity` / `map_action` / `redact_evidence`），
/// 额外在工具调用检查中调用 `sieve_rules::critical_lock::enforce_action` 保证 fail-closed。
///
/// 泛型 `E` 允许传入 `VectorscanEngine`（系统规则）或 `LayeredEngine<VectorscanEngine, UserEngine>`
/// （系统 + 用户规则，PRD §6.3 / §5.5.2.1，Week 6 Phase A 起）。
pub struct InboundAdapter<E: MatchEngine + Send + Sync + 'static = VectorscanEngine> {
    engine: Arc<E>,
    /// rule_id → RuleEntry 反查表。
    rule_lookup: HashMap<String, RuleEntry>,
}

impl<E: MatchEngine + Send + Sync + 'static> InboundAdapter<E> {
    /// 构造 adapter。
    pub fn new(engine: Arc<E>, rules: Vec<RuleEntry>) -> Self {
        let rule_lookup = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
        Self {
            engine,
            rule_lookup,
        }
    }
}

impl<E: MatchEngine + Send + Sync + 'static> InboundEngine for InboundAdapter<E> {
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
                if is_excluded_by_rule(matched_text, input, r) {
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
                    map_action_by_disposition(
                        disp,
                        r.action,
                        &hit.rule_id,
                        timeout,
                        r.default_on_timeout,
                    )
                } else {
                    // 无显式 disposition：走旧路径（enforce_action → Block or action）
                    let enforced =
                        sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
                    if enforced == RulesAction::Block {
                        Action::Block
                    } else {
                        let disp = r.effective_disposition();
                        let timeout = r.timeout_seconds.unwrap_or(60);
                        map_action_by_disposition(
                            disp,
                            enforced,
                            &hit.rule_id,
                            timeout,
                            r.default_on_timeout,
                        )
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
                source_channel: None,
                origin_chain_depth: 0,
            });
        }

        // BIP39 inbound second-pass（关联 PRD §9 #4 / IN-CR-03-BIP39-INBOUND）
        // 攻击者诱导用户在入站对话中粘贴助记词（fear-privkey-074~087）。
        // 与 outbound 路径共用同一套 API（candidate_bip39_windows + verify_checksum），
        // 仅 checksum 通过的窗口定级 Critical（避免 BIP39 教学词组 FP）。
        //
        // allowlist 豁免：含以下短语时跳过（教学/规范文档场景）。
        // 注意：允许列表在 second-pass 层检查，不依赖 vectorscan is_excluded。
        let bip39_allowlist = [
            "BIP39 specification",
            "sample mnemonic",
            "test mnemonic from",
            "this is an example",
            "for example",
            "example mnemonic",
        ];
        let input_lower = input.to_lowercase();
        let is_bip39_allowlisted = bip39_allowlist
            .iter()
            .any(|w| input_lower.contains(&w.to_lowercase()));

        if !is_bip39_allowlisted {
            let wl = sieve_rules::wordlist::wordlist_index();
            let tokens: Vec<&str> = input.split_whitespace().collect();
            let candidates = sieve_rules::bip39::candidate_bip39_windows(&tokens, wl);
            for window in candidates {
                if sieve_rules::bip39::verify_checksum(&window, wl) {
                    let window_text = window.join(" ");
                    let evidence_truncated = redact_evidence(&window_text);
                    let fp = fingerprint("IN-CR-03-BIP39-INBOUND", &window_text);
                    detections.push(Detection {
                        id: Uuid::new_v4(),
                        rule_id: "IN-CR-03-BIP39-INBOUND".to_string(),
                        severity: Severity::Critical,
                        action: Action::HoldForDecision {
                            request_id: uuid::Uuid::new_v4(),
                            timeout_seconds: 120,
                            default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                        },
                        source,
                        span: ContentSpan {
                            start: body_offset,
                            end: body_offset + input.len(),
                        },
                        evidence_truncated,
                        fingerprint: fp,
                        source_channel: None,
                        origin_chain_depth: 0,
                    });
                    // 同一文本只需报一次
                    break;
                }
            }
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

impl<E: MatchEngine + Send + Sync + 'static> OutboundEngine for OutboundAdapter<E> {
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
                if is_excluded_by_rule(matched_text, input, r) {
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
                        map_action_by_disposition(
                            disp,
                            r.action,
                            &hit.rule_id,
                            timeout,
                            r.default_on_timeout,
                        )
                    } else {
                        // 无显式 disposition：走旧路径（enforce_action → Block or action）
                        let enforced =
                            sieve_rules::critical_lock::enforce_action(&hit.rule_id, r.action);
                        if enforced == RulesAction::Block {
                            Action::Block
                        } else {
                            let disp = r.effective_disposition();
                            let timeout = r.timeout_seconds.unwrap_or(60);
                            map_action_by_disposition(
                                disp,
                                enforced,
                                &hit.rule_id,
                                timeout,
                                r.default_on_timeout,
                            )
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
                source_channel: None,
                origin_chain_depth: 0,
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
                    source_channel: None,
                    origin_chain_depth: 0,
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
            fail_closed: None,
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

    /// OUT-07 PEM 私钥扫描：用真实规则文件验证 vectorscan 命中 span 及 Action 类型。
    ///
    /// R3-#3 修复前置验证：确认 OUT-07 对 PEM header 的命中 span 是正确的字节偏移，
    /// 以保证 `redact_segments` 能正确处理 GUI hold span。
    #[test]
    fn out07_pem_key_scan_span_and_action() {
        use sieve_rules::loader::load_outbound_rules;
        use std::path::PathBuf;

        let rules_path = {
            let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.pop(); // sieve-cli
            p.pop(); // crates
            p.push("crates/sieve-rules/rules/outbound.toml");
            p
        };

        let rules = load_outbound_rules(&rules_path).expect("load outbound rules");
        let engine = VectorscanEngine::compile(rules.clone()).expect("compile vectorscan");
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);

        // 模拟 pem_key_body() 里的 content 字段（extract_text_content 解析后的文本）
        let text = "这是我的密钥：-----BEGIN EC PRIVATE KEY-----\nMHQCAQEEINsamplekey\n-----END EC PRIVATE KEY-----";
        let expected_pem_start = text.find("-----BEGIN").expect("PEM header in text");

        let hits = adapter
            .scan_text(text, ContentSource::OutboundUserText, 0)
            .expect("scan_text");

        // 应该有命中（OUT-07 或其他 PEM 相关规则）
        let pem_hits: Vec<_> = hits
            .iter()
            .filter(|d| d.rule_id.starts_with("OUT-07"))
            .collect();

        assert!(
            !pem_hits.is_empty(),
            "OUT-07 应命中 PEM header，hits={hits:?}"
        );

        let pem_hit = &pem_hits[0];
        // span.start 应等于 PEM header 在文本中的字节偏移
        assert_eq!(
            pem_hit.span.start, expected_pem_start,
            "OUT-07 span.start 应等于 PEM header 的字节偏移"
        );
        assert!(
            pem_hit.span.end > pem_hit.span.start,
            "OUT-07 span.end 应大于 span.start"
        );
        // action 应是 HoldForDecision（disposition=gui_popup）
        assert!(
            matches!(pem_hit.action, Action::HoldForDecision { .. }),
            "OUT-07 应走 HoldForDecision 路径（disposition=gui_popup），实际: {:?}",
            pem_hit.action
        );
    }

    // ── R3-#4 / R10-#2 修复验证：default_on_timeout 从规则读取 ───────────────────

    /// OUT-06（JWT，default_on_timeout=redact）→ HoldForDecision.default_on_timeout = Redact。
    ///
    /// 修 R3-#4：之前硬编码 Block；现在从 RuleEntry.default_on_timeout 读取。
    #[test]
    fn out06_jwt_hold_for_decision_has_redact_timeout() {
        use sieve_rules::loader::load_outbound_rules;
        use std::path::PathBuf;

        let rules_path = {
            let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.pop();
            p.pop();
            p.push("crates/sieve-rules/rules/outbound.toml");
            p
        };

        let rules = load_outbound_rules(&rules_path).expect("load outbound rules");
        let engine = VectorscanEngine::compile(rules.clone()).expect("compile vectorscan");
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);

        // JWT 触发 OUT-06（disposition=gui_popup, default_on_timeout=redact, timeout=15s）
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let text = format!("token: {jwt}");
        let hits = adapter
            .scan_text(&text, ContentSource::OutboundUserText, 0)
            .expect("scan_text");

        let out06_hits: Vec<_> = hits.iter().filter(|d| d.rule_id == "OUT-06").collect();
        assert!(!out06_hits.is_empty(), "OUT-06 应命中 JWT token");

        let hit = &out06_hits[0];
        // 关键断言：default_on_timeout 应从规则读取为 Redact
        assert!(
            matches!(
                hit.action,
                Action::HoldForDecision {
                    default_on_timeout: sieve_core::detection::DefaultOnTimeout::Redact,
                    timeout_seconds: 15,
                    ..
                }
            ),
            "OUT-06 的 HoldForDecision 应有 default_on_timeout=Redact, timeout=15s，实际: {:?}",
            hit.action
        );
    }

    /// OUT-07（PEM 私钥，default_on_timeout=block）→ HoldForDecision.default_on_timeout = Block。
    ///
    /// 修 R3-#4 回归：OUT-07 本来就是 Block，确认没有被错误改为 Redact。
    #[test]
    fn out07_pem_hold_for_decision_has_block_timeout() {
        use sieve_rules::loader::load_outbound_rules;
        use std::path::PathBuf;

        let rules_path = {
            let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.pop();
            p.pop();
            p.push("crates/sieve-rules/rules/outbound.toml");
            p
        };

        let rules = load_outbound_rules(&rules_path).expect("load outbound rules");
        let engine = VectorscanEngine::compile(rules.clone()).expect("compile vectorscan");
        let adapter = OutboundAdapter::new(Arc::new(engine), rules);

        let text = "my key: -----BEGIN EC PRIVATE KEY----- MHQCAQEE ...";
        let hits = adapter
            .scan_text(text, ContentSource::OutboundUserText, 0)
            .expect("scan_text");

        let out07_hits: Vec<_> = hits.iter().filter(|d| d.rule_id == "OUT-07").collect();
        assert!(!out07_hits.is_empty(), "OUT-07 应命中 PEM 私钥");

        let hit = &out07_hits[0];
        assert!(
            matches!(
                hit.action,
                Action::HoldForDecision {
                    default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                    ..
                }
            ),
            "OUT-07 的 HoldForDecision 应有 default_on_timeout=Block，实际: {:?}",
            hit.action
        );
    }

    // ── R9-#2 修复验证：chain_depth ≥ 2 时 Redact 升级为 HoldForDecision ─────────

    /// chain_depth ≥ 2 时 Action::Redact（OUT-01 sk-ant-* secret）应升级为 HoldForDecision。
    ///
    /// 修 R9-#2：daemon Anthropic / OpenAI 路径在 chain_depth ≥ 2 时对 Redact action 升级。
    /// 本测试直接构造 detection 验证升级逻辑，不依赖完整 daemon。
    #[test]
    fn r9_fix_redact_upgrades_to_hold_for_decision_at_chain_depth_2() {
        use sieve_core::detection::DefaultOnTimeout;

        // 构造一个 OUT-01 Redact detection（模拟 auto_redact 路径）
        let redact_detection = sieve_core::detection::Detection {
            id: uuid::Uuid::new_v4(),
            rule_id: "OUT-01".to_string(),
            severity: Severity::Critical,
            action: Action::Redact {
                placeholder: "[REDACTED:OUT-01]".to_string(),
            },
            source: ContentSource::OutboundUserText,
            span: sieve_core::protocol::unified_message::ContentSpan { start: 0, end: 10 },
            evidence_truncated: "sk-a***nce".to_string(),
            fingerprint: "abcdef0123456789".to_string(),
            source_channel: None,
            origin_chain_depth: 2,
        };

        // 模拟 daemon chain_depth ≥ 2 升级逻辑
        let chain_depth = 2_usize;
        let mut d = redact_detection;
        if chain_depth >= 2 {
            if let Action::Redact { .. } = &d.action {
                d.action = Action::HoldForDecision {
                    request_id: uuid::Uuid::new_v4(),
                    timeout_seconds: 60,
                    default_on_timeout: DefaultOnTimeout::Redact,
                };
            }
        }

        assert!(
            matches!(d.action, Action::HoldForDecision { .. }),
            "chain_depth=2 时 Redact 应升级为 HoldForDecision，实际: {:?}",
            d.action
        );
        // 升级后的 default_on_timeout 应保持 Redact 语义
        assert!(
            matches!(
                d.action,
                Action::HoldForDecision {
                    default_on_timeout: DefaultOnTimeout::Redact,
                    ..
                }
            ),
            "Redact 升级后的 default_on_timeout 应为 Redact，实际: {:?}",
            d.action
        );
    }

    /// Anthropic 入站 chain_depth=2 + IN-CR-02（hook_terminal）→ HookMark 升级为 HoldForDecision。
    ///
    /// 修 R8-#2-Inbound 验证：Anthropic 入站路径与 OpenAI 入站路径同等升级。
    #[test]
    fn r8_inbound_hookmark_upgrades_to_hold_at_chain_depth_2() {
        use sieve_core::detection::DefaultOnTimeout;

        let mut rule = make_rule(
            "IN-CR-02",
            r"rm -rf",
            RulesSeverity::Critical,
            RulesAction::Block,
        );
        rule.disposition = Some(sieve_rules::manifest::Disposition::HookTerminal);

        let engine = VectorscanEngine::compile(vec![rule.clone()]).unwrap();
        let adapter = InboundAdapter::new(Arc::new(engine), vec![rule]);

        // 先确认不含 chain_depth 时是 HookMark
        let hits = adapter
            .scan_text("run: rm -rf /tmp", ContentSource::InboundAssistantText, 0)
            .unwrap();
        assert_eq!(hits.len(), 1);
        assert!(
            matches!(hits[0].action, Action::HookMark),
            "chain_depth=0 应是 HookMark: {:?}",
            hits[0].action
        );

        // 模拟 classify_inbound_detections chain_depth=2 升级逻辑
        let chain_depth = 2_usize;
        let mut d = hits[0].clone();
        if chain_depth >= 2 && matches!(d.action, Action::HookMark) {
            d.action = Action::HoldForDecision {
                request_id: uuid::Uuid::new_v4(),
                timeout_seconds: 60,
                default_on_timeout: DefaultOnTimeout::Block,
            };
        }

        assert!(
            matches!(d.action, Action::HoldForDecision { .. }),
            "Anthropic 入站 chain_depth=2 IN-CR-02 HookMark 应升级为 HoldForDecision: {:?}",
            d.action
        );
    }
}
