//! 规则管理、preset 控制、evaluate 沙盒及灰名单 wire schema。
//!
//! 包含 sieve.list_rules / sieve.reload_config / sieve.set_paused /
//! sieve.set_preset / sieve.set_preset_overrides / sieve.evaluate /
//! sieve.list_graylist / sieve.remove_graylist 等 RPC 的参数和响应类型，
//! 对应 SPEC-005 §10–§11（控制面 RPC）。
//!
//! **零 IO 约束**：本文件仅 import serde / chrono / std，
//! 禁止引入 tokio / fd-lock / 任何 IO / 异步 / 运行时依赖。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::decision::SourceAgent;

// ── Preset 控制 ──────────────────────────────────────────────────────────────

/// `sieve.set_paused` 请求参数。
///
/// `minutes ∈ [0, 60]`：0 = 立刻恢复；上限 60（防止"事实上的关闭"）。
/// Critical 锁规则不受暂停影响（[PRD v2.0 §9 #3 #8]）。
///
/// 关联：ADR-013 §S.4 / SPEC-002 §9.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPausedRequest {
    pub minutes: u32,
}

/// `sieve.set_paused` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPausedResult {
    pub paused: bool,
    /// 暂停截止时间（UTC，SPEC-005 §4A）；`paused=false` 时为 None。
    #[serde(default, serialize_with = "crate::ts_serde::serialize_opt_utc_millis")]
    pub paused_until: Option<DateTime<Utc>>,
    /// 受暂停影响的 disposition 集合（Critical 锁规则的 disposition 永不出现在此列表）。
    pub applies_to: Vec<String>,
}

/// `sieve.set_preset` 请求参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPresetRequest {
    /// `"strict"` | `"default"` | `"relaxed"` | `"custom"`。
    pub mode: String,
}

/// `sieve.set_preset` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPresetResult {
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub applied_at: DateTime<Utc>,
}

/// 单条 preset override（custom preset 下逐规则覆盖）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetOverride {
    pub timeout_seconds: u32,
    /// `"block"` | `"allow"` | `"redact"`。
    pub default_on_timeout: String,
}

/// `sieve.set_preset_overrides` 请求参数。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SetPresetOverridesRequest {
    /// rule_id → override 映射。
    pub overrides: std::collections::HashMap<String, PresetOverride>,
}

/// 单条被拒绝的 override。
///
/// `reason ∈ { "critical_lock" | "unknown_rule" | "invalid_value" }`（ADR-013 §S.4）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedOverride {
    pub rule_id: String,
    pub reason: String,
}

/// `sieve.set_preset_overrides` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPresetOverridesResult {
    pub applied: Vec<String>,
    pub rejected: Vec<RejectedOverride>,
}

/// `sieve.reload_config` 请求参数（空对象）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReloadConfigRequest {}

/// `sieve.reload_config` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReloadConfigResult {
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub reloaded_at: DateTime<Utc>,
    pub system_rules_count: u32,
    pub user_rules_count: u32,
    /// 用户规则 lint 错误清单（仅警告，不阻断）。
    pub user_rules_errors: Vec<String>,
}

// ── Evaluate 沙盒 ────────────────────────────────────────────────────────────

/// evaluate sandbox 的内容种类。
///
/// 跟 `sieve_rules::ContentKind` 对应，IPC 层独立定义避免循环依赖。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluateContentKind {
    RawText,
    ToolUseInput,
    ModelResponse,
}

/// evaluate sandbox 的方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluateDirection {
    Outbound,
    Inbound,
}

/// `sieve.evaluate` 请求参数。
///
/// payload 上限 64KB（daemon 端校验），超过返回 -32003 payload_too_large。
/// 关联：ADR-013 §S.4。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateRequest {
    pub direction: EvaluateDirection,
    pub content_kind: EvaluateContentKind,
    /// 触发此次 evaluate 的上游 agent（SPEC-005 §5.7）。
    ///
    /// 旧版本发来 `"claude-code"` 字符串时，serde 无法匹配任何 SourceAgent 变体，
    /// 因为 `SourceAgent` 使用 snake_case（`"claude"` / `"open_claw"` 等）。
    /// 为保持向后兼容，字段类型维持为 `SourceAgent`；旧 GUI 应更新为 `"claude"`。
    #[serde(default)]
    pub source_agent: SourceAgent,
    pub payload: String,
}

/// 单条 evaluate 命中。
///
/// **敏感数据保护**：critical_lock 规则命中时，`matched_pattern_summary` 仅含规则类型摘要
/// （如 "BIP39 with checksum match"），daemon 端**禁止**回填 matched_canonical 或原 payload 片段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateMatch {
    pub rule_id: String,
    /// `"system"` | `"user"`。
    pub rule_kind: String,
    /// `"critical"` | `"high"` | `"medium"` | `"low"`。
    pub severity: String,
    pub disposition: super::decision::Disposition,
    pub matched_pattern_summary: String,
    pub fields_triggered: Vec<String>,
    /// `"allow"` | `"deny"` | `"redact_and_allow"`，daemon 模拟决策。
    pub would_decision: String,
    #[serde(default)]
    pub would_recommendation: Option<EvaluateRecommendation>,
}

/// evaluate 的 daemon 推荐结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateRecommendation {
    pub decision: String,
    /// `"high"` | `"medium"` | `"low"`。
    pub confidence: String,
    pub reason: String,
}

/// `sieve.evaluate` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateResult {
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub evaluated_at: DateTime<Utc>,
    pub matches: Vec<EvaluateMatch>,
    /// 未命中的规则 ID 抽样（不保证完整列表）。
    #[serde(default)]
    pub no_match: Vec<String>,
}

// ── 规则列表 ──────────────────────────────────────────────────────────────────

