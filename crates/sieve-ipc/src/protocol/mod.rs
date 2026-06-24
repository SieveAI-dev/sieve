//! Wire schema 模块（SPEC-005 wire types 唯一权威源）。
//!
//! # 零 IO 硬约束
//!
//! 本模块及所有子模块**只能 import** `serde / chrono / uuid / std`，
//! **禁止 import** `tokio / hyper / fd-lock / bytes / memchr / tracing / 任何 IO crate`。
//!
//! 违反此约束会导致 protocol/ 将来升级为独立 crate 时出现编译错误。
//! 关联：sieve-ipc 内部模块化设计。
//!
//! # 修改流程
//!
//! 1. 先修改 `docs/specs/SPEC-005-ipc-protocol.md`（wire schema 权威源）
//! 2. 再改本目录代码，保持字段名 / 类型 100% 与 SPEC 一致
//! 3. 更新 CHANGELOG

pub mod audit;
pub mod decision;
pub mod envelope;
pub mod handshake;
pub mod health;
pub mod notify;
pub mod rules;

// ── 公开 re-export（保持 crate::protocol::* 的向后兼容 flat 接口）────────────

pub use audit::{PurgeHistoryRequest, PurgeHistoryResult};

pub use decision::{
    CancelReason, DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout,
    DetectionPayload, Disposition, OriginHop, RequestDecisionCanceledNotify, Severity, SourceAgent,
    UiPhase,
};

pub use handshake::{HelloParams, ReloadUserRules};

pub use health::{
    AuditDbSnapshot, GraylistSnapshot, HealthRequest, HealthResult, IpcSnapshot, ListenSnapshot,
    ListenerSnapshot, PresetSnapshot, RulesSnapshot,
};

pub use notify::{NotifyKind, PausedChangedNotify, PresetChangedNotify, StatusBarNotify};

pub use rules::{
    EvaluateContentKind, EvaluateDirection, EvaluateMatch, EvaluateRecommendation, EvaluateRequest,
    EvaluateResult, GraylistEntrySummary, JudgeToolCallRequest, JudgeToolCallResult,
    ListGraylistRequest, ListGraylistResult, ListRulesResult, PresetOverride, RejectedOverride,
    ReloadConfigRequest, ReloadConfigResult, RemoveGraylistRequest, RemoveGraylistResult,
    RuleSummary, SetPausedRequest, SetPausedResult, SetPresetOverridesRequest,
    SetPresetOverridesResult, SetPresetRequest, SetPresetResult,
};

/// JSON-RPC 2.0 envelope（向后兼容路径：`protocol::jsonrpc::Request` 等）。
pub mod jsonrpc {
    pub use super::envelope::{ErrorObject, Request, Response};
}
