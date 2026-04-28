// sieve-hook lib target：供 criterion bench 和集成测试调用核心逻辑。
// main.rs 通过 use sieve_hook_lib::* 复用这些定义。

pub mod decision;
pub mod error;
pub mod pending;
pub mod protocol;

use std::path::Path;
use uuid::Uuid;

use decision::{write_decision, DecisionOutcome};
use error::PendingError;
use pending::{read_pending_checked, scan_pending_dir};

const STALE_THRESHOLD_SECS: i64 = 600;

/// 核心运行逻辑（不含 clap 解析），供 bench 和测试直接调用。
///
/// pending 文件不存在 → exit 0（fail-open）
/// pending 文件存在但已过期 → exit 1（fail-closed）
/// JSON 解析失败 → exit 1（fail-closed）
/// 文件正常 → 按 default_on_timeout 决定（非 TTY 路径，不显示提示）
///
/// 返回进程退出码：0 = 允许，1 = 拒绝。
/// 关联：SPEC-001 §4（hook 决策流程）。
pub fn run_check(request_id: Uuid, base: &Path) -> i32 {
    match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
        Err(PendingError::NotFound) => 0,
        Err(PendingError::Stale) => {
            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
            1
        }
        Err(PendingError::ParseError(e)) => {
            eprintln!("sieve-hook: failed to parse pending file: {e}");
            1
        }
        Err(PendingError::IoError(e)) => {
            eprintln!("sieve-hook: IO error reading pending file: {e}");
            1
        }
        Ok(req) => {
            // 非 TTY 场景（bench/测试）：直接按 default_on_timeout 决定。
            let outcome = match req.default_on_timeout {
                protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
                _ => DecisionOutcome::Deny,
            };
            if let Err(e) = write_decision(request_id, &outcome, base) {
                eprintln!("sieve-hook: failed to write decision: {e}");
            }
            match outcome {
                DecisionOutcome::Allow => 0,
                DecisionOutcome::Deny => 1,
            }
        }
    }
}

/// 启发式运行逻辑：无 request_id 时扫目录。
///
/// 优先级 3（SPEC-001 §4.3）：
/// - 零 fresh pending → fail-open（exit 0）
/// - stale 文件 → 删除 + warn + fail-open（exit 0）
/// - 有 fresh pending → 合并所有 detection，按 default_on_timeout 决定（非 TTY 路径）
///   多 pending 时用户一次决策广播给所有 request_id。
///
/// 返回进程退出码：0 = 允许，1 = 拒绝。
/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
pub fn run_check_heuristic(base: &Path) -> i32 {
    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);

    // 删除 stale 文件 + 打 warning。
    for stale_path in &scan.stale_paths {
        eprintln!(
            "sieve-hook: warning: stale pending file deleted: {}",
            stale_path.display()
        );
        let _ = std::fs::remove_file(stale_path);
    }

    if scan.fresh.is_empty() {
        // 零 pending：Sieve 代理未标记任何请求，fail-open。
        return 0;
    }

    // 有 fresh pending：合并所有 detection，按所有请求中最严的 default_on_timeout 决定。
    // （非 TTY 路径：直接按策略决定，不弹提示。）
    let outcome = decide_outcome_for_requests(&scan.fresh);

    // 广播决策给所有 pending request_id。
    for req in &scan.fresh {
        if let Err(e) = write_decision(req.request_id, &outcome, base) {
            eprintln!(
                "sieve-hook: failed to write decision for {}: {e}",
                req.request_id
            );
        }
    }

    match outcome {
        DecisionOutcome::Allow => 0,
        DecisionOutcome::Deny => 1,
    }
}

