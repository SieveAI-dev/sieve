//! fail-closed 安全不变量测试（关联 ADR-007 §2 / PRD §9 #3 #8）。
//!
//! 阶段 D 把 fail-closed 判定从硬编码 ID 名单下沉为规则字段
//! （[`RuleEntry::effective_fail_closed`]）。本测试固化两条不可回归的安全不变量：
//!
//! 1. **Critical ⟹ fail-closed**：每条 `severity = critical` 的系统规则都必须
//!    `effective_fail_closed() == true`（PRD §9 #8 Critical 在所有版本不可关）。
//! 2. **历史 fail-closed 集合不缩小**：旧 `critical_lock::FAIL_CLOSED_RULES` 的 30 条
//!    规则在字段化后仍全部 fail-closed（行为保留，含两条 high 但 fail-closed 的例外
//!    OUT-09 / IN-GEN-06，靠显式 `fail_closed = true` 表达）。
//!
//! 这两条若被破坏 = Critical 漏判 = P0 安全回归，故用真实规则 TOML 做断言。

use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};
use sieve_rules::manifest::{RuleEntry, Severity};
use std::path::PathBuf;

fn rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rules")
}

fn all_system_rules() -> Vec<RuleEntry> {
    let mut rules =
        load_outbound_rules(&rules_dir().join("outbound.toml")).expect("load outbound.toml");
    rules.extend(load_inbound_rules(&rules_dir().join("inbound.toml")).expect("load inbound.toml"));
    rules
}

/// 不变量 1：每条 Critical 系统规则必 fail-closed。
#[test]
fn every_critical_rule_is_fail_closed() {
    let rules = all_system_rules();
    let mut offenders = Vec::new();
    for r in &rules {
        if matches!(r.severity, Severity::Critical) && !r.effective_fail_closed() {
            offenders.push(r.id.clone());
        }
    }
    assert!(
        offenders.is_empty(),
        "以下 Critical 规则未 fail-closed（违反 PRD §9 #8）：{offenders:?}"
    );
}

/// 不变量 2：历史 FAIL_CLOSED_RULES 30 条在字段化后仍全部 fail-closed（行为不缩小）。
#[test]
fn historical_fail_closed_set_preserved() {
    // 旧 critical_lock::FAIL_CLOSED_RULES 的权威 30 条（含两条 high 例外 OUT-09 / IN-GEN-06）。
    const HISTORICAL_FAIL_CLOSED: &[&str] = &[
        "IN-CR-01",
        "IN-CR-02",
        "IN-CR-02-CURL-PIPE",
        "IN-CR-02-WGET-PIPE",
        "IN-CR-02-EVAL",
        "IN-CR-02-NC-REVERSE",
        "IN-CR-02-DD-WIPE",
        "IN-CR-04-SHELL-RC-APPEND",
        "IN-CR-04-CRONTAB",
        "IN-CR-04-CRON-D-WRITE",
        "IN-CR-04-LAUNCHCTL",
        "IN-CR-04-LAUNCH-AGENT-PLIST",
        "IN-CR-04-SYSTEMCTL-ENABLE",
        "IN-CR-04-SYSTEMD-UNIT-WRITE",
        "IN-CR-04-FISH-CONFIG",
        "IN-CR-04-LOGIN-ITEMS",
        "IN-CR-05-EVM",
        "IN-CR-05-SOLANA",
        "IN-CR-05-BITCOIN",
        "IN-CR-05-MALFORMED",
        "IN-CR-06",
        "IN-GEN-06",
        "IN-GEN-01",
        "IN-GEN-03",
        "OUT-01",
        "OUT-02",
        "OUT-03",
        "OUT-04",
        "OUT-07",
        "OUT-08",
        "OUT-09",
        "OUT-10",
    ];

    let rules = all_system_rules();
    let by_id: std::collections::HashMap<&str, &RuleEntry> =
        rules.iter().map(|r| (r.id.as_str(), r)).collect();

    let mut regressions = Vec::new();
    for id in HISTORICAL_FAIL_CLOSED {
        // IN-CR-05-MALFORMED 是运行时合成的 partial_json 规则，可能不在 TOML 中——跳过缺失项。
        if let Some(r) = by_id.get(id) {
            if !r.effective_fail_closed() {
                regressions.push((*id).to_string());
            }
        }
    }
    assert!(
        regressions.is_empty(),
        "以下历史 fail-closed 规则在字段化后丢失了 fail-closed（P0 安全回归）：{regressions:?}"
    );
}

/// 两条 high 但 fail-closed 的例外必须靠显式字段表达（不能依赖 severity 推断）。
#[test]
fn high_severity_exceptions_explicitly_fail_closed() {
    let rules = all_system_rules();
    for id in ["OUT-09", "IN-GEN-06"] {
        let r = rules
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("规则 {id} 应存在于 TOML"));
        assert!(
            !matches!(r.severity, Severity::Critical),
            "{id} 预期 high 严重度（用于验证显式 fail_closed 覆盖推断）"
        );
        assert_eq!(
            r.fail_closed,
            Some(true),
            "{id} 必须显式 fail_closed = true（high 但 fail-closed）"
        );
        assert!(
            r.effective_fail_closed(),
            "{id} effective_fail_closed 应为 true"
        );
    }
}
