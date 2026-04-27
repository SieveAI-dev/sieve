//! Vectorscan 多模式正则引擎（关联 ADR-001 / ADR-002 / PRD §6.4）。
//!
//! Phase 1 用 block mode（出站请求一次性扫描）；Week 3 起 stream mode 处理 SSE 流式。
//!
//! # 生命周期设计说明
//!
//! `vectorscan_rs::BlockScanner<'db>` 借用 `&'db BlockDatabase`，无法与 db 同存于同一 struct
//! 而不引入 unsafe self-referential pattern。鉴于 `lib.rs` 已有 `#![deny(unsafe_code)]`，
//! 本实现选择每次 `scan()` 调用时从 `BlockDatabase` 创建 `BlockScanner`（alloc scratch）。
//! scratch 分配代价远小于实际扫描代价，在 P99 < 20ms 目标下可接受。
//! Week 3 如需优化，可改为 `thread_local!` scratch 复用方案（仍无 unsafe）。

use crate::error::{SieveRulesError, SieveRulesResult};
use crate::manifest::RuleEntry;
use crate::placeholder::is_placeholder;
use std::collections::HashMap;
use vectorscan_rs::{BlockDatabase, BlockScanner, Flag, Pattern, Scan};

/// 一次匹配的位置信息。
#[derive(Debug, Clone)]
pub struct MatchHit {
    /// 命中的规则 ID（如 OUT-01）。
    pub rule_id: String,
    /// 命中位置在输入字节流的起始偏移（闭区间，需 SOM_LEFTMOST flag）。
    pub start: usize,
    /// 命中位置的结束偏移（开区间）。
    pub end: usize,
}

/// 多模式匹配引擎 trait。
pub trait MatchEngine: Send + Sync {
    /// 对输入字节流执行多模式匹配，返回所有命中。
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>>;
}

/// Vectorscan 多模式正则引擎。
///
/// 编译后的 `BlockDatabase` 线程安全（`Send + Sync`）；扫描时按需创建 `BlockScanner`（含 scratch）。
pub struct VectorscanEngine {
    db: BlockDatabase,
    rules: HashMap<u32, RuleEntry>,
}

impl VectorscanEngine {
    /// 编译规则集为 vectorscan database。
    ///
    /// 每条规则的 `pattern` 编译为带 `SOM_LEFTMOST` flag（精确报告 start offset）。
    pub fn compile(rules: Vec<RuleEntry>) -> SieveRulesResult<Self> {
        let patterns: Vec<Pattern> = rules
            .iter()
            .enumerate()
            .map(|(i, r)| {
                Pattern::new(
                    r.pattern.as_bytes().to_vec(),
                    Flag::SOM_LEFTMOST,
                    Some(i as u32),
                )
            })
            .collect();

        let db = BlockDatabase::new(patterns)
            .map_err(|e| SieveRulesError::Engine(format!("compile vectorscan db: {e}")))?;

        let rules_map: HashMap<u32, RuleEntry> = rules
            .into_iter()
            .enumerate()
            .map(|(i, r)| (i as u32, r))
            .collect();

        Ok(Self {
            db,
            rules: rules_map,
        })
    }

    /// 获取规则元信息（用于上层组装 Detection）。
    pub fn rule_meta(&self, pattern_id: u32) -> Option<&RuleEntry> {
        self.rules.get(&pattern_id)
    }

    /// 候选文本是否被 placeholder / per-rule allowlist 排除。
    pub fn is_excluded(&self, candidate: &str, rule: &RuleEntry) -> bool {
        // 全局 placeholder 黑名单
        if is_placeholder(candidate) {
            return true;
        }
        // per-rule allowlist regexes
        for r in &rule.allowlist_regexes {
            if let Ok(re) = regex::Regex::new(r) {
                if re.is_match(candidate) {
                    return true;
                }
            }
        }
        // per-rule allowlist stopwords
        for sw in &rule.allowlist_stopwords {
            if candidate.contains(sw.as_str()) {
                return true;
            }
        }
        false
    }
}

impl MatchEngine for VectorscanEngine {
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>> {
        // 每次 scan 创建新 scanner（alloc scratch）。
        // 参见模块文档中关于生命周期设计的说明。
        let mut scanner = BlockScanner::new(&self.db)
            .map_err(|e| SieveRulesError::Engine(format!("create scanner: {e}")))?;

        let mut hits: Vec<MatchHit> = Vec::new();
        scanner
            .scan(input, |id, from, to, _flags| {
                let rule_id = self
                    .rules
                    .get(&id)
                    .map(|r| r.id.clone())
                    .unwrap_or_default();
                hits.push(MatchHit {
                    rule_id,
                    start: from as usize,
                    end: to as usize,
                });
                Scan::Continue
            })
            .map_err(|e| SieveRulesError::Engine(format!("scan failed: {e}")))?;

        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{Action, Severity};

    fn rule(id: &str, pattern: &str, severity: Severity) -> RuleEntry {
        RuleEntry {
            id: id.into(),
            description: id.into(),
            pattern: pattern.into(),
            severity,
            action: Action::Block,
            entropy_min: None,
            keywords: vec![],
            allowlist_regexes: vec![],
            allowlist_stopwords: vec![],
        }
    }

    #[test]
    fn compile_and_scan_simple() {
        let rules = vec![rule("OUT-TEST", r"hello", Severity::Critical)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let hits = engine.scan(b"say hello world").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "OUT-TEST");
        assert_eq!(hits[0].start, 4);
        assert_eq!(hits[0].end, 9);
    }

    #[test]
    fn no_match_returns_empty() {
        let rules = vec![rule("OUT-TEST", r"hello", Severity::Critical)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let hits = engine.scan(b"goodbye world").unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn multiple_patterns_match() {
        let rules = vec![
            rule("OUT-A", r"foo", Severity::High),
            rule("OUT-B", r"bar", Severity::Low),
        ];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let hits = engine.scan(b"foobar").unwrap();
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn is_excluded_placeholder() {
        let rules = vec![rule("OUT-01", r"sk-ant-api03", Severity::Critical)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let rule_entry = engine.rule_meta(0).unwrap();
        assert!(engine.is_excluded("sk-ant-api03-XXXXXXXX", rule_entry));
        assert!(!engine.is_excluded("sk-ant-api03-real-mixed-content-xyz", rule_entry));
    }

    #[test]
    fn allowlist_stopword_excludes() {
        let mut r = rule("OUT-01", r"secret", Severity::High);
        r.allowlist_stopwords = vec!["example".to_string()];
        let rules = vec![r];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let rule_entry = engine.rule_meta(0).unwrap();
        assert!(engine.is_excluded("my example secret", rule_entry));
        assert!(!engine.is_excluded("my real secret", rule_entry));
    }

    #[test]
    fn allowlist_regex_excludes() {
        let mut r = rule("OUT-01", r"private_key", Severity::High);
        r.allowlist_regexes = vec![r"(?i)test".to_string()];
        let rules = vec![r];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let rule_entry = engine.rule_meta(0).unwrap();
        assert!(engine.is_excluded("test_private_key", rule_entry));
        assert!(!engine.is_excluded("prod_private_key", rule_entry));
    }
}
