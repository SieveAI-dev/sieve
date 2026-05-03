//! `sieve.request_decision` wire DTO。
//!
//! SPEC-005 §6.0 要求 daemon 在序列化 `sieve.request_decision` 时经过显式 wire DTO
//! 适配层，而非直接序列化内部 `DecisionRequest` struct。
//!
//! 本模块定义两种 wire 形态：
//! - [`RequestDecisionWire`]：`detections.len() == 1` 时，字段平铺到顶层（§6.1.1）。
//! - [`MergedRequestDecisionWire`]：`detections.len() > 1` 时，顶层含 `merged: true` +
//!   `issues[]` 数组（§6.1.2）。
//!
//! 内部 `DecisionRequest` struct 不变——wire DTO 是新增转换层，不替换内部结构。
//!
//! ### §6.0 field mapping
//!
//! | daemon 内部字段 | wire 字段 | 说明 |
//! |---|---|---|
//! | `created_at` | `received_at_daemon` | P2-2：字段重命名（P1-5 实施） |
//! | `detections[0].rule_id` | 顶层 `rule_id` | 单 issue 平铺 |
//! | `detections[0].title` | 顶层 `title` | 单 issue 平铺 |
//! | `detections[0].severity` | 顶层 `severity` | 单 issue 平铺 |
//! | `detections[0].disposition` | 顶层 `disposition` | 单 issue 平铺 |
//! | `detections[0].details` | 顶层 `context` | 单 issue 平铺，可为 null |
//! | `detections > 1` | `merged: true` + `issues[]` | 多 issue 合并形式 |
//!
//! 关联：SPEC-005 §6.0 / §6.1 / §6.1.1 / §6.1.2 / PROGRESS P1-5 P2-2 P2-4。

use chrono::SecondsFormat;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::protocol::{
    DecisionRequest, DefaultOnTimeout, Disposition, OriginHop, Severity, SourceAgent,
};

// ── 辅助序列化 ───────────────────────────────────────────────────────────────

/// 序列化 `Severity` 为 spec §5.1 lowercase 字符串。
fn severity_str(s: &Severity) -> &'static str {
    match s {
        Severity::Critical => "critical",
        Severity::High => "high",
        Severity::Medium => "medium",
        Severity::Low => "low",
    }
}

/// 序列化 `DefaultOnTimeout` 为 spec §5.5 snake_case 字符串。
fn default_on_timeout_str(d: &DefaultOnTimeout) -> &'static str {
    match d {
        DefaultOnTimeout::Block => "block",
        DefaultOnTimeout::Allow => "allow",
        DefaultOnTimeout::Redact => "redact",
    }
}

/// 序列化 `Disposition` 为 spec §5.3 snake_case 字符串。
fn disposition_str(d: &Disposition) -> &'static str {
    match d {
        Disposition::GuiPopup => "gui_popup",
        Disposition::HookTerminal => "hook_terminal",
        Disposition::AutoRedact => "auto_redact",
        Disposition::StatusBar => "status_bar",
    }
}

/// 从 `SourceAgent` 推导 wire 字符串（§5.7）。
fn source_agent_str(a: &SourceAgent) -> &'static str {
    match a {
        SourceAgent::Claude => "claude",
        SourceAgent::OpenClaw => "open_claw",
        SourceAgent::Hermes => "hermes",
        SourceAgent::Unknown => "unknown",
    }
}

// ── issue 子对象（多 issue 合并形式用）───────────────────────────────────────

/// 单 issue 在多 issue 合并请求的 `issues[]` 数组中的表示（§6.1.2）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueWire {
    /// 本次合并请求内唯一，形如 `"i-1"` / `"i-2"`（§6.1.2）。
    pub issue_id: String,
    pub rule_id: String,
    pub title: String,
    /// severity enum（§5.1 lowercase）。
    pub severity: String,
    /// 单 issue 维度 allow_remember（daemon 计算）。
    pub allow_remember: bool,
    /// 渲染上下文（§6.1.3 context.template）；null 时 GUI 走通用渲染。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    /// daemon 推荐（§6.1.4）；null 时缺失。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<Value>,
}

// ── 单 issue 平铺 wire 形态（§6.1.1）────────────────────────────────────────

