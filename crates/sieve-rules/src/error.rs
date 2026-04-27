//! sieve-rules 错误类型（thiserror，禁止 anyhow）。

use thiserror::Error;

/// sieve-rules 顶层错误。
#[derive(Debug, Error)]
pub enum SieveRulesError {
    /// 规则文件解析失败（manifest JSON / tar.zst 解压失败）。
    #[error("manifest parse error: {0}")]
    Manifest(String),

    /// vectorscan 引擎错误（模式编译失败 / 匹配失败）。
    #[error("engine error: {0}")]
    Engine(String),

    /// Ed25519 签名验证失败。
    #[error("signature verification failed: {0}")]
    Signature(String),

    /// IO 错误。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<serde_json::Error> for SieveRulesError {
    fn from(e: serde_json::Error) -> Self {
        Self::Manifest(e.to_string())
    }
}

/// sieve-rules Result 别名。
pub type SieveRulesResult<T> = Result<T, SieveRulesError>;
