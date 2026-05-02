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

use crate::critical_lock::is_fail_closed;
use crate::error::{SieveRulesError, SieveRulesResult};
use crate::manifest::RuleEntry;
use crate::placeholder::is_placeholder;
use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;
use vectorscan_rs::{BlockDatabase, BlockScanner, Flag, Pattern, Scan};

/// 扫描方向（PRD v2.0 §6.3.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// 入站（model → user，拦截危险 tool_use 输出）。
    Inbound,
    /// 出站（user → API，拦截敏感数据上传）。
    Outbound,
}

/// 上游协议类型（PRD v2.0 §6.3.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// Anthropic Messages API。
    Anthropic,
    /// OpenAI Chat Completions API（Phase 2，ADR-018）。
    OpenAI,
}

/// 扫描内容类型（PRD v2.0 §6.3.1）。
///
/// 让规则引擎知道自己在哪条路径生效，v1.5.4 P0 教训之一。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    /// SSE event data delta（流式 `data: {...}` 行）。
    SseEventDelta,
    /// 非流式 JSON 响应体（`stream=false`）。
    JsonResponseBody,
    /// 工具调用输入（`tool_use.input` 字段）。
    ToolUseInput,
    /// 出站请求体。
    RequestBody,
}

/// 扫描请求上下文（PRD v2.0 §6.3.1）。
///
/// 相比裸 `&[u8]`，携带路由上下文与业务上下文，用于 fingerprint 计算、
/// 灰名单查询、序列窗口（Phase B）。内部全为引用/Copy，实现 [`Clone`]。
#[derive(Debug, Clone, Copy)]
pub struct ScanRequest<'a> {
    /// 待扫描的原始字节流。
    pub bytes: &'a [u8],
    /// 流量方向（入站 / 出站）。
    pub direction: Direction,
    /// 上游协议。
    pub protocol: Protocol,
    /// 内容类型。
    pub content_kind: ContentKind,
    /// 工具名（仅 [`ContentKind::ToolUseInput`] 时有意义，如 `"Bash"`）。
    pub tool_name: Option<&'a str>,
    /// 调用方 Agent 标识（如 `"claude-code"` / `"openclaw"`）。
    pub source_agent: Option<&'a str>,
    /// 调用方进程可执行路径（进程上下文，PRD §5.6）。
    pub caller_exe: Option<&'a std::path::Path>,
}

/// 一次扫描的汇总报告（PRD v2.0 §6.3.1）。
#[derive(Debug)]
pub struct ScanReport {
    /// 所有命中列表。
    pub hits: Vec<MatchHit>,
    /// 本次扫描耗时（微秒）。
    pub elapsed_us: u64,
    /// 引擎名称（用于 audit / 调试）。
    pub engine_name: String,
    /// 本次生效的规则条数。
    pub rule_count: usize,
}

/// 一次匹配的位置信息。
#[derive(Debug, Clone)]
pub struct MatchHit {
    /// 命中的规则 ID（如 OUT-01；用户规则携带 `user:` 前缀）。
    pub rule_id: String,
    /// 命中位置在输入字节流的起始偏移（闭区间，需 SOM_LEFTMOST flag）。
    pub start: usize,
    /// 命中位置的结束偏移（开区间）。
    pub end: usize,
}

/// 多模式匹配引擎 trait（PRD v2.0 §6.3.1）。
///
/// v2.0 扩展：保留原 `scan(&[u8])` 向后兼容接口，新增带上下文的
/// `scan_with_context(ScanRequest)` 及引擎元信息方法。
pub trait MatchEngine: Send + Sync {
    /// 对输入字节流执行多模式匹配，返回所有命中（向后兼容接口）。
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>>;

    /// 带上下文的扫描（v2.0 新增，PRD §6.3.1）。
    ///
    /// 默认实现委托给 [`MatchEngine::scan`]，携带耗时统计与引擎元信息。
    /// [`LayeredEngine`] 等组合引擎会覆盖此方法以实现合并逻辑。
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

    /// 引擎标识名（用于 audit 与调试）。
    fn engine_name(&self) -> &str {
        "unknown"
    }