/// `sieve.request_decision` 单 issue wire 形态（`merged: false`，字段平铺到顶层）。
///
/// 对应 SPEC-005 §6.1.1 字段表。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDecisionWire {
    pub request_id: Uuid,
    pub rule_id: String,
    pub title: String,
    /// severity enum（§5.1 lowercase）。
    pub severity: String,
    /// direction（`"inbound"` / `"outbound"`），来自 `DecisionRequest.wire_direction`。
    pub direction: String,
    /// disposition enum（§5.3 snake_case）。
    pub disposition: String,
    pub timeout_seconds: u32,
    /// default_on_timeout enum（§5.5 snake_case）。
    pub default_on_timeout: String,
    pub allow_remember: bool,
    /// 固定 `false`（单 issue 形式）。
    pub merged: bool,
    /// daemon 收到检测请求的时刻（UTC 毫秒 + Z 后缀，SPEC-005 §4A；P2-2 字段重命名 created_at→received_at_daemon）。
    ///
    /// Wire 层使用 `String` 以保证格式可控（ISO 8601 + 毫秒 + Z 后缀）。
    pub received_at_daemon: String,
    /// 渲染上下文（§6.1.3）；null 或缺失时 GUI 走通用渲染。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    /// daemon 推荐（§6.1.4）；null 或缺失时 GUI 默认拒绝按钮。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<Value>,
    /// 来源 agent（§5.7 snake_case）。
    pub source_agent: String,
    /// sub-agent 嵌套调用链（§6.1.1 OriginHop[]）。
    pub origin_chain: Vec<OriginHop>,
    /// OpenClaw 跨通道来源（§6.1.1）；null 或缺失时 GUI 忽略。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_channel: Option<String>,
    /// `X-Sieve-Origin` header 真实嵌套深度；null 时回退到 `origin_chain.length`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit_chain_depth: Option<u32>,
}

// ── 多 issue 合并 wire 形态（§6.1.2）────────────────────────────────────────

/// `sieve.request_decision` 多 issue 合并 wire 形态（`merged: true` + `issues[]`）。
///
/// 对应 SPEC-005 §6.1.2。顶层保留聚合字段，单 issue 字段（`rule_id` / `context`）
/// 不发（避免 GUI 误用），详情在 `issues[]` 里。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedRequestDecisionWire {
    pub request_id: Uuid,
    /// daemon 生成的合并标题（如"Sieve 检测到 N 个安全问题"，§6.1.2）。
    pub title: String,
    /// 取所有 issue 最严重的 severity（§6.1.2 顶层取值规则）。
    pub severity: String,
    /// 全部 issue 同方向（§6.1.2 要求方向不同时拆成两个独立请求）。
    pub direction: String,
    /// 全部 issue 同 disposition（§6.1.2 要求 disposition 不同时拆请求）。
    pub disposition: String,
    /// 取所有 issue 最小 timeout（§6.1.2）。
    pub timeout_seconds: u32,
    /// 取最严格 default_on_timeout（§6.1.2：block > redact > allow）。
    pub default_on_timeout: String,
    /// 任一 issue allow_remember=false → 顶层强制 false（§6.1.2）。
    pub allow_remember: bool,
    /// 固定 `true`。
    pub merged: bool,
    /// daemon 收到检测请求的时刻（UTC 毫秒 + Z 后缀，§4A；P2-2 字段重命名）。
    pub received_at_daemon: String,
    pub source_agent: String,
    pub origin_chain: Vec<OriginHop>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit_chain_depth: Option<u32>,
    /// issue 明细数组（§6.1.2）。
    pub issues: Vec<IssueWire>,
}

// ── 转换枚举 ─────────────────────────────────────────────────────────────────

/// `sieve.request_decision` wire DTO，根据 detection 数量选择形态。
///
/// - `Single`：`detections.len() == 1`（§6.1.1）
/// - `Merged`：`detections.len() > 1`（§6.1.2）
/// - `Empty`：`detections.len() == 0`（异常降级，按单 issue 处理，rule_id = "unknown"）
#[derive(Debug, Clone)]
pub enum RequestDecisionWireKind {
    Single(RequestDecisionWire),
    Merged(MergedRequestDecisionWire),
}

