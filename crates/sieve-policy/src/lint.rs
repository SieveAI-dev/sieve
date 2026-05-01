//! 用户规则 lint 校验（PRD v2.0 §5.5.3 11 类约束）。
//!
//! 分为三类：
//! - **A 类（语义边界）**：防止用户规则越权（5 类）
//! - **B 类（资源上限）**：防止 ReDoS / 文件系统攻击（3+ 类）
//! - **C 类（文件系统）**：由 [`crate::loader`] 负责，本模块不重复
//!
//! 入口：[`lint`] 纯函数，返回所有 [`LintViolation`]。

use crate::loader::{UserRuleEntry, UserRulesFile};
use sieve_rules::critical_lock::FAIL_CLOSED_RULES;
use std::collections::HashSet;
use std::time::Instant;

/// lint 违规类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintKind {
    // --- A. 语义边界 ---
    /// severity 为 "critical" 或 action 为 "block" 或 disposition 为 "hook_terminal"。
    ForbiddenSeverityActionDisposition,
    /// pattern 含 `__` 前缀（系统占位符保留前缀）。
    ForbiddenPatternPrefix,
    /// id 与已有用户规则重复（含禁用规则）。
    DuplicateRuleId,
    /// id 与系统 Critical rule_id 冲突。
    SystemRuleIdConflict,
    /// 入站方向规则尝试 disposition = "auto_redact"（用户不能改写入站 model 输出）。
    InboundAutoRedactForbidden,
    /// allowlist_* 字段试图豁免系统 Critical rule_id。
    AllowlistTargetsSystemCritical,

    // --- B. 资源上限 ---
    /// user.toml 文件大小超过 1MB（由调用方检查后传入参数标志）。
    FileTooLarge,
    /// 总规则条数超过 200 条。
    TooManyRules,
    /// 单个 pattern vectorscan 编译超时（100ms 上限）。
    PatternCompileTimeout,
    /// 单个 pattern 编译后 db size 超过 1MB。
    PatternDbTooLarge,
    /// allowlist_stopwords 中含有 < 4 字节的字符串。
    StopwordTooShort,
    /// keywords 字段为空。
    KeywordsEmpty,
}

/// 单个 lint 违规。
#[derive(Debug, Clone)]
pub struct LintViolation {
    /// 违规的规则 ID（文件级违规用 `"<file>"` 代替）。
    pub rule_id: String,
    /// 违规类型。
    pub kind: LintKind,
    /// 人类可读的违规说明。
    pub message: String,
}

/// 对 `UserRulesFile` 执行全部 11 类 lint 约束，返回所有违规。
///
/// 返回空 Vec 表示通过。**纯函数**，无副作用。
///
/// # 参数
///
/// - `file`: 待校验的用户规则文件
/// - `file_size_bytes`: 调用方读取文件时获得的文件大小（用于 B 类文件大小上限校验）
pub fn lint(file: &UserRulesFile, file_size_bytes: u64) -> Vec<LintViolation> {
    let mut violations: Vec<LintViolation> = Vec::new();

    // B. 文件大小上限 1MB
    if file_size_bytes > 1024 * 1024 {
        violations.push(LintViolation {
            rule_id: "<file>".into(),
            kind: LintKind::FileTooLarge,
            message: format!("user.toml size {} bytes exceeds 1MB limit", file_size_bytes),
        });
    }

    // B. 总规则条数上限 200
    if file.rules.len() > 200 {
        violations.push(LintViolation {
            rule_id: "<file>".into(),
            kind: LintKind::TooManyRules,
            message: format!(
                "user.toml contains {} rules, exceeds limit of 200",
                file.rules.len()
            ),
        });
    }

    // 收集用户规则 ID（用于重复检测）
    let mut seen_ids: HashSet<String> = HashSet::new();

    // 系统 Critical rule_id 集合
    let system_critical: HashSet<&str> = FAIL_CLOSED_RULES.iter().copied().collect();

    for entry in &file.rules {
        lint_entry(entry, &mut seen_ids, &system_critical, &mut violations);
    }

    violations
}