/// `sieve.list_rules` 响应（SPEC-005 §11A，Since v2.0 兼容扩展）。
///
/// 旧版本 daemon 不实现此方法，GUI 端应在收到 `-32601 method_not_found` 时降级
/// （禁用规则总览 UI，不崩溃）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRulesResult {
    /// 当前已加载的全部规则快照（系统规则 + 用户规则合并）。
    pub rules: Vec<RuleSummary>,
}

/// 单条规则摘要（SPEC-005 §11A `RuleSummary` 字段表）。
///
/// 11 个字段对应 SPEC 字段表，GUI 端可直接用于渲染 Detection 规则总览 Table。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSummary {
    /// 规则唯一标识（如 `"IN-CR-01"`；用户规则含 `"user:"` 前缀）。
    pub rule_id: String,
    /// UI 显示标题（fallback 为 rule_id，因系统 RuleEntry 只有 description 无独立 title）。
    pub title: String,
    /// 严重等级：`"low"` / `"medium"` / `"high"` / `"critical"`。
    pub severity: String,
    /// 流量方向：`"inbound"` / `"outbound"`。
    pub direction: String,
    /// 处置形式：`"gui_popup"` / `"auto_redact"` / `"status_bar"` / `"hook_terminal"`。
    pub disposition: String,
    /// 超时后默认处置（仅 `disposition == "gui_popup"` 时有意义）。
    #[serde(default)]
    pub default_on_timeout: Option<String>,
    /// 弹窗自动确认超时秒数（仅 `disposition == "gui_popup"` 时有意义）。
    #[serde(default)]
    pub timeout_seconds: Option<u32>,
    /// `true` 时 GUI 端禁止编辑此规则（Critical 级系统规则强制为 true）。
    pub critical_lock: bool,
    /// 规则是否启用。
    pub enabled: bool,
    /// 规则来源：`"system"` / `"user"`。
    pub rule_kind: String,
    /// 规则描述/备注，可能为 `null`。
    #[serde(default)]
    pub description: Option<String>,
}

// ── 灰名单管理 ────────────────────────────────────────────────────────────────

/// `sieve.list_graylist` 请求参数。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListGraylistRequest {
    /// 分页大小（None = 默认 50）。
    #[serde(default)]
    pub limit: Option<u32>,
    /// 分页游标（None = 第一页）。
    #[serde(default)]
    pub cursor: Option<String>,
}

/// 灰名单条目摘要（去敏感字段）。
///
/// **隐私保护**：daemon 返回时**不**包含 `fingerprint_inputs.matched_canonical`，避免 GUI
/// 间接拿到敏感片段。完整 inputs 查看路径推 v2.1。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraylistEntrySummary {
    pub fingerprint: String,
    pub rule_id: String,
    /// `"system"` | `"user"`。
    pub rule_kind: String,
    /// unix ms。
    pub added_at: i64,
    pub added_by: String,
    #[serde(default)]
    pub context_hint: Option<String>,
    pub match_count_since: u64,
    /// unix ms（None = 永不过期）。
    #[serde(default)]
    pub expires_at: Option<i64>,
}

/// `sieve.list_graylist` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListGraylistResult {
    pub entries: Vec<GraylistEntrySummary>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// `sieve.remove_graylist` 请求参数。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveGraylistRequest {
    pub fingerprint: String,
}

/// `sieve.remove_graylist` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveGraylistResult {
    pub removed: bool,
    pub audit_event_id: String,
}

// ── judge_tool_call（SPEC-005 §11C，Since v2.x 向后兼容扩展）────────────────────

/// `sieve.judge_tool_call` 请求参数。
///
/// client（如 agent 的 PreToolUse hook）把 agent **即将执行**的结构化工具调用喂给
/// daemon，daemon 跑入站规则引擎判危、命中 Critical 时走 GUI 弹窗确认，回裁决。
/// 让不解析上游响应体的 client 也能借 daemon 的规则引擎拿到入站危险工具拦截。
///
/// **v2 向后兼容扩展**：加 method 不 bump 协议版本（SPEC-005 §2 白名单管版本号非方法名）；
/// 不认识此方法的旧 daemon 返 `-32601`，client 据此 fail-closed。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeToolCallRequest {
    /// 工具名（不同 agent 取值不同，如 codex 的 `exec_command` / `apply_patch`）。
    pub tool_name: String,
    /// 工具输入对象（任意结构；daemon 对其全文扫描判危）。
    pub tool_input: serde_json::Value,
    /// 对应上游 response 的 tool_use_id（关联键，可空）。
    #[serde(default)]
    pub tool_use_id: String,
    /// 工具执行的工作目录（敏感路径判定用，可空）。
    #[serde(default)]
    pub cwd: String,
    /// 来源 agent（审计 / 规则上下文用）。默认 Unknown。
    #[serde(default)]
    pub source_agent: SourceAgent,
    /// client 愿意等待的最长毫秒数（即 client 内部 deadline）。
    ///
    /// daemon 据此 cap GUI 弹窗 timeout（取 `min(规则 timeout, timeout_ms)`），
    /// 保证在 client 放弃前回裁决；client 端到点仍须自行 fail-closed 兜底。
    /// `0` / 缺省 → daemon 用规则默认 timeout。
    #[serde(default)]
    pub timeout_ms: u32,
}

/// `sieve.judge_tool_call` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeToolCallResult {
    /// 裁决：`"allow"`（放行）| `"deny"`（拒绝）。
    ///
    /// 注：`"rewrite"`（改写工具输入做脱敏）为后续扩展，本期只产 allow/deny。
    pub verdict: String,
    /// 触发裁决的规则 ID（无命中时为 None）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    /// 给用户的拒绝原因（deny 时填；client 写进面向用户的提示）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
