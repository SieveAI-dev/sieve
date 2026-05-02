//! 灰名单存储（PRD v2.0 §5.4.2）。
//!
//! 灰名单允许用户对非 Critical 规则命中选择"永久允许此次场景"（Allow + Remember）。
//! 每条灰名单一个 JSON 文件，存放于 `~/.sieve/decisions/<fingerprint>.json`。
//! 文件名是 fingerprint hex digest，不直接暴露 rule_id。
//!
//! # Critical 锁约束（PRD §5.4.2 / §9 #3 #8 #14）
//!
//! - 写入前**必须**验证 rule_id 不在 `critical_lock.rs::FAIL_CLOSED_RULES`
//! - Critical 规则命中**永不允许** Remember（GUI 端同步禁用 checkbox）
//! - 任何形式绕过 Critical 锁视为 P0 安全漏洞

use crate::error::{PolicyError, PolicyResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sieve_rules::critical_lock::is_fail_closed;
use std::path::Path;

/// fingerprint 计算的输入字段（PRD §5.4.2 `fingerprint_inputs`）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintInputs {
    /// 命中的规则 ID。
    pub rule_id: String,
    /// 命中的规范化片段（去空白、统一大小写后的命中文本）。
    pub matched_canonical: String,
    /// 工具名（如 `"Bash"`，无则 `""`）。
    pub tool_name: String,
    /// 协议（`"anthropic"` / `"openai"`）。
    pub protocol: String,
    /// 内容类型（`"tool_use_input"` 等）。
    pub content_kind: String,
    /// 调用方 Agent（如 `"claude-code"`，未知则 `""`）。
    pub source_agent: String,
}

/// 灰名单条目（PRD §5.4.2 schema）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraylistEntry {
    /// schema 版本（当前 = 1）。
    pub schema_version: u32,
    /// fingerprint 版本（当前 = 1，用于 breaking schema 升级）。
    pub fingerprint_version: u32,
    /// 命中的规则 ID。
    pub rule_id: String,
    /// 规则版本（如 `"v1.5.4"`，用于 cache invalidation）。
    pub rule_version: String,
    /// sha256 64 字符 hex fingerprint。
    pub fingerprint: String,
    /// fingerprint 计算输入。
    pub fingerprint_inputs: FingerprintInputs,
    /// 决策（当前仅 `"allow"`）。
    pub decision: String,
    /// 过期时间（`null` 表示永不过期）。
    pub expires_at: Option<DateTime<Utc>>,
    /// 添加时间（Unix 毫秒时间戳）。
    pub added_at: i64,
    /// 添加来源（`"gui_user_decision"` 等）。
    pub added_by: String,
    /// 用户备注（GUI 表单输入）。
    #[serde(default)]
    pub context_hint: Option<String>,
    /// 自添加以来该 fingerprint 被查询到的次数。
    #[serde(default)]
    pub match_count_since: u64,
    /// 关联 audit 事件 ID（UUID v4）。
    pub audit_event_id: String,
}

/// 计算 fingerprint（sha256 hex，64 字符）。
///
/// 对 [`FingerprintInputs`] 做 JSON 序列化后取 sha256。
/// JSON key 按字典序排列（serde 默认 struct 顺序保证一致性）。
pub fn compute_fingerprint(inputs: &FingerprintInputs) -> String {
    use sha2::{Digest, Sha256};
    // 用 serde_json 序列化保证结构一致（字段顺序 = struct 定义顺序）
    let canonical =
        serde_json::to_string(inputs).unwrap_or_else(|_| format!("{:?}", inputs.rule_id));
    let hash = Sha256::digest(canonical.as_bytes());
    hex::encode(hash)
}

/// 添加灰名单条目。
///
/// # Critical 锁
///
/// `entry.rule_id` 在 `FAIL_CLOSED_RULES` 中时返回 [`PolicyError::CriticalRuleNotGraylistable`]。
///
/// # 写入安全
///
/// - 先写 `<fingerprint>.json.tmp`，再 atomic rename 到 `<fingerprint>.json`
/// - 文件权限 `0600`，目录权限 `0700`（Unix only）
/// - no-follow symlink 校验
pub fn add_entry(dir: &Path, entry: GraylistEntry) -> PolicyResult<()> {
    // Critical 锁：禁止将 fail-closed 规则加入灰名单（PRD §5.4.2）
    if is_fail_closed(&entry.rule_id) {
        return Err(PolicyError::CriticalRuleNotGraylistable {
            rule_id: entry.rule_id.clone(),
        });
    }

    ensure_dir(dir)?;

    let filename = format!("{}.json", entry.fingerprint);
    let final_path = dir.join(&filename);
    let tmp_path = dir.join(format!("{}.json.tmp", entry.fingerprint));

    let json = serde_json::to_string_pretty(&entry)?;
    std::fs::write(&tmp_path, &json)?;

    // 设文件权限 0600（Unix only）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600))?;
    }

    // Atomic rename（防止 daemon reload 读到部分写入）
    std::fs::rename(&tmp_path, &final_path)?;

    tracing::debug!(
        "graylist: added entry fingerprint={} rule_id={}",
        entry.fingerprint,
        entry.rule_id
    );
    Ok(())
}

