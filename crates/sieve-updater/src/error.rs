//! Error types for sieve-updater（客户端实现）.

use thiserror::Error;

/// All errors that can occur inside sieve-updater.
///
/// Lib crates mandate thiserror (no anyhow).
#[derive(Debug, Error)]
pub enum UpdaterError {
    /// The current operating system is not supported.
    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),

    /// I/O error (filesystem operations).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// HTTP transport error (hyper).
    #[error("http error: {0}")]
    Http(String),

    /// TLS configuration error.
    #[error("tls error: {0}")]
    Tls(String),

    /// JSON (de)serialization error.
    #[error("json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// SHA-256 digest did not match the expected value.
    #[error("sha256 mismatch: expected {expected}, got {actual}")]
    Sha256Mismatch { expected: String, actual: String },

    /// Ed25519 signature verification failed.
    #[error("ed25519 signature verification failed: {0}")]
    Ed25519Failed(String),

    /// All retry attempts exhausted without success.
    #[error("retry exhausted after {attempts} attempts: {last_error}")]
    RetryExhausted { attempts: u32, last_error: String },

    /// zstd decompression failed.
    #[error("zstd decompression failed: {0}")]
    DecompressFailed(String),

    /// Downloaded payload exceeded the configured maximum size.
    #[error("response too large: got {size} bytes, max {max} bytes")]
    ResponseTooLarge { size: usize, max: usize },

    /// The manifest `version` string is not a safe single path component
    /// (path separator / parent ref / empty). Rejected fail-closed before any
    /// filesystem use, because `version` is server-controlled and the ed25519
    /// signature is fail-open while the signing trust key is not yet configured.
    #[error("invalid version string (path-unsafe): {0}")]
    InvalidVersion(String),

    /// The manifest offered a rules-pack version that is <= the one already
    /// installed. Rejected to prevent a **signed-downgrade replay**: an attacker
    /// (or a stale mirror) serving an older but still-signature-valid pack could
    /// otherwise roll back a detection rule. Checked *after* signature
    /// verification, so `attempted` is an authentic-but-stale package. Non-fatal
    /// at the runner: it logs a warning and keeps the current pack.
    #[error("rules version downgrade rejected: server offered {attempted}, but {installed} is already installed")]
    VersionDowngrade {
        /// The version the manifest offered (authentic but <= installed).
        attempted: String,
        /// The version currently installed (kept).
        installed: String,
    },
}
