//! sieve-core 错误类型 (thiserror，禁止 anyhow，见 .cursorrules §3.2)

use thiserror::Error;

/// sieve-core 顶层错误。
#[derive(Debug, Error)]
pub enum SieveCoreError {
    /// 协议解析失败 (AnthropicRequest 反序列化等)。
    #[error("protocol parse error: {0}")]
    Protocol(String),

    /// 上游 HTTP 请求构造或转发失败。
    #[error("forwarder error: {0}")]
    Forwarder(String),

    /// SSE 透传失败。
    #[error("sse passthrough error: {0}")]
    Sse(String),

    /// IO 错误。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// hyper 错误（包装为 String 以避免暴露内部类型）。
    #[error("hyper error: {0}")]
    Hyper(String),

    /// rustls TLS 配置错误。
    #[error("tls config error: {0}")]
    TlsConfig(String),
}

#[cfg(feature = "forwarder")]
impl From<hyper::Error> for SieveCoreError {
    fn from(e: hyper::Error) -> Self {
        Self::Hyper(e.to_string())
    }
}

impl From<serde_json::Error> for SieveCoreError {
    fn from(e: serde_json::Error) -> Self {
        Self::Protocol(e.to_string())
    }
}

/// sieve-core Result 别名。
pub type SieveCoreResult<T> = Result<T, SieveCoreError>;
