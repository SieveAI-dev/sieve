use std::path::{Path, PathBuf};

use fd_lock::RwLock;
use uuid::Uuid;

use crate::{
    error::IpcError,
    paths::{ensure_dirs, pending_dir},
    protocol::DecisionRequest,
};

/// 将 [`DecisionRequest`] 写入 `<base>/pending/<request_id>.json`。
///
/// 写入前用 fd-lock 对目标文件加独占写锁，防止并发写入同一 request_id（极少见
/// 但理论可行）。文件以 pretty JSON 格式写入，方便调试和 hook 侧直接读取。
///
/// 关联：SPEC-001 §3.1（pending 文件写入规约）。
pub fn write_pending(req: &DecisionRequest, base: &Path) -> Result<PathBuf, IpcError> {
    ensure_dirs(base)?;
    let dir = pending_dir(base);
    let path = dir.join(format!("{}.json", req.request_id));

    // 打开（或创建）文件，然后加独占写锁再写内容。
    // 使用 std::fs::OpenOptions 而非 std::fs::write，以便 fd-lock 持有文件描述符。
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    let mut lock = RwLock::new(file);
    {
        let mut guard = lock
            .write()
            .map_err(|e| IpcError::FileLock(e.to_string()))?;
        let json = serde_json::to_string_pretty(req)?;
        use std::io::Write;
        guard.write_all(json.as_bytes())?;
    }

    Ok(path)
}

/// 读取并解析 `<base>/pending/<request_id>.json`。
///
/// 返回：
/// - `Ok(DecisionRequest)` 成功
/// - `Err(IpcError::PendingNotFound)` 文件不存在
/// - `Err(IpcError::Json)` 解析失败
pub fn read_pending(request_id: Uuid, base: &Path) -> Result<DecisionRequest, IpcError> {
    let path = pending_dir(base).join(format!("{request_id}.json"));
    let content = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            IpcError::PendingNotFound { request_id }
        } else {
            IpcError::Socket(e)
        }
    })?;
    let req: DecisionRequest = serde_json::from_str(&content)?;
    Ok(req)
}
