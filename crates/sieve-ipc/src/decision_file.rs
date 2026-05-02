use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;
use fd_lock::RwLock;
use uuid::Uuid;

use crate::{
    error::IpcError,
    paths::{decisions_dir, ensure_dirs, locks_dir},
    protocol::{DecisionAction, DecisionResponse},
};

/// 将 [`DecisionResponse`] 写入 `<base>/decisions/<request_id>.json`。
///
/// 写入前在 `<base>/locks/<request_id>.lock` 加独占写锁，确保并发写入安全
///（hook 与 GUI 极少同时操作同一 request_id，但防御性加锁是正确做法）。
///
/// 关联：SPEC-001 §3.3（决策文件写入规约）。
pub fn write_decision(resp: &DecisionResponse, base: &Path) -> Result<PathBuf, IpcError> {
    ensure_dirs(base)?;
    let lock_path = locks_dir(base).join(format!("{}.lock", resp.request_id));
    let dec_path = decisions_dir(base).join(format!("{}.json", resp.request_id));

    // 创建锁文件（若不存在），然后加独占写锁。
    let lock_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)?;

    let mut lock = RwLock::new(lock_file);
    {
        let _guard = lock
            .write()
            .map_err(|e| IpcError::FileLock(e.to_string()))?;

        let json = serde_json::to_string_pretty(resp)?;
        std::fs::write(&dec_path, json.as_bytes())?;
    }

    // decisions 写入成功后，清理对应的 pending 文件。
    // 删除失败不是致命错误（竞争/权限），仅打 warning，不向上返回错误。
    // Unix 上 unlink 不受 fd-lock 影响，可安全删除。
    // 关联：SPEC-001 §4.3（清理机制）。
    let pending_path = crate::paths::pending_dir(base).join(format!("{}.json", resp.request_id));
    if let Err(e) = std::fs::remove_file(&pending_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!(
                "sieve-ipc: warning: failed to remove pending file {}: {e}",
                pending_path.display()
            );
        }
    }

    Ok(dec_path)
}

/// 轮询等待 `<base>/decisions/<request_id>.json` 出现并读取。
///
/// 轮询间隔 50 ms，对 30–120 s 的用户响应超时来说 CPU 开销可忽略。
/// 选择轮询而非 inotify/notify 是为了跨平台简单性；Phase 1 仅 macOS，
/// 但未来 Linux 支持时轮询同样生效，不需要额外适配。
///
/// 超时后按 `default_on_timeout` 构造兜底响应。关联：ADR-013 §4.2。
pub async fn wait_for_decision(
    request_id: Uuid,
    base: &Path,
    timeout: Duration,
    default_on_timeout: crate::protocol::DefaultOnTimeout,
) -> Result<DecisionResponse, IpcError> {
    let path = decisions_dir(base).join(format!("{request_id}.json"));
    let deadline = tokio::time::Instant::now() + timeout;
    let poll_interval = Duration::from_millis(50);

    loop {
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            let resp: DecisionResponse = serde_json::from_str(&content)?;
            return Ok(resp);
        }

        if tokio::time::Instant::now() >= deadline {
            // 超时：按 default_on_timeout 构造兜底响应。
            let action = match default_on_timeout {
                crate::protocol::DefaultOnTimeout::Block => DecisionAction::Deny,
                crate::protocol::DefaultOnTimeout::Allow => DecisionAction::Allow,
                crate::protocol::DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
            };
            return Ok(DecisionResponse {
                request_id,
                decision: action,
                decided_at: Utc::now(),
                by_user: false,
                remember: false,
                context_hint: None,
                ui_phase_when_clicked: None,
            });
        }

        tokio::time::sleep(poll_interval).await;
    }
}

/// 同步版读取决策文件（hook 侧使用，不依赖 tokio）。
pub fn read_decision(request_id: Uuid, base: &Path) -> Result<DecisionResponse, IpcError> {
    let path = decisions_dir(base).join(format!("{request_id}.json"));
    let content = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            // 不存在时通过 IpcError::PendingNotFound 复用（语义相近）
            IpcError::PendingNotFound { request_id }
        } else {
            IpcError::Socket(e)
        }
    })?;
    let resp: DecisionResponse = serde_json::from_str(&content)?;
    Ok(resp)
}
