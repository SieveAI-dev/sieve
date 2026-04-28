use thiserror::Error;

/// IPC 层错误枚举。
///
/// 关联规格：ADR-013（IPC 协议）、SPEC-001（sieve-hook 文件协议）。
#[derive(Debug, Error)]
pub enum IpcError {
    /// Unix socket 绑定或连接失败。
    #[error("socket error: {0}")]
    Socket(#[from] std::io::Error),

    /// JSON 序列化 / 反序列化失败。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// 请求在规定超时内未收到决策响应。
    #[error("decision timeout for request {request_id}")]
    Timeout { request_id: uuid::Uuid },

    /// pending 文件已超过 stale 阈值（10 分钟），视为过期拒绝。
    ///
    /// fail-closed：过期请求不允许放行，防止残留文件被重放。
    #[error("pending file is stale (created_at too old) for request {request_id}")]
    StalePending { request_id: uuid::Uuid },

    /// pending 文件不存在——此请求未经代理标记，可 fail-open。
    #[error("pending file not found for request {request_id}")]
    PendingNotFound { request_id: uuid::Uuid },

    /// 文件加锁失败。
    #[error("file lock error: {0}")]
    FileLock(String),

    /// $HOME 环境变量缺失，无法确定 sieve_home 路径。
    #[error("$HOME environment variable is not set")]
    HomeNotFound,

    /// JSON-RPC 响应中携带了错误对象。
    #[error("json-rpc error {code}: {message}")]
    JsonRpcError { code: i64, message: String },

    /// 对端发送了无法识别的 JSON-RPC method 或响应格式异常。
    #[error("unexpected json-rpc response: {0}")]
    UnexpectedResponse(String),
}
