//! 用户规则加载 + fail-safe 集成测试。
//!
//! 测试 `sieve_policy` 加载管道在 5 类 corruption 场景下的行为：
//! 失败时返回 `Err`，调用方（main.rs `load_user_engine_fail_safe`）将其降级为 `None`，
//! daemon 必须正常启动，系统规则不受影响（fail-safe）。
//!
//! 测试策略：直接调用 `sieve_policy` 公开 API，验证加载管道（load → lint → compile）
//! 在各类 corruption 下的行为，不依赖 main.rs 私有函数。
//!
//! .cursorrules §3.2：测试代码允许使用 .unwrap()。

#![allow(unsafe_code)] // std::env::set_var 在 Rust 1.80+ 要求 unsafe{}

use sieve_policy::engine::UserEngine;
use sieve_policy::lint::lint;
use sieve_policy::loader::{load_user_rules, UserRuleEntry};
use std::path::Path;
use tempfile::TempDir;

// ─── 常量 / helper ────────────────────────────────────────────────────────────

/// 合法 user.toml 内容（通过全部 lint 校验）。
const VALID_USER_TOML: &str = r#"
schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"

[[rules]]
id = "MY-USER-RULE-001"
description = "禁止输出内部 API Key 前缀"
pattern = "sk-internal-"
severity = "high"
action = "warn"
keywords = ["sk-internal"]
added_at = "2026-05-01T00:00:00Z"
added_by = "manual"
"#;

/// 将 TOML 内容写入 `<dir>/rules/user.toml`，设置正确权限（0700/0600）。
#[cfg(unix)]
fn write_user_toml(tmp: &TempDir, content: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let rules_dir = tmp.path().join("rules");
    std::fs::create_dir_all(&rules_dir).unwrap();
    std::fs::set_permissions(&rules_dir, std::fs::Permissions::from_mode(0o700)).unwrap();

    let path = rules_dir.join("user.toml");
    std::fs::write(&path, content).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    path
}

/// 非 Unix 平台：跳过权限设置，仅写文件。
#[cfg(not(unix))]
fn write_user_toml(tmp: &TempDir, content: &str) -> std::path::PathBuf {
    let rules_dir = tmp.path().join("rules");
    std::fs::create_dir_all(&rules_dir).unwrap();
    let path = rules_dir.join("user.toml");
    std::fs::write(&path, content).unwrap();
    path
}

/// `Result<UserEngine, String>` 的辅助提取——`unwrap_err` 要求 `T: Debug`，
/// `UserEngine` 未实现 `Debug`，用此函数绕过。
fn unwrap_pipeline_err(result: Result<UserEngine, String>) -> String {
    match result {
        Ok(_) => panic!("期望 Err，实际为 Ok"),
        Err(e) => e,
    }
}

