//! B-R2：不可变守护配置——集中回归基线。
//!
//! 把「守护配置对 agent 不可变」这条架构不变量锁成一处端到端回归基线：
//! agent 经由配置/规则/hook 的任何削弱尝试都被结构性拒绝。每条攻击路径一个具名断言，
//! 任一防御回归即 CI 变红。是 fail-closed 不可关在 config/规则加载层的姊妹篇。
//!
//! 覆盖以下 10 条攻击路径：
//!   - #1/#7 未知字段/危险开关偷渡：`deny_unknown_fields` —— 在 sieve-cli `config.rs` 内测
//!     （`unknown_field_rejected` / `upstream_listener_rejects_unknown_field` / `audit_rejects_unknown_field`）。
//!     sieve-cli 是 binary crate（无 lib），`Config` 不可从集成测试 use，故 config 层断言留在其内测。
//!   - #10 非 loopback bind / 配置坏：`check_safety_invariants` —— config.rs `check_invariants_rejects_non_loopback_bind`。
//!   - #2 CLI flag 一键放行：该 flag 不存在（fail-closed 不可关 + CI hard-fail），无可断言对象。
//!   - #8 / #3a / #3b / #3c / #4 / #5 / #6 / #9：本文件覆盖（求值/lint/加载/合并层）。
//!
//! 跑法：`cargo test -p sieve-policy --test immutable_guardian`

use chrono::Utc;
use sieve_policy::{
    engine::UserEngine,
    error::PolicyError,
    lint::{lint, LintKind},
    loader::{load_user_rules, RuleDirection, UserRuleEntry, UserRulesFile},
};
use sieve_rules::critical_lock::{enforce_action, is_fail_closed, register_rules};
use sieve_rules::manifest::{Action, DefaultOnTimeout, RuleEntry, Severity};
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// ─────────────────────────── 辅助构造 ───────────────────────────

/// 合法用户规则条目（默认 high/warn，符合用户规则等级上限）。
fn user_entry(id: &str) -> UserRuleEntry {
    UserRuleEntry {
        id: id.into(),
        description: "immutable guardian regression rule".into(),
        pattern: "test_pattern".into(),
        severity: "high".into(),
        action: "warn".into(),
        keywords: vec!["keyword".into()],
        allowlist_stopwords: vec![],
        disposition: None,
        direction: RuleDirection::Both,
        enabled: true,
        added_at: Utc::now(),
        added_by: "manual".into(),
    }
}

fn file_of(rules: Vec<UserRuleEntry>) -> UserRulesFile {
    UserRulesFile {
        schema_version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        rules,
    }
}

/// 系统 Critical 规则（fail-closed），用于把 ID 注册进全局分类注册表。
/// 用 `IMMUT-` 唯一前缀避免污染其他测试（注册表 accumulate，只增不删）。
fn sys_critical(id: &str) -> RuleEntry {
    RuleEntry {
        id: id.into(),
        description: id.into(),
        pattern: "x".into(),
        severity: Severity::Critical,
        action: Action::Block,
        entropy_min: None,
        keywords: vec![],
        allowlist_regexes: vec![],
        allowlist_stopwords: vec![],
        disposition: None,
        fail_closed: None,
        timeout_seconds: None,
        default_on_timeout: DefaultOnTimeout::Block,
    }
}

// ─────────────────────────── 求值层（#8）───────────────────────────

/// #8：系统 Critical 规则在求值期被强制 Block，且与 `dry_run` 正交。
/// `enforce_action` 不接受 dry_run 参数——fail-closed 恒 Block，dry_run 无法削弱它。
#[test]
fn b_r2_eval_fail_closed_forces_block_regardless_of_dry_run() {
    register_rules(&[sys_critical("IMMUT-FC-EVAL")]);
    assert!(is_fail_closed("IMMUT-FC-EVAL"));
    // 请求 Allow / Warn 一律被强制 Block（dry_run 是 daemon 层的 body 改写开关，
    // 不进入 enforce_action 的求值；fail-closed 求值对 dry_run 不可见 = 正交）。
    assert_eq!(
        enforce_action("IMMUT-FC-EVAL", Action::Allow),
        Action::Block
    );
    assert_eq!(enforce_action("IMMUT-FC-EVAL", Action::Warn), Action::Block);
    // 未注册规则保持请求 action（对照）。
    assert_eq!(
        enforce_action("IMMUT-DEFINITELY-ABSENT", Action::Mark),
        Action::Mark
    );
}

// ─────────────────────────── 用户规则 lint 层（#3a/#3b/#3c/#4/#5）─────────────

/// #3a：用户规则声明 severity=critical → A-1 拒绝。
#[test]
fn b_r2_lint_rejects_user_severity_critical() {
    let mut e = user_entry("IMMUT-USER-CRIT");
    e.severity = "critical".into();
    let v = lint(&file_of(vec![e]), 100);
    assert!(
        v.iter()
            .any(|x| x.kind == LintKind::ForbiddenSeverityActionDisposition),
        "severity=critical 应被 A-1 拒绝: {v:?}"
    );
}