    /// 当前生效的规则条数。
    fn rule_count(&self) -> usize {
        0
    }

    /// 编译后的 vectorscan pattern database 大小（字节）。
    fn compiled_pattern_size_bytes(&self) -> usize {
        0
    }
}

/// 合并系统规则与用户规则的分层引擎（PRD v2.0 §6.3.1）。
///
/// # 合并顺序（严格保证，不可调整）
///
/// 1. 系统规则全量扫描
/// 2. 系统规则命中任意 fail-closed（Critical）规则 → 立即返回，不评估用户规则
/// 3. 否则追加用户规则命中（用户规则命中的 `rule_id` 已携带 `user:` 前缀，由 UserEngine 保证）
///
/// 此顺序保证用户规则无法 suppress 系统 Critical 命中（PRD §9 #3 + §5.5.2.1）。
///
/// # Hot Swap（PRD §5.5.5 / v2.1 zero-downtime reload）
///
/// `user` 字段由 [`arc_swap::ArcSwap`] 包装，允许 daemon reload listener 通过
/// [`LayeredEngine::swap_user`] 原子替换用户引擎，无需重启 daemon：
/// - scan 路径（hot path）调用 `ArcSwap::load()` 取快照，**零锁零开销**（lock-free read）。
/// - swap 路径调用 `ArcSwap::store()` 原子写入新指针，所有后续 scan 立即看到新引擎。
/// - 正在进行中的 scan 持有旧 `Arc<U>`，结束后旧引擎自动释放（引用计数归零）。
pub struct LayeredEngine<S: MatchEngine, U: MatchEngine> {
    system: S,
    /// 原子可替换的用户引擎（PRD §5.5.5 hot reload）。
    ///
    /// `ArcSwap<Option<Arc<U>>>` 允许：
    /// - `None`：无用户规则，纯系统引擎模式
    /// - `Some(Arc<U>)`：系统 + 用户规则分层模式
    user: ArcSwap<Option<Arc<U>>>,
}

impl<S: MatchEngine, U: MatchEngine> LayeredEngine<S, U> {
    /// 构造分层引擎。`user` 为 `None` 时退化为纯系统规则引擎。
    pub fn new(system: S, user: Option<U>) -> Self {
        Self {
            system,
            user: ArcSwap::from(Arc::new(user.map(Arc::new))),
        }
    }

    /// Atomic swap 用户规则引擎（PRD §5.5.5 zero-downtime hot reload）。
    ///
    /// daemon reload listener 调用此方法替换 user engine，无需重启 daemon。
    /// 调用完成后所有后续 [`LayeredEngine::scan`] 调用立即使用新引擎；
    /// 已在进行中的 scan 持有旧 `Arc<U>` 快照，完成后旧引擎自动释放。
    /// 传入 `None` 时退化为纯系统规则引擎（等同于构造时 `user = None`）。
    pub fn swap_user(&self, new_user: Option<U>) {
        self.user.store(Arc::new(new_user.map(Arc::new)));
    }
}

impl<S: MatchEngine, U: MatchEngine> MatchEngine for LayeredEngine<S, U> {
    fn scan(&self, input: &[u8]) -> SieveRulesResult<Vec<MatchHit>> {
        // 构造 dummy ScanRequest 复用 scan_with_context 合并逻辑，避免重复代码。
        let req = ScanRequest {
            bytes: input,
            direction: Direction::Outbound,
            protocol: Protocol::Anthropic,
            content_kind: ContentKind::RequestBody,
            tool_name: None,
            source_agent: None,
            caller_exe: None,
        };
        self.scan_with_context(req).map(|r| r.hits)
    }

