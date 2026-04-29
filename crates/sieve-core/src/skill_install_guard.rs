//! IN-CR-06 OpenClaw 动态 skill 安装检测（PRD v1.5 §4.6）。
//!
//! ## 设计说明
//!
//! OpenClaw 的 skill 动态安装流量形态：
//! 1. HTTP POST 到类似 `/openclaw/skills/install` 的 endpoint（Week 7 实测确认）。
//! 2. 请求 body 包含 skill manifest（含 source URL、作者、权限列表等）。
//!
//! 本模块实现**占位检测**：
//! - 路径匹配：`/openclaw/skills/install`（或 `/api/v1/skills/install` 等候选路径）
//! - Body 匹配：JSON 含 `"type"` 或 `"kind"` 字段值含 "skill"，且含 `"install"` 或 `"source"` 字段
//!
//! 任何命中都构造 IN-CR-06 Detection，fail-closed 等待用户确认。
//!
//! ## TODO（Week 7）
//!
//! - 实测 OpenClaw skill install 真实 HTTP endpoint 路径与 manifest schema
//! - 完善 manifest 解析：提取 `source_url`、`author`、`permissions` 到 Detection details
//! - 接入黑名单查询（source domain 黑名单、权限级别评分）
//!
//! 关联：PRD v1.5 §4.6 / ADR-016（处置矩阵）。

use crate::detection::{fingerprint, Action, ContentSource, Detection, Severity};
use crate::protocol::unified_message::ContentSpan;
use uuid::Uuid;

/// 不可信外部 channel 列表（PRD v1.5 §4.5）。
///
/// 当 IN-GEN-06 命中且 `source_channel` 在此列表中时，severity 从 High 提级为 Critical。
///
/// v1.5 第一版：硬编码白名单；v1.6 计划开放 GUI 配置。
pub const UNTRUSTED_CHANNELS: &[&str] = &[
    "whatsapp",
    "slack",
    "telegram",
    "discord",
    "imessage",
    "wechat",
    "line",
    "signal",
    "messenger",
    "teams",
    "sms",
];

/// OpenClaw skill 安装 endpoint 路径候选（Week 7 实测前占位）。
///
/// # TODO（Week 7）
///
/// 实测 OpenClaw 真实 API 路径后替换此列表。
const SKILL_INSTALL_PATH_PATTERNS: &[&str] = &[
    "/openclaw/skills/install",
    "/api/v1/skills/install",
    "/skills/install",
    "/mcp/install",
];

/// 检测请求路径是否疑似 OpenClaw skill 安装 endpoint。
///
/// # Examples
/// ```
/// use sieve_core::skill_install_guard::is_skill_install_path;
///
/// assert!(is_skill_install_path("/openclaw/skills/install"));
/// assert!(!is_skill_install_path("/v1/messages"));
/// ```
pub fn is_skill_install_path(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    SKILL_INSTALL_PATH_PATTERNS
        .iter()
        .any(|p| path_lower.contains(p))
}

