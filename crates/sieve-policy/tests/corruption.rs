//! sieve-policy 破坏性输入端到端测试（用户规则 fail-safe）。
//!
//! 覆盖 loader + lint 在异常输入下的防御行为，每类破坏一个 `#[test]`。
//! 不启动 daemon，纯粹测 sieve-policy 层。
//!
//! 跑法：
//! ```bash
//! cargo test -p sieve-policy --test corruption
//! ```

use chrono::Utc;
use sieve_policy::{
    error::PolicyError,
    lint::{lint, LintKind},
    loader::{load_user_rules, RuleDirection, UserRuleEntry, UserRulesFile},
};
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// ─────────────────────────── 辅助函数 ───────────────────────────

/// 构造合法的 user.toml 字符串，`rules` 段由调用方传入。
fn make_toml(rules: &str) -> String {
    format!(
        r#"schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"

{rules}
"#
    )
}

/// 写文件到 tmp 目录，同时设置正确的安全权限（0700/0600），
/// 返回文件路径。
fn write_secure(tmp: &TempDir, content: &str) -> PathBuf {
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

/// 构造含 N 条合法 dummy 规则的 `[[rules]]` 段。
fn make_n_rules(n: usize) -> String {
    (0..n)
        .map(|i| {
            format!(
                r#"[[rules]]
id = "DUMMY-{:03}"
description = "dummy rule {}"
pattern = "dummy_pattern_{}"
severity = "high"
action = "warn"
keywords = ["dummy"]
added_at = "2026-05-01T00:00:00Z"
added_by = "manual"
"#,
                i, i, i
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// 从 `UserRulesFile` 构造 `UserRuleEntry` 的辅助函数，用于直接构造 lint 输入。
fn valid_entry(id: &str) -> UserRuleEntry {
    UserRuleEntry {
        id: id.into(),
        description: "corruption test rule".into(),
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

fn make_file(rules: Vec<UserRuleEntry>) -> UserRulesFile {
    UserRulesFile {
        schema_version: 1,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        rules,
    }
}

// ─────────────────────────── 破坏测试 ───────────────────────────

/// 破坏 1：TOML 语法错误 → load_user_rules 返 PolicyError::TomlParse。
///
/// 覆盖：破坏的文件不能导致 daemon panic，必须明确返回错误。
#[test]
fn corruption_toml_syntax_error() {
    let tmp = TempDir::new().unwrap();
    // `schema_version = invalid_value` 不是合法 TOML（裸标识符无法作为值）
    let bad_toml = "schema_version = invalid_value\n";
    let path = write_secure(&tmp, bad_toml);

    let err = load_user_rules(&path).unwrap_err();
    assert!(
        matches!(err, PolicyError::TomlParse(_)),
        "TOML 语法错误应返回 TomlParse，实际: {err:?}"
    );
}

/// 破坏 2：lint 违规 — severity=critical → lint() 返 ForbiddenSeverityActionDisposition。
///
/// 覆盖：用户规则禁止声明 critical 等级。
#[test]
fn corruption_lint_severity_critical() {
    let mut entry = valid_entry("USER-CRITICAL");
    entry.severity = "critical".into();
    let file = make_file(vec![entry]);

    let violations = lint(&file, 100);
    assert!(
        violations
            .iter()
            .any(|v| v.kind == LintKind::ForbiddenSeverityActionDisposition),
        "severity=critical 应触发 ForbiddenSeverityActionDisposition，实际: {violations:?}"
    );
}

/// 破坏 3：lint 违规 — action=block → lint() 返 ForbiddenSeverityActionDisposition。
///
/// 覆盖：用户规则禁止声明 block 处置。
#[test]
fn corruption_lint_action_block() {
    let mut entry = valid_entry("USER-BLOCK");
    entry.action = "block".into();
    let file = make_file(vec![entry]);

    let violations = lint(&file, 100);
    assert!(
        violations
            .iter()
            .any(|v| v.kind == LintKind::ForbiddenSeverityActionDisposition),
        "action=block 应触发 ForbiddenSeverityActionDisposition，实际: {violations:?}"
    );
}

/// 破坏 4：lint 违规 — keywords=[] → lint() 返 KeywordsEmpty。
///
/// 覆盖：keywords 预过滤为必填非空字段。
#[test]
fn corruption_lint_keywords_empty() {
    let mut entry = valid_entry("USER-EMPTY-KW");
    entry.keywords = vec![];
    let file = make_file(vec![entry]);

    let violations = lint(&file, 100);
    assert!(
        violations.iter().any(|v| v.kind == LintKind::KeywordsEmpty),
        "keywords=[] 应触发 KeywordsEmpty，实际: {violations:?}"
    );
}

/// 破坏 5：lint 违规 — 201 条规则超过上限 → lint() 返 TooManyRules。
///
/// 覆盖：用户规则条数上限 200。
#[test]
fn corruption_lint_too_many_rules() {
    let rules: Vec<_> = (0..201)
        .map(|i| valid_entry(&format!("USER-RULE-{:03}", i)))
        .collect();
    let file = make_file(rules);

    let violations = lint(&file, 100);
    assert!(
        violations.iter().any(|v| v.kind == LintKind::TooManyRules),
        "201 条规则应触发 TooManyRules，实际: {violations:?}"
    );
}

/// 破坏 6：文件大小超过 1MB → lint() 返 FileTooLarge。
///
/// 覆盖：文件大小上限 1MB，防止文件系统攻击。
#[test]
fn corruption_lint_file_too_large() {
    let file = make_file(vec![valid_entry("USER-RULE")]);
    // 传入超 1MB 的 file_size_bytes 参数（1.5MB = 1_572_864 bytes）
    let violations = lint(&file, 1_572_864);

    assert!(
        violations.iter().any(|v| v.kind == LintKind::FileTooLarge),
        "文件大小 1.5MB 应触发 FileTooLarge，实际: {violations:?}"
    );
}

/// 破坏 7：文件权限 0644（非 0600）→ load_user_rules 返 PolicyError::FilePermissions。
///
/// 覆盖：owner-only 权限保护，防止其他用户读取规则文件。
#[test]
#[cfg(unix)]
fn corruption_file_permissions_wrong() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("rules");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();

    let path = dir.join("user.toml");
    let content = make_toml(&make_n_rules(1));
    std::fs::write(&path, &content).unwrap();
    // 故意设置 0644（权限太宽松，任意用户可读）
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    let err = load_user_rules(&path).unwrap_err();
    assert!(
        matches!(err, PolicyError::FilePermissions(_)),
        "0644 权限应返回 FilePermissions，实际: {err:?}"
    );
}

/// 破坏 8：符号链接指向真实文件 → load_user_rules 返 PolicyError::SymlinkRejected。
///
/// 覆盖：no-follow symlink 策略，防止 ln -s /etc/passwd 攻击。
#[test]
#[cfg(unix)]
fn corruption_symlink_rejected() {
    let tmp = TempDir::new().unwrap();
    let real_file = tmp.path().join("real_user.toml");
    let content = make_toml(&make_n_rules(1));
    std::fs::write(&real_file, &content).unwrap();
    std::fs::set_permissions(&real_file, std::fs::Permissions::from_mode(0o600)).unwrap();

    // 在规则目录创建符号链接
    let rules_dir = tmp.path().join("rules");
    std::fs::create_dir_all(&rules_dir).unwrap();
    std::fs::set_permissions(&rules_dir, std::fs::Permissions::from_mode(0o700)).unwrap();

    let link = rules_dir.join("user.toml");
    std::os::unix::fs::symlink(&real_file, &link).unwrap();

    let err = load_user_rules(&link).unwrap_err();
    assert!(
        matches!(err, PolicyError::SymlinkRejected(_)),
        "符号链接应返回 SymlinkRejected，实际: {err:?}"
    );
}

/// 破坏 9：目录权限 0755（文件 0600 正确，但目录太宽松）→ load_user_rules 返 FilePermissions。
///
/// 覆盖：目录权限校验，目录 0755 允许任意用户遍历，不符合最小权限原则。
#[test]
#[cfg(unix)]
fn corruption_dir_permissions_wrong() {
    let tmp = TempDir::new().unwrap();
    let dir = tmp.path().join("rules");
    std::fs::create_dir_all(&dir).unwrap();
    // 目录故意设置 0755（非 0700）
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();

    let path = dir.join("user.toml");
    let content = make_toml(&make_n_rules(1));
    std::fs::write(&path, &content).unwrap();
    // 文件本身 0600 正确
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();

    let err = load_user_rules(&path).unwrap_err();
    assert!(
        matches!(err, PolicyError::FilePermissions(_)),
        "目录 0755 应返回 FilePermissions，实际: {err:?}"
    );
}

/// 破坏 10：两条规则 id 重复 → lint() 返 DuplicateRuleId。
///
/// 覆盖：用户规则 ID 必须唯一（含禁用规则），防止规则集歧义。
#[test]
fn corruption_lint_duplicate_rule_id() {
    let entry1 = valid_entry("DUPLICATE-ID");
    let entry2 = valid_entry("DUPLICATE-ID"); // 同一 ID
    let file = make_file(vec![entry1, entry2]);

    let violations = lint(&file, 100);
    assert!(
        violations
            .iter()
            .any(|v| v.kind == LintKind::DuplicateRuleId),
        "重复 ID 应触发 DuplicateRuleId，实际: {violations:?}"
    );
}

/// 破坏 11：TOML 包含未知字段 → load_user_rules 返 PolicyError::TomlParse（deny_unknown_fields）。
///
/// 覆盖防御深度：serde deny_unknown_fields 拒绝格式漂移，避免静默接受意外字段。
#[test]
fn corruption_toml_unknown_field() {
    let tmp = TempDir::new().unwrap();
    // 在顶层加入未知字段
    let bad_toml = r#"schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"
injected_field = "attacker_value"
"#;
    let path = write_secure(&tmp, bad_toml);

    let err = load_user_rules(&path).unwrap_err();
    assert!(
        matches!(err, PolicyError::TomlParse(_)),
        "未知字段应返回 TomlParse（deny_unknown_fields），实际: {err:?}"
    );
}

/// 破坏 11b：direction=inbound + disposition=auto_redact → lint() 返 InboundAutoRedactForbidden。
///
/// 覆盖：入站方向规则禁止 disposition=auto_redact，
/// 用户不能改写 model 输出。
#[test]
fn corruption_lint_inbound_auto_redact_forbidden() {
    let mut entry = valid_entry("USER-INBOUND-REDACT");
    entry.direction = RuleDirection::Inbound;
    entry.disposition = Some("auto_redact".into());
    let file = make_file(vec![entry]);

    let violations = lint(&file, 100);
    assert!(
        violations
            .iter()
            .any(|v| v.kind == LintKind::InboundAutoRedactForbidden),
        "direction=inbound + disposition=auto_redact 应触发 InboundAutoRedactForbidden，实际: {violations:?}"
    );
}

/// direction=outbound + disposition=auto_redact 合法（出站自动脱敏）。
#[test]
fn outbound_auto_redact_is_allowed() {
    let mut entry = valid_entry("USER-OUTBOUND-REDACT");
    entry.direction = RuleDirection::Outbound;
    entry.disposition = Some("auto_redact".into());
    let file = make_file(vec![entry]);

    let violations = lint(&file, 100);
    assert!(
        !violations
            .iter()
            .any(|v| v.kind == LintKind::InboundAutoRedactForbidden),
        "direction=outbound + disposition=auto_redact 不应触发 InboundAutoRedactForbidden，实际: {violations:?}"
    );
}

/// 破坏 12：超大有效文件（201 条规则写入磁盘后加载）→ load 成功但 lint 返 TooManyRules。
///
/// 端到端验证：loader 不负责规则数量限制，lint 负责；两层分工正确。
#[test]
fn corruption_e2e_too_many_rules_from_disk() {
    let tmp = TempDir::new().unwrap();
    // 201 条合法 dummy 规则写入 TOML
    let content = make_toml(&make_n_rules(201));
    let path = write_secure(&tmp, &content);

    // load 本身应该成功（loader 不做数量限制）
    let file = load_user_rules(&path).expect("load_user_rules 应成功，数量限制在 lint 层");
    assert_eq!(file.rules.len(), 201, "应加载到 201 条规则");

    // lint 应报 TooManyRules
    let toml_size = std::fs::metadata(&path).unwrap().len();
    let violations = lint(&file, toml_size);
    assert!(
        violations.iter().any(|v| v.kind == LintKind::TooManyRules),
        "201 条规则 lint 应触发 TooManyRules，实际: {violations:?}"
    );
}
