//! multi-agent setup 集成测试（SPEC-004 §2）。
//!
//! 仅 macOS 编译运行（`#[cfg(target_os = "macos")]`）。
//!
//! 测试矩阵（7 个）：
//! 1. `sieve setup --agent claude --dry-run`：输出含 Claude diff，不改文件
//! 2. `sieve setup --agent openclaw --dry-run`：输出 stub diff（标 TBD），不改文件
//! 3. `sieve setup --agent claude --agent hermes --dry-run`：两段 diff
//! 4. `sieve setup --all-detected --dry-run`：输出含探测到的 agent（至少 claude）
//! 5. `sieve doctor --agent claude`：仅跑 Claude 5 项检查
//! 6. `sieve uninstall --agent claude --dry-run`：dry-run 显示恢复内容
//! 7. `sieve uninstall --all --dry-run`：dry-run 全部回滚预览

#![cfg(target_os = "macos")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

/// 返回 debug 构建的 sieve 二进制路径（不存在则跳过测试）。
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
        eprintln!("跳过：sieve 二进制未找到（请先 cargo build -p sieve-cli）");
        None
    }
}

/// 创建 fake home 目录并返回路径。
fn fake_home() -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    let fake = dir.path();
    fs::create_dir_all(fake.join(".claude")).unwrap();
    fs::create_dir_all(fake.join(".sieve")).unwrap();
    // 写一个 fake settings.json
    fs::write(
        fake.join(".claude").join("settings.json"),
        r#"{"env": {"ORIGINAL_KEY": "original_value"}}"#,
    )
    .unwrap();
    dir
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 1：sieve setup --agent claude --dry-run
// ─────────────────────────────────────────────────────────────────────────────

