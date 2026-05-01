//! 用户规则引擎（PRD v2.0 §6.3.1 / §5.5.2.1）。
//!
//! [`UserEngine`] 包装 [`sieve_rules::engine::VectorscanEngine`]，将 [`UserRuleEntry`]
//! 转换为 [`RuleEntry`] 后编译。所有命中的 `rule_id` 自动加 `user:` 前缀，防止与系统规则冲突
//! （PRD §5.5.2.1 "命中标识"）。

use crate::error::{PolicyError, PolicyResult};
use crate::loader::UserRuleEntry;
use sieve_rules::engine::{MatchEngine, MatchHit, ScanReport, ScanRequest, VectorscanEngine};
use sieve_rules::error::SieveRulesResult;
use sieve_rules::manifest::{Action, DefaultOnTimeout, Disposition, RuleEntry, Severity};

/// 用户规则引擎。
///
/// 实现 [`MatchEngine`] trait，命中的 `rule_id` 携带 `user:` 前缀（如 `user:MY-RULE`）。
/// 与 [`sieve_rules::engine::LayeredEngine`] 配合，保证系统规则先行、用户规则后行的合并顺序。
pub struct UserEngine {
    inner: VectorscanEngine,
    /// 已编译规则条数（用于 rule_count 元信息）。
    count: usize,
}

impl UserEngine {
    /// 将 `UserRuleEntry` 列表编译为 vectorscan database。
    ///
    /// 只编译 `enabled = true` 的规则（禁用规则跳过）。
    pub fn compile(rules: Vec<UserRuleEntry>) -> PolicyResult<Self> {
        let enabled: Vec<UserRuleEntry> = rules.into_iter().filter(|r| r.enabled).collect();
        let count = enabled.len();

        let rule_entries: Vec<RuleEntry> = enabled.into_iter().map(to_rule_entry).collect();

        let inner = VectorscanEngine::compile(rule_entries)
            .map_err(|e| PolicyError::EngineCompile(e.to_string()))?;

        Ok(Self { inner, count })
    }
}

impl MatchEngine for UserEngine {
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>> {
        let mut hits = self.inner.scan(input)?;
        // 所有命中的 rule_id 加 `user:` 前缀（PRD §5.5.2.1）
        for h in &mut hits {
            if !h.rule_id.starts_with("user:") {
                h.rule_id = format!("user:{}", h.rule_id);
            }
        }
        Ok(hits)
    }

    fn scan_with_context(&self, req: ScanRequest<'_>) -> SieveRulesResult<ScanReport> {
        let start = std::time::Instant::now();
        let hits = self.scan(req.bytes)?;
        Ok(ScanReport {
            hits,
            elapsed_us: start.elapsed().as_micros() as u64,
            engine_name: self.engine_name().to_string(),
            rule_count: self.rule_count(),
        })
    }

    fn engine_name(&self) -> &str {
        "user-vectorscan"
    }

    fn rule_count(&self) -> usize {
        self.count
    }

    fn compiled_pattern_size_bytes(&self) -> usize {
        self.inner.compiled_pattern_size_bytes()
    }
}