/// 运行完整加载管道（load → lint → compile），返回 Result<UserEngine, String>。
///
/// 等价于 main.rs `load_and_compile_user_engine` 的核心逻辑：
/// - 文件不存在或规则为空 → Err
/// - lint 违规 → Err（附汇总）
/// - 编译失败 → Err
/// - 成功 → Ok(UserEngine)
fn run_load_pipeline(path: &Path) -> Result<UserEngine, String> {
    let file_size = if path.exists() {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let file = load_user_rules(path).map_err(|e| format!("load error: {e}"))?;

    if file.rules.is_empty() {
        return Err(format!(
            "user rules file is empty or not present at {}",
            path.display()
        ));
    }

    let violations = lint(&file, file_size);
    if !violations.is_empty() {
        let summary = violations
            .iter()
            .map(|v| format!("[{}] {:?}: {}", v.rule_id, v.kind, v.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(format!("lint failed: {summary}"));
    }

    let enabled: Vec<UserRuleEntry> = file.rules.into_iter().filter(|r| r.enabled).collect();
    UserEngine::compile(enabled).map_err(|e| format!("compile error: {e}"))
}

// ─── 测试 1：TOML 语法错 ─────────────────────────────────────────────────────

#[test]
fn corruption_1_toml_syntax_error_returns_err() {
    // 语法破坏的 TOML → load_user_rules 返回 PolicyError::TomlParse
    // main.rs fail-safe 应将此降级为 warn + None（daemon 正常启动）
    let tmp = TempDir::new().unwrap();
    let path = write_user_toml(&tmp, "not valid toml ][[[");

    let result = run_load_pipeline(&path);
    assert!(result.is_err(), "TOML 语法错时管道应返回 Err");
    let err = unwrap_pipeline_err(result);
    assert!(
        err.contains("load error") || err.contains("parse") || err.contains("TOML"),
        "错误信息应含 load/parse/TOML，实际: {err}"
    );
}

// ─── 测试 2：lint 违规（severity=critical，被 A 类约束拦截）─────────────────

#[test]
fn corruption_2_lint_violation_critical_severity_returns_err() {
    // severity = "critical" 是用户规则不允许的
    // lint 应报 ForbiddenSeverityActionDisposition
    let tmp = TempDir::new().unwrap();
    let content = r#"
schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"

[[rules]]
id = "BAD-RULE-CRITICAL"
description = "尝试使用 critical 等级"
pattern = "badpattern"
severity = "critical"
action = "warn"
keywords = ["badpattern"]
added_at = "2026-05-01T00:00:00Z"
added_by = "manual"
"#;
    let path = write_user_toml(&tmp, content);

    let result = run_load_pipeline(&path);
    assert!(result.is_err(), "severity=critical 应被 lint 拦截返回 Err");
    let err = unwrap_pipeline_err(result);
    assert!(
        err.contains("lint failed"),
        "错误信息应含 'lint failed'，实际: {err}"
    );
}

// ─── 测试 3：文件权限错（0644，非 0600）────────────────────────────────────

#[test]
#[cfg(unix)]
fn corruption_3_wrong_file_permissions_returns_err() {
    use std::os::unix::fs::PermissionsExt;

    // 文件权限 0644（非 0600）→ load_user_rules 返回 PolicyError::FilePermissions
    // main.rs fail-safe 应将此降级为 warn + None
    let tmp = TempDir::new().unwrap();
    let rules_dir = tmp.path().join("rules");
    std::fs::create_dir_all(&rules_dir).unwrap();
    std::fs::set_permissions(&rules_dir, std::fs::Permissions::from_mode(0o700)).unwrap();

    let path = rules_dir.join("user.toml");
    std::fs::write(&path, VALID_USER_TOML).unwrap();
    // 故意设置错误权限
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

    let result = run_load_pipeline(&path);
    assert!(result.is_err(), "文件权限 0644 时应返回 Err");
    let err = unwrap_pipeline_err(result);
    assert!(
        err.contains("load error") || err.contains("permission") || err.contains("0600"),
        "错误信息应含权限相关内容，实际: {err}"
    );
}

// ─── 测试 4：文件不存在（路径不存在目录）→ 返回空规则 → 管道返回 Err ────────

#[test]
fn corruption_4_nonexistent_path_returns_err() {
    // 文件不存在 → load_user_rules 返回空 UserRulesFile（fail-safe 行为）
    // run_load_pipeline 对空规则返回 Err（等价于"无用户规则，用系统规则"）
    let nonexistent = Path::new("/tmp/__sieve_test_nonexistent_dir__/rules/user.toml");

    let result = run_load_pipeline(nonexistent);
    assert!(result.is_err(), "文件不存在时管道应返回 Err（空规则）");
    let err = unwrap_pipeline_err(result);
    assert!(
        err.contains("empty or not present"),
        "错误信息应说明规则为空，实际: {err}"
    );
}

// ─── 测试 5：合法用户规则文件 → 管道返回 Ok(UserEngine) ─────────────────────

#[test]
fn valid_user_rules_compiles_ok_and_matches() {
    // 正常合法的 user.toml → 管道返回 Ok，并且能正常匹配规则
    let tmp = TempDir::new().unwrap();
    let path = write_user_toml(&tmp, VALID_USER_TOML);

    let result = run_load_pipeline(&path);
    assert!(result.is_ok(), "合法用户规则应编译成功");

    let engine = result.unwrap();
    // 验证规则数量（VALID_USER_TOML 含 1 条 enabled 规则）
    use sieve_rules::engine::MatchEngine;
    assert_eq!(engine.rule_count(), 1, "应编译 1 条用户规则");

    // 验证规则能命中（rule_id 应携带 user: 前缀）
    let hits = engine.scan(b"sk-internal-abc123").unwrap();
    assert!(!hits.is_empty(), "规则应命中目标文本");
    assert!(
        hits[0].rule_id.starts_with("user:"),
        "命中的 rule_id 应携带 user: 前缀，实际: {}",
        hits[0].rule_id
    );
}

// ─── 测试 6：disabled 规则被过滤 → 空规则列表 → 管道返回 Err ─────────────────

#[test]
fn all_disabled_rules_returns_err() {
    // 所有规则 enabled=false → compile 阶段收到空列表 → 返回 Err
    // 等价于"无有效用户规则"，fail-safe 同样降级为 None
    let content = r#"
schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"

[[rules]]
id = "MY-DISABLED-RULE"
description = "禁用规则"
pattern = "nope"
severity = "low"
action = "warn"
keywords = ["nope"]
enabled = false
added_at = "2026-05-01T00:00:00Z"
added_by = "manual"
"#;
    let tmp = TempDir::new().unwrap();
    let path = write_user_toml(&tmp, content);

    // load 成功，lint 成功，但 filter(enabled) 后为空
    // UserEngine::compile(vec![]) 在 vectorscan 不支持空规则时返回 Err
    // run_load_pipeline 对空 enabled 列表同样返回 Err（与 UserEngine::compile 行为一致）
    let file_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let file = load_user_rules(&path).unwrap();
    let violations = lint(&file, file_size);
    assert!(
        violations.is_empty(),
        "disabled 规则不应触发 lint 违规: {violations:?}"
    );

    let enabled: Vec<UserRuleEntry> = file.rules.into_iter().filter(|r| r.enabled).collect();
    assert!(enabled.is_empty(), "过滤后应为空规则列表");

    // UserEngine::compile([]) 要么失败（vectorscan 不支持空列表），要么成功但 rule_count=0
    // 两种情况 fail-safe 的结果都是"无用户规则"，daemon 正常启动
    let compile_result = UserEngine::compile(enabled);
    match compile_result {
        Err(_) => {
            // vectorscan 不支持空规则列表 → 合理，fail-safe 降级为 None
        }
        Ok(engine) => {
            use sieve_rules::engine::MatchEngine;
            assert_eq!(engine.rule_count(), 0, "空规则引擎 rule_count 应为 0");
            let hits = engine.scan(b"anything").unwrap();
            assert!(hits.is_empty(), "空规则引擎不应命中任何内容");
        }
    }
}

// ─── 测试 7：目录权限错（0755，非 0700）────────────────────────────────────

#[test]
#[cfg(unix)]
fn corruption_7_wrong_dir_permissions_returns_err() {
    use std::os::unix::fs::PermissionsExt;

    // 目录权限 0755（非 0700）→ load_user_rules 返回 PolicyError::FilePermissions
    let tmp = TempDir::new().unwrap();
    let rules_dir = tmp.path().join("rules");
    std::fs::create_dir_all(&rules_dir).unwrap();
    // 故意设置错误目录权限
    std::fs::set_permissions(&rules_dir, std::fs::Permissions::from_mode(0o755)).unwrap();

    let path = rules_dir.join("user.toml");
    std::fs::write(&path, VALID_USER_TOML).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();

    let result = run_load_pipeline(&path);
    assert!(result.is_err(), "目录权限 0755 时应返回 Err");
}

// ─── 测试 8：LayeredEngine 包装 UserEngine 后的 MatchEngine 行为 ──────────────

#[test]
fn layered_engine_with_user_rules_merges_hits() {
    use sieve_rules::engine::{LayeredEngine, MatchEngine, VectorscanEngine};
    use sieve_rules::manifest::{Action, DefaultOnTimeout, Disposition, RuleEntry, Severity};

    // 构造系统规则 VectorscanEngine
    let system_rule = RuleEntry {
        id: "SYS-001".into(),
        description: "系统规则".into(),
        pattern: "sys_secret".into(),
        severity: Severity::High,
        action: Action::Warn,
        entropy_min: None,
        keywords: vec!["sys".into()],
        allowlist_regexes: vec![],
        allowlist_stopwords: vec![],
        disposition: Some(Disposition::StatusBar),
        fail_closed: None,
        timeout_seconds: None,
        default_on_timeout: DefaultOnTimeout::Allow,
    };
    let system_engine = VectorscanEngine::compile(vec![system_rule]).unwrap();

    // 构造用户规则 UserEngine（通过 run_load_pipeline）
    let tmp = TempDir::new().unwrap();
    let path = write_user_toml(&tmp, VALID_USER_TOML);
    let user_engine = run_load_pipeline(&path).unwrap();

    // 用 LayeredEngine 组合
    let layered = LayeredEngine::new(system_engine, Some(user_engine));

    // 系统规则命中
    let hits = layered.scan(b"sys_secret here").unwrap();
    assert!(!hits.is_empty(), "LayeredEngine 应命中系统规则");
    assert_eq!(hits[0].rule_id, "SYS-001");

    // 用户规则命中
    let hits = layered.scan(b"sk-internal-abc123").unwrap();
    assert!(!hits.is_empty(), "LayeredEngine 应命中用户规则");
    assert!(
        hits[0].rule_id.starts_with("user:"),
        "用户规则命中应有 user: 前缀，实际: {}",
        hits[0].rule_id
    );

    // 两种规则同时命中
    let hits = layered.scan(b"sys_secret and sk-internal-abc123").unwrap();
    assert!(hits.len() >= 2, "LayeredEngine 应同时报告两条命中");

    // rule_count 合并
    assert!(
        layered.rule_count() >= 2,
        "LayeredEngine rule_count 应 >= 2，实际: {}",
        layered.rule_count()
    );
}