/// 对单条规则执行全部校验。
fn lint_entry(
    entry: &UserRuleEntry,
    seen_ids: &mut HashSet<String>,
    system_critical: &HashSet<&str>,
    violations: &mut Vec<LintViolation>,
) {
    // A-1. 禁止 severity=critical / action=block / disposition=hook_terminal
    let sev = entry.severity.to_lowercase();
    let act = entry.action.to_lowercase();
    let disp = entry.disposition.as_deref().unwrap_or("").to_lowercase();

    if sev == "critical" || act == "block" || disp == "hook_terminal" {
        violations.push(LintViolation {
            rule_id: entry.id.clone(),
            kind: LintKind::ForbiddenSeverityActionDisposition,
            message: format!(
                "rule '{}': severity=critical/action=block/disposition=hook_terminal forbidden \
                 in user rules (PRD §5.5.3-A); got severity={}, action={}, disposition={:?}",
                entry.id, entry.severity, entry.action, entry.disposition
            ),
        });
    }

    // A-2. 禁止 pattern 含 `__` 前缀（系统占位符保留前缀）
    if entry.pattern.contains("__") {
        violations.push(LintViolation {
            rule_id: entry.id.clone(),
            kind: LintKind::ForbiddenPatternPrefix,
            message: format!(
                "rule '{}': pattern contains '__' prefix which is reserved for system \
                 placeholder bypass (PRD §5.5.3-A)",
                entry.id
            ),
        });
    }

    // A-3. id 重复（含禁用规则）
    if !seen_ids.insert(entry.id.clone()) {
        violations.push(LintViolation {
            rule_id: entry.id.clone(),
            kind: LintKind::DuplicateRuleId,
            message: format!("rule '{}': duplicate rule id (PRD §5.5.2.1)", entry.id),
        });
    }

    // A-4. id 与系统 Critical rule_id 冲突
    if system_critical.contains(entry.id.as_str()) {
        violations.push(LintViolation {
            rule_id: entry.id.clone(),
            kind: LintKind::SystemRuleIdConflict,
            message: format!(
                "rule '{}': conflicts with system Critical rule id (PRD §5.5.3-A)",
                entry.id
            ),
        });
    }

    // A-5. 入站方向规则禁止 disposition=auto_redact（用户不能改写入站 model 输出，PRD §9 #11）
    // 用户规则没有明确的方向字段，但 disposition=auto_redact 本意是出站改写，
    // 入站方向下使用 auto_redact 无效且具误导性；PRD 明确禁止用户规则使用 auto_redact。
    if disp == "auto_redact" {
        violations.push(LintViolation {
            rule_id: entry.id.clone(),
            kind: LintKind::InboundAutoRedactForbidden,
            message: format!(
                "rule '{}': disposition=auto_redact forbidden in user rules \
                 (PRD §5.5.3-A: 用户不能改写 model 输出)",
                entry.id
            ),
        });
    }

    // A-6. allowlist_stopwords 试图豁免系统 Critical rule_id
    // 语义：用户 allowlist 仅作用于自己的规则，但若 stopword 与系统 Critical ID 重合
    // 说明用户有意绕过系统规则（LayeredEngine 保证不影响，但 lint 提前拦截）。
    for sw in &entry.allowlist_stopwords {
        if system_critical.contains(sw.as_str()) {
            violations.push(LintViolation {
                rule_id: entry.id.clone(),
                kind: LintKind::AllowlistTargetsSystemCritical,
                message: format!(
                    "rule '{}': allowlist_stopwords contains system Critical rule id '{}' \
                     (PRD §5.5.3-A)",
                    entry.id, sw
                ),
            });
        }
    }

    // B. keywords 非空（强制 keywords 预过滤，PRD §5.5.3-B）
    if entry.keywords.is_empty() {
        violations.push(LintViolation {
            rule_id: entry.id.clone(),
            kind: LintKind::KeywordsEmpty,
            message: format!(
                "rule '{}': keywords field must be non-empty (PRD §5.5.3-B: \
                 强制 keywords 预过滤避免 match-all pattern 拖慢扫描)",
                entry.id
            ),
        });
    }

    // B. allowlist_stopwords 每条 >= 4 字节
    for sw in &entry.allowlist_stopwords {
        if sw.len() < 4 {
            violations.push(LintViolation {
                rule_id: entry.id.clone(),
                kind: LintKind::StopwordTooShort,
                message: format!(
                    "rule '{}': allowlist_stopword '{}' is {} bytes, minimum 4 bytes \
                     (PRD §5.5.3-B: 防止超短停用词污染所有匹配)",
                    entry.id,
                    sw,
                    sw.len()
                ),
            });
        }
    }

    // B. pattern vectorscan 编译时间上限 100ms（及 db size 上限 1MB）
    check_pattern_compile_limits(entry, violations);
}

