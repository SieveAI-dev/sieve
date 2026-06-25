//! sieve-policy 错误类型。
//!
//! 库 crate 使用 `thiserror`，禁止 `anyhow`（CLAUDE.md Rust 规范）。

use thiserror::Error;

/// sieve-policy 顶层错误。
#[derive(Debug, Error)]
pub enum PolicyError {
    /// 文件权限不符合要求（须 0600 for file / 0700 for dir）。
    ///
    /// 关联：no-follow symlink + 文件权限校验。
    #[error("file permissions error: {0}")]
    FilePermissions(String),

    /// 符号链接被拒绝（no-follow symlink 策略）。
    ///
    /// 防止 `ln -s /etc/passwd ~/.sieve/rules/user.toml` 攻击向量。
    #[error("symlink rejected: {0}")]
    SymlinkRejected(String),

    /// TOML 解析失败。
    #[error("toml parse error: {0}")]
    TomlParse(String),

    /// Lint 校验失败（违反 §5.5.3 11 类约束之一）。
    #[error("lint violation: {violations:?}")]
    LintViolations {
        /// 所有违规列表。
        violations: Vec<String>,
    },

    /// 灰名单条目 fingerprint 不一致（防止文件被人为修改）。
    #[error("graylist fingerprint mismatch: stored={stored}, computed={computed}")]
    FingerprintMismatch {
        /// 文件中存储的 fingerprint。
        stored: String,
        /// 重新计算的 fingerprint。
        computed: String,
    },

    /// 尝试将 Critical/fail-closed 规则加入灰名单（Critical 锁）。
    #[error("critical rule cannot be graylisted: {rule_id}")]
    CriticalRuleNotGraylistable {
        /// 被拒绝的规则 ID。
        rule_id: String,
    },

    /// 用户引擎编译失败（vectorscan pattern 编译错误）。
    #[error("user engine compile error: {0}")]
    EngineCompile(String),

    /// IO 错误。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// sieve-policy Result 别名。
pub type PolicyResult<T> = Result<T, PolicyError>;