    fn scan_with_context(&self, req: ScanRequest<'_>) -> SieveRulesResult<ScanReport> {
        // 第一层：系统规则全量扫描
        let mut report = self.system.scan_with_context(req)?;

        // 系统规则命中任意 fail-closed（Critical）规则 → 立即返回（PRD §6.3.1 合并顺序 #1）
        if report.hits.iter().any(|h| is_fail_closed(&h.rule_id)) {
            return Ok(report);
        }

        // 第二层：追加用户规则命中（`user:` 前缀由 UserEngine 保证）。
        // ArcSwap::load() 返回 Guard（内部含 Arc 快照），零锁 lock-free read，hot path 安全。
        let user_guard = self.user.load();
        if let Some(user_arc) = user_guard.as_ref().as_ref() {
            let user_report = user_arc.scan_with_context(req)?;
            report.rule_count += user_report.rule_count;
            report.elapsed_us += user_report.elapsed_us;
            report.hits.extend(user_report.hits);
        }

        Ok(report)
    }

    fn engine_name(&self) -> &str {
        "layered"
    }

    fn rule_count(&self) -> usize {
        let user_guard = self.user.load();
        self.system.rule_count()
            + user_guard
                .as_ref()
                .as_ref()
                .map(|u| u.rule_count())
                .unwrap_or(0)
    }

    fn compiled_pattern_size_bytes(&self) -> usize {
        let user_guard = self.user.load();
        self.system.compiled_pattern_size_bytes()
            + user_guard
                .as_ref()
                .as_ref()
                .map(|u| u.compiled_pattern_size_bytes())
                .unwrap_or(0)
    }
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
    fn engine_name(&self) -> &str {
        "vectorscan"
    }

    fn rule_count(&self) -> usize {
        self.rules.len()
    }

    fn compiled_pattern_size_bytes(&self) -> usize {
        // vectorscan_rs 暂未暴露 `hs_database_size()`，无法精确测量 compiled DB 体积；
        // 返回 0 作占位，由 lint 阶段编译时间 100ms 上限（PRD §5.5.3-B）兜底间接限流。
        // 上游暴露 API 后此处与 sieve-policy/src/lint.rs 同步补 1MB 真值校验
        //（跟踪：tasks/v2-pending.md TODO-EXT-1）。
        0
    }

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

    // -------------------------------------------------------------------------
    // v2.0 新增：engine_name / rule_count / scan_with_context 测试
    // -------------------------------------------------------------------------

    #[test]
    fn vectorscan_engine_meta() {
        let rules = vec![
            rule("OUT-A", r"foo", Severity::High),
            rule("OUT-B", r"bar", Severity::Low),
        ];
        let engine = VectorscanEngine::compile(rules).unwrap();
        assert_eq!(engine.engine_name(), "vectorscan");
        assert_eq!(engine.rule_count(), 2);
    }

    /// scan(&[u8]) 向后兼容回归：v2.0 新增 trait 方法不破坏旧接口行为。
    #[test]
    fn scan_bytes_backward_compat() {
        let rules = vec![rule("OUT-TEST", r"hello", Severity::Critical)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        // 旧接口
        let hits = engine.scan(b"say hello world").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "OUT-TEST");
    }

    /// scan_with_context 默认实现携带 elapsed_us + engine_name。
    #[test]
    fn scan_with_context_returns_report() {
        let rules = vec![rule("OUT-TEST", r"hello", Severity::Critical)];
        let engine = VectorscanEngine::compile(rules).unwrap();
        let req = ScanRequest {
            bytes: b"say hello world",
            direction: Direction::Outbound,
            protocol: Protocol::Anthropic,
            content_kind: ContentKind::RequestBody,
            tool_name: None,
            source_agent: None,
            caller_exe: None,
        };
        let report = engine.scan_with_context(req).unwrap();
        assert_eq!(report.hits.len(), 1);
        assert_eq!(report.engine_name, "vectorscan");
        assert_eq!(report.rule_count, 1);
    }

    // -------------------------------------------------------------------------
    // v2.0 新增：LayeredEngine 测试
    // -------------------------------------------------------------------------

