//! 灰名单端到端集成测试（PRD v2.0 §5.4.2）。
//!
//! 测试范围：
//! 1. compute_fingerprint 对相同输入产生相同摘要（确定性）
//! 2. 灰名单 add_entry / lookup 端到端 roundtrip
//! 3. Critical 规则被 add_entry 拒绝（CriticalRuleNotGraylistable）
//! 4. fingerprint 被篡改后 lookup 返回 FingerprintMismatch 错误（而非 Ok）
//! 5. allow_remember 计算：非 Critical 规则 = true，Critical 规则 = false
//!
//! .cursorrules §3.2：测试代码允许使用 .unwrap()。

use sieve_policy::graylist::{
    add_entry, compute_fingerprint, lookup, FingerprintInputs, GraylistEntry,
};
use tempfile::TempDir;

// ─── 辅助函数 ─────────────────────────────────────────────────────────────────

fn make_inputs(rule_id: &str) -> FingerprintInputs {
    FingerprintInputs {
        rule_id: rule_id.into(),
        matched_canonical: "0xdeadbeef".into(),
        tool_name: "Bash".into(),
        protocol: "anthropic".into(),
        content_kind: "tool_use_input".into(),
        source_agent: "claude-code".into(),
    }
}

fn make_entry(rule_id: &str) -> GraylistEntry {
    let inputs = make_inputs(rule_id);
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
        added_at: chrono::Utc::now().timestamp_millis(),
        added_by: "gui_user_decision".into(),
        context_hint: Some("测试备注".into()),
        match_count_since: 0,
        audit_event_id: "test-audit-uuid".into(),
    }
}

// ─── Test 1: fingerprint 确定性 ───────────────────────────────────────────────

/// 同一输入 compute_fingerprint 两次结果必须相同（PRD §5.4.2 fingerprint 设计）。
#[test]
fn fingerprint_is_deterministic() {
    let inputs = make_inputs("IN-GEN-04");
    let fp1 = compute_fingerprint(&inputs);
    let fp2 = compute_fingerprint(&inputs);
    assert_eq!(fp1, fp2, "fingerprint 必须是确定性的");
    assert_eq!(fp1.len(), 64, "sha256 hex 应为 64 字符");
}

/// 不同 rule_id 产生不同 fingerprint。
#[test]
fn fingerprint_differs_by_rule_id() {
    let fp1 = compute_fingerprint(&make_inputs("IN-GEN-04"));
    let fp2 = compute_fingerprint(&make_inputs("IN-GEN-05"));
    assert_ne!(fp1, fp2, "不同 rule_id 应产生不同 fingerprint");
}

/// 不同 matched_canonical 产生不同 fingerprint。
#[test]
fn fingerprint_differs_by_canonical() {
    let mut i1 = make_inputs("IN-GEN-04");
    let mut i2 = make_inputs("IN-GEN-04");
    i1.matched_canonical = "addr_a".into();
    i2.matched_canonical = "addr_b".into();
    assert_ne!(
        compute_fingerprint(&i1),
        compute_fingerprint(&i2),
        "不同 matched_canonical 应产生不同 fingerprint"
    );
}

// ─── Test 2: add_entry / lookup roundtrip ────────────────────────────────────

/// 写入灰名单后可以通过 fingerprint lookup 读回，数据完整。
#[test]
fn add_and_lookup_roundtrip() {
    let tmp = TempDir::new().unwrap();
    // IN-GEN-04 不在 FAIL_CLOSED_RULES 中，可以加入灰名单
    let entry = make_entry("IN-GEN-04");
    let fp = entry.fingerprint.clone();

    add_entry(tmp.path(), entry).unwrap();

    let found = lookup(tmp.path(), &fp).unwrap();
    assert!(found.is_some(), "add 后 lookup 应能找到条目");

    let e = found.unwrap();
    assert_eq!(e.rule_id, "IN-GEN-04");
    assert_eq!(e.decision, "allow");
    assert_eq!(e.context_hint.as_deref(), Some("测试备注"));
    assert_eq!(e.schema_version, 1);
}

/// lookup 不存在的 fingerprint 返回 Ok(None)，不报错。
#[test]
fn lookup_missing_returns_none() {
    let tmp = TempDir::new().unwrap();
    // 64 字符全零 fingerprint 不会存在
    let fp = "a".repeat(64);
    let result = lookup(tmp.path(), &fp).unwrap();
    assert!(result.is_none(), "不存在的 fingerprint 应返回 None");
}

/// 写入后再查，fingerprint 校验一致。
#[test]
fn fingerprint_consistency_after_write() {
    let tmp = TempDir::new().unwrap();
    let inputs = make_inputs("MY-CUSTOM-RULE");
    let fp = compute_fingerprint(&inputs);

    let entry = GraylistEntry {
        schema_version: 1,
        fingerprint_version: 1,
        rule_id: "MY-CUSTOM-RULE".into(),
        rule_version: "v2.0".into(),
        fingerprint: fp.clone(),
        fingerprint_inputs: inputs,
        decision: "allow".into(),
        expires_at: None,
        added_at: chrono::Utc::now().timestamp_millis(),
        added_by: "test".into(),
        context_hint: None,
        match_count_since: 0,
        audit_event_id: "test-uuid".into(),
    };

    add_entry(tmp.path(), entry).unwrap();

    let found = lookup(tmp.path(), &fp).unwrap().expect("应能找到");
    // fingerprint 字段与文件名一致
    assert_eq!(found.fingerprint, fp);
}

