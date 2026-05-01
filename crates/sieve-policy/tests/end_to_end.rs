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
    loader::{load_user_rules, UserRuleEntry, UserRulesFile},
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
