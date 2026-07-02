//! Headless 决策查询与解决 wire schema。
//!
//! 包含 `sieve.list_pending`（只读枚举待决策）与 `sieve.resolve_decision`
//! （headless 解决单个待决策，A 方案授权）的参数和响应类型，
//! 对应 SPEC-005 §11D / §11E。
//!
//! **零 IO 约束**：本文件仅 import serde / chrono / uuid / std，
//! 禁止引入 tokio / fd-lock / 任何 IO / 异步 / 运行时依赖。
//!
//! ## A 方案授权（决策授权模型）
//!
//! headless CLI 对 `Critical` 类决策一律静默 deny；`High` 及以下允许 headless 批准。
//! 判定在 daemon 端按 `max_severity` 做（daemon 侧从 detections 计算并存入 pending 快照），
//! **不信 CLI 自报的 severity**。`resolve_decision` 的 `remember` 字段恒为 false
//! （不给 CLI 开永久白名单）。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::decision::{DecisionAction, DefaultOnTimeout, Severity, SourceAgent};

// ── list_pending ─────────────────────────────────────────────────────────────

/// `sieve.list_pending` 请求参数（空对象；过滤在 client 侧做，daemon 保持薄）。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListPendingRequest {}

/// 单条待决策在 `list_pending` 响应中的只读投影。
///
/// 从 daemon 侧 `DecisionRequest` 裁剪：只暴露 GUI/CLI 渲染与过滤所需字段，
/// 不含内部 responder / oneshot 等运行时对象。`max_severity` 是 daemon 侧
/// 从 detections 计算的权威值，A 方案授权门禁据此判定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSnapshot {
    /// 待决策请求 ID（与发出的 `request_decision` 一致）。
    pub request_id: Uuid,
    /// 本次请求涉及的最高严重等级（daemon 侧计算，A 方案门禁依据）。
    pub max_severity: Severity,
    /// 检测命中摘要列表（裁剪版，不含正则/offset 内部细节）。
    pub detections: Vec<PendingDetectionSummary>,
    /// 用户响应超时时长（秒，已钳制在 [30,120]）。
    pub timeout_seconds: u32,
    /// 超时后的默认决策。
    pub default_on_timeout: DefaultOnTimeout,
    /// 流量方向（`"inbound"` / `"outbound"`）。
    pub direction: String,
    /// 触发本次决策的来源 agent。
    pub source_agent: SourceAgent,
    /// 触发本次决策的 listener 上游 provider_id（多 listener 路由）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    /// daemon 收到检测、发起本次决策请求的时刻（UTC，SPEC-005 §4A）。
    #[serde(serialize_with = "crate::ts_serde::serialize_utc_millis")]
    pub created_at: DateTime<Utc>,
    /// 自 `created_at` 起已等待的秒数（daemon 应答时计算）。
    pub age_seconds: u64,
}

/// `list_pending` 中单条检测命中的摘要（GUI/CLI 渲染 + severity 过滤用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDetectionSummary {
    pub rule_id: String,
    pub severity: Severity,
    pub title: String,
    pub one_line_summary: String,
}

/// `sieve.list_pending` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPendingResult {
    /// 当前所有待决策的只读快照；空集返回 `[]`（空 ≠ 错误）。
    pub pending: Vec<PendingSnapshot>,
}

// ── resolve_decision ─────────────────────────────────────────────────────────

/// `sieve.resolve_decision` 请求参数（headless 解决单个待决策）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveDecisionRequest {
    /// 目标待决策请求 ID。
    pub request_id: Uuid,
    /// 请求的决策动作（`"allow"` / `"deny"` / `"redact_and_allow"`）。
    ///
    /// 注意：daemon 对 `Critical` 类的 `allow` / `redact_and_allow` 会**静默改写为 deny**
    /// （A 方案），`effective_decision` 反映实际处置。
    pub decision: DecisionAction,
    /// 决策理由（可选，映射到 CLI `--reason`，透传写入 audit context）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_hint: Option<String>,
}

/// `resolve_decision` 的处置状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolveStatus {
    /// 成功处置（`effective_decision` 反映实际动作，可能因 A 方案被改写）。
    Resolved,
    /// 目标 request_id 不存在（已超时 / 已被解决 / id 不存在，三种情况天然合一）。
    NotFound,
}

/// `sieve.resolve_decision` 响应。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveDecisionResult {
    /// 处置状态。
    pub status: ResolveStatus,
    /// 实际生效的决策动作；`status == not_found` 时为 `None`。
    ///
    /// A 方案：Critical 类的 `allow` / `redact_and_allow` 会被静默改写为 `deny`，
    /// 此字段反映改写后的实际值（可能 ≠ 请求的 `decision`）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_decision: Option<DecisionAction>,
}