// ─── Test 3: Critical 规则被拒绝 ─────────────────────────────────────────────

/// OUT-01 是 fail-closed Critical 规则，add_entry 应返回 CriticalRuleNotGraylistable。
#[test]
fn critical_rule_cannot_be_added_to_graylist() {
    let tmp = TempDir::new().unwrap();
    let entry = make_entry("OUT-01");
    let err = add_entry(tmp.path(), entry).unwrap_err();
    assert!(
        matches!(
            err,
            sieve_policy::error::PolicyError::CriticalRuleNotGraylistable { .. }
        ),
        "OUT-01 应被 CriticalRuleNotGraylistable 拒绝，实际: {err:?}"
    );
}

/// IN-CR-05-EVM（签名工具）同样不可灰名单化。
#[test]
fn in_cr_05_evm_cannot_be_graylisted() {
    let tmp = TempDir::new().unwrap();
    let entry = make_entry("IN-CR-05-EVM");
    let err = add_entry(tmp.path(), entry).unwrap_err();
    assert!(
        matches!(
            err,
            sieve_policy::error::PolicyError::CriticalRuleNotGraylistable { .. }
        ),
        "IN-CR-05-EVM 应被 CriticalRuleNotGraylistable 拒绝，实际: {err:?}"
    );
}

// ─── Test 4: fingerprint 篡改检测 ────────────────────────────────────────────

/// 手动修改 graylist JSON 文件后，lookup 应返回 FingerprintMismatch 错误。
#[test]
fn tampered_fingerprint_detected_on_lookup() {
    let tmp = TempDir::new().unwrap();
    let entry = make_entry("MY-CUSTOM-RULE");
    let fp = entry.fingerprint.clone();
    let original_canonical = entry.fingerprint_inputs.matched_canonical.clone();

    add_entry(tmp.path(), entry).unwrap();

    // 直接修改文件中的 matched_canonical，破坏 fingerprint 一致性
    let path = tmp.path().join(format!("{fp}.json"));
    let content = std::fs::read_to_string(&path).unwrap();
    let tampered = content.replace(&original_canonical, "tampered_value_xyz");
    std::fs::write(&path, tampered).unwrap();

    let err = lookup(tmp.path(), &fp).unwrap_err();
    assert!(
        matches!(
            err,
            sieve_policy::error::PolicyError::FingerprintMismatch { .. }
        ),
        "篡改后 lookup 应返回 FingerprintMismatch，实际: {err:?}"
    );
}

// ─── Test 5: allow_remember 计算 ─────────────────────────────────────────────

/// 非 Critical 规则（不在 FAIL_CLOSED_RULES）allow_remember 应为 true。
#[test]
fn allow_remember_true_for_non_critical_rule() {
    // IN-GEN-04 不在 FAIL_CLOSED_RULES 中
    let is_fail_closed = sieve_rules::critical_lock::is_fail_closed("IN-GEN-04");
    assert!(!is_fail_closed, "IN-GEN-04 不应在 fail-closed 名单中");
    // allow_remember = !is_fail_closed
    assert!(
        !is_fail_closed,
        "非 Critical 规则 allow_remember 应为 true（即 is_fail_closed=false）"
    );
}

/// Critical 规则（在 FAIL_CLOSED_RULES）allow_remember 应为 false。
#[test]
fn allow_remember_false_for_critical_rule() {
    // OUT-01、IN-CR-05-EVM、IN-CR-02 都是 Critical fail-closed 规则
    for rule_id in &["OUT-01", "IN-CR-05-EVM", "IN-CR-02"] {
        assert!(
            sieve_rules::critical_lock::is_fail_closed(rule_id),
            "{rule_id} 应在 fail-closed 名单中（allow_remember 必须 false）"
        );
    }
}

/// 多条 detection 时：有任一 Critical 规则 → 整批 allow_remember = false。
#[test]
fn allow_remember_false_when_any_critical_in_batch() {
    let rule_ids = ["IN-GEN-04", "OUT-01"]; // 第二条是 Critical
    let any_fail_closed = rule_ids
        .iter()
        .any(|id| sieve_rules::critical_lock::is_fail_closed(id));
    assert!(
        any_fail_closed,
        "批次中含 Critical 规则时，allow_remember 应为 false"
    );
}

/// 批次中全部非 Critical 规则 → allow_remember = true。
#[test]
fn allow_remember_true_when_no_critical_in_batch() {
    let rule_ids = ["IN-GEN-04", "MY-CUSTOM-RULE"];
    let any_fail_closed = rule_ids
        .iter()
        .any(|id| sieve_rules::critical_lock::is_fail_closed(id));
    assert!(
        !any_fail_closed,
        "批次中无 Critical 规则时，allow_remember 应为 true"
    );
}

// ─── Test 6: 文件权限 ────────────────────────────────────────────────────────

/// graylist 文件权限应为 0600（Unix only）。
#[cfg(unix)]
#[test]
fn graylist_entry_file_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new().unwrap();
    let entry = make_entry("IN-GEN-04");
    let fp = entry.fingerprint.clone();
    add_entry(tmp.path(), entry).unwrap();

    let path = tmp.path().join(format!("{fp}.json"));
    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "灰名单文件权限应为 0600");
}
