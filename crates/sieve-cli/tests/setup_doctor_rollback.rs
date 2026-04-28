//! R5-#1 修复验证：setup 调 doctor 失败时自动回滚。
//!
//! 仅 macOS 编译运行（`#[cfg(target_os = "macos")]`）。
//!
//! 测试矩阵：
//! - T1（happy-path）：`sieve setup --yes` 在 dry-run 模式下成功，settings.json 保持不变
//! - T2（doctor 失败回滚）：通过子进程运行 `sieve setup --yes`，
//!   daemon 不在线时 doctor 必然失败，验证 setup 返回非零 exit code，
//!   并且 settings.json 恢复原内容（没有停留在半配置状态）

#![cfg(target_os = "macos")]

use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

/// 返回 debug 构建的 sieve 二进制路径（如不存在则跳过测试）。
fn sieve_bin() -> Option<PathBuf> {
    let bin = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join("debug")
        .join("sieve");
    if bin.exists() {
        Some(bin)
    } else {
        None
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// T1：dry-run 模式下 setup 不改 settings.json（happy-path 基线）
// ─────────────────────────────────────────────────────────────────────────────

/// dry-run 不修改任何文件，且进程退出码为 0。
///
/// 这里直接复用 sieve_setup_dry_run.rs 已有逻辑作为基线，
/// 确认 R5-#1 修复没有破坏 dry-run 路径。
#[test]
fn dry_run_exits_zero_without_modifying_settings() {
    let Some(sieve_bin) = sieve_bin() else {
        eprintln!("跳过 dry_run_exits_zero_without_modifying_settings：sieve 二进制未找到");
        return;
    };

    let dir = tempdir().unwrap();
    let fake_home = dir.path().to_path_buf();
    let claude_dir = fake_home.join(".claude");
    let sieve_dir = fake_home.join(".sieve");
    fs::create_dir_all(&claude_dir).unwrap();
    fs::create_dir_all(&sieve_dir).unwrap();

    let settings_path = claude_dir.join("settings.json");
    let original = r#"{"env": {"ORIGINAL_KEY": "original_value"}}"#;
    fs::write(&settings_path, original).unwrap();

    let status = std::process::Command::new(&sieve_bin)
        .args(["setup", "--dry-run"])
        .env("HOME", fake_home.to_str().unwrap())
        .env("SIEVE_HOME", sieve_dir.to_str().unwrap())
        .env_remove("SIEVE_LOG")
        .status()
        .expect("运行 sieve setup --dry-run 失败");

    // dry-run 应该成功
    assert!(
        status.success(),
        "sieve setup --dry-run 应以 0 退出，实际：{status}"
    );

    // settings.json 不应被修改
    let after = fs::read_to_string(&settings_path).unwrap();
    assert_eq!(after, original, "dry-run 不应修改 settings.json");
}

// ─────────────────────────────────────────────────────────────────────────────
// T2：doctor 失败时 setup 回滚，settings.json 恢复原内容
// ─────────────────────────────────────────────────────────────────────────────

/// `sieve setup --yes` 在 daemon 未启动环境下：
/// - doctor 必然失败（daemon 未在 :11453 监听）
/// - setup 应返回非零 exit code
/// - settings.json 应恢复为原始内容（不留半配置状态）
///
/// 验证 R5-#1 修复：doctor 失败分支现在会调用 ctx.rollback()。
#[test]
fn setup_rolls_back_when_doctor_fails() {
    let Some(sieve_bin) = sieve_bin() else {
        eprintln!("跳过 setup_rolls_back_when_doctor_fails：sieve 二进制未找到");
        return;
    };

    let dir = tempdir().unwrap();
    let fake_home = dir.path().to_path_buf();
    let claude_dir = fake_home.join(".claude");
    let sieve_dir = fake_home.join(".sieve");
    fs::create_dir_all(&claude_dir).unwrap();
    fs::create_dir_all(&sieve_dir).unwrap();

    let settings_path = claude_dir.join("settings.json");
    let original = r#"{"env": {"ORIGINAL_KEY": "original_value"}}"#;
    fs::write(&settings_path, original).unwrap();

    // --yes 跳过确认，直接执行。
    // F-2 修复后 setup 会把规则文件部署到 $SIEVE_HOME/rules/，使 doctor canary 可以运行。
    // 为了让 doctor 仍然失败（测试回滚路径），把 SIEVE_RULES_PATH 指向不存在的文件，
    // 强制优先级 1 返回不存在路径 → canary 规则引擎初始化失败 → doctor 返回 Err。
    let output = std::process::Command::new(&sieve_bin)
        .args(["setup", "--yes"])
        .env("HOME", fake_home.to_str().unwrap())
        .env("SIEVE_HOME", sieve_dir.to_str().unwrap())
        // 指向不存在文件：doctor resolve_rules_path 优先级 1 命中，
        // 但文件不存在 → VectorscanEngine::compile 失败 → canary 检查失败 → doctor Err
        .env("SIEVE_RULES_PATH", "/nonexistent/sieve/rules/outbound.toml")
        .env_remove("SIEVE_LOG")
        .output()
        .expect("运行 sieve setup --yes 失败");

    // doctor 失败 → setup 应返回非零 exit code
    assert!(
        !output.status.success(),
        "doctor 失败时 sieve setup 应以非零 exit code 退出，实际：{}",
        output.status
    );

    // 错误输出应提示"已自动回滚"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("已自动回滚") || stderr.contains("rollback") || stderr.contains("回滚"),
        "stderr 应含回滚提示，实际：{stderr}"
    );

    // settings.json 必须恢复为原始内容（核心验证）
    let after = fs::read_to_string(&settings_path).unwrap_or_default();
    assert_eq!(
        after, original,
        "doctor 失败后 settings.json 应恢复为原始内容，但实际内容为：{after}"
    );
}
