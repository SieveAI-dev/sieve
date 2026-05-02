use std::path::Path;

use chrono::Utc;
use fd_lock::RwLock;
use uuid::Uuid;

use crate::protocol::DecisionResponse;

/// hook 侧决策结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionOutcome {
    /// 用户允许，hook 返回 exit 0。
    Allow,
    /// 用户拒绝或超时 fail-closed，hook 返回 exit 1。
    Deny,
}

/// 将决策结果写入 `<base>/decisions/<request_id>.json`。
///
/// 写入前在 `<base>/locks/<request_id>.lock` 加独占写锁。
///
/// Critical 规则 `remember` 永远 `false`，由调用方（main.rs）强制传入 false。
/// 关联：SPEC-001 §3.3（决策文件写入）、ADR-014（Critical 不可记住）。
pub fn write_decision(
    request_id: Uuid,
    outcome: &DecisionOutcome,
    base: &Path,
) -> Result<(), String> {
    // 确保目录存在。
    let decisions_dir = base.join("decisions");
    let locks_dir = base.join("locks");
    std::fs::create_dir_all(&decisions_dir).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&locks_dir).map_err(|e| e.to_string())?;

    let lock_path = locks_dir.join(format!("{request_id}.lock"));
    let dec_path = decisions_dir.join(format!("{request_id}.json"));

    let lock_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| e.to_string())?;

    let mut lock = RwLock::new(lock_file);
    let _guard = lock.write().map_err(|e| e.to_string())?;

    let decision_str = match outcome {
        DecisionOutcome::Allow => "allow",
        DecisionOutcome::Deny => "deny",
    };

    let resp = DecisionResponse {
        request_id,
        decision: decision_str.to_owned(),
        decided_at: Utc::now(),
        by_user: true,
        // Critical 规则 remember 强制 false（SPEC-001 §4.4）。
        remember: false,
        ui_phase_when_clicked: None,
    };

    let json = serde_json::to_string_pretty(&resp).map_err(|e| e.to_string())?;
    std::fs::write(&dec_path, json.as_bytes()).map_err(|e| e.to_string())?;

    // decisions 写入成功后，清理对应的 pending 文件。
    // 删除失败不是致命错误（竞争/权限），仅打 warning。
    // Unix 上持有 fd-lock 的文件仍可 unlink，先删 pending 再释放锁是安全的。
    // 关联：SPEC-001 §4.3（清理机制）。
    let pending_path = base.join("pending").join(format!("{request_id}.json"));
    if let Err(e) = std::fs::remove_file(&pending_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!(
                "sieve-hook: warning: failed to remove pending file {}: {e}",
                pending_path.display()
            );
        }
    }

    Ok(())
}
