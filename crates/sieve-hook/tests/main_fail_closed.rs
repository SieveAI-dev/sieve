// 集成测试：验证生产 binary（sieve-hook check）启发式路径的 fail-closed 行为。
//
// 覆盖 known-issues-v1.4.md §P1-R3-#6 修复：生产 binary 损坏 pending 文件应
// fail-closed（exit 1），与 lib::run_check_heuristic 行为保持一致。
//
// 关联：SPEC-001 §4.3（启发式查 pending 目录）。

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

// ── 测试辅助：写入损坏的 pending 文件 ────────────────────────────────────────

fn write_corrupt_pending(base: &std::path::Path, filename: &str) {
    let dir = base.join("pending");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(filename), b"not valid json{{{").unwrap();
}

fn write_valid_pending(base: &std::path::Path, filename: &str) {
    let dir = base.join("pending");
    fs::create_dir_all(&dir).unwrap();
    // 写入合法的 DecisionRequest JSON（default_on_timeout=Allow，新鲜时间戳）。
    let json = serde_json::json!({
        "request_id": "01960000-0000-7000-8000-000000000001",
        "created_at": chrono::Utc::now().to_rfc3339(),
        "timeout_seconds": 30,
        "default_on_timeout": "allow",
        "detections": []
    });
    fs::write(
        dir.join(filename),
        serde_json::to_vec_pretty(&json).unwrap(),
    )
    .unwrap();
}

// ── 测试 1：生产 binary corrupt fail-closed ───────────────────────────────────

/// 损坏 pending 文件 → exit 1，stderr 含 "corrupt"。
#[test]
fn binary_corrupt_pending_fails_closed() {
    let tmp = tempfile::tempdir().unwrap();
    write_corrupt_pending(tmp.path(), "A.json");

    Command::cargo_bin("sieve-hook")
        .unwrap()
        .args(["check", "--sieve-home", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("corrupt"));
}

// ── 测试 2：生产 binary 无 pending fail-open（无回归）────────────────────────

/// pending 目录为空 → exit 0（fail-open）。
#[test]
fn binary_empty_pending_fails_open() {
    let tmp = tempfile::tempdir().unwrap();
    // 确保 pending 目录存在但为空。
    fs::create_dir_all(tmp.path().join("pending")).unwrap();

    Command::cargo_bin("sieve-hook")
        .unwrap()
        .args(["check", "--sieve-home", tmp.path().to_str().unwrap()])
        .assert()
        .success()
        .code(0);
}

// ── 测试 3：混合 fresh + corrupt → exit 1（保守 fail-closed）────────────────

/// 合法 pending + 损坏 pending → exit 1（corrupt 优先，fail-closed）。
#[test]
fn binary_mixed_fresh_and_corrupt_fails_closed() {
    let tmp = tempfile::tempdir().unwrap();
    write_valid_pending(tmp.path(), "A.json");
    write_corrupt_pending(tmp.path(), "B.json");

    Command::cargo_bin("sieve-hook")
        .unwrap()
        .args(["check", "--sieve-home", tmp.path().to_str().unwrap()])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("corrupt"));
}