    /// 系统引擎命中 fail-closed（Critical）规则时，用户引擎不被评估。
    ///
    /// 使用 OUT-01（在 FAIL_CLOSED_RULES 中）模拟 Critical 命中。
    #[test]
    fn layered_engine_critical_hit_blocks_user_engine() {
        // 系统引擎：OUT-01 是 fail-closed
        let system_rules = vec![rule("OUT-01", r"critical_secret", Severity::Critical)];
        let system = VectorscanEngine::compile(system_rules).unwrap();

        // 用户引擎：独立 pattern，但不应被评估
        let user_rules = vec![rule("MY-RULE", r"critical_secret", Severity::High)];
        let user = VectorscanEngine::compile(user_rules).unwrap();

        let layered = LayeredEngine::new(system, Some(user));
        let hits = layered.scan(b"critical_secret leak").unwrap();

        // 只有系统规则 hit，用户规则 hit 被短路
        assert!(
            hits.iter().all(|h| !h.rule_id.starts_with("user:")),
            "用户规则命中不应出现，系统 Critical 短路后应立即返回: {hits:?}"
        );
        assert!(
            hits.iter().any(|h| h.rule_id == "OUT-01"),
            "系统规则 OUT-01 应命中: {hits:?}"
        );
    }

    /// 系统引擎命中非 Critical 规则时，用户引擎被评估并合并。
    #[test]
    fn layered_engine_non_critical_merges_user_hits() {
        // 系统引擎：非 fail-closed ID
        let system_rules = vec![rule("IN-GEN-04", r"system_pattern", Severity::High)];
        let system = VectorscanEngine::compile(system_rules).unwrap();

        // 用户引擎：命中同一输入的另一 pattern
        let user_rules = vec![rule("MY-RULE", r"user_pattern", Severity::Medium)];
        let user = VectorscanEngine::compile(user_rules).unwrap();

        let layered = LayeredEngine::new(system, Some(user));
        let hits = layered
            .scan(b"system_pattern and user_pattern here")
            .unwrap();

        // 系统规则与用户规则均应命中
        assert!(
            hits.iter().any(|h| h.rule_id == "IN-GEN-04"),
            "系统规则应命中: {hits:?}"
        );
        assert!(
            hits.iter().any(|h| h.rule_id == "MY-RULE"),
            "用户规则应命中并合并: {hits:?}"
        );
    }

    /// 无系统命中时仍评估用户规则。
    #[test]
    fn layered_engine_no_system_hit_evaluates_user() {
        let system_rules = vec![rule("SYS-PATTERN", r"nonexistent_xyz", Severity::High)];
        let system = VectorscanEngine::compile(system_rules).unwrap();

        let user_rules = vec![rule("MY-USER-RULE", r"user_only", Severity::Medium)];
        let user = VectorscanEngine::compile(user_rules).unwrap();

        let layered = LayeredEngine::new(system, Some(user));
        let hits = layered.scan(b"user_only content").unwrap();

        assert_eq!(hits.len(), 1, "仅用户规则应命中: {hits:?}");
        assert_eq!(hits[0].rule_id, "MY-USER-RULE");
    }

    /// rule_count 与 compiled_pattern_size_bytes 是系统 + 用户之和。
    #[test]
    fn layered_engine_meta_aggregates() {
        let system_rules = vec![
            rule("SYS-A", r"foo", Severity::High),
            rule("SYS-B", r"bar", Severity::High),
        ];
        let system = VectorscanEngine::compile(system_rules).unwrap();

        let user_rules = vec![rule("MY-RULE", r"baz", Severity::Medium)];
        let user = VectorscanEngine::compile(user_rules).unwrap();

        let layered = LayeredEngine::new(system, Some(user));
        assert_eq!(layered.rule_count(), 3);
        assert_eq!(layered.engine_name(), "layered");
    }