/// 查询灰名单条目（通过 fingerprint hex 字符串）。
///
/// 找到后**重新计算 fingerprint 并校验一致性**，不一致返回
/// [`PolicyError::FingerprintMismatch`]（防文件被人为修改，PRD §5.4.2）。
pub fn lookup(dir: &Path, fingerprint: &str) -> PolicyResult<Option<GraylistEntry>> {
    let path = dir.join(format!("{fingerprint}.json"));
    if !path.exists() {
        return Ok(None);
    }

    // no-follow symlink 校验
    let meta = path.symlink_metadata()?;
    if meta.file_type().is_symlink() {
        return Err(PolicyError::SymlinkRejected(format!(
            "graylist entry is a symlink: {:?}",
            path
        )));
    }

    let content = std::fs::read_to_string(&path)?;
    let entry: GraylistEntry = serde_json::from_str(&content)?;

    // 重新计算 fingerprint 验证一致性（防人为修改，PRD §5.4.2）
    let computed = compute_fingerprint(&entry.fingerprint_inputs);
    if computed != entry.fingerprint {
        return Err(PolicyError::FingerprintMismatch {
            stored: entry.fingerprint.clone(),
            computed,
        });
    }

    // 校验 fingerprint 与文件名一致
    if entry.fingerprint != fingerprint {
        return Err(PolicyError::FingerprintMismatch {
            stored: fingerprint.to_string(),
            computed: entry.fingerprint.clone(),
        });
    }

    Ok(Some(entry))
}

/// 列出目录下所有灰名单条目（按 added_at 倒序）。
///
/// 实现细节：
/// - 仅枚举 `<dir>` 顶层 `*.json` 文件，不递归子目录。
/// - 对每个文件做 no-follow symlink 校验 + fingerprint 重新计算校验；
///   损坏 / 篡改 / 解析失败的文件**跳过**（写 WARN 日志），不抛错——
///   保证一个坏文件不会让整个 list 操作失败（GUI list_graylist 调用方期望 fail-soft）。
/// - 返回结果按 `added_at` 倒序（最近添加的在前）。
///
/// 关联：ADR-013 §S.4 sieve.list_graylist。
pub fn list_entries(dir: &Path) -> PolicyResult<Vec<GraylistEntry>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    // no-follow symlink 校验
    let meta = dir.symlink_metadata()?;
    if meta.file_type().is_symlink() {
        return Err(PolicyError::SymlinkRejected(format!(
            "decisions directory is a symlink: {:?}",
            dir
        )));
    }

    let mut entries: Vec<GraylistEntry> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("graylist: read_dir entry failed: {e}");
                continue;
            }
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
            continue;
        };
        if ext != "json" {
            continue;
        }

        // no-follow symlink 校验
        let item_meta = match path.symlink_metadata() {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("graylist: stat {} failed: {e}", path.display());
                continue;
            }
        };
        if item_meta.file_type().is_symlink() {
            tracing::warn!(
                "graylist: skipping symlink {} (no-follow policy)",
                path.display()
            );
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("graylist: read {} failed: {e}", path.display());
                continue;
            }
        };
        let parsed: GraylistEntry = match serde_json::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("graylist: parse {} failed: {e}", path.display());
                continue;
            }
        };

        // fingerprint 完整性校验：重新计算并与文件名一致
        let computed = compute_fingerprint(&parsed.fingerprint_inputs);
        if computed != parsed.fingerprint {
            tracing::warn!(
                "graylist: fingerprint mismatch for {} (stored={}, computed={}); skipping",
                path.display(),
                parsed.fingerprint,
                computed
            );
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if stem != parsed.fingerprint {
            tracing::warn!(
                "graylist: filename {} does not match fingerprint {}; skipping",
                stem,
                parsed.fingerprint
            );
            continue;
        }

        entries.push(parsed);
    }

    entries.sort_by(|a, b| b.added_at.cmp(&a.added_at));
    Ok(entries)
}

/// 删除灰名单条目。
///
/// 返回 `true` 表示文件存在并已删除，`false` 表示文件不存在（幂等）。
pub fn remove_entry(dir: &Path, fingerprint: &str) -> PolicyResult<bool> {
    let path = dir.join(format!("{fingerprint}.json"));
    if !path.exists() {
        return Ok(false);
    }

    // no-follow symlink 校验
    let meta = path.symlink_metadata()?;
    if meta.file_type().is_symlink() {
        return Err(PolicyError::SymlinkRejected(format!(
            "graylist entry is a symlink: {:?}",
            path
        )));
    }

    std::fs::remove_file(&path)?;
    tracing::debug!("graylist: removed entry fingerprint={}", fingerprint);
    Ok(true)
}

