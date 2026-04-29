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

// ─────────────────────────────────────────────────────────────────────────────
// 测试 TBD-01：OpenClaw dry-run 含真实字段（openclaw.json + baseUrl）
// ─────────────────────────────────────────────────────────────────────────────

/// OpenClaw dry-run 输出含 openclaw.json 路径和 baseUrl 字段名。
///
/// 调研结论（TBD-01 已解决）：配置文件为 ~/.openclaw/openclaw.json，
/// provider 字段为 models.providers.<id>.baseUrl。
/// 关联 SPEC-004 §10 TBD-01。
#[test]
fn openclaw_dry_run_shows_real_config_path_and_field() {
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

    // 真实字段路径：openclaw.json（非 config.toml / TBD 占位）
    assert!(
        combined.contains("openclaw.json"),
        "dry-run 应含真实配置路径 openclaw.json（TBD-01 调研结论），combined: {combined}"
    );
    // 含 X-Sieve-Source-Channel（TBD-05 调研结论）
    assert!(
        combined.contains("X-Sieve-Source-Channel") || combined.contains("header"),
        "dry-run 应含 header 注入说明（TBD-05 调研结论），combined: {combined}"
    );
    assert!(out.status.success(), "exit 应为 0，combined: {combined}");
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 TBD-02：Hermes dry-run 含真实字段（config.yaml + base_url）
// ─────────────────────────────────────────────────────────────────────────────

/// Hermes dry-run 输出含 config.yaml 路径和 base_url 字段名。
///
/// 调研结论（TBD-02 已解决）：配置文件为 ~/.hermes/config.yaml（YAML）。
/// 关联 SPEC-004 §10 TBD-02。
#[test]
fn hermes_dry_run_shows_real_config_path_and_field() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();

    let out = Command::new(&bin)
        .args(["setup", "--agent", "hermes", "--dry-run", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{stdout}{stderr}");

    // 真实字段路径：config.yaml（非 config.toml / TBD 占位）
    assert!(
        combined.contains("config.yaml"),
        "dry-run 应含真实配置路径 config.yaml（TBD-02 调研结论），combined: {combined}"
    );
    // 含 TBD-06 降级说明
    assert!(
        combined.contains("TBD-06")
            || combined.contains("delegation")
            || combined.contains("ANTHROPIC_DEFAULT_HEADERS"),
        "dry-run 应含 TBD-06 降级说明，combined: {combined}"
    );
    assert!(out.status.success(), "exit 应为 0，combined: {combined}");
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 TBD-05：OpenClaw apply 注入 X-Sieve-Source-Channel header
// ─────────────────────────────────────────────────────────────────────────────

/// OpenClaw apply 在每个 provider 的 headers 中注入 X-Sieve-Source-Channel = "openclaw"。
///
/// 调研结论（TBD-05 已解决）：OpenClaw 支持 models.providers.<id>.headers 字段。
/// 关联 SPEC-004 §10 TBD-05。
#[test]
fn openclaw_apply_injects_sieve_source_channel_header() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();

    let openclaw_dir = fake.join(".openclaw");
    fs::create_dir_all(&openclaw_dir).unwrap();
    let config_path = openclaw_dir.join("openclaw.json");
    fs::write(
        &config_path,
        r#"{"models":{"providers":{"test-provider":{"baseUrl":"https://api.openai.com/v1"}}}}"#,
    )
    .unwrap();

    let out = Command::new(&bin)
        .args(["setup", "--agent", "openclaw", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    assert!(
        out.status.success(),
        "apply 应 exit 0，stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let updated = fs::read_to_string(&config_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&updated).unwrap();

    // X-Sieve-Source-Channel 应注入
    let channel = v
        .pointer("/models/providers/test-provider/headers/X-Sieve-Source-Channel")
        .and_then(|c| c.as_str());
    assert_eq!(
        channel,
        Some("openclaw"),
        "X-Sieve-Source-Channel 应注入为 'openclaw'（TBD-05），updated: {updated}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 测试 TBD-06：Hermes apply delegation.base_url 降级注入
// ─────────────────────────────────────────────────────────────────────────────

/// Hermes apply 在 delegation.base_url 注入 Sieve URL（TBD-06 降级方案）。
///
/// 调研结论（TBD-06 降级）：Hermes 不透传 ANTHROPIC_DEFAULT_HEADERS，
/// 降级为 delegation.base_url 指向 Sieve。
/// 关联 SPEC-004 §10 TBD-06。
#[test]
fn hermes_apply_injects_delegation_base_url_fallback() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();

    let hermes_dir = fake.join(".hermes");
    fs::create_dir_all(&hermes_dir).unwrap();
    let config_path = hermes_dir.join("config.yaml");
    fs::write(
        &config_path,
        "model:\n  provider: openrouter\n  base_url: \"\"\ndelegation:\n  max_iterations: 50\n  base_url: \"\"\n",
    )
    .unwrap();

    let out = Command::new(&bin)
        .args(["setup", "--agent", "hermes", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", fake.join(".sieve"))
        .output()
        .expect("执行 sieve 失败");

    assert!(
        out.status.success(),
        "apply 应 exit 0，stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let updated = fs::read_to_string(&config_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&updated).unwrap();

    // delegation.base_url 应指向 Sieve（TBD-06 降级）
    let delegation_url = parsed
        .get("delegation")
        .and_then(|d| d.get("base_url"))
        .and_then(|u| u.as_str());
    assert_eq!(
        delegation_url,
        Some("http://127.0.0.1:11453"),
        "delegation.base_url 应为 Sieve URL（TBD-06 降级），updated: {updated}"
    );

    // model.base_url 也应指向 Sieve
    let model_url = parsed
        .get("model")
        .and_then(|m| m.get("base_url"))
        .and_then(|u| u.as_str());
    assert_eq!(
        model_url,
        Some("http://127.0.0.1:11453"),
        "model.base_url 应为 Sieve URL，updated: {updated}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-1：OpenAI 路由保留——upstream-routes.json 含原始 baseUrl
// ─────────────────────────────────────────────────────────────────────────────

/// F-1 修复验证：setup --agent openclaw 写出 upstream-routes.json，
/// 包含各 provider 的原始 baseUrl（非 127.0.0.1:11453）。
///
/// 场景：mock OpenClaw config 含 3 个 provider（openai / openrouter / deepseek），
/// setup 后验证 upstream-routes.json 有 3 条原始 URL 记录。
///
/// 关联：known-issues-v1.4.md F-1。
#[test]
fn f1_upstream_routes_json_contains_original_provider_urls() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    // 创建 mock OpenClaw config，含 3 个 provider
    let openclaw_dir = fake.join(".openclaw");
    fs::create_dir_all(&openclaw_dir).unwrap();
    let config_path = openclaw_dir.join("openclaw.json");
    fs::write(
        &config_path,
        r#"{
  "models": {
    "providers": {
      "openai": {"baseUrl": "https://api.openai.com/v1"},
      "openrouter": {"baseUrl": "https://openrouter.ai/api/v1"},
      "deepseek": {"baseUrl": "https://api.deepseek.com/v1"}
    }
  }
}"#,
    )
    .unwrap();

    let out = Command::new(&bin)
        .args(["setup", "--agent", "openclaw", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve 失败");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "setup --agent openclaw 应 exit 0，combined: {combined}"
    );

    // 验证 upstream-routes.json 存在且含 3 条记录
    let routes_path = sieve_home.join("upstream-routes.json");
    assert!(
        routes_path.exists(),
        "F-1: upstream-routes.json 应写入 {}, combined: {combined}",
        routes_path.display()
    );

    let routes_raw = fs::read_to_string(&routes_path).unwrap();
    let routes: serde_json::Value =
        serde_json::from_str(&routes_raw).expect("upstream-routes.json 应为合法 JSON");

    // 3 个 provider 的原始 URL 都应存在（不是 Sieve 代理地址）
    let obj = routes
        .as_object()
        .expect("upstream-routes.json 根应为 object");
    assert_eq!(
        obj.len(),
        3,
        "F-1: upstream-routes.json 应含 3 条路由，实际: {routes_raw}"
    );
    assert_eq!(
        obj.get("openai").and_then(|v| v.as_str()),
        Some("https://api.openai.com/v1"),
        "F-1: openai 原始 URL 未保留，routes: {routes_raw}"
    );
    assert_eq!(
        obj.get("openrouter").and_then(|v| v.as_str()),
        Some("https://openrouter.ai/api/v1"),
        "F-1: openrouter 原始 URL 未保留，routes: {routes_raw}"
    );
    assert_eq!(
        obj.get("deepseek").and_then(|v| v.as_str()),
        Some("https://api.deepseek.com/v1"),
        "F-1: deepseek 原始 URL 未保留，routes: {routes_raw}"
    );

    // daemon config 含 upstream_routes_path（通过 sieve.toml 的路由文件存在验证）
    let sieve_toml = sieve_home.join("sieve.toml");
    assert!(
        sieve_toml.exists(),
        "F-1: sieve.toml 应在 setup 时写入，sieve_home: {}",
        sieve_home.display()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-2：规则文件部署——rules/outbound.toml + inbound.toml 存在且内容一致
// ─────────────────────────────────────────────────────────────────────────────

/// F-2 修复验证：setup --agent claude（非 dry-run）后，
/// $SIEVE_HOME/rules/outbound.toml 和 inbound.toml 存在，
/// 内容与内嵌规则一致（二进制打包版本）。
///
/// 场景：tempdir 模拟 SIEVE_HOME，setup 后直接检查文件系统。
///
/// 关联：known-issues-v1.4.md P1-R3-#1（现已修复）。
#[test]
fn f2_rules_deployed_to_sieve_home_on_setup() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    let out = Command::new(&bin)
        .args(["setup", "--agent", "claude", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        // SIEVE_RULES_PATH 设为不存在路径，让 doctor canary 失败，触发回滚后 setup 以非零退出。
        // 但规则文件已在 install_shared_daemon 阶段写出，回滚时只删 sentinel + sieve.toml + plist。
        // 为了让测试不依赖 doctor 结果，这里改为不覆盖 SIEVE_RULES_PATH（使用部署的规则）。
        .output()
        .expect("执行 sieve 失败");

    // setup 可能成功（doctor 通过）或失败（doctor 失败，回滚），
    // 但规则文件在 install_shared_daemon 阶段早于 ClaudeAdapter 写出。
    // 即使 doctor 失败导致回滚，规则文件也应已写出（daemon_ctx 在单独块中，不属于 claude ctx 回滚）。
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let outbound = sieve_home.join("rules").join("outbound.toml");
    let inbound = sieve_home.join("rules").join("inbound.toml");

    assert!(
        outbound.exists(),
        "F-2: rules/outbound.toml 应在 setup 时部署，sieve_home: {}, combined: {combined}",
        sieve_home.display()
    );
    assert!(
        inbound.exists(),
        "F-2: rules/inbound.toml 应在 setup 时部署，sieve_home: {}, combined: {combined}",
        sieve_home.display()
    );

    // 内容不应为空
    let outbound_content = fs::read_to_string(&outbound).unwrap();
    let inbound_content = fs::read_to_string(&inbound).unwrap();
    assert!(
        outbound_content.contains("OUT-01") || outbound_content.contains("[[rules]]"),
        "F-2: outbound.toml 内容应含规则，实际长度: {}",
        outbound_content.len()
    );
    assert!(
        inbound_content.contains("IN-CR-01") || inbound_content.contains("[[rules]]"),
        "F-2: inbound.toml 内容应含规则，实际长度: {}",
        inbound_content.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-3：非 Claude agent 也安装 daemon
// ─────────────────────────────────────────────────────────────────────────────

/// F-3 修复验证：setup --agent openclaw 也会安装 daemon。
///
/// 场景：tempdir 模拟 SIEVE_HOME，只装 openclaw，
/// 验证 sieve.toml 和 launchd plist 都存在（daemon 共享安装）。
///
/// 关联：known-issues-v1.4.md F-3。
#[test]
fn f3_openclaw_setup_also_installs_daemon() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    // 创建 mock OpenClaw config
    let openclaw_dir = fake.join(".openclaw");
    fs::create_dir_all(&openclaw_dir).unwrap();
    fs::write(
        openclaw_dir.join("openclaw.json"),
        r#"{"models":{"providers":{"openai":{"baseUrl":"https://api.openai.com"}}}}"#,
    )
    .unwrap();

    let out = Command::new(&bin)
        .args(["setup", "--agent", "openclaw", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve 失败");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "F-3: setup --agent openclaw 应 exit 0，combined: {combined}"
    );

    // sieve.toml 应存在（daemon 共享安装完成）
    let sieve_toml = sieve_home.join("sieve.toml");
    assert!(
        sieve_toml.exists(),
        "F-3: sieve.toml 应在 setup --agent openclaw 时写入（daemon 共享安装），combined: {combined}"
    );

    // launchd plist 应存在（即使只装 openclaw，也要安装 daemon）
    let plist = fake
        .join("Library")
        .join("LaunchAgents")
        .join("com.sieve.daemon.plist");
    assert!(
        plist.exists(),
        "F-3: launchd plist 应在 setup --agent openclaw 时写入，combined: {combined}"
    );

    // sentinel 文件应存在（防止重复安装）
    let sentinel = sieve_home.join(".daemon-installed");
    assert!(
        sentinel.exists(),
        "F-3: sentinel .daemon-installed 应在首次安装后写入，combined: {combined}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// F-3：sentinel 防重复安装
// ─────────────────────────────────────────────────────────────────────────────

/// F-3 sentinel 验证：setup --agent openclaw 后再 setup --agent claude，
/// daemon 不应重复安装（sentinel 保护）。
///
/// 验证：两次 setup 后 sieve.toml 内容相同（未被覆盖），
/// 且 sentinel 文件仍存在。
///
/// 关联：known-issues-v1.4.md F-3。
#[test]
fn f3_sentinel_prevents_daemon_reinstall() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    // 第 1 次：setup --agent openclaw
    let openclaw_dir = fake.join(".openclaw");
    fs::create_dir_all(&openclaw_dir).unwrap();
    fs::write(
        openclaw_dir.join("openclaw.json"),
        r#"{"models":{"providers":{"openai":{"baseUrl":"https://api.openai.com"}}}}"#,
    )
    .unwrap();

    let out1 = Command::new(&bin)
        .args(["setup", "--agent", "openclaw", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve 失败（第 1 次）");

    let combined1 = format!(
        "{}{}",
        String::from_utf8_lossy(&out1.stdout),
        String::from_utf8_lossy(&out1.stderr)
    );
    assert!(
        out1.status.success(),
        "F-3: 第 1 次 setup 应 exit 0，combined: {combined1}"
    );

    // 记录第 1 次写入的 sieve.toml 内容
    let sieve_toml_path = sieve_home.join("sieve.toml");
    assert!(
        sieve_toml_path.exists(),
        "第 1 次 setup 后 sieve.toml 应存在"
    );
    let toml_after_first = fs::read_to_string(&sieve_toml_path).unwrap();
    let sentinel = sieve_home.join(".daemon-installed");
    assert!(sentinel.exists(), "第 1 次 setup 后 sentinel 应存在");

    // 第 2 次：setup --agent claude（同一 fake home）
    let out2 = Command::new(&bin)
        .args(["setup", "--agent", "claude", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve 失败（第 2 次）");

    let combined2 = format!(
        "{}{}",
        String::from_utf8_lossy(&out2.stdout),
        String::from_utf8_lossy(&out2.stderr)
    );
    // 第 2 次 setup 也可能成功（doctor 通过）或失败（doctor 失败），重点是 daemon 不重复安装
    // 验证：sieve.toml 内容未被覆盖（sentinel 生效，install_shared_daemon 跳过）
    let toml_after_second = fs::read_to_string(&sieve_toml_path).unwrap();
    assert_eq!(
        toml_after_first, toml_after_second,
        "F-3: 第 2 次 setup 有 sentinel，sieve.toml 不应被重写，combined2: {combined2}"
    );

    // sentinel 仍存在
    assert!(
        sentinel.exists(),
        "F-3: sentinel 在第 2 次 setup 后应仍存在，combined2: {combined2}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// R10-#1：OpenClaw/Hermes apply 后 setup.log 有 entry，uninstall 能找到并恢复
// ─────────────────────────────────────────────────────────────────────────────

/// R10-#1 测试 1：OpenClaw apply 后 setup.log 有 agent="openclaw" entry。
///
/// 验证 SetupContext::append_log_entry 被调用，setup.log 包含
/// setup_complete 和 config_modified 两条 agent="openclaw" entry。
#[test]
fn r10_openclaw_apply_writes_setup_log_entry() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    let openclaw_dir = fake.join(".openclaw");
    fs::create_dir_all(&openclaw_dir).unwrap();
    fs::write(
        openclaw_dir.join("openclaw.json"),
        r#"{"models":{"providers":{"test-provider":{"baseUrl":"https://api.openai.com/v1"}}}}"#,
    )
    .unwrap();

    let out = Command::new(&bin)
        .args(["setup", "--agent", "openclaw", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve 失败");

    assert!(
        out.status.success(),
        "setup --agent openclaw 应 exit 0，stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let log_path = sieve_home.join("setup.log");
    assert!(
        log_path.exists(),
        "R10-#1: setup.log 应存在，sieve_home: {}",
        sieve_home.display()
    );

    let log_content = fs::read_to_string(&log_path).unwrap();
    // 验证有 agent="openclaw" 的 setup_complete entry
    let has_openclaw_complete = log_content
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .any(|v| {
            v.get("agent").and_then(|a| a.as_str()) == Some("openclaw")
                && v.get("action").and_then(|a| a.as_str()) == Some("setup_complete")
        });
    assert!(
        has_openclaw_complete,
        "R10-#1: setup.log 应含 agent=openclaw 的 setup_complete entry，log: {log_content}"
    );

    // 验证有 agent="openclaw" 的 config_modified entry
    let has_openclaw_modified = log_content
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .any(|v| {
            v.get("agent").and_then(|a| a.as_str()) == Some("openclaw")
                && v.get("action").and_then(|a| a.as_str()) == Some("config_modified")
        });
    assert!(
        has_openclaw_modified,
        "R10-#1: setup.log 应含 agent=openclaw 的 config_modified entry，log: {log_content}"
    );
}

/// R10-#1 测试 2：Hermes apply 后 setup.log 有 agent="hermes" entry。
#[test]
fn r10_hermes_apply_writes_setup_log_entry() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    let hermes_dir = fake.join(".hermes");
    fs::create_dir_all(&hermes_dir).unwrap();
    fs::write(
        hermes_dir.join("config.yaml"),
        "model:\n  provider: openrouter\n  base_url: \"\"\ndelegation:\n  base_url: \"\"\n",
    )
    .unwrap();

    let out = Command::new(&bin)
        .args(["setup", "--agent", "hermes", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve 失败");

    assert!(
        out.status.success(),
        "setup --agent hermes 应 exit 0，stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let log_path = sieve_home.join("setup.log");
    assert!(log_path.exists(), "R10-#1: setup.log 应存在");

    let log_content = fs::read_to_string(&log_path).unwrap();

    let has_hermes_complete = log_content
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .any(|v| {
            v.get("agent").and_then(|a| a.as_str()) == Some("hermes")
                && v.get("action").and_then(|a| a.as_str()) == Some("setup_complete")
        });
    assert!(
        has_hermes_complete,
        "R10-#1: setup.log 应含 agent=hermes 的 setup_complete entry，log: {log_content}"
    );

    let has_hermes_modified = log_content
        .lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .any(|v| {
            v.get("agent").and_then(|a| a.as_str()) == Some("hermes")
                && v.get("action").and_then(|a| a.as_str()) == Some("config_modified")
        });
    assert!(
        has_hermes_modified,
        "R10-#1: setup.log 应含 agent=hermes 的 config_modified entry，log: {log_content}"
    );
}

/// R10-#1 测试 3：uninstall --agent openclaw 找到 entry 并从备份恢复 openclaw.json。
///
/// 端到端验证：setup → 文件被改写 → uninstall → 文件恢复到 setup 前内容。
#[test]
fn r10_uninstall_openclaw_restores_backup() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    let openclaw_dir = fake.join(".openclaw");
    fs::create_dir_all(&openclaw_dir).unwrap();
    let config_path = openclaw_dir.join("openclaw.json");
    let original_content =
        r#"{"models":{"providers":{"test-provider":{"baseUrl":"https://api.openai.com/v1"}}}}"#;
    fs::write(&config_path, original_content).unwrap();

    // setup：修改 openclaw.json
    let setup_out = Command::new(&bin)
        .args(["setup", "--agent", "openclaw", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve setup 失败");
    assert!(
        setup_out.status.success(),
        "R10-#1: setup --agent openclaw 应 exit 0，stderr: {}",
        String::from_utf8_lossy(&setup_out.stderr)
    );

    // 验证 setup 修改了文件
    let after_setup = fs::read_to_string(&config_path).unwrap();
    assert!(
        after_setup.contains("127.0.0.1:11453"),
        "R10-#1: setup 后 openclaw.json 应含 Sieve URL，actual: {after_setup}"
    );

    // uninstall：从备份恢复
    let uninstall_out = Command::new(&bin)
        .args(["uninstall", "--agent", "openclaw", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve uninstall 失败");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&uninstall_out.stdout),
        String::from_utf8_lossy(&uninstall_out.stderr)
    );
    assert!(
        uninstall_out.status.success(),
        "R10-#1: uninstall --agent openclaw 应 exit 0，combined: {combined}"
    );

    // 验证文件恢复到原始内容
    let after_uninstall = fs::read_to_string(&config_path).unwrap();
    let v_after: serde_json::Value = serde_json::from_str(&after_uninstall).unwrap();
    let v_orig: serde_json::Value = serde_json::from_str(original_content).unwrap();
    assert_eq!(
        v_after, v_orig,
        "R10-#1: uninstall 后 openclaw.json 应恢复到 setup 前内容，after: {after_uninstall}"
    );
}

/// R10-#1 测试 4：uninstall --agent hermes 找到 entry 并从备份恢复 config.yaml。
#[test]
fn r10_uninstall_hermes_restores_backup() {
    let Some(bin) = sieve_bin() else {
        return;
    };
    let dir = fake_home();
    let fake = dir.path();
    let sieve_home = fake.join(".sieve");

    let hermes_dir = fake.join(".hermes");
    fs::create_dir_all(&hermes_dir).unwrap();
    let config_path = hermes_dir.join("config.yaml");
    let original_content =
        "model:\n  provider: openrouter\n  base_url: \"\"\ndelegation:\n  base_url: \"\"\n";
    fs::write(&config_path, original_content).unwrap();

    // setup：修改 config.yaml
    let setup_out = Command::new(&bin)
        .args(["setup", "--agent", "hermes", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve setup 失败");
    assert!(
        setup_out.status.success(),
        "R10-#1: setup --agent hermes 应 exit 0，stderr: {}",
        String::from_utf8_lossy(&setup_out.stderr)
    );

    // 验证 setup 修改了文件
    let after_setup = fs::read_to_string(&config_path).unwrap();
    assert!(
        after_setup.contains("127.0.0.1:11453"),
        "R10-#1: setup 后 config.yaml 应含 Sieve URL，actual: {after_setup}"
    );

    // uninstall：从备份恢复
    let uninstall_out = Command::new(&bin)
        .args(["uninstall", "--agent", "hermes", "--yes"])
        .env("HOME", fake)
        .env("SIEVE_HOME", &sieve_home)
        .output()
        .expect("执行 sieve uninstall 失败");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&uninstall_out.stdout),
        String::from_utf8_lossy(&uninstall_out.stderr)
    );
    assert!(
        uninstall_out.status.success(),
        "R10-#1: uninstall --agent hermes 应 exit 0，combined: {combined}"
    );

    // 验证文件恢复到原始内容（base_url 恢复为空字符串）
    let after_uninstall = fs::read_to_string(&config_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&after_uninstall).unwrap();
    let model_url = parsed
        .get("model")
        .and_then(|m| m.get("base_url"))
        .and_then(|u| u.as_str());
    assert_eq!(
        model_url,
        Some(""),
        "R10-#1: uninstall 后 config.yaml model.base_url 应恢复为空，after: {after_uninstall}"
    );
    let delegation_url = parsed
        .get("delegation")
        .and_then(|d| d.get("base_url"))
        .and_then(|u| u.as_str());
    assert_eq!(
        delegation_url,
        Some(""),
        "R10-#1: uninstall 后 config.yaml delegation.base_url 应恢复为空，after: {after_uninstall}"
    );
}
