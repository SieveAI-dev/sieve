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

/// JSON-RPC 2.0 错误码常量（ADR-013 Supplement 2026-05-02 §S.2）。
///
/// 标准段（-32700 ~ -32600）保留给 JSON-RPC 协议本身；
/// -32000 ~ -32099 为 Sieve 自定义实现段。
pub mod rpc_codes {
    // ── JSON-RPC 标准段 ────────────────────────────────────────
    /// 请求无效（缺字段 / 类型错）。
    pub const INVALID_REQUEST: i64 = -32600;
    /// 方法未找到。
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// 参数无效。
    pub const INVALID_PARAMS: i64 = -32602;
    /// 服务端内部错误。
    pub const INTERNAL_ERROR: i64 = -32603;

    // ── Sieve 自定义段（ADR-013 §S.2）──────────────────────────
    /// 客户端协议版本不被接受。
    pub const PROTOCOL_VERSION_MISMATCH: i64 = -32000;
    /// 操作触碰 critical_lock 名单（ADR-021 防线二）。
    pub const CRITICAL_LOCK_VIOLATED: i64 = -32001;
    /// daemon 忙（reload / restart 进行中）。
    pub const DAEMON_BUSY: i64 = -32002;
    /// payload 超过 evaluate 64KB 上限。
    pub const PAYLOAD_TOO_LARGE: i64 = -32003;
    /// list / remove graylist 找不到目标 fingerprint。
    pub const UNKNOWN_FINGERPRINT: i64 = -32004;
    /// 当前 paused 状态不允许此操作（保留，目前为空集）。
    pub const UNSUPPORTED_IN_PAUSED: i64 = -32005;
}