/// #3b：用户规则声明 action=block → A-1 拒绝。
#[test]
fn b_r2_lint_rejects_user_action_block() {
    let mut e = user_entry("IMMUT-USER-BLOCK");
    e.action = "block".into();
    let v = lint(&file_of(vec![e]), 100);
    assert!(
        v.iter()
            .any(|x| x.kind == LintKind::ForbiddenSeverityActionDisposition),
        "action=block 应被 A-1 拒绝: {v:?}"
    );
}

/// #3c：用户规则 id 与系统 Critical rule_id 撞号 → A-4 拒绝。
#[test]
fn b_r2_lint_rejects_system_critical_id_conflict() {
    register_rules(&[sys_critical("IMMUT-CONFLICT-ID")]);
    let e = user_entry("IMMUT-CONFLICT-ID");
    let v = lint(&file_of(vec![e]), 100);
    assert!(
        v.iter().any(|x| x.kind == LintKind::SystemRuleIdConflict),
        "id 撞系统 Critical 应被 A-4 拒绝: {v:?}"
    );
}

/// #4：用户规则 allowlist_stopwords 含系统 Critical rule_id → A-6 拒绝。
#[test]
fn b_r2_lint_rejects_allowlist_targets_system_critical() {
    register_rules(&[sys_critical("IMMUT-ALLOWLIST-TARGET")]);
    let mut e = user_entry("IMMUT-USER-ALLOW");
    e.allowlist_stopwords = vec!["IMMUT-ALLOWLIST-TARGET".into()];
    let v = lint(&file_of(vec![e]), 100);
    assert!(
        v.iter()
            .any(|x| x.kind == LintKind::AllowlistTargetsSystemCritical),
        "allowlist 豁免系统 Critical 应被 A-6 拒绝: {v:?}"
    );
}

/// #5：用户规则 direction=inbound + disposition=auto_redact → A-5 拒绝
/// （用户不得改写模型输出）。
#[test]
fn b_r2_lint_rejects_inbound_auto_redact() {
    let mut e = user_entry("IMMUT-USER-INBOUND-REDACT");
    e.direction = RuleDirection::Inbound;
    e.disposition = Some("auto_redact".into());
    let v = lint(&file_of(vec![e]), 100);
    assert!(
        v.iter()
            .any(|x| x.kind == LintKind::InboundAutoRedactForbidden),
        "入站 auto_redact 用户规则应被 A-5 拒绝: {v:?}"
    );
}

// ─────────────────────────── 文件加载层（#6）───────────────────────────

/// #6：user.toml 为 symlink → 加载期 no-follow 拒绝。
#[test]
#[cfg(unix)]
fn b_r2_loader_rejects_symlink() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("rules");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();

    // 真实目标文件（合法内容）
    let target = tmp.path().join("real_user.toml");
    std::fs::write(&target, "schema_version = 1\ncreated_at = \"2026-05-01T00:00:00Z\"\nupdated_at = \"2026-05-01T00:00:00Z\"\n").unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o600)).unwrap();

    // user.toml 做成指向目标的 symlink
    let link = dir.join("user.toml");
    std::os::unix::fs::symlink(&target, &link).unwrap();

    let err = load_user_rules(&link).unwrap_err();
    assert!(
        matches!(err, PolicyError::SymlinkRejected(_)),
        "symlink user.toml 应被 SymlinkRejected: {err:?}"
    );
}

// ─────────────────────────── 规则合并层（#9）───────────────────────────

/// #9：用户规则经合并永不 fail-closed（兜底——即便 lint 漏网，`engine.rs` 的
/// `to_rule_entry` 强制 `fail_closed=Some(false)`，用户规则 ID 绝不进 fail-closed 注册表）。
#[test]
fn b_r2_user_rule_never_fail_closed() {
    // 合法用户规则（high/warn）编译为用户引擎；不应使其 ID 进入 fail-closed 注册表。
    let e = user_entry("IMMUT-USER-NEVER-FC");
    let _engine = UserEngine::compile(vec![e]).expect("user engine compile");
    assert!(
        !is_fail_closed("IMMUT-USER-NEVER-FC"),
        "用户规则永不 fail-closed（合并层 fail_closed=Some(false) 兜底，engine.rs:155）"
    );
}

// ─────────────────────────── fail-safe 不变量（验收标准 3）───────────────

/// 注入恶意用户规则后，系统 Critical 仍全功能：lint 拒绝恶意条但系统 fail-closed 不受影响。
#[test]
fn b_r2_failsafe_system_critical_unaffected_by_malicious_user_rule() {
    register_rules(&[sys_critical("IMMUT-FAILSAFE-SYS")]);
    // 恶意用户规则（severity=critical）被 lint 拒绝……
    let mut bad = user_entry("IMMUT-FAILSAFE-USER");
    bad.severity = "critical".into();
    let v = lint(&file_of(vec![bad]), 100);
    assert!(!v.is_empty(), "恶意用户规则应产生 lint 违规（被拒绝）");
    // ……而系统 Critical 仍 fail-closed、求值期仍强制 Block。
    assert!(is_fail_closed("IMMUT-FAILSAFE-SYS"));
    assert_eq!(
        enforce_action("IMMUT-FAILSAFE-SYS", Action::Allow),
        Action::Block
    );
}
