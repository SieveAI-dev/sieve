//! 出站 / 入站规则命中后的 Detection 数据结构（关联 docs/design/data-model.md §Detection）。

use crate::protocol::unified_message::ContentSpan;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// 严重等级（关联 PRD §5.1 / §9 公理 12 Critical FP < 0.5%）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// 低风险，仅审计。
    Low,
    /// 中风险，弹窗提示。
    Medium,
    /// 高风险，需确认。
    High,
    /// 严重风险，强制阻断（FP < 0.5%，PRD §9 公理 12）。
    Critical,
}

/// 命中处置动作（关联 PRD v1.4 §5.4 / ADR-016 二维处置矩阵）。
///
/// v1.4 重构：按 `Disposition` 路由，废弃 `WarnConfirm`。
/// - `HookMark`：Hook 类命中，写 IPC pending 文件，SSE 流原样转发（ADR-014 §Hook 路径）。
/// - `HoldForDecision`：GUI 类命中，hold 住 SSE 流等待用户决策（ADR-014 §GUI 路径）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// 直接拦截（极端场景 / 出站 Critical fail-closed）。
    Block,
    /// 自动脱敏：替换为 `[REDACTED:<rule_id>]` 占位符（AutoRedact disposition，OUT-01~05/12）。
    Redact {
        /// 替换用占位符文本。
        placeholder: String,
    },
    /// Hook 类：写 IPC pending 文件，SSE 流原样转发（IN-CR-02~04、IN-GEN-01~03）。
    ///
    /// 关联 ADR-014 §Hook 路径、SPEC-001。
    HookMark,
    /// GUI 类：hold 住 SSE 流，通过 IpcServer 等待用户决策（IN-CR-01/05、IN-GEN-04）。
    ///
    /// 关联 ADR-014 §GUI 路径、SPEC-002。
    HoldForDecision {
        /// 请求唯一标识（UUIDv4），用于 IPC 匹配。
        request_id: uuid::Uuid,
        /// 等待超时秒数（来自 `RuleEntry.timeout_seconds`）。
        timeout_seconds: u32,
    },
    /// 仅审计，不影响流量（StatusBar disposition）。
    MarkOnly,
    /// 静默记录（用于 dry_run / canary）。
    SilentLog,
}

/// 命中内容来源（关联 PRD §6.2 Pipeline 节点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentSource {
    /// 出站用户消息文本。
    OutboundUserText,
    /// 出站系统提示文本。
    OutboundSystemText,
    /// 入站 assistant 消息文本（Week 3 起使用）。
    InboundAssistantText,
    /// 入站工具调用 input（Week 3 起使用）。
    InboundToolUseInput,
}

/// 单次规则命中的完整记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    /// 命中事件 UUID（用于审计日志关联）。
    pub id: Uuid,
    /// 命中规则 ID（如 OUT-01）。
    pub rule_id: String,
    /// 严重等级。
    pub severity: Severity,
    /// 处置动作。
    pub action: Action,
    /// 内容来源。
    pub source: ContentSource,
    /// 命中内容字节偏移（half-open [start, end)）。
    pub span: ContentSpan,
    /// 已脱敏的证据片段（便于审计追溯，**绝不存原始密钥**）。
    pub evidence_truncated: String,
    /// 命中指纹（用于 .sieveignore 匹配）。
    pub fingerprint: String,
}

/// 计算命中指纹（关联 docs/design/data-model.md §155-161）。
///
/// 算法：`SHA-256("{rule_id}:{normalized_content}")[..16]` 转 hex。
/// `normalized_content`：UTF-8 trim + 密钥类截断到前 32 字节。
pub fn fingerprint(rule_id: &str, content: &str) -> String {
    let normalized = content.trim();
    // Phase 1 简化：直接截断到 32 字节（UTF-8 字符边界安全截断）
    let truncated = if normalized.len() <= 32 {
        normalized.to_string()
    } else {
        let mut end = 32;
        while !normalized.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        normalized[..end].to_string()
    };
    let mut hasher = Sha256::new();
    hasher.update(rule_id.as_bytes());
    hasher.update(b":");
    hasher.update(truncated.as_bytes());
    let hash = hasher.finalize();
    hex_encode(&hash[..8]) // 16 hex chars = 8 bytes
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_16_hex_chars() {
        let fp = fingerprint("OUT-01", "sk-ant-fake");
        assert_eq!(fp.len(), 16);
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn same_input_same_fingerprint() {
        let fp1 = fingerprint("OUT-01", "  sk-ant-fake  ");
        let fp2 = fingerprint("OUT-01", "sk-ant-fake");
        assert_eq!(fp1, fp2); // trim 应让二者相同
    }

    #[test]
    fn different_rule_id_different_fingerprint() {
        let fp1 = fingerprint("OUT-01", "secret");
        let fp2 = fingerprint("OUT-02", "secret");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn long_content_truncated() {
        let fp1 = fingerprint("OUT-01", &"a".repeat(40));
        let fp2 = fingerprint("OUT-01", &"a".repeat(32));
        assert_eq!(fp1, fp2); // 截断到 32 字节后相同
    }

    #[test]
    fn severity_serde_lowercase() {
        let s = serde_json::to_string(&Severity::Critical).unwrap();
        assert_eq!(s, "\"critical\"");
    }

    #[test]
    fn action_block_serde() {
        let a = Action::Block;
        let s = serde_json::to_string(&a).unwrap();
        assert!(s.contains("block"));
    }
}
