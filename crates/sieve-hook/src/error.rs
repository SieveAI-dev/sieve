/// pending 文件读取阶段的错误。
///
/// 独立定义（不依赖 sieve-ipc）以保持 sieve-hook 零重依赖目标。
/// 关联：SPEC-001 §4（hook 决策流程）。
pub enum PendingError {
    /// pending 文件不存在——Sieve 代理未标记此请求，可 fail-open。
    NotFound,
    /// pending 文件存在但 created_at > stale 阈值，fail-closed。
    Stale,
    /// JSON 解析失败，fail-closed。
    ParseError(String),
    /// 其他 IO 错误。
    IoError(String),
}