impl RequestDecisionWireKind {
    /// 将内部 `DecisionRequest` 转换为 wire DTO。
    ///
    /// `direction` 参数：`"inbound"` / `"outbound"`，调用方根据上下文传入。
    ///
    /// **严重等级比较顺序**（SPEC-005 §6.1.2）：
    /// `critical > high > medium > low`（按枚举 discriminant 反向）
    pub fn from_request(req: &DecisionRequest, direction: &str) -> Self {
        if req.detections.len() <= 1 {
            let (rule_id, title, severity, disposition, context) =
                if let Some(d) = req.detections.first() {
                    (
                        d.rule_id.clone(),
                        d.title.clone(),
                        severity_str(&d.severity).to_owned(),
                        disposition_str(&d.disposition).to_owned(),
                        // `details` 映射到 wire `context`（§6.1.3）
                        if d.details.is_null() {
                            None
                        } else {
                            Some(d.details.clone())
                        },
                    )
                } else {
                    // detections 空：降级占位符
                    (
                        "unknown".to_owned(),
                        "Unknown".to_owned(),
                        "low".to_owned(),
                        "gui_popup".to_owned(),
                        None,
                    )
                };

            RequestDecisionWireKind::Single(RequestDecisionWire {
                request_id: req.request_id,
                rule_id,
                title,
                severity,
                direction: direction.to_owned(),
                disposition,
                timeout_seconds: req.timeout_seconds,
                default_on_timeout: default_on_timeout_str(&req.default_on_timeout).to_owned(),
                allow_remember: req.allow_remember,
                merged: false,
                received_at_daemon: req.created_at.to_rfc3339_opts(SecondsFormat::Millis, true),
                context,
                // `DetectionPayload` 暂无 recommendation 字段（§6.1.4）；
                // 将来 daemon 可在此注入聚合推荐。
                recommendation: None,
                source_agent: source_agent_str(&req.source_agent).to_owned(),
                origin_chain: req.origin_chain.clone(),
                source_channel: req.source_channel.clone(),
                explicit_chain_depth: req.explicit_chain_depth.map(|d| d as u32),
            })
        } else {
            // 多 issue：按 §6.1.2 聚合规则
            let severity = req
                .detections
                .iter()
                .map(|d| d.severity)
                .max_by_key(severity_rank)
                .map(|s| severity_str(&s).to_owned())
                .unwrap_or_else(|| "low".to_owned());

            // DetectionPayload 没有 per-issue timeout；直接使用顶层 timeout_seconds。
            let timeout_seconds = req.timeout_seconds;

            // allow_remember 顶层值：任一 issue allow_remember=false → 顶层强制 false（§6.1.2）
            // 当前 DecisionRequest 无 per-issue allow_remember，统一用顶层值。
            let allow_remember = req.allow_remember;

            let issues: Vec<IssueWire> = req
                .detections
                .iter()
                .enumerate()
                .map(|(i, d)| IssueWire {
                    issue_id: format!("i-{}", i + 1),
                    rule_id: d.rule_id.clone(),
                    title: d.title.clone(),
                    severity: severity_str(&d.severity).to_owned(),
                    // 单 issue 的 allow_remember 与顶层相同（DecisionRequest 无 per-issue 字段）
                    allow_remember: req.allow_remember,
                    context: if d.details.is_null() {
                        None
                    } else {
                        Some(d.details.clone())
                    },
                    recommendation: None,
                })
                .collect();

            let disposition = req
                .detections
                .first()
                .map(|d| disposition_str(&d.disposition).to_owned())
                .unwrap_or_else(|| "gui_popup".to_owned());

            let title = format!("Sieve 检测到 {} 个安全问题", req.detections.len());

            RequestDecisionWireKind::Merged(MergedRequestDecisionWire {
                request_id: req.request_id,
                title,
                severity,
                direction: direction.to_owned(),
                disposition,
                timeout_seconds,
                default_on_timeout: default_on_timeout_str(&req.default_on_timeout).to_owned(),
                allow_remember,
                merged: true,
                received_at_daemon: req.created_at.to_rfc3339_opts(SecondsFormat::Millis, true),
                source_agent: source_agent_str(&req.source_agent).to_owned(),
                origin_chain: req.origin_chain.clone(),
                source_channel: req.source_channel.clone(),
                explicit_chain_depth: req.explicit_chain_depth.map(|d| d as u32),
                issues,
            })
        }
    }

    /// 序列化为 `serde_json::Value`。
    pub fn to_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        match self {
            RequestDecisionWireKind::Single(w) => serde_json::to_value(w),
            RequestDecisionWireKind::Merged(w) => serde_json::to_value(w),
        }
    }
}

/// severity → 排序权重（越高越严重，用于 max_by_key）。
fn severity_rank(s: &Severity) -> u8 {
    match s {
        Severity::Low => 0,
        Severity::Medium => 1,
        Severity::High => 2,
        Severity::Critical => 3,
    }
}