/// 确保目录存在，并设置正确权限（0700 on Unix）。
fn ensure_dir(dir: &Path) -> PolicyResult<()> {
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }

    // no-follow symlink 校验
    let meta = dir.symlink_metadata()?;
    if meta.file_type().is_symlink() {
        return Err(PolicyError::SymlinkRejected(format!(
            "decisions directory is a symlink: {:?}",
            dir
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_inputs(rule_id: &str) -> FingerprintInputs {
        FingerprintInputs {
            rule_id: rule_id.into(),
            matched_canonical: "test_match".into(),
            tool_name: "Bash".into(),
            protocol: "anthropic".into(),
            content_kind: "tool_use_input".into(),
            source_agent: "claude-code".into(),
        }
    }

    fn make_entry(rule_id: &str) -> GraylistEntry {
        let inputs = make_inputs(rule_id);
        let fp = compute_fingerprint(&inputs);
        GraylistEntry {
            schema_version: 1,
            fingerprint_version: 1,
            rule_id: rule_id.into(),
            rule_version: "v2.0".into(),
            fingerprint: fp,
            fingerprint_inputs: inputs,
            decision: "allow".into(),
            expires_at: None,
            added_at: Utc::now().timestamp_millis(),
            added_by: "gui_user_decision".into(),
            context_hint: None,
            match_count_since: 0,
            audit_event_id: "test-uuid".into(),
        }
    }

    #[test]
    fn compute_fingerprint_deterministic() {
        let inputs = make_inputs("IN-GEN-04");
        let fp1 = compute_fingerprint(&inputs);
        let fp2 = compute_fingerprint(&inputs);
        assert_eq!(fp1, fp2, "fingerprint must be deterministic");
        assert_eq!(fp1.len(), 64, "sha256 hex should be 64 chars");
    }

    #[test]
    fn fingerprint_changes_on_input_change() {
        let inputs1 = make_inputs("IN-GEN-04");
        let mut inputs2 = make_inputs("IN-GEN-04");
        inputs2.matched_canonical = "different_match".into();
        assert_ne!(compute_fingerprint(&inputs1), compute_fingerprint(&inputs2));
    }

    #[test]
    fn add_and_lookup_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let entry = make_entry("IN-GEN-04");
        let fp = entry.fingerprint.clone();

        add_entry(tmp.path(), entry).unwrap();

        let found = lookup(tmp.path(), &fp).unwrap();
        assert!(found.is_some(), "entry should be found after add");
        assert_eq!(found.unwrap().rule_id, "IN-GEN-04");
    }

    #[test]
    fn lookup_missing_returns_none() {
        let tmp = TempDir::new().unwrap();
        let result = lookup(
            tmp.path(),
            "nonexistent_fingerprint_64chars_aaaabbbbccccdddd1234",
        )
        .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn remove_entry_returns_true_when_exists() {
        let tmp = TempDir::new().unwrap();
        let entry = make_entry("IN-GEN-04");
        let fp = entry.fingerprint.clone();
        add_entry(tmp.path(), entry).unwrap();
        assert!(remove_entry(tmp.path(), &fp).unwrap());
    }

    #[test]
    fn remove_entry_returns_false_when_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(!remove_entry(tmp.path(), "nonexistent_fingerprint_64chars").unwrap());
    }

    #[test]
    fn remove_then_lookup_returns_none() {
        let tmp = TempDir::new().unwrap();
        let entry = make_entry("MY-USER-RULE");
        // MY-USER-RULE 不在 FAIL_CLOSED_RULES 中，可以加入灰名单
        let fp = entry.fingerprint.clone();
        add_entry(tmp.path(), entry).unwrap();
        remove_entry(tmp.path(), &fp).unwrap();
        let found = lookup(tmp.path(), &fp).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn critical_rule_cannot_be_graylisted() {
        let tmp = TempDir::new().unwrap();
        // OUT-01 在 FAIL_CLOSED_RULES 中
        let entry = make_entry("OUT-01");
        let err = add_entry(tmp.path(), entry).unwrap_err();
        assert!(
            matches!(err, PolicyError::CriticalRuleNotGraylistable { .. }),
            "expected CriticalRuleNotGraylistable, got: {err:?}"
        );
    }

    #[test]
    fn fingerprint_mismatch_detected() {
        let tmp = TempDir::new().unwrap();
        let entry = make_entry("MY-USER-RULE");
        let fp = entry.fingerprint.clone();
        add_entry(tmp.path(), entry.clone()).unwrap();

        // 直接修改文件内容，破坏 fingerprint 一致性
        let path = tmp.path().join(format!("{fp}.json"));
        let content = std::fs::read_to_string(&path).unwrap();
        let tampered = content.replace(
            &entry.fingerprint_inputs.matched_canonical,
            "tampered_match",
        );
        std::fs::write(&path, tampered).unwrap();

        let err = lookup(tmp.path(), &fp).unwrap_err();
        assert!(
            matches!(err, PolicyError::FingerprintMismatch { .. }),
            "expected FingerprintMismatch after tampering, got: {err:?}"
        );
    }

    #[test]
    #[cfg(unix)]
    fn graylist_file_has_correct_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = TempDir::new().unwrap();
        let entry = make_entry("IN-GEN-04");
        let fp = entry.fingerprint.clone();
        add_entry(tmp.path(), entry).unwrap();

        let path = tmp.path().join(format!("{fp}.json"));
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "graylist file must have 0600 permissions");
    }
}