    /// LayeredEngine 无用户规则时退化为纯系统引擎。
    #[test]
    fn layered_engine_no_user_engine() {
        let system_rules = vec![rule("SYS-ONLY", r"target", Severity::High)];
        let system = VectorscanEngine::compile(system_rules).unwrap();
        let layered: LayeredEngine<VectorscanEngine, VectorscanEngine> =
            LayeredEngine::new(system, None);
        let hits = layered.scan(b"target found").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].rule_id, "SYS-ONLY");
        assert_eq!(layered.rule_count(), 1);
    }

    // -------------------------------------------------------------------------
    // v2.1 hot swap 测试（PRD §5.5.5 zero-downtime reload）
    // -------------------------------------------------------------------------

    /// swap_user 能原子替换用户引擎，scan 立即看到新规则。
    ///
    /// 验证步骤：v1 引擎命中 → swap 到 v2 → v2 命中 v1 不命中 → swap 到 None → 纯系统规则。
    #[test]
    fn layered_engine_hot_swap_works() {
        let system_rules = vec![rule("SYS-KEEP", r"system_token", Severity::High)];
        let system = VectorscanEngine::compile(system_rules).unwrap();

        // 用户引擎 v1：匹配 "user_v1_secret"
        let user_v1 =
            VectorscanEngine::compile(vec![rule("USER-V1", r"user_v1_secret", Severity::Medium)])
                .unwrap();

        let layered = LayeredEngine::new(system, Some(user_v1));

        // 初始状态：v1 规则命中
        let hits1 = layered.scan(b"user_v1_secret found").unwrap();
        assert!(
            hits1.iter().any(|h| h.rule_id == "USER-V1"),
            "v1 规则应命中: {hits1:?}"
        );

        // swap 到 v2（匹配 "user_v2_token"）
        let user_v2 =
            VectorscanEngine::compile(vec![rule("USER-V2", r"user_v2_token", Severity::Medium)])
                .unwrap();
        layered.swap_user(Some(user_v2));

        // swap 后：v2 命中，v1 不再命中
        let hits2 = layered.scan(b"user_v2_token appeared").unwrap();
        assert!(
            hits2.iter().any(|h| h.rule_id == "USER-V2"),
            "swap 后 v2 规则应命中: {hits2:?}"
        );
        let hits2_on_v1 = layered.scan(b"user_v1_secret found").unwrap();
        assert!(
            !hits2_on_v1.iter().any(|h| h.rule_id == "USER-V1"),
            "swap 后 v1 规则不应命中: {hits2_on_v1:?}"
        );

        // swap 到 None → 纯系统规则
        layered.swap_user(None::<VectorscanEngine>);
        assert_eq!(
            layered.rule_count(),
            1,
            "swap None 后 rule_count 应等于系统规则数"
        );
        let hits3 = layered.scan(b"user_v2_token appeared").unwrap();
        assert!(
            !hits3.iter().any(|h| h.rule_id.starts_with("USER-")),
            "swap None 后无用户规则命中: {hits3:?}"
        );
        // 系统规则仍正常工作
        let sys_hits = layered.scan(b"system_token here").unwrap();
        assert!(
            sys_hits.iter().any(|h| h.rule_id == "SYS-KEEP"),
            "系统规则始终有效: {sys_hits:?}"
        );
    }

    /// swap_user 期间并发 scan 不阻塞、不 panic（ArcSwap lock-free 保证）。
    #[test]
    fn layered_engine_swap_does_not_block_concurrent_reads() {
        use std::sync::Arc;
        use std::thread;

        let system_rules = vec![rule("SYS-A", r"sys_pattern", Severity::High)];
        let system = VectorscanEngine::compile(system_rules).unwrap();
        let user_init =
            VectorscanEngine::compile(vec![rule("USER-INIT", r"init_data", Severity::Low)])
                .unwrap();
        let layered = Arc::new(LayeredEngine::new(system, Some(user_init)));

        let layered_read = Arc::clone(&layered);
        let layered_swap = Arc::clone(&layered);

        // reader 线程：反复 scan，验证不阻塞不 panic
        let reader = thread::spawn(move || {
            for _ in 0..200 {
                let _ = layered_read
                    .scan(b"sys_pattern init_data swap_data")
                    .unwrap();
            }
        });

        // swapper 线程：交替 swap 不同引擎
        let swapper = thread::spawn(move || {
            for i in 0..10u32 {
                let new_user =
                    VectorscanEngine::compile(vec![rule("USER-SWAP", r"swap_data", Severity::Low)])
                        .unwrap();
                if i % 2 == 0 {
                    layered_swap.swap_user(Some(new_user));
                } else {
                    layered_swap.swap_user(None::<VectorscanEngine>);
                }
            }
        });

        reader.join().expect("reader 线程不应 panic");
        swapper.join().expect("swapper 线程不应 panic");
    }
}
