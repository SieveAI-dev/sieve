//! Pipeline 节点（架构图 ②⑦）及 v1.4 统一 dispatch 入口。
//!
//! `dispatch` 根据 Detection 的 `action` 路由到：
//! - `Redact` → [`outbound_redact`] 脱敏路径（AutoRedact disposition）
//! - `HookMark` → [`inbound_hook`] 写 pending 文件（SSE 原样转发）
//! - `HoldForDecision` → [`inbound_hold`] hold 流 + keep-alive（GuiPopup disposition）
//! - `MarkOnly` / `SilentLog` → StatusBarOnly 透传
//!
//! `dispatch` 及 hold/hook 子模块仅在 `forwarder` feature 下编译（依赖 bytes + tokio async），
//! 与 `cargo fuzz --no-default-features` 场景隔离。
//!
//! 关联：ADR-014（双层防御）、ADR-016（二维处置矩阵）、PRD v1.4 §6.1 §6.7。

pub mod inbound;
pub mod outbound;
pub mod outbound_redact;
pub mod streaming;

// forwarder feature 下才编译 hold / hook（依赖 bytes + tokio async）
#[cfg(feature = "forwarder")]
pub mod inbound_hold;
#[cfg(feature = "forwarder")]
pub mod inbound_hook;

use crate::detection::Detection;
use crate::error::SieveCoreResult;
use crate::protocol::unified_message::UnifiedMessage;

pub use outbound_redact::{align_to_utf8_char_start, redact_body_bytes, RedactHit, RedactResult};

#[cfg(feature = "forwarder")]
pub use inbound_hold::{HoldError, HoldOutcome};
#[cfg(feature = "forwarder")]
pub use inbound_hook::HookError;

// ── Pipeline Node trait ──────────────────────────────────────────────────────

/// Pipeline 节点 trait。
///
/// Week 2 起 process 返回命中列表；Week 3 起入站节点也返回 Vec<Detection>
/// （地址替换 / 工具调用拦截）。
///
/// 关联架构图节点 ②（出站过滤）和节点 ⑦（入站过滤）。
pub trait PipelineNode: Send + Sync {
    /// 节点名（用于审计日志，需稳定不变）。
    fn name(&self) -> &str;

    /// 处理一个 UnifiedMessage，返回所有命中的 Detection 列表。
    ///
    /// # Errors
    /// 处理失败时返回对应 [`crate::error::SieveCoreError`]。
    fn process(&self, msg: &mut UnifiedMessage) -> SieveCoreResult<Vec<Detection>>;
}

// ── dispatch（仅 forwarder feature）─────────────────────────────────────────

#[cfg(feature = "forwarder")]
pub use dispatch_impl::{dispatch, Direction, DispatchResult, PipelineError};

#[cfg(feature = "forwarder")]
mod dispatch_impl {
    use std::sync::Arc;

    use bytes::Bytes;
    use thiserror::Error;
    use tokio::sync::mpsc;
    use uuid::Uuid;

    use crate::detection::{Action, Detection, Severity};
    use crate::pipeline::inbound_hold::{self, HoldError, HoldOutcome};
    use crate::pipeline::inbound_hook::HookError;
    use crate::pipeline::outbound_redact::{self, RedactHit};

    /// 流量方向。
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Direction {
        /// 出站（客户端 → Anthropic API）。
        Outbound,
        /// 入站（Anthropic API → 客户端）。
        Inbound,
    }

