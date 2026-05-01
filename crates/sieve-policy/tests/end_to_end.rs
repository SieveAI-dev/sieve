//! sieve-policy 端到端集成测试（PRD v2.0 §5.4 §5.5 §6.3）。
//!
//! 覆盖完整链路：加载 user.toml → lint → 编译 UserEngine → 扫描 → graylist 全链路。

use chrono::Utc;
use sieve_policy::{
    engine::UserEngine,
    error::PolicyError,
    graylist::{
        add_entry, compute_fingerprint, lookup, remove_entry, FingerprintInputs, GraylistEntry,
    },
    lint::lint,
    loader::{load_user_rules, RuleDirection, UserRuleEntry, UserRulesFile},
};
use sieve_rules::engine::MatchEngine;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// ───────────────────────── 辅助函数 ─────────────────────────

fn tmp_rules_file(tmp: &TempDir, content: &str) -> PathBuf {
    let dir = tmp.path().join("rules");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("user.toml");
    std::fs::write(&path, content).unwrap();
    #[cfg(unix)]
    {
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    path
}

fn make_valid_toml(id: &str, pattern: &str) -> String {
    format!(
        r#"
schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"

[[rules]]
id = "{id}"
description = "End to end test rule"
pattern = "{pattern}"
severity = "high"
action = "warn"
keywords = ["keyword"]
allowlist_stopwords = ["allowed_context"]
disposition = "status_bar"
enabled = true
added_at = "2026-05-01T00:00:00Z"
added_by = "manual"
"#
    )
}

fn make_graylist_entry(rule_id: &str) -> GraylistEntry {
    let inputs = FingerprintInputs {
        rule_id: rule_id.into(),
        matched_canonical: "test_canonical_match".into(),
        tool_name: "Bash".into(),
        protocol: "anthropic".into(),
        content_kind: "tool_use_input".into(),
        source_agent: "claude-code".into(),
    };
    let fp = compute_fingerprint(&inputs);
    GraylistEntry {
        schema_version: 1,
        fingerprint_version: 1,
        rule_id: rule_id.into(),
        rule_version: "v2.0".into(),
        fingerprint: fp,
        fingerprint_inputs: inputs,
        decision: "allow".into(),
        expires_at: None,
        added_at: Utc::now().timestamp_millis(),
        added_by: "gui_user_decision".into(),
        context_hint: Some("e2e test".into()),
        match_count_since: 0,
        audit_event_id: "e2e-test-uuid-0001".into(),
    }
}

// ───────────────────────── 测试 ─────────────────────────

/// 完整链路：load → lint → UserEngine::compile → scan → graylist add/lookup/remove。
#[test]
fn full_pipeline_load_lint_compile_scan_graylist() {
    let tmp = TempDir::new().unwrap();

    // 1. 加载 user.toml
    let path = tmp_rules_file(&tmp, &make_valid_toml("E2E-RULE", "e2e_secret_pattern"));
    let file = load_user_rules(&path).unwrap();
    assert_eq!(file.rules.len(), 1, "should have 1 rule");

    // 2. lint 通过
    let toml_size = std::fs::metadata(&path).unwrap().len();
    let violations = lint(&file, toml_size);
    assert!(
        violations.is_empty(),
        "valid rule should pass lint: {violations:?}"
    );

    // 3. 编译 UserEngine
    let engine = UserEngine::compile(file.rules).unwrap();
    assert_eq!(engine.rule_count(), 1);
    assert_eq!(engine.engine_name(), "user-vectorscan");

    // 4. 扫描：命中
    let hits = engine
        .scan(b"found e2e_secret_pattern here keyword")
        .unwrap();
    assert_eq!(hits.len(), 1, "should have 1 hit: {hits:?}");
    assert_eq!(hits[0].rule_id, "user:E2E-RULE");

    // 5. 扫描：不命中
    let no_hits = engine.scan(b"no match here at all").unwrap();
    assert!(no_hits.is_empty(), "should not match: {no_hits:?}");

    // 6. 灰名单 add → lookup → remove
    let decisions_dir = tmp.path().join("decisions");
    let entry = make_graylist_entry("E2E-RULE");
    let fp = entry.fingerprint.clone();

    add_entry(&decisions_dir, entry).unwrap();
    let found = lookup(&decisions_dir, &fp).unwrap();
    assert!(found.is_some(), "graylist entry should be found");
    assert_eq!(found.unwrap().rule_id, "E2E-RULE");

    assert!(remove_entry(&decisions_dir, &fp).unwrap());
    assert!(lookup(&decisions_dir, &fp).unwrap().is_none());
}

/// lint 拒绝非法规则，UserEngine::compile 不会接到危险规则。
#[test]
fn lint_blocks_forbidden_rules() {
    let rule = UserRuleEntry {
        id: "FORBIDDEN".into(),
        description: "should be blocked".into(),
        pattern: "some_pattern".into(),
        severity: "critical".into(), // 禁止
        action: "block".into(),      // 禁止
        keywords: vec!["kw".into()],
        allowlist_stopwords: vec![],
        disposition: None,
        direction: RuleDirection::Both,
        enabled: true,
        added_at: Utc::now(),
        added_by: "manual".into(),
    };
    let file = UserRulesFile {
        schema_version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        rules: vec![rule],
    };
    let violations = lint(&file, 100);
    assert!(
        !violations.is_empty(),
        "critical+block rule should fail lint"
    );
}

/// 文件不存在时返回空 UserRulesFile（daemon 正常启动，PRD §5.5.2.1）。
#[test]
fn missing_file_returns_empty_ruleset() {
    let path = PathBuf::from("/tmp/sieve_e2e_nonexistent_user_rules.toml");
    let file = load_user_rules(&path).unwrap();
    assert!(
        file.rules.is_empty(),
        "missing file should return empty rules"
    );
}

/// Critical 规则不可加入灰名单（PRD §5.4.2 Critical 锁）。
#[test]
fn critical_rule_graylist_blocked() {
    let tmp = TempDir::new().unwrap();
    let decisions = tmp.path().join("decisions");

    // OUT-01 在 FAIL_CLOSED_RULES 中
    let entry = make_graylist_entry("OUT-01");
    let err = add_entry(&decisions, entry).unwrap_err();
    assert!(
        matches!(err, PolicyError::CriticalRuleNotGraylistable { .. }),
        "OUT-01 should not be graylisted: {err:?}"
    );
}

/// 灰名单文件被篡改后 lookup 返回 FingerprintMismatch。
#[test]
fn tampered_graylist_detected() {
    let tmp = TempDir::new().unwrap();
    let decisions = tmp.path().join("decisions");

    let entry = make_graylist_entry("MY-USER-RULE");
    let fp = entry.fingerprint.clone();
    add_entry(&decisions, entry).unwrap();

    // 篡改文件内容
    let path = decisions.join(format!("{fp}.json"));
    let content = std::fs::read_to_string(&path).unwrap();
    let tampered = content.replace("test_canonical_match", "evil_canonical_match");
    std::fs::write(&path, tampered).unwrap();

    let err = lookup(&decisions, &fp).unwrap_err();
    assert!(
        matches!(err, PolicyError::FingerprintMismatch { .. }),
        "tampered file should return FingerprintMismatch: {err:?}"
    );
}

/// direction 字段按方向过滤：outbound 规则不进入 inbound 引擎（PRD v2.0 §5.5）。
#[test]
fn direction_field_filters_rules_correctly() {
    use sieve_policy::engine::UserEngine;

    let outbound_rule = UserRuleEntry {
        id: "OUTBOUND-ONLY".into(),
        description: "only scans outbound".into(),
        pattern: "outbound_secret".into(),
        severity: "high".into(),
        action: "warn".into(),
        keywords: vec!["outbound".into()],
        allowlist_stopwords: vec![],
        disposition: None,
        direction: RuleDirection::Outbound,
        enabled: true,
        added_at: Utc::now(),
        added_by: "manual".into(),
    };
    let inbound_rule = UserRuleEntry {
        id: "INBOUND-ONLY".into(),
        description: "only scans inbound".into(),
        pattern: "inbound_secret".into(),
        severity: "medium".into(),
        action: "warn".into(),
        keywords: vec!["inbound".into()],
        allowlist_stopwords: vec![],
        disposition: None,
        direction: RuleDirection::Inbound,
        enabled: true,
        added_at: Utc::now(),
        added_by: "manual".into(),
    };
    let both_rule = UserRuleEntry {
        id: "BOTH-RULE".into(),
        description: "scans both sides".into(),
        pattern: "both_secret".into(),
        severity: "low".into(),
        action: "mark".into(),
        keywords: vec!["both".into()],
        allowlist_stopwords: vec![],
        disposition: None,
        direction: RuleDirection::Both,
        enabled: true,
        added_at: Utc::now(),
        added_by: "manual".into(),
    };
    let all_rules = vec![
        outbound_rule.clone(),
        inbound_rule.clone(),
        both_rule.clone(),
    ];

    // 出站引擎：只应编译 Outbound + Both（2 条）
    let outbound_engine =
        UserEngine::compile_for_direction(all_rules.clone(), RuleDirection::Outbound).unwrap();
    assert_eq!(
        outbound_engine.rule_count(),
        2,
        "出站引擎应有 2 条规则（Outbound + Both）"
    );
    // 出站 pattern 命中
    let hits = outbound_engine
        .scan(b"outbound_secret here outbound")
        .unwrap();
    assert_eq!(hits.len(), 1, "出站引擎应命中 OUTBOUND-ONLY");
    assert_eq!(hits[0].rule_id, "user:OUTBOUND-ONLY");
    // 入站 pattern 不在出站引擎中
    let hits = outbound_engine
        .scan(b"inbound_secret here inbound")
        .unwrap();
    assert!(hits.is_empty(), "出站引擎不应命中 INBOUND-ONLY：{hits:?}");

    // 入站引擎：只应编译 Inbound + Both（2 条）
    let inbound_engine =
        UserEngine::compile_for_direction(all_rules.clone(), RuleDirection::Inbound).unwrap();
    assert_eq!(
        inbound_engine.rule_count(),
        2,
        "入站引擎应有 2 条规则（Inbound + Both）"
    );
    // 入站 pattern 命中
    let hits = inbound_engine.scan(b"inbound_secret here inbound").unwrap();
    assert_eq!(hits.len(), 1, "入站引擎应命中 INBOUND-ONLY");
    assert_eq!(hits[0].rule_id, "user:INBOUND-ONLY");
    // 出站 pattern 不在入站引擎中
    let hits = inbound_engine
        .scan(b"outbound_secret here outbound")
        .unwrap();
    assert!(hits.is_empty(), "入站引擎不应命中 OUTBOUND-ONLY：{hits:?}");

    // Both 规则两侧都能命中
    let hits = outbound_engine.scan(b"both_secret here both").unwrap();
    assert_eq!(hits.len(), 1, "出站引擎应命中 BOTH-RULE");
    let hits = inbound_engine.scan(b"both_secret here both").unwrap();
    assert_eq!(hits.len(), 1, "入站引擎应命中 BOTH-RULE");
}

/// 旧 user.toml（无 direction 字段）默认 Both，出站入站两侧都能命中（向后兼容，PRD §5.5）。
#[test]
fn legacy_toml_without_direction_defaults_to_both() {
    // 旧格式 TOML：无 direction 字段
    let legacy_toml = r#"
schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"

[[rules]]
id = "LEGACY-RULE"
description = "legacy rule without direction field"
pattern = "legacy_pattern"
severity = "high"
action = "warn"
keywords = ["legacy"]
enabled = true
added_at = "2026-05-01T00:00:00Z"
added_by = "manual"
"#;
    let tmp = TempDir::new().unwrap();
    let path = tmp_rules_file(&tmp, legacy_toml);
    let file = load_user_rules(&path).unwrap();

    assert_eq!(file.rules.len(), 1);
    // 无 direction 字段 → 默认 Both（向后兼容）
    assert_eq!(
        file.rules[0].direction,
        RuleDirection::Both,
        "旧格式规则应默认 direction=Both"
    );

    // 出站引擎能编译并命中
    let outbound_engine =
        UserEngine::compile_for_direction(file.rules.clone(), RuleDirection::Outbound).unwrap();
    let hits = outbound_engine.scan(b"legacy_pattern here legacy").unwrap();
    assert_eq!(hits.len(), 1, "出站引擎应命中旧格式规则");

    // 入站引擎能编译并命中
    let inbound_engine =
        UserEngine::compile_for_direction(file.rules.clone(), RuleDirection::Inbound).unwrap();
    let hits = inbound_engine.scan(b"legacy_pattern here legacy").unwrap();
    assert_eq!(hits.len(), 1, "入站引擎应命中旧格式规则");
}

/// 多条规则中禁用规则不参与扫描。
#[test]
fn disabled_rules_excluded_from_engine() {
    let rules = vec![
        UserRuleEntry {
            id: "ENABLED-RULE".into(),
            description: "enabled".into(),
            pattern: "enabled_pattern".into(),
            severity: "high".into(),
            action: "warn".into(),
            keywords: vec!["enabled".into()],
            allowlist_stopwords: vec![],
            disposition: None,
            direction: RuleDirection::Both,
            enabled: true,
            added_at: Utc::now(),
            added_by: "manual".into(),
        },
        UserRuleEntry {
            id: "DISABLED-RULE".into(),
            description: "disabled".into(),
            pattern: "disabled_pattern".into(),
            severity: "medium".into(),
            action: "mark".into(),
            keywords: vec!["disabled".into()],
            allowlist_stopwords: vec![],
            disposition: None,
            direction: RuleDirection::Both,
            enabled: false,
            added_at: Utc::now(),
            added_by: "manual".into(),
        },
    ];

    let engine = UserEngine::compile(rules).unwrap();
    assert_eq!(
        engine.rule_count(),
        1,
        "only enabled rule should be compiled"
    );

    // disabled_pattern 不应命中
    let hits = engine.scan(b"disabled_pattern here").unwrap();
    assert!(hits.is_empty(), "disabled rule should not match: {hits:?}");

    // enabled_pattern 应命中
    let hits = engine.scan(b"enabled_pattern here").unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].rule_id, "user:ENABLED-RULE");
}