/// 从 JSON body 检测是否含 skill manifest schema。
///
/// 判定依据：JSON 对象同时含以下任一特征组合：
/// 1. `type` 或 `kind` 字段值包含 "skill"
/// 2. 含 `install`、`source`、`manifest` 或 `plugin` 顶层字段
///
/// # TODO（Week 7）
///
/// 实测 manifest schema 后改为严格字段匹配。
fn body_looks_like_skill_manifest(body: &serde_json::Value) -> bool {
    let obj = match body.as_object() {
        Some(o) => o,
        None => return false,
    };

    // 判定 type/kind 字段
    let type_hint = obj
        .get("type")
        .or_else(|| obj.get("kind"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_lowercase().contains("skill"))
        .unwrap_or(false);

    // 判定 skill 安装相关字段
    let has_install_field = obj.contains_key("install")
        || obj.contains_key("source")
        || obj.contains_key("manifest")
        || obj.contains_key("plugin");

    type_hint || has_install_field
}

/// 解析 skill manifest 摘要（用于 Detection.evidence_truncated）。
///
/// 提取 `name`、`source`、`author` 字段（若存在）拼接为可读摘要。
/// 所有值截断到 64 字符，避免超长日志。
///
/// # TODO（Week 7）
///
/// 补充权限列表（`permissions`）解析与风险评分。
fn extract_manifest_summary(body: &serde_json::Value) -> String {
    let obj = match body.as_object() {
        Some(o) => o,
        None => return "[manifest unparsed]".to_string(),
    };

    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let source = obj
        .get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown-source");
    let author = obj
        .get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown-author");

    let summary = format!("skill='{name}' source='{source}' author='{author}'");
    if summary.len() > 128 {
        format!("{}...", &summary[..125])
    } else {
        summary
    }
}

/// 检查 HTTP 请求路径 + body JSON 是否疑似 OpenClaw skill 安装。
///
/// 返回 IN-CR-06 Detection 列表（0 或 1 个）。
///
/// # Arguments
/// - `path`：HTTP 请求路径（如 `/openclaw/skills/install`）
/// - `body`：请求 body 的 JSON 值（可以是 `serde_json::Value::Null` 若 body 不存在）
/// - `source`：内容来源（一般为 `ContentSource::InboundToolUseInput`）
///
/// # Errors
///
/// 本函数不产生 IO，不返回错误；若无法判定则返回空 Vec（fail-open，依靠路径匹配兜底）。
///
/// # TODO（Week 7）
///
/// 补充 manifest source URL 黑名单查询。
///
/// PRD v1.5 §4.6；关联 ADR-016。
pub fn check_openclaw_skill_install(
    path: &str,
    body: &serde_json::Value,
    source: ContentSource,
) -> Vec<Detection> {
    // 路径匹配或 body manifest 匹配，任一触发即构造 Detection
    let path_hit = is_skill_install_path(path);
    let body_hit = body_looks_like_skill_manifest(body);

    if !path_hit && !body_hit {
        return Vec::new();
    }

    let summary = extract_manifest_summary(body);
    let fp = fingerprint("IN-CR-06", &format!("{path}:{summary}"));

    vec![Detection {
        id: Uuid::new_v4(),
        rule_id: "IN-CR-06".into(),
        severity: Severity::Critical,
        action: Action::HoldForDecision {
            request_id: Uuid::new_v4(),
            timeout_seconds: 120,
            default_on_timeout: crate::detection::DefaultOnTimeout::Block,
        },
        source,
        span: ContentSpan { start: 0, end: 0 },
        evidence_truncated: summary,
        fingerprint: fp,
        source_channel: None,
        origin_chain_depth: 0,
    }]
}

/// 检查 source_channel 是否在不可信外部 channel 列表中（大小写不敏感）。
///
/// 用于 IN-GEN-06 运行时提级逻辑。
///
/// # Examples
/// ```
/// use sieve_core::skill_install_guard::is_untrusted_channel;
///
/// assert!(is_untrusted_channel("WhatsApp"));
/// assert!(is_untrusted_channel("SLACK"));
/// assert!(!is_untrusted_channel("internal-api"));
/// ```
pub fn is_untrusted_channel(channel: &str) -> bool {
    let lower = channel.to_lowercase();
    UNTRUSTED_CHANNELS.iter().any(|c| lower == *c)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_skill_install_path ─────────────────────────────────────────────────

    #[test]
    fn skill_path_openclaw_detected() {
        assert!(is_skill_install_path("/openclaw/skills/install"));
        assert!(is_skill_install_path("/OPENCLAW/SKILLS/INSTALL")); // case-insensitive
        assert!(is_skill_install_path("/api/v1/skills/install"));
        assert!(is_skill_install_path("/mcp/install"));
    }

    #[test]
    fn non_skill_path_not_detected() {
        assert!(!is_skill_install_path("/v1/messages"));
        assert!(!is_skill_install_path("/health"));
        assert!(!is_skill_install_path("/skills/list")); // list ≠ install
    }

    // ── body_looks_like_skill_manifest ────────────────────────────────────────

    #[test]
    fn body_with_skill_type_detected() {
        let body = serde_json::json!({
            "type": "skill",
            "name": "evil-skill",
            "source": "https://evil.com/skill.js"
        });
        assert!(body_looks_like_skill_manifest(&body));
    }

    #[test]
    fn body_with_source_field_detected() {
        let body = serde_json::json!({
            "name": "my-plugin",
            "source": "https://example.com/plugin",
            "version": "1.0"
        });
        assert!(body_looks_like_skill_manifest(&body));
    }

    #[test]
    fn normal_message_body_not_detected() {
        let body = serde_json::json!({
            "model": "claude-opus-4-5",
            "messages": [{"role": "user", "content": "hello"}]
        });
        assert!(!body_looks_like_skill_manifest(&body));
    }

    // ── check_openclaw_skill_install ──────────────────────────────────────────

    #[test]
    fn path_hit_produces_detection() {
        let body = serde_json::Value::Null;
        let dets = check_openclaw_skill_install(
            "/openclaw/skills/install",
            &body,
            ContentSource::InboundToolUseInput,
        );
        assert_eq!(dets.len(), 1);
        assert_eq!(dets[0].rule_id, "IN-CR-06");
        assert_eq!(dets[0].severity, Severity::Critical);
        assert!(matches!(
            dets[0].action,
            Action::HoldForDecision {
                timeout_seconds: 120,
                ..
            }
        ));
    }

    #[test]
    fn body_hit_produces_detection() {
        let body = serde_json::json!({
            "type": "skill",
            "name": "bad-skill",
            "author": "attacker",
            "source": "https://evil.com"
        });
        let dets =
            check_openclaw_skill_install("/v1/messages", &body, ContentSource::InboundToolUseInput);
        assert_eq!(dets.len(), 1);
        assert_eq!(dets[0].rule_id, "IN-CR-06");
    }

    #[test]
    fn no_hit_returns_empty() {
        let body = serde_json::json!({"model": "claude", "messages": []});
        let dets =
            check_openclaw_skill_install("/v1/messages", &body, ContentSource::InboundToolUseInput);
        assert!(dets.is_empty());
    }

    #[test]
    fn evidence_contains_manifest_summary() {
        let body = serde_json::json!({
            "type": "skill",
            "name": "test-skill",
            "author": "test-author",
            "source": "https://example.com/skill"
        });
        let dets = check_openclaw_skill_install(
            "/openclaw/skills/install",
            &body,
            ContentSource::InboundToolUseInput,
        );
        assert_eq!(dets.len(), 1);
        assert!(dets[0].evidence_truncated.contains("test-skill"));
        assert!(dets[0].evidence_truncated.contains("test-author"));
    }

    // ── is_untrusted_channel ──────────────────────────────────────────────────

    #[test]
    fn known_untrusted_channels() {
        for ch in &["whatsapp", "slack", "telegram", "discord", "imessage"] {
            assert!(is_untrusted_channel(ch), "{ch} should be untrusted channel");
        }
    }

    #[test]
    fn untrusted_channel_case_insensitive() {
        assert!(is_untrusted_channel("WhatsApp"));
        assert!(is_untrusted_channel("SLACK"));
        assert!(is_untrusted_channel("Telegram"));
    }

    #[test]
    fn trusted_or_unknown_channel_not_untrusted() {
        assert!(!is_untrusted_channel("internal-api"));
        assert!(!is_untrusted_channel(""));
        assert!(!is_untrusted_channel("email")); // email 不在列表
    }
}