    /// Pipeline dispatch 专用错误。
    ///
    /// 关联 `.cursorrules §3.2`：库 crate 用 `thiserror`，禁 `anyhow`。
    #[derive(Debug, Error)]
    pub enum PipelineError {
        /// Hook 类 pending 文件写入失败。
        #[error("hook error: {0}")]
        Hook(#[from] HookError),
        /// GUI 类 hold 失败（IPC 错误）。
        #[error("hold error: {0}")]
        Hold(#[from] HoldError),
        /// IPC 服务未初始化（GuiPopup detection 但 ipc 参数为 None）。
        #[error("IPC server not initialized for GuiPopup detection")]
        IpcNotInitialized,
        /// keep-alive channel 未提供（GuiPopup detection 但 keep_alive_tx 参数为 None）。
        #[error("keep-alive channel not provided for GuiPopup detection")]
        KeepAliveChannelMissing,
    }

    /// `dispatch` 的返回值，指示 daemon 下一步动作。
    ///
    /// 关联 ADR-016 二维处置矩阵 / ADR-014 双层防御路径。
    #[derive(Debug)]
    pub enum DispatchResult {
        /// 透传原样 body / SSE 流（无任何命中，或 StatusBar 静默）。
        Passthrough,
        /// 改写 body bytes 后转发（出站 AutoRedact）。
        RewriteBody(Bytes),
        /// 用户允许（GUI 类 hold 后通过）→ daemon 继续转发剩余 SSE。
        AllowAfterHold,
        /// 用户拒绝（GUI 类 hold 后拒绝）→ daemon 截流注入 `sieve_blocked` event。
        DenyWithBlock(String),
        /// Hook 类已写 IPC pending 文件 → daemon 原样转发 SSE 流。
        HookMarked,
        /// StatusBar 静默通知（不打断流程）。
        StatusBarOnly,
    }

    /// 根据 detection 的 `action` 决定下一步动作，这是 daemon `proxy_inner` 调用的统一入口。
    ///
    /// # 路由优先级（高 → 低）
    /// `Block` > `HoldForDecision`（GuiPopup）> `HookMark`（HookTerminal）> `Redact`（AutoRedact）> `MarkOnly`
    ///
    /// 关联：ADR-016 §dispatch 路由、ADR-014 §双层防御。
    pub async fn dispatch(
        _direction: Direction,
        detections: Vec<Detection>,
        ipc: Option<Arc<sieve_ipc::IpcServer>>,
        request_id: Uuid,
        body_bytes: Option<&[u8]>,
        keep_alive_tx: Option<mpsc::Sender<Bytes>>,
    ) -> Result<DispatchResult, PipelineError> {
        if detections.is_empty() {
            return Ok(DispatchResult::Passthrough);
        }

        let mut has_block = false;
        let mut hold_detections: Vec<&Detection> = Vec::new();
        let mut hook_detections: Vec<&Detection> = Vec::new();
        let mut redact_hits: Vec<RedactHit> = Vec::new();
        let mut all_status_only = true;

        for d in &detections {
            match &d.action {
                Action::Block => {
                    has_block = true;
                    all_status_only = false;
                }
                Action::HoldForDecision { .. } => {
                    hold_detections.push(d);
                    all_status_only = false;
                }
                Action::HookMark => {
                    hook_detections.push(d);
                    all_status_only = false;
                }
                Action::Redact { .. } => {
                    redact_hits.push(RedactHit {
                        rule_id: d.rule_id.clone(),
                        start: d.span.start,
                        end: d.span.end,
                    });
                    all_status_only = false;
                }
                Action::MarkOnly | Action::SilentLog => {
                    // 静默 / 状态栏，不改变 all_status_only 的含义
                }
            }
        }

        if all_status_only {
            return Ok(DispatchResult::StatusBarOnly);
        }

        // Block 优先
        if has_block {
            let reason = detections
                .iter()
                .filter(|d| d.action == Action::Block)
                .map(|d| d.rule_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Ok(DispatchResult::DenyWithBlock(format!(
                "block（rules: {reason}）"
            )));
        }

        // GuiPopup：hold 流等待用户决策
        if !hold_detections.is_empty() {
            let ipc = ipc.ok_or(PipelineError::IpcNotInitialized)?;
            let ka_tx = keep_alive_tx.ok_or(PipelineError::KeepAliveChannelMissing)?;

            let (hold_request_id, timeout_seconds) = hold_detections
                .iter()
                .find_map(|d| {
                    if let Action::HoldForDecision {
                        request_id,
                        timeout_seconds,
                        default_on_timeout: _,
                    } = d.action
                    {
                        Some((request_id, timeout_seconds))
                    } else {
                        None
                    }
                })
                .unwrap_or((request_id, 60));

            use chrono::Utc;
            use sieve_ipc::protocol::DetectionPayload;

            let ipc_detections: Vec<DetectionPayload> = hold_detections
                .iter()
                .map(|d| DetectionPayload {
                    rule_id: d.rule_id.clone(),
                    severity: map_severity_to_ipc(d.severity),
                    disposition: sieve_ipc::Disposition::GuiPopup,
                    title: format!("检测命中：{}", d.rule_id),
                    one_line_summary: d.evidence_truncated.clone(),
                    details: serde_json::json!({}),
                })
                .collect();

            let ipc_req = sieve_ipc::DecisionRequest {
                request_id: hold_request_id,
                created_at: Utc::now(),
                timeout_seconds,
                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                detections: ipc_detections,
                source_agent: sieve_ipc::SourceAgent::Unknown,
                origin_chain: vec![],
                source_channel: None,
                explicit_chain_depth: None,
                allow_remember: false, // v2.0：Week 6 由 daemon 根据规则计算后传入
            };

            let outcome = inbound_hold::hold_and_decide(ipc, ipc_req, ka_tx).await?;
            return match outcome {
                // `remember` / `context_hint` 由 daemon 消费写灰名单（PRD §5.4.2），
                // dispatch 层只做放行 / 拒绝路由，不处理灰名单逻辑。
                HoldOutcome::Allow { .. } | HoldOutcome::RedactAndAllow { .. } => {
                    Ok(DispatchResult::AllowAfterHold)
                }
                HoldOutcome::Deny { reason } => Ok(DispatchResult::DenyWithBlock(reason)),
            };
        }

        // HookTerminal：写 pending 文件，SSE 原样转发
        if !hook_detections.is_empty() {
            use chrono::Utc;
            use sieve_ipc::protocol::DetectionPayload;

            let sieve_home = sieve_ipc::paths::sieve_home()
                .map_err(|e| PipelineError::Hook(HookError::Ipc(e)))?;

            let ipc_detections: Vec<DetectionPayload> = hook_detections
                .iter()
                .map(|d| DetectionPayload {
                    rule_id: d.rule_id.clone(),
                    severity: map_severity_to_ipc(d.severity),
                    disposition: sieve_ipc::Disposition::HookTerminal,
                    title: format!("检测命中：{}", d.rule_id),
                    one_line_summary: d.evidence_truncated.clone(),
                    details: serde_json::json!({}),
                })
                .collect();

            let ipc_req = sieve_ipc::DecisionRequest {
                request_id,
                created_at: Utc::now(),
                timeout_seconds: 60,
                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                detections: ipc_detections,
                source_agent: sieve_ipc::SourceAgent::Unknown,
                origin_chain: vec![],
                source_channel: None,
                explicit_chain_depth: None,
                allow_remember: false, // v2.0：Week 6 由 daemon 根据规则计算后传入
            };

            sieve_ipc::pending_file::write_pending(&ipc_req, &sieve_home)
                .map_err(|e| PipelineError::Hook(HookError::Ipc(e)))?;

            return Ok(DispatchResult::HookMarked);
        }

        // AutoRedact：脱敏 body bytes
        if !redact_hits.is_empty() {
            let body = body_bytes.unwrap_or(&[]);
            let result = outbound_redact::redact_body_bytes(body, &redact_hits);
            return Ok(DispatchResult::RewriteBody(Bytes::from(result.body)));
        }

        Ok(DispatchResult::Passthrough)
    }

    /// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
    fn map_severity_to_ipc(s: Severity) -> sieve_ipc::Severity {
        match s {
            Severity::Critical => sieve_ipc::Severity::Critical,
            Severity::High => sieve_ipc::Severity::High,
            Severity::Medium => sieve_ipc::Severity::Medium,
            Severity::Low => sieve_ipc::Severity::Low,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::detection::{Action, ContentSource, Detection, Severity};
        use crate::protocol::unified_message::ContentSpan;

        fn make_detection(rule_id: &str, action: Action) -> Detection {
            Detection {
                id: Uuid::new_v4(),
                rule_id: rule_id.to_string(),
                severity: Severity::Critical,
                action,
                source: ContentSource::InboundAssistantText,
                span: ContentSpan { start: 0, end: 5 },
                evidence_truncated: "sk-an".to_string(),
                fingerprint: "abc123".to_string(),
                source_channel: None,
                origin_chain_depth: 0,
            }
        }

        // ── 1. 空 detections → Passthrough ───────────────────────────────────

        #[tokio::test]
        async fn dispatch_empty_returns_passthrough() {
            let result = dispatch(Direction::Inbound, vec![], None, Uuid::new_v4(), None, None)
                .await
                .unwrap();
            assert!(matches!(result, DispatchResult::Passthrough));
        }

        // ── 2. MarkOnly → StatusBarOnly ───────────────────────────────────────

        #[tokio::test]
        async fn dispatch_mark_only_returns_status_bar() {
            let detections = vec![make_detection("OUT-11", Action::MarkOnly)];
            let result = dispatch(
                Direction::Outbound,
                detections,
                None,
                Uuid::new_v4(),
                None,
                None,
            )
            .await
            .unwrap();
            assert!(matches!(result, DispatchResult::StatusBarOnly));
        }

        // ── 3. Block → DenyWithBlock ──────────────────────────────────────────

        #[tokio::test]
        async fn dispatch_block_returns_deny() {
            let detections = vec![make_detection("IN-CR-99", Action::Block)];
            let result = dispatch(
                Direction::Inbound,
                detections,
                None,
                Uuid::new_v4(),
                None,
                None,
            )
            .await
            .unwrap();
            assert!(matches!(result, DispatchResult::DenyWithBlock(_)));
        }

        // ── 4. Redact → RewriteBody ───────────────────────────────────────────

        #[tokio::test]
        async fn dispatch_redact_returns_rewrite_body() {
            let mut d = make_detection(
                "OUT-01",
                Action::Redact {
                    placeholder: "[REDACTED]".to_string(),
                },
            );
            d.span = ContentSpan { start: 0, end: 5 };

            let body = b"sk-antXXXXX rest of body";
            let result = dispatch(
                Direction::Outbound,
                vec![d],
                None,
                Uuid::new_v4(),
                Some(body),
                None,
            )
            .await
            .unwrap();
            assert!(matches!(result, DispatchResult::RewriteBody(_)));
            if let DispatchResult::RewriteBody(new_body) = result {
                let s = String::from_utf8(new_body.to_vec()).unwrap();
                assert!(s.contains("[REDACTED:OUT-01]"), "body: {s}");
            }
        }

        // ── 5. HookMark → HookMarked ──────────────────────────────────────────

        #[tokio::test]
        async fn dispatch_hook_mark_returns_hook_marked() {
            let tmp = tempfile::tempdir().unwrap();
            std::env::set_var("SIEVE_HOME", tmp.path().to_str().unwrap());

            let detections = vec![make_detection("IN-CR-02", Action::HookMark)];
            let result = dispatch(
                Direction::Inbound,
                detections,
                None,
                Uuid::new_v4(),
                None,
                None,
            )
            .await
            .unwrap();
            assert!(matches!(result, DispatchResult::HookMarked));

            std::env::remove_var("SIEVE_HOME");
        }
    }
}
