//! 出站 / 入站规则命中后的 Detection 数据结构（关联 docs/design/data-model.md §Detection）。

use crate::protocol::unified_message::ContentSpan;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// 超时后的默认处置（sieve-core 内部镜像，避免对 sieve-rules 循环依赖）。
///
/// 与 `sieve_rules::manifest::DefaultOnTimeout` 语���完全一致；
/// engine_adapter 在构造 `Detection` 时从 `RuleEntry.default_on_timeout` 转换。
///
/// 关联：R3-#5 修复（IN-CR-01 disposition 配置生效）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultOnTimeout {
    /// 脱敏后放行。
    Redact,
    /// 阻断（fail-closed 默认）。
    Block,
    /// 放行（仅低优先级通知类规则）。
    Allow,
}

/// 严重等级（关联出站检测表 / Critical FP < 0.5% 公理）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// 低风险，仅审计。
    Low,
    /// 中风险，弹窗提示。
    Medium,
    /// 高风险，需确认。
    High,
    /// 严重风险，强制阻断（FP < 0.5% 公理）。
    Critical,
}

/// 命中处置动作（关联二维处置矩阵）。
///
/// v1.4 重构：按 `Disposition` 路由，废弃 `WarnConfirm`。
/// - `HookMark`：Hook 类命中，写 IPC pending 文件，SSE 流原样转发（Hook 路径）。
/// - `HoldForDecision`：GUI 类命中，hold 住 SSE 流等待用户决策（GUI 路径）。
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
    /// 关联 Hook 路径、SPEC-001。
    HookMark,
    /// GUI 类：hold 住 SSE 流，通过 IpcServer 等待用户决策（IN-CR-01/05、IN-GEN-04）。
    ///
    /// 关联 GUI 路径、SPEC-002。
    HoldForDecision {
        /// 请求唯一标识（UUIDv4），用于 IPC 匹配。
        request_id: uuid::Uuid,
        /// 等待超时秒数（来自 `RuleEntry.timeout_seconds`）。
        timeout_seconds: u32,
        /// 超时或 GUI 未连接时的默认处置（来自 `RuleEntry.default_on_timeout`）。
        ///
        /// 修 R3-#5：不再硬编码 Block，由规则配置决定。
        /// IN-CR-01 配置 `block`（fail-closed）；其他规则按 TOML 读取。
        default_on_timeout: DefaultOnTimeout,
    },
    /// 仅审计，不影响流量（StatusBar disposition）。
    MarkOnly,
    /// 静默记录（用于 dry_run / canary）。
    SilentLog,
}

/// 命中内容来源（关联 Pipeline 节点）。
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
    /// 来源 channel 标识（来自 `X-Sieve-Source-Channel` 请求头）。
    ///
    /// 用于 IN-GEN-06 运行时提级逻辑：当 source_channel 属于不可信外部 channel
    /// （WhatsApp / Slack / Telegram / Discord / iMessage 等）时，severity 提级为 Critical。
    ///
    /// `serde(default)` 保证旧序列化格式向后兼容。
    #[serde(default)]
    pub source_channel: Option<String>,
    /// 嵌套调用链深度（来自 `X-Sieve-Origin` 请求头，解析后计数）。
    ///
    /// 0 = 直接调用；> 0 = 经过中间层转发。超过阈值（如 3）时可作为额外风险信号。
    ///
    /// `serde(default)` 保证向后兼容。
    #[serde(default)]
    pub origin_chain_depth: usize,
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