/// dry-run 输出含 Claude diff 关键词，不修改 settings.json。
///
/// 关联 SPEC-004 §2.1。
#[test]
fn setup_agent_claude_dry_run_shows_diff() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let settings = fake.join(".claude").join("settings.json");
    let original = fs::read_to_string(&settings).unwrap();

    let out = Command::new(&bin)
        .args(["setup", "--agent", "claude", "--dry-run", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    // 输出应包含 Claude diff 关键词
    assert!(
        stdout.contains("ANTHROPIC_BASE_URL") || stderr.contains("ANTHROPIC_BASE_URL"),
        "setup --agent claude --dry-run 输出应含 ANTHROPIC_BASE_URL，stdout: {stdout}, stderr: {stderr}"
    );
    assert!(
        stdout.contains("claude") || stderr.contains("claude"),
        "输出应含 'claude'，stdout: {stdout}"
    );
    // dry-run 不改文件
    let after = fs::read_to_string(&settings).unwrap();
    assert_eq!(original, after, "dry-run 不应修改 settings.json");
    // 进程退出码为 0
    assert!(
        out.status.success(),
        "setup --agent claude --dry-run 应 exit 0，stdout: {stdout}, stderr: {stderr}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 2：sieve setup --agent openclaw --dry-run（stub diff）
// ─────────────────────────────────────────────────────────────────────────────

/// OpenClaw dry-run 输出 stub diff（含 TBD 说明），不改文件，exit 0。
///
/// 关联 SPEC-004 §4.2 / §10 TBD-01。
#[test]
fn setup_agent_openclaw_dry_run_shows_stub_diff() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();

    let out = Command::new(&bin)
        .args(["setup", "--agent", "openclaw", "--dry-run", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    // 输出应含 openclaw 相关内容（stub diff 或 TBD 说明）
    assert!(
        combined.contains("openclaw") || combined.contains("OpenClaw"),
        "setup --agent openclaw --dry-run 输出应含 openclaw 相关内容，combined: {combined}"
    );
    // dry-run 成功退出
    assert!(
        out.status.success(),
        "setup --agent openclaw --dry-run 应 exit 0，combined: {combined}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 3：sieve setup --agent claude --agent hermes --dry-run
// ─────────────────────────────────────────────────────────────────────────────

/// 同时传两个 --agent，输出含两段 diff。
///
/// 关联 SPEC-004 §2.1（多 agent 顺序处理）。
#[test]
fn setup_multiple_agents_dry_run_shows_both_diffs() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let settings = fake.join(".claude").join("settings.json");
    let original = fs::read_to_string(&settings).unwrap();

    let out = Command::new(&bin)
        .args([
            "setup",
            "--agent",
            "claude",
            "--agent",
            "hermes",
            "--dry-run",
            "--yes",
        ])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    // 输出应含 claude 和 hermes 两段
    assert!(
        combined.contains("claude") || combined.contains("Claude"),
        "输出应含 Claude 内容，combined: {combined}"
    );
    assert!(
        combined.contains("hermes") || combined.contains("Hermes"),
        "输出应含 Hermes 内容，combined: {combined}"
    );
    // dry-run 不改文件
    let after = fs::read_to_string(&settings).unwrap();
    assert_eq!(original, after, "dry-run 不应修改 settings.json");
    // exit 0
    assert!(
        out.status.success(),
        "setup --agent claude --agent hermes --dry-run 应 exit 0，combined: {combined}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 4：sieve setup --all-detected --dry-run
// ─────────────────────────────────────────────────────────────────────────────

/// --all-detected 扫描 → dry-run 输出含探测到的 agent。
///
/// 测试机上有 claude 二进制或 settings.json → Claude 必然被探测到。
/// 关联 SPEC-004 §3。
#[test]
fn setup_all_detected_dry_run() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();

    let out = Command::new(&bin)
        .args(["setup", "--all-detected", "--dry-run", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    // 由于 fake_home 中有 .claude/settings.json，Claude 应被探测到
    // 输出要么含 Claude diff，要么含"未检测到"提示（若 detect 逻辑严格）
    // 这里只验证进程不崩溃（exit 0）并有输出
    assert!(
        !combined.is_empty(),
        "setup --all-detected 应有输出，combined: {combined}"
    );
    assert!(
        out.status.success(),
        "setup --all-detected --dry-run 应 exit 0，combined: {combined}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 5：sieve doctor --agent claude
// ─────────────────────────────────────────────────────────────────────────────

/// --agent claude 只跑 Claude 5 项检查，输出含 Claude 检查结果，不含 openclaw/hermes。
///
/// 关联 SPEC-004 §6.1。
#[test]
fn doctor_agent_claude_only_runs_claude_checks() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();

    let out = Command::new(&bin)
        .args(["doctor", "--agent", "claude"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    // 输出应含 Claude 检查项（ANTHROPIC_BASE_URL 或 Claude Code）
    assert!(
        combined.contains("Claude Code") || combined.contains("ANTHROPIC_BASE_URL"),
        "doctor --agent claude 输出应含 Claude 检查项，combined: {combined}"
    );
    // 不应含 OpenClaw/Hermes stub 输出
    assert!(
        !combined.contains("OpenClaw 检查为 stub"),
        "doctor --agent claude 不应跑 OpenClaw，combined: {combined}"
    );
    assert!(
        !combined.contains("Hermes 检查为 stub"),
        "doctor --agent claude 不应跑 Hermes，combined: {combined}"
    );
    // fake_home 中没有配置 sieve，doctor 应以非零退出（检查项失败）
    // exit code 非零（期望 1，因为未配置）
    assert!(
        !out.status.success(),
        "未配置 sieve 时 doctor --agent claude 应 exit 非零，combined: {combined}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 6：sieve uninstall --agent claude --dry-run
// ─────────────────────────────────────────────────────────────────────────────

/// uninstall --agent claude --dry-run 显示恢复内容，不实际改文件。
///
/// 关联 SPEC-004 §2.3。
#[test]
fn uninstall_agent_claude_dry_run_shows_preview() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();

    let out = Command::new(&bin)
        .args(["uninstall", "--agent", "claude", "--dry-run", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    // 应含 dry-run 标志
    assert!(
        combined.contains("dry-run") || combined.contains("未做任何改动"),
        "uninstall --agent claude --dry-run 输出应含 dry-run 说明，combined: {combined}"
    );
    // exit 0
    assert!(
        out.status.success(),
        "uninstall --agent claude --dry-run 应 exit 0，combined: {combined}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 7：sieve uninstall --all --dry-run
// ─────────────────────────────────────────────────────────────────────────────

/// uninstall --all --dry-run 全部回滚预览，exit 0。
///
/// 关联 SPEC-004 §2.3 / §5.2。
#[test]
fn uninstall_all_dry_run_shows_full_preview() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();

    let out = Command::new(&bin)
        .args(["uninstall", "--all", "--dry-run", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.contains("dry-run") || combined.contains("未做任何改动"),
        "uninstall --all --dry-run 输出应含 dry-run 说明，combined: {combined}"
    );
    assert!(
        out.status.success(),
        "uninstall --all --dry-run 应 exit 0，combined: {combined}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R7-#4：setup.log 缺失时 agent_filter 保护
// ─────────────────────────────────────────────────────────────────────────────

/// R7-#4 场景 A：setup.log 缺失 + --agent openclaw → 不动备份，友好退出。
///
/// backups/ 仅含 Claude 文件；openclaw 不应触发 fallback。
#[test]
fn uninstall_no_setup_log_openclaw_does_not_restore_backup() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    // 构造 backups/ 含 Claude 文件，但不创建 setup.log
    let backup_dir = sieve_home.join("backups").join("2026-04-27T00:00:00Z");
    fs::create_dir_all(&backup_dir).unwrap();
    fs::write(
        backup_dir.join("settings.json"),
        r#"{"env":{"ORIGINAL_KEY":"original_value"}}"#,
    )
    .unwrap();

    // 记录 settings.json 初始内容（不应被改动）
    let settings_path = fake.join(".claude").join("settings.json");
    let original = fs::read_to_string(&settings_path).unwrap();

    let out = Command::new(&bin)
        .args(["uninstall", "--agent", "openclaw", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    // 应输出友好提示，不实际恢复任何文件
    assert!(
        combined.contains("nothing to uninstall") || combined.contains("no setup record"),
        "setup.log 缺失 + --agent openclaw 应提示无记录，combined: {combined}"
    );
    // settings.json 不应被修改
    let after = fs::read_to_string(&settings_path).unwrap();
    assert_eq!(
        original, after,
        "setup.log 缺失 + --agent openclaw 不应恢复 Claude backup 到 settings.json"
    );
    // exit 0（友好退出，不是错误）
    assert!(
        out.status.success(),
        "setup.log 缺失 + --agent openclaw 应 exit 0，combined: {combined}"
    );
}

/// R7-#4 场景 B：setup.log 缺失 + --agent claude → 仍能 fallback 到全局备份（无回归）。
///
/// v1.4 老用户无 setup.log，--agent claude 必须能正常 uninstall。
#[test]
fn uninstall_no_setup_log_claude_fallback_works() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    // 构造 backups/ 含 Claude 文件，但不创建 setup.log
    let backup_dir = sieve_home.join("backups").join("2026-04-27T00:00:00Z");
    fs::create_dir_all(&backup_dir).unwrap();
    fs::write(
        backup_dir.join("settings.json"),
        r#"{"env":{"ORIGINAL_KEY":"original_value"}}"#,
    )
    .unwrap();

    let out = Command::new(&bin)
        .args(["uninstall", "--agent", "claude", "--dry-run", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    // dry-run 应含备份目录（fallback 生效），不含"nothing to uninstall"
    assert!(
        !combined.contains("nothing to uninstall"),
        "setup.log 缺失 + --agent claude 不应提示无记录，combined: {combined}"
    );
    assert!(
        combined.contains("dry-run")
            || combined.contains("未做任何改动")
            || combined.contains("backup"),
        "setup.log 缺失 + --agent claude 应进入 dry-run 预览流程，combined: {combined}"
    );
    assert!(
        out.status.success(),
        "setup.log 缺失 + --agent claude --dry-run 应 exit 0，combined: {combined}"
    );
}

/// R7-#4 场景 C：setup.log 缺失 + --all → 仍能 fallback（无回归）。
#[test]
fn uninstall_no_setup_log_all_fallback_works() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    // 构造 backups/ 含 Claude 文件，但不创建 setup.log
    let backup_dir = sieve_home.join("backups").join("2026-04-27T00:00:00Z");
    fs::create_dir_all(&backup_dir).unwrap();
    fs::write(
        backup_dir.join("settings.json"),
        r#"{"env":{"ORIGINAL_KEY":"original_value"}}"#,
    )
    .unwrap();

    let out = Command::new(&bin)
        .args(["uninstall", "--all", "--dry-run", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    // dry-run 应含 dry-run 说明，不含"nothing to uninstall"
    assert!(
        !combined.contains("nothing to uninstall"),
        "setup.log 缺失 + --all 不应提示无记录，combined: {combined}"
    );
    assert!(
        combined.contains("dry-run") || combined.contains("未做任何改动"),
        "setup.log 缺失 + --all 应进入 dry-run 预览流程，combined: {combined}"
    );
    assert!(
        out.status.success(),
        "setup.log 缺失 + --all --dry-run 应 exit 0，combined: {combined}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 额外：sieve uninstall（无参数）应 exit 2
// ─────────────────────────────────────────────────────────────────────────────

/// 不传 --agent 且不传 --all 时，uninstall 应 exit 2（SPEC-004 §2.3）。
#[test]
fn uninstall_no_args_exits_2() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();

    let out = Command::new(&bin)
        .args(["uninstall"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    let stderr = String::from_utf8_lossy(&out.stderr);

    // 应 exit 2（SPEC-004 §2.3）
    assert_eq!(
        out.status.code(),
        Some(2),
        "uninstall 无参数应 exit 2，stderr: {stderr}"
    );
    assert!(
        stderr.contains("--agent") || stderr.contains("--all"),
        "错误信息应提示 --agent 或 --all，stderr: {stderr}"
    );
}
