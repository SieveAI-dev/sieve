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
    ///
    /// - `candidate`：vectorscan 命中的 matched text（短，仅命中片段）。
    /// - `full_context`：完整文档内容，用于 `allowlist_stopwords` 上下文感知匹配。
    ///   传空字符串时退化为仅检查 `candidate`（向后兼容）。
    pub fn is_excluded(&self, candidate: &str, full_context: &str, rule: &RuleEntry) -> bool {
        // 全局 placeholder 黑名单
        if is_placeholder(candidate) {
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
        // per-rule allowlist stopwords：在 full_context（全文）中查找，
        // 使得文档中的教学/警告语境可以豁免短 matched text（如 `eval "$(` / `rm -rf /`）。
        // full_context 为空时退化为 candidate 内查找。
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
}

impl MatchEngine for VectorscanEngine {
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>> {
        // 每次 scan 创建新 scanner（alloc scratch）。
        // 参见模块文档中关于生命周期设计的说明。
        let mut scanner = BlockScanner::new(&self.db)
            .map_err(|e| SieveRulesError::Engine(format!("create scanner: {e}")))?;

        // vectorscan 对带量词的 pattern（`{m,n}` / `(?:..)*` 等）会在每个合法 end
        // 位置都触发回调。例如 `\.env\b(?:\.[a-z]+)*` 在 `.env.example` 上会从
        // start=0 emit end=4,6,7,...,12 多次。下游 allowlist 只能看到 matched_text，
        // 短 match（仅 `.env`）拿不到完整文件名上下文，会绕过 `\.env\.example` 白名单。
        //
        // 此处按 (rule_id, start) 保留**最长** end，给上层 longest-match 语义。
        // 关联：IN-CR-03-DOTENV / IN-CR-03-SSH-DIR allowlist 正确性。
        let mut by_key: HashMap<(String, usize), MatchHit> = HashMap::new();
        scanner
            .scan(input, |id, from, to, _flags| {
                let rule_id = self
                    .rules
                    .get(&id)
                    .map(|r| r.id.clone())
                    .unwrap_or_default();
                let key = (rule_id.clone(), from as usize);
                by_key
                    .entry(key)
                    .and_modify(|existing| {
                        if (to as usize) > existing.end {
                            existing.end = to as usize;
                        }
                    })
                    .or_insert(MatchHit {
                        rule_id,
                        start: from as usize,
                        end: to as usize,
                    });
                Scan::Continue
            })
            .map_err(|e| SieveRulesError::Engine(format!("scan failed: {e}")))?;

        // 输出排序保证测试与下游处理的确定性
        let mut hits: Vec<MatchHit> = by_key.into_values().collect();
        hits.sort_by(|a, b| {
            a.start
                .cmp(&b.start)
                .then_with(|| a.rule_id.cmp(&b.rule_id))
        });
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
            disposition: None,
            timeout_seconds: None,
            default_on_timeout: crate::manifest::DefaultOnTimeout::Block,
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
        assert!(engine.is_excluded("sk-ant-api03-XXXXXXXX", "", rule_entry));
        assert!(!engine.is_excluded("sk-ant-api03-real-mixed-content-xyz", "", rule_entry));
    }

    #[test]
    fn allowlist_stopword_excludes() {
        let mut r = rule("OUT-01", r"secret", Severity::High);
        r.allowlist_stopwords = vec!["example".to_string()];
        let rules = vec![r];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let rule_entry = engine.rule_meta(0).unwrap();
        assert!(engine.is_excluded("my example secret", "", rule_entry));
        assert!(!engine.is_excluded("my real secret", "", rule_entry));
        // full_context 里包含 stopword 时也应豁免（即使 candidate 本身没有）
        assert!(engine.is_excluded("secret", "this is an example of a secret", rule_entry));
        assert!(!engine.is_excluded("secret", "this is a real secret", rule_entry));
    }

    #[test]
    fn allowlist_regex_excludes() {
        let mut r = rule("OUT-01", r"private_key", Severity::High);
        r.allowlist_regexes = vec![r"(?i)test".to_string()];
        let rules = vec![r];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let rule_entry = engine.rule_meta(0).unwrap();
        assert!(engine.is_excluded("test_private_key", "", rule_entry));
        assert!(!engine.is_excluded("prod_private_key", "", rule_entry));
    }

    /// vectorscan 对带量词的 pattern 会触发多个 endpoint 回调；引擎必须保留最长 end，
    /// 否则 allowlist 看不到完整 matched_text 会漏过短 match。关联 IN-CR-03-DOTENV。
    #[test]
    fn longest_match_per_start_dedup() {
        let rules = vec![rule("TEST-DOTENV", r"\.env\b(?:\.[a-z]+)*", Severity::High)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let hits = engine.scan(b"read .env.example").unwrap();
        // 期望：仅 1 个 hit，匹配整段 `.env.example`（end=17），而非短 `.env`（end=9）
        let dotenv_hits: Vec<_> = hits.iter().filter(|h| h.rule_id == "TEST-DOTENV").collect();
        assert_eq!(
            dotenv_hits.len(),
            1,
            "expected single longest-match per start, got: {hits:?}"
        );
        assert_eq!(dotenv_hits[0].start, 5);
        assert_eq!(
            dotenv_hits[0].end, 17,
            "should keep longest end (.env.example = 12 chars), got {hits:?}"
        );
    }
}
