//! 用户规则文件加载（PRD v2.0 §5.5.1 §5.5.4）。
//!
//! 负责从 `~/.sieve/rules/user.toml` 加载用户规则，并做文件系统安全校验。
//!
//! # 文件系统安全约束（PRD §5.5.3-C）
//!
//! - 文件权限必须 `0600`（owner-only），目录权限 `0700`
//! - 拒绝符号链接（no-follow symlink，防 `ln -s /etc/passwd` 攻击）
//! - 文件不存在时返回空 [`UserRulesFile`]（daemon 正常启动）

use crate::error::{PolicyError, PolicyResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 规则扫描方向（PRD v2.0 §5.5）。
///
/// 控制用户规则应用于哪一侧的流量：
/// - `Outbound`：仅扫出站请求（user prompt + system prompt）
/// - `Inbound`：仅扫入站响应（assistant text + tool_use）
/// - `Both`：两侧都扫（默认，向后兼容旧 user.toml）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleDirection {
    /// 仅扫出站请求（user prompt + system prompt）。
    Outbound,
    /// 仅扫入站响应（assistant text + tool_use）。
    Inbound,
    /// 两侧都扫（默认，向后兼容旧 user.toml）。
    Both,
}

impl Default for RuleDirection {
    fn default() -> Self {
        Self::Both
    }
}

/// 用户规则文件顶层结构（PRD §5.5.4 user.toml schema）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserRulesFile {
    /// 文件 schema 版本（当前 = 1）。
    pub schema_version: u32,
    /// 文件创建时间（UTC）。
    pub created_at: DateTime<Utc>,
    /// 文件最后修改时间（UTC）。
    pub updated_at: DateTime<Utc>,
    /// 规则条目列表。
    #[serde(default)]
    pub rules: Vec<UserRuleEntry>,
}

impl Default for UserRulesFile {
    fn default() -> Self {
        Self {
            schema_version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            rules: Vec::new(),
        }
    }
}

/// 单条用户规则（PRD §5.5.4）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserRuleEntry {
    /// 规则 ID（用户自定义，不能与系统规则 ID 重复）。
    pub id: String,
    /// 规则描述。
    pub description: String,
    /// vectorscan 兼容 PCRE 子集模式串。
    pub pattern: String,
    /// 严重等级（用户规则只允许 high/medium/low，禁止 critical）。
    pub severity: String,
    /// 处置动作（用户规则只允许 warn/mark/ask，禁止 block）。
    pub action: String,
    /// 快速预过滤关键词（必填且非空，PRD §5.5.3-B）。
    pub keywords: Vec<String>,
    /// 允许豁免的停用词列表（每个 >= 4 字节，PRD §5.5.3-B）。
    #[serde(default)]
    pub allowlist_stopwords: Vec<String>,
    /// 处置形式（用户规则禁止 hook_terminal，PRD §5.5.3-A）。
    #[serde(default)]
    pub disposition: Option<String>,
    /// 规则扫描方向（PRD v2.0 §5.5）。
    ///
    /// 缺省为 `Both`（两侧都扫），向后兼容无此字段的旧 user.toml。
    #[serde(default)]
    pub direction: RuleDirection,
    /// 是否启用（false 表示禁用但保留条目）。
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 规则添加时间（UTC）。
    pub added_at: DateTime<Utc>,
    /// 规则来源（"manual" | "imported"）。
    pub added_by: String,
}

fn default_enabled() -> bool {
    true
}

/// 从 `path` 加载用户规则文件。
///
/// # 文件不存在
///
/// 返回空的 [`UserRulesFile`]（不报错），允许 daemon 在没有用户规则文件时正常启动。
///
/// # 安全校验（PRD §5.5.3-C）
///
/// 1. 拒绝符号链接
/// 2. 文件权限必须 `0600`
/// 3. 目录权限必须 `0700`（如果目录存在）
pub fn load_user_rules(path: &Path) -> PolicyResult<UserRulesFile> {
    // 文件不存在 → 返回空 UserRulesFile，daemon 正常启动（PRD §5.5.2.1）
    if !path.exists() {
        tracing::debug!(
            "user rules file not found at {:?}, using empty ruleset",
            path
        );
        return Ok(UserRulesFile::default());
    }

    // No-follow symlink 校验（PRD §5.5.3-C）
    let meta = path.symlink_metadata()?;
    if meta.file_type().is_symlink() {
        return Err(PolicyError::SymlinkRejected(format!(
            "user rules file is a symlink: {:?}",
            path
        )));
    }

    // 文件权限 0600 校验（仅 Unix，PRD §5.5.3-C）
    check_file_permissions(path, &meta)?;

    // 目录权限 0700 校验
    if let Some(dir) = path.parent() {
        if dir.exists() {
            let dir_meta = dir.symlink_metadata()?;
            if dir_meta.file_type().is_symlink() {
                return Err(PolicyError::SymlinkRejected(format!(
                    "user rules directory is a symlink: {:?}",
                    dir
                )));
            }
            check_dir_permissions(dir, &dir_meta)?;
        }
    }

    let content = std::fs::read_to_string(path)?;
    toml::from_str::<UserRulesFile>(&content).map_err(|e| PolicyError::TomlParse(e.to_string()))
}

