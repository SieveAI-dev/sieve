use std::path::PathBuf;

use crate::error::IpcError;

/// 计算 sieve home 目录。
///
/// 优先级：`$SIEVE_HOME` 环境变量 > `$HOME/.sieve`。
/// $HOME 缺失时返回 [`IpcError::HomeNotFound`]。
///
/// 关联：SPEC-001 §2.1（目录结构）。
pub fn sieve_home() -> Result<PathBuf, IpcError> {
    if let Ok(val) = std::env::var("SIEVE_HOME") {
        return Ok(PathBuf::from(val));
    }
    let home = std::env::var("HOME").map_err(|_| IpcError::HomeNotFound)?;
    Ok(PathBuf::from(home).join(".sieve"))
}

/// `<sieve_home>/pending/` 目录，存放主代理写入的待决策文件。
pub fn pending_dir(base: &std::path::Path) -> PathBuf {
    base.join("pending")
}

/// `<sieve_home>/decisions/` 目录，存放 hook/GUI 写入的决策文件。
pub fn decisions_dir(base: &std::path::Path) -> PathBuf {
    base.join("decisions")
}

/// `<sieve_home>/locks/` 目录，存放文件锁占位符。
pub fn locks_dir(base: &std::path::Path) -> PathBuf {
    base.join("locks")
}

/// `<sieve_home>/ipc.sock` Unix socket 路径（主代理监听，GUI 连接）。
pub fn ipc_socket_path(base: &std::path::Path) -> PathBuf {
    base.join("ipc.sock")
}

/// 确保所有子目录存在，不存在时递归创建。
///
/// 幂等——多次调用安全。
pub fn ensure_dirs(base: &std::path::Path) -> Result<(), IpcError> {
    for dir in [pending_dir(base), decisions_dir(base), locks_dir(base)] {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}
