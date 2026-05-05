//! Audit（审计）管理 wire schema。
//!
//! 包含 sieve.purge_history 请求/响应类型，
//! 对应 SPEC-005 §11B（审计清除 RPC）。
//!
//! **零 IO 约束**：本文件仅 import serde / std，
//! 禁止引入 tokio / fd-lock / 任何 IO / 异步 / 运行时依赖。

use serde::{Deserialize, Serialize};

/// `sieve.purge_history` 请求参数（SPEC-005 §11B，Since v2.0 兼容扩展）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeHistoryRequest {
    /// GUI 端 Touch ID 通过的时刻（UTC，Unix ms）；用于审计，不作为幂等 key。
    pub confirmed_at: i64,
}

/// `sieve.purge_history` 响应（SPEC-005 §11B）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeHistoryResult {
    /// daemon 实际执行删除完成的时刻（UTC，Unix ms）。
    pub purged_at: i64,
    /// 本次删除的 audit event 行数；`0` 表示历史本就为空，视为成功。
    pub rows_deleted: u64,
}