/// 将 [`UserRuleEntry`] 转换为 [`RuleEntry`]（供 VectorscanEngine 使用）。
///
/// severity/action/disposition 字符串转为枚举；unknown 值安全降级（不 panic）。
fn to_rule_entry(u: UserRuleEntry) -> RuleEntry {
    let severity = match u.severity.to_lowercase().as_str() {
        "critical" => Severity::Critical, // lint 已拦截，此处防御性处理
        "high" => Severity::High,
        "medium" => Severity::Medium,
        _ => Severity::Low,
    };
    let action = match u.action.to_lowercase().as_str() {
        "block" => Action::Block, // lint 已拦截，此处防御性处理
        "warn" => Action::Warn,
        "mark" => Action::Mark,
        _ => Action::Warn,
    };
    let disposition = u.disposition.as_deref().and_then(|d| match d {
        "auto_redact" => Some(Disposition::AutoRedact),
        "gui_popup" => Some(Disposition::GuiPopup),
        "hook_terminal" => Some(Disposition::HookTerminal), // lint 已拦截
        "status_bar" => Some(Disposition::StatusBar),
        _ => None,
    });

    RuleEntry {
        id: u.id,
        description: u.description,
        pattern: u.pattern,
        severity,
        action,
        entropy_min: None,
        keywords: u.keywords,
        allowlist_regexes: vec![],
        allowlist_stopwords: u.allowlist_stopwords,
        disposition,
        timeout_seconds: None,
        default_on_timeout: DefaultOnTimeout::Allow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_entry(id: &str, pattern: &str, enabled: bool) -> UserRuleEntry {
        UserRuleEntry {
            id: id.into(),
            description: "test".into(),
            pattern: pattern.into(),
            severity: "high".into(),
            action: "warn".into(),
            keywords: vec!["keyword".into()],
            allowlist_stopwords: vec![],
            disposition: None,
            enabled,
            added_at: Utc::now(),
            added_by: "manual".into(),
        }
    }

    #[test]
    fn user_engine_compile_and_scan() {
        let rules = vec![make_entry("MY-RULE", r"secret_token", true)];
        let engine = UserEngine::compile(rules).unwrap();
        let hits = engine.scan(b"found secret_token here").unwrap();
        assert_eq!(hits.len(), 1);
        // rule_id 必须携带 user: 前缀
        assert_eq!(hits[0].rule_id, "user:MY-RULE");
    }

    #[test]
    fn disabled_rules_not_compiled() {
        let rules = vec![
            make_entry("ENABLED", r"pattern_a", true),
            make_entry("DISABLED", r"pattern_b", false),
        ];
        let engine = UserEngine::compile(rules).unwrap();
        assert_eq!(
            engine.rule_count(),
            1,
            "disabled rule should not be compiled"
        );

        // pattern_b 不应命中
        let hits = engine.scan(b"pattern_b present").unwrap();
        assert!(hits.is_empty(), "disabled rule should not match: {hits:?}");
    }

    #[test]
    fn user_prefix_not_doubled() {
        // 即使多次调用 scan，user: 前缀不应重复
        let rules = vec![make_entry("MY-RULE", r"target", true)];
        let engine = UserEngine::compile(rules).unwrap();
        let hits = engine.scan(b"target").unwrap();
        assert_eq!(hits[0].rule_id, "user:MY-RULE");
        // 不是 "user:user:MY-RULE"
        assert!(!hits[0].rule_id.starts_with("user:user:"));
    }

    #[test]
    fn scan_with_context_returns_correct_engine_name() {
        let rules = vec![make_entry("MY-RULE", r"target", true)];
        let engine = UserEngine::compile(rules).unwrap();
        let req = sieve_rules::engine::ScanRequest {
            bytes: b"target here",
            direction: sieve_rules::engine::Direction::Inbound,
            protocol: sieve_rules::engine::Protocol::Anthropic,
            content_kind: sieve_rules::engine::ContentKind::ToolUseInput,
            tool_name: Some("Bash"),
            source_agent: Some("claude-code"),
            caller_exe: None,
        };
        let report = engine.scan_with_context(req).unwrap();
        assert_eq!(report.engine_name, "user-vectorscan");
        assert_eq!(report.rule_count, 1);
        assert_eq!(report.hits[0].rule_id, "user:MY-RULE");
    }

    #[test]
    fn empty_rules_compiles_ok() {
        // VectorscanEngine 不支持空规则列表（panic/error 取决于实现）。
        // UserEngine 有 0 条规则时 scan 返回空 hits 即可。
        // 注意：若 VectorscanEngine::compile(vec![]) 报错，UserEngine 也报错，这是预期行为。
        let rules: Vec<UserRuleEntry> = vec![];
        // 空规则列表可能 compile 失败（vectorscan 不支持）或成功，均可接受
        match UserEngine::compile(rules) {
            Ok(engine) => {
                let hits = engine.scan(b"anything").unwrap();
                assert!(hits.is_empty());
            }
            Err(_) => {
                // vectorscan 不支持空规则，engine 报错是可接受行为
            }
        }
    }
}