// ── 单元测试 ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{DetectionPayload, Disposition, Severity, SourceAgent};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_detection(
        rule_id: &str,
        severity: Severity,
        disposition: Disposition,
    ) -> DetectionPayload {
        DetectionPayload {
            rule_id: rule_id.to_owned(),
            severity,
            disposition,
            title: format!("{rule_id} 标题"),
            one_line_summary: "摘要".to_owned(),
            details: serde_json::json!({}),
        }
    }

    fn make_req(detections: Vec<DetectionPayload>) -> DecisionRequest {
        DecisionRequest {
            request_id: Uuid::now_v7(),
            created_at: Utc::now(),
            timeout_seconds: 60,
            default_on_timeout: DefaultOnTimeout::Block,
            detections,
            source_agent: SourceAgent::Claude,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: Some(0),
            allow_remember: false,
        }
    }

    /// 单 issue：wire 平铺字段，无 `issues`，`merged: false`。
    #[test]
    fn single_issue_wire_flattened() {
        let req = make_req(vec![make_detection(
            "IN-CR-05",
            Severity::Critical,
            Disposition::GuiPopup,
        )]);
        let wire = RequestDecisionWireKind::from_request(&req, "inbound");
        let val = wire.to_value().expect("serialize");

        assert_eq!(val["rule_id"], "IN-CR-05", "rule_id must be top-level");
        assert_eq!(val["severity"], "critical");
        assert_eq!(val["direction"], "inbound");
        assert_eq!(val["disposition"], "gui_popup");
        assert_eq!(val["merged"], false);
        assert!(
            val.get("issues").is_none(),
            "single issue must not have issues[]"
        );
        assert!(
            val.get("received_at_daemon").is_some(),
            "received_at_daemon must exist (P2-2)"
        );
        assert!(
            val.get("created_at").is_none(),
            "old field created_at must not appear (P2-2)"
        );
    }

    /// 多 issue：wire 含 `merged: true` + `issues[N]`，顶层无 `rule_id`。
    #[test]
    fn multi_issue_wire_merged() {
        let req = make_req(vec![
            make_detection("IN-CR-05", Severity::Critical, Disposition::GuiPopup),
            make_detection("IN-GEN-04", Severity::High, Disposition::GuiPopup),
        ]);
        let wire = RequestDecisionWireKind::from_request(&req, "inbound");
        let val = wire.to_value().expect("serialize");

        assert_eq!(val["merged"], true, "merged must be true for multi-issue");
        let issues = val["issues"].as_array().expect("issues must be array");
        assert_eq!(issues.len(), 2, "must have 2 issue entries");
        assert_eq!(issues[0]["issue_id"], "i-1");
        assert_eq!(issues[0]["rule_id"], "IN-CR-05");
        assert_eq!(issues[1]["issue_id"], "i-2");
        assert_eq!(issues[1]["rule_id"], "IN-GEN-04");
        // 顶层 severity 取最高
        assert_eq!(val["severity"], "critical");
        // 顶层无 rule_id（避免 GUI 误用）
        assert!(
            val.get("rule_id").is_none(),
            "merged form must not have top-level rule_id"
        );
    }

    /// `received_at_daemon` 字段存在，`created_at` 字段不出现（P2-2）。
    #[test]
    fn field_name_received_at_daemon_not_created_at() {
        let req = make_req(vec![make_detection(
            "IN-CR-01",
            Severity::High,
            Disposition::GuiPopup,
        )]);
        let wire = RequestDecisionWireKind::from_request(&req, "inbound");
        let val = wire.to_value().expect("serialize");

        let json_str = serde_json::to_string(&val).expect("to_string");
        assert!(
            json_str.contains("received_at_daemon"),
            "wire must contain received_at_daemon (P2-2)"
        );
        assert!(
            !json_str.contains("\"created_at\""),
            "wire must NOT contain created_at (P2-2 rename)"
        );
    }

    /// 时间戳带 `Z` 后缀 + 毫秒精度（P2-3 联动）。
    #[test]
    fn timestamp_has_z_suffix_and_millis() {
        let req = make_req(vec![make_detection(
            "IN-CR-02",
            Severity::Critical,
            Disposition::HookTerminal,
        )]);
        let wire = RequestDecisionWireKind::from_request(&req, "inbound");
        let val = wire.to_value().expect("serialize");
        let ts = val["received_at_daemon"].as_str().expect("must be string");

        assert!(
            ts.ends_with('Z'),
            "timestamp must end with Z suffix, got: {ts}"
        );
        // 含毫秒（格式 T12:34:56.789Z，.后3位）
        let has_millis = ts.contains('.') && {
            let dot_pos = ts.rfind('.').unwrap();
            ts[dot_pos + 1..].trim_end_matches('Z').len() == 3
        };
        assert!(has_millis, "timestamp must have 3-digit millis, got: {ts}");
    }
}