/// 从多个 pending 请求中计算合并决策：任一 Block/Redact → Deny，全 Allow → Allow。
fn decide_outcome_for_requests(reqs: &[protocol::DecisionRequest]) -> DecisionOutcome {
    for req in reqs {
        match req.default_on_timeout {
            protocol::DefaultOnTimeout::Allow => {}
            _ => return DecisionOutcome::Deny,
        }
    }
    DecisionOutcome::Allow
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use std::path::Path;
    use uuid::Uuid;

    use crate::decision::{self, DecisionOutcome};
    use crate::pending;
    use crate::protocol::{DecisionRequest, DefaultOnTimeout, DetectionPayload};

    fn write_pending_json(base: &Path, req: &DecisionRequest) {
        let dir = base.join("pending");
        std::fs::create_dir_all(&dir).unwrap();
        let json = serde_json::to_string_pretty(req).unwrap();
        std::fs::write(dir.join(format!("{}.json", req.request_id)), json).unwrap();
    }

    fn make_req(
        id: Uuid,
        dot: DefaultOnTimeout,
        created_at: chrono::DateTime<Utc>,
    ) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at,
            timeout_seconds: 30,
            default_on_timeout: dot,
            detections: vec![],
        }
    }

    // ── pending 文件不存在 → exit 0（fail-open） ────────────────────────────

    #[test]
    fn pending_not_found_returns_0() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 0, "file not found should fail-open (exit 0)");
    }

    // ── pending 文件过期 → exit 1（fail-closed） ────────────────────────────

    #[test]
    fn pending_stale_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // created_at 设为 11 分钟前，超过 stale 阈值（10 分钟）。
        let stale_time = Utc::now() - Duration::minutes(11);
        let req = make_req(id, DefaultOnTimeout::Allow, stale_time);
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "stale pending should fail-closed (exit 1)");
    }

    // ── JSON 解析失败 → exit 1（fail-closed） ───────────────────────────────

    #[test]
    fn pending_parse_error_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let dir = tmp.path().join("pending");
        std::fs::create_dir_all(&dir).unwrap();
        // 写入非法 JSON。
        std::fs::write(dir.join(format!("{id}.json")), b"{ not valid json }").unwrap();
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "parse error should fail-closed (exit 1)");
    }

    // ── default_on_timeout=Allow → exit 0 ──────────────────────────────────

    #[test]
    fn pending_allow_on_timeout_returns_0() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 0, "default_on_timeout=Allow should return exit 0");
    }

    // ── default_on_timeout=Block → exit 1 ──────────────────────────────────

    #[test]
    fn pending_block_on_timeout_returns_1() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Block, Utc::now());
        write_pending_json(tmp.path(), &req);
        let code = super::run_check(id, tmp.path());
        assert_eq!(code, 1, "default_on_timeout=Block should return exit 1");
    }

    // ── Critical detection 记录的 decision.remember 永远 false ─────────────

    #[test]
    fn critical_decision_remember_is_false() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Allow,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-01".to_owned(),
                severity: "critical".to_owned(),
                disposition: "hook_terminal".to_owned(),
                title: "Test".to_owned(),
                one_line_summary: "test".to_owned(),
                details: serde_json::Value::Null,
            }],
        };
        write_pending_json(tmp.path(), &req);
        super::run_check(id, tmp.path());

        // 读取写入的 decision 文件，验证 remember=false。
        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
        let content = std::fs::read_to_string(dec_path).unwrap();
        let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(resp["remember"], serde_json::Value::Bool(false));
    }

    // ════════════════════════════════════════════════════════════════════════
    // 启发式匹配路径（run_check_heuristic）的 7 个新测试
    // ════════════════════════════════════════════════════════════════════════

    // 测试 1：零 pending 文件 → exit 0（fail-open）
    #[test]
    fn heuristic_zero_pending_fail_open() {
        let tmp = tempfile::tempdir().unwrap();
        // pending 目录不存在，模拟全新安装。
        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 0, "zero pending should fail-open (exit 0)");
    }

    // 测试 2：单 pending 文件 + default_on_timeout=Allow → exit 0
    #[test]
    fn heuristic_single_pending_allow() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Allow, Utc::now());
        write_pending_json(tmp.path(), &req);

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 0, "single Allow pending should return exit 0");

        // 验证 decision 文件已写入。
        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
        assert!(dec_path.exists(), "decision file should be written");
        let content = std::fs::read_to_string(&dec_path).unwrap();
        let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(resp["decision"], "allow");
    }

    // 测试 3：单 pending 文件 + default_on_timeout=Block → exit 1
    #[test]
    fn heuristic_single_pending_block() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let req = make_req(id, DefaultOnTimeout::Block, Utc::now());
        write_pending_json(tmp.path(), &req);

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 1, "single Block pending should return exit 1");

        // 验证 decision 文件已写入且 decision=deny。
        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
        let content = std::fs::read_to_string(&dec_path).unwrap();
        let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(resp["decision"], "deny");
    }

    // 测试 4：多 pending 文件 → 所有 decision 文件写入，最严策略生效
    #[test]
    fn heuristic_multi_pending_all_decisions_written() {
        let tmp = tempfile::tempdir().unwrap();
        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        // id1 = Allow，id2 = Block → 合并后应 Deny。
        write_pending_json(
            tmp.path(),
            &make_req(id1, DefaultOnTimeout::Allow, Utc::now()),
        );
        write_pending_json(
            tmp.path(),
            &make_req(id2, DefaultOnTimeout::Block, Utc::now()),
        );

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 1, "mixed pending: Block wins, should return exit 1");

        // 两个 request_id 都应写入 decision 文件。
        for id in [id1, id2] {
            let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
            assert!(dec_path.exists(), "decision for {id} should be written");
            let content = std::fs::read_to_string(&dec_path).unwrap();
            let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert_eq!(resp["decision"], "deny", "all decisions should be deny");
        }
    }

    // 测试 5：stale pending 文件 → 删除 stale + exit 0（fail-open）
    #[test]
    fn heuristic_stale_pending_deleted_and_fail_open() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        let stale_time = Utc::now() - Duration::minutes(11);
        let req = make_req(id, DefaultOnTimeout::Block, stale_time);
        write_pending_json(tmp.path(), &req);

        let pending_file = tmp.path().join("pending").join(format!("{id}.json"));
        assert!(
            pending_file.exists(),
            "stale pending file should exist before run"
        );

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 0, "stale-only pending should fail-open (exit 0)");
        // stale 文件应被删除。
        assert!(
            !pending_file.exists(),
            "stale pending file should be deleted"
        );
    }

    // 测试 6：SIEVE_REQUEST_ID 优先级 — env 设了就走 run_check 路径，不扫目录
    #[test]
    fn env_request_id_takes_priority_over_heuristic() {
        let tmp = tempfile::tempdir().unwrap();
        // 只有 id_env 对应的 pending 文件，另写一个 id_other（不应命中）。
        let id_env = Uuid::now_v7();
        let id_other = Uuid::now_v7();
        write_pending_json(
            tmp.path(),
            &make_req(id_env, DefaultOnTimeout::Allow, Utc::now()),
        );
        write_pending_json(
            tmp.path(),
            &make_req(id_other, DefaultOnTimeout::Block, Utc::now()),
        );

        // 直接调 run_check（模拟 env 优先级路径）：只查 id_env，应 Allow。
        let code = super::run_check(id_env, tmp.path());
        assert_eq!(
            code, 0,
            "run_check with explicit id should only check that id"
        );

        // id_other 没有对应 decision 文件（未被启发式路径处理）。
        let dec_other = tmp
            .path()
            .join("decisions")
            .join(format!("{id_other}.json"));
        assert!(
            !dec_other.exists(),
            "heuristic should not run when explicit id is provided"
        );
    }

    // 测试 7：多 pending 全 Allow → exit 0
    #[test]
    fn heuristic_multi_pending_all_allow() {
        let tmp = tempfile::tempdir().unwrap();
        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        write_pending_json(
            tmp.path(),
            &make_req(id1, DefaultOnTimeout::Allow, Utc::now()),
        );
        write_pending_json(
            tmp.path(),
            &make_req(id2, DefaultOnTimeout::Allow, Utc::now()),
        );

        let code = super::run_check_heuristic(tmp.path());
        assert_eq!(code, 0, "all-Allow multi pending should return exit 0");

        for id in [id1, id2] {
            let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
            let content = std::fs::read_to_string(&dec_path).unwrap();
            let resp: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert_eq!(resp["decision"], "allow");
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // P2-#4 修复：scan 跳过已决策 + write_decision 删 pending 的 5 个新测试
    // ════════════════════════════════════════════════════════════════════════

    // 测试 8：scan_pending_dir 跳过已决策的 pending（decisions/<id>.json 存在）
    #[test]
    fn scan_skips_already_decided_pending() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // 写入 pending 文件。
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Block, Utc::now()),
        );
        // 模拟已写入的 decision 文件。
        let dec_dir = tmp.path().join("decisions");
        std::fs::create_dir_all(&dec_dir).unwrap();
        std::fs::write(dec_dir.join(format!("{id}.json")), b"{}").unwrap();

        let result = pending::scan_pending_dir(tmp.path(), 600);
        assert!(
            result.fresh.is_empty(),
            "scan should skip pending that has a corresponding decision file"
        );
        assert!(
            result.stale_paths.is_empty(),
            "decided pending should not appear in stale_paths either"
        );
    }

    // 测试 9：scan_pending_dir 正常返回无对应 decision 的 fresh pending
    #[test]
    fn scan_returns_undecided_pending() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Allow, Utc::now()),
        );
        // 无 decisions/<id>.json → 应进 fresh。

        let result = pending::scan_pending_dir(tmp.path(), 600);
        assert_eq!(
            result.fresh.len(),
            1,
            "undecided pending should appear in fresh"
        );
        assert_eq!(result.fresh[0].request_id, id);
    }

    // 测试 10：write_decision 完成后 pending/<id>.json 应被删除
    #[test]
    fn write_decision_removes_pending_file() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // 先写 pending 文件。
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Allow, Utc::now()),
        );
        let pending_path = tmp.path().join("pending").join(format!("{id}.json"));
        assert!(
            pending_path.exists(),
            "pending file should exist before write_decision"
        );

        decision::write_decision(id, &DecisionOutcome::Allow, tmp.path()).unwrap();

        assert!(
            !pending_path.exists(),
            "write_decision should delete the pending file"
        );
    }

    // 测试 11：write_decision 删 pending 失败时不报错（容错）
    #[cfg(unix)]
    #[test]
    fn write_decision_tolerates_pending_delete_failure() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        // 写 pending 文件，然后让 pending 目录不可写（删除权限受限）。
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Allow, Utc::now()),
        );
        let pending_dir = tmp.path().join("pending");
        // 移除目录写权限，使 remove_file 失败。
        std::fs::set_permissions(&pending_dir, std::fs::Permissions::from_mode(0o555)).unwrap();

        // write_decision 本身不应因删 pending 失败而返回错误。
        let result = decision::write_decision(id, &DecisionOutcome::Allow, tmp.path());

        // 恢复权限（tempdir drop 时需要能清理）。
        std::fs::set_permissions(&pending_dir, std::fs::Permissions::from_mode(0o755)).unwrap();

        assert!(
            result.is_ok(),
            "write_decision should succeed even if pending file deletion fails"
        );
        // decisions 文件应已写入。
        let dec_path = tmp.path().join("decisions").join(format!("{id}.json"));
        assert!(dec_path.exists(), "decision file should still be written");
    }

    // 测试 12：完整生命周期——scan → write_decision → 再 scan → fresh=[]（无重复）
    #[test]
    fn full_lifecycle_no_repeat_popup() {
        let tmp = tempfile::tempdir().unwrap();
        let id = Uuid::now_v7();
        write_pending_json(
            tmp.path(),
            &make_req(id, DefaultOnTimeout::Allow, Utc::now()),
        );

        // 第一次 scan：应看到 fresh=[id]。
        let result1 = pending::scan_pending_dir(tmp.path(), 600);
        assert_eq!(
            result1.fresh.len(),
            1,
            "first scan should return fresh pending"
        );

        // 模拟用户决策（write_decision 写 decisions + 删 pending）。
        decision::write_decision(id, &DecisionOutcome::Allow, tmp.path()).unwrap();

        // 第二次 scan：pending 已删且 decisions 已存在 → fresh=[]。
        let result2 = pending::scan_pending_dir(tmp.path(), 600);
        assert!(
            result2.fresh.is_empty(),
            "second scan after decision should return empty fresh (no repeated popup)"
        );
        assert!(
            result2.stale_paths.is_empty(),
            "second scan should return empty stale_paths"
        );
    }
}