/// 检查 pattern 编译时间（< 100ms）和 db 大小（< 1MB）。
///
/// 实际编译 vectorscan pattern 做 dryrun，用计时器判断是否超时。
/// db size 当前 vectorscan_rs 不暴露 API，用编译时间作为代理指标。
fn check_pattern_compile_limits(entry: &UserRuleEntry, violations: &mut Vec<LintViolation>) {
    use sieve_rules::engine::VectorscanEngine;
    use sieve_rules::manifest::{Action, DefaultOnTimeout, RuleEntry, Severity};

    let dummy_rule = RuleEntry {
        id: entry.id.clone(),
        description: entry.description.clone(),
        pattern: entry.pattern.clone(),
        severity: Severity::Medium,
        action: Action::Warn,
        entropy_min: None,
        keywords: entry.keywords.clone(),
        allowlist_regexes: vec![],
        allowlist_stopwords: entry.allowlist_stopwords.clone(),
        disposition: None,
        timeout_seconds: None,
        default_on_timeout: DefaultOnTimeout::Allow,
    };

    let start = Instant::now();
    let compile_result = VectorscanEngine::compile(vec![dummy_rule]);
    let elapsed = start.elapsed();

    match compile_result {
        Err(e) => {
            // 编译失败视为违规（防止非法 pattern 进入引擎）
            violations.push(LintViolation {
                rule_id: entry.id.clone(),
                kind: LintKind::PatternCompileTimeout,
                message: format!(
                    "rule '{}': pattern failed to compile: {} (PRD §5.5.3-B)",
                    entry.id, e
                ),
            });
        }
        Ok(_engine) => {
            // 编译时间上限 100ms
            if elapsed.as_millis() > 100 {
                violations.push(LintViolation {
                    rule_id: entry.id.clone(),
                    kind: LintKind::PatternCompileTimeout,
                    message: format!(
                        "rule '{}': pattern compile took {}ms, exceeds 100ms limit \
                         (PRD §5.5.3-B: ReDoS 防御)",
                        entry.id,
                        elapsed.as_millis()
                    ),
                });
            }
            // db size 暂无 vectorscan_rs API，此处记录 TODO（Week 6 跟进）
            // TODO(Week 6): 等 vectorscan_rs 暴露 hs_database_size() 后补充 1MB 上限校验
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::UserRulesFile;
    use chrono::Utc;

    fn make_file(rules: Vec<UserRuleEntry>) -> UserRulesFile {
        UserRulesFile {
            schema_version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            rules,
        }
    }

    fn valid_rule(id: &str) -> UserRuleEntry {
        UserRuleEntry {
            id: id.into(),
            description: "test rule".into(),
            pattern: "secret_pattern".into(),
            severity: "high".into(),
            action: "warn".into(),
            keywords: vec!["secret".into()],
            allowlist_stopwords: vec![],
            disposition: None,
            enabled: true,
            added_at: Utc::now(),
            added_by: "manual".into(),
        }
    }

    #[test]
    fn valid_rule_no_violations() {
        let file = make_file(vec![valid_rule("MY-RULE")]);
        let violations = lint(&file, 100);
        assert!(
            violations.is_empty(),
            "expected no violations: {violations:?}"
        );
    }

    #[test]
    fn rejects_critical_severity() {
        let mut r = valid_rule("MY-RULE");
        r.severity = "critical".into();
        let file = make_file(vec![r]);
        let v = lint(&file, 100);
        assert!(
            v.iter()
                .any(|x| x.kind == LintKind::ForbiddenSeverityActionDisposition),
            "expected ForbiddenSeverityActionDisposition: {v:?}"
        );
    }

    #[test]
    fn rejects_block_action() {
        let mut r = valid_rule("MY-RULE");
        r.action = "block".into();
        let file = make_file(vec![r]);
        let v = lint(&file, 100);
        assert!(
            v.iter()
                .any(|x| x.kind == LintKind::ForbiddenSeverityActionDisposition),
            "expected ForbiddenSeverityActionDisposition: {v:?}"
        );
    }

    #[test]
    fn rejects_hook_terminal_disposition() {
        let mut r = valid_rule("MY-RULE");
        r.disposition = Some("hook_terminal".into());
        let file = make_file(vec![r]);
        let v = lint(&file, 100);
        assert!(
            v.iter()
                .any(|x| x.kind == LintKind::ForbiddenSeverityActionDisposition),
            "expected ForbiddenSeverityActionDisposition: {v:?}"
        );
    }

    #[test]
    fn rejects_auto_redact_disposition() {
        let mut r = valid_rule("MY-RULE");
        r.disposition = Some("auto_redact".into());
        let file = make_file(vec![r]);
        let v = lint(&file, 100);
        assert!(
            v.iter()
                .any(|x| x.kind == LintKind::InboundAutoRedactForbidden),
            "expected InboundAutoRedactForbidden: {v:?}"
        );
    }

    #[test]
    fn rejects_pattern_with_double_underscore() {
        let mut r = valid_rule("MY-RULE");
        r.pattern = "__placeholder__".into();
        let file = make_file(vec![r]);
        let v = lint(&file, 100);
        assert!(
            v.iter().any(|x| x.kind == LintKind::ForbiddenPatternPrefix),
            "expected ForbiddenPatternPrefix: {v:?}"
        );
    }

    #[test]
    fn rejects_duplicate_rule_id() {
        let file = make_file(vec![valid_rule("MY-RULE"), valid_rule("MY-RULE")]);
        let v = lint(&file, 100);
        assert!(
            v.iter().any(|x| x.kind == LintKind::DuplicateRuleId),
            "expected DuplicateRuleId: {v:?}"
        );
    }

    #[test]
    fn rejects_system_critical_id_conflict() {
        // OUT-01 在 FAIL_CLOSED_RULES 中
        let file = make_file(vec![valid_rule("OUT-01")]);
        let v = lint(&file, 100);
        assert!(
            v.iter().any(|x| x.kind == LintKind::SystemRuleIdConflict),
            "expected SystemRuleIdConflict: {v:?}"
        );
    }

    #[test]
    fn rejects_empty_keywords() {
        let mut r = valid_rule("MY-RULE");
        r.keywords = vec![];
        let file = make_file(vec![r]);
        let v = lint(&file, 100);
        assert!(
            v.iter().any(|x| x.kind == LintKind::KeywordsEmpty),
            "expected KeywordsEmpty: {v:?}"
        );
    }

    #[test]
    fn rejects_short_stopword() {
        let mut r = valid_rule("MY-RULE");
        r.allowlist_stopwords = vec!["ok".into()]; // 2 bytes < 4
        let file = make_file(vec![r]);
        let v = lint(&file, 100);
        assert!(
            v.iter().any(|x| x.kind == LintKind::StopwordTooShort),
            "expected StopwordTooShort: {v:?}"
        );
    }

    #[test]
    fn rejects_allowlist_targeting_system_critical() {
        let mut r = valid_rule("MY-RULE");
        // OUT-01 是系统 Critical rule_id，不应出现在用户 allowlist 中
        r.allowlist_stopwords = vec!["OUT-01".into()];
        let file = make_file(vec![r]);
        let v = lint(&file, 100);
        assert!(
            v.iter()
                .any(|x| x.kind == LintKind::AllowlistTargetsSystemCritical),
            "expected AllowlistTargetsSystemCritical: {v:?}"
        );
    }

    #[test]
    fn rejects_too_many_rules() {
        let rules: Vec<_> = (0..201)
            .map(|i| valid_rule(&format!("MY-RULE-{:03}", i)))
            .collect();
        let file = make_file(rules);
        let v = lint(&file, 100);
        assert!(
            v.iter().any(|x| x.kind == LintKind::TooManyRules),
            "expected TooManyRules: {v:?}"
        );
    }

    #[test]
    fn rejects_file_too_large() {
        let file = make_file(vec![valid_rule("MY-RULE")]);
        // 传入超过 1MB 的文件大小
        let v = lint(&file, 1024 * 1024 + 1);
        assert!(
            v.iter().any(|x| x.kind == LintKind::FileTooLarge),
            "expected FileTooLarge: {v:?}"
        );
    }

    #[test]
    fn multiple_violations_collected() {
        let mut r = valid_rule("MY-RULE");
        r.severity = "critical".into();
        r.keywords = vec![];
        let file = make_file(vec![r]);
        let v = lint(&file, 100);
        // 至少有两个违规
        assert!(v.len() >= 2, "expected multiple violations: {v:?}");
    }
}