/// 检查文件权限是否为 0600（Unix only）。
#[cfg(unix)]
fn check_file_permissions(path: &Path, meta: &std::fs::Metadata) -> PolicyResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = meta.permissions().mode() & 0o777;
    if mode != 0o600 {
        return Err(PolicyError::FilePermissions(format!(
            "file {:?} must have permissions 0600, got {:04o}",
            path, mode
        )));
    }
    Ok(())
}

/// 检查目录权限是否为 0700（Unix only）。
#[cfg(unix)]
fn check_dir_permissions(dir: &Path, meta: &std::fs::Metadata) -> PolicyResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = meta.permissions().mode() & 0o777;
    if mode != 0o700 {
        return Err(PolicyError::FilePermissions(format!(
            "directory {:?} must have permissions 0700, got {:04o}",
            dir, mode
        )));
    }
    Ok(())
}

/// 非 Unix 平台：跳过权限校验（Windows 推 Phase 3，ADR-006 Tier 2）。
#[cfg(not(unix))]
fn check_file_permissions(_path: &Path, _meta: &std::fs::Metadata) -> PolicyResult<()> {
    Ok(())
}

#[cfg(not(unix))]
fn check_dir_permissions(_dir: &Path, _meta: &std::fs::Metadata) -> PolicyResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn write_toml(dir: &TempDir, content: &str) -> std::path::PathBuf {
        let path = dir.path().join("user.toml");
        std::fs::write(&path, content).unwrap();
        #[cfg(unix)]
        {
            // 先设目录 0700
            std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
            // 再设文件 0600
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        path
    }

    const MINIMAL_TOML: &str = r#"
schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"

[[rules]]
id = "MY-TEST-RULE"
description = "test rule"
pattern = "secret"
severity = "high"
action = "warn"
keywords = ["secret"]
added_at = "2026-05-01T00:00:00Z"
added_by = "manual"
"#;

    #[test]
    fn load_missing_file_returns_empty() {
        let path = std::path::Path::new("/tmp/nonexistent_sieve_test_user_rules.toml");
        let result = load_user_rules(path).unwrap();
        assert_eq!(result.schema_version, 1);
        assert!(result.rules.is_empty());
    }

    #[test]
    fn load_valid_toml() {
        let tmp = TempDir::new().unwrap();
        let path = write_toml(&tmp, MINIMAL_TOML);
        let f = load_user_rules(&path).unwrap();
        assert_eq!(f.schema_version, 1);
        assert_eq!(f.rules.len(), 1);
        assert_eq!(f.rules[0].id, "MY-TEST-RULE");
        assert!(f.rules[0].enabled);
    }

    #[test]
    fn unknown_field_rejected() {
        let tmp = TempDir::new().unwrap();
        let content = r#"
schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"
unknown_field = "oops"
"#;
        let path = write_toml(&tmp, content);
        assert!(load_user_rules(&path).is_err());
    }

    #[test]
    #[cfg(unix)]
    fn rejects_wrong_file_permissions() {
        let tmp = TempDir::new().unwrap();
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let path = tmp.path().join("user.toml");
        std::fs::write(&path, MINIMAL_TOML).unwrap();
        // 故意设置 0644（非 0600）
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let err = load_user_rules(&path).unwrap_err();
        assert!(
            matches!(err, PolicyError::FilePermissions(_)),
            "expected FilePermissions error, got: {err:?}"
        );
    }

    #[test]
    #[cfg(unix)]
    fn rejects_symlink() {
        let tmp = TempDir::new().unwrap();
        let real_file = tmp.path().join("real.toml");
        std::fs::write(&real_file, MINIMAL_TOML).unwrap();
        let link = tmp.path().join("user.toml");
        std::os::unix::fs::symlink(&real_file, &link).unwrap();
        let err = load_user_rules(&link).unwrap_err();
        assert!(
            matches!(err, PolicyError::SymlinkRejected(_)),
            "expected SymlinkRejected error, got: {err:?}"
        );
    }
}
