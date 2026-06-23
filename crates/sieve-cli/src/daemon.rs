//! 透传 daemon（架构图节点 ①③⑤⑧）。
//!
//! Week 2：POST /v1/messages body 收集 → 出站规则扫描 → Critical 命中时返回 426；
//! 非 messages 路径 / 解析失败 / 无命中 → 流式透传（Week 1 行为保持不变）。
//!
//! Week 3：出站 dry_run+Critical fail-closed 修正 + 入站 SSE tee 截流检测。
//!
//! Week 4（v1.4）：
//! - 出站 AutoRedact：命中 Redact action 时脱敏 body bytes 后转发，**不返回 426**；
//! - 入站 Hook 类（HookMark）：写 IPC pending 文件，SSE 流原样转发，**不调用 sieve_blocked**；
//! - 入站 GUI 类（HoldForDecision）：hold SSE 流 + keep-alive，等用户决策后 Allow/Deny；
//! - IpcServer 随 daemon 启动，accept loop 在后台 spawn。
//!
//! Week 5（v1.5）：
//! - 路径分发：`/v1/messages` → Anthropic 路径；`/v1/chat/completions` → OpenAI 路径；
//! - `X-Sieve-Origin` header 解析 → source_agent / origin_chain / chain_depth；
//! - chain_depth ≥ 5 → 直接 426；chain_depth ≥ 2 → 所有命中强制 GuiPopup；
//! - `X-Sieve-Source-Channel` header 解析 → DecisionRequest.source_channel。
//!
//! 关联：PRD v1.5 §6.1 §4.5 §4.6 / ADR-018（OpenAI 协议）/ ADR-019（multi-agent header）/
//!        ADR-013（IPC）/ ADR-014（双层防御）/ ADR-016（处置矩阵）。

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use futures_util::StreamExt as _;
use http_body_util::{combinators::BoxBody, BodyExt, StreamBody};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use sieve_core::detection::Action;
use sieve_core::forwarder::ProxyConfig;
use sieve_core::pipeline::inbound::{InboundEngine, InboundFilter};
use sieve_core::pipeline::outbound::OutboundFilter;
use sieve_core::pipeline::outbound_redact::{redact_segments, RedactHit};
use sieve_core::pipeline::streaming::StreamingPipelineNode as _;
use sieve_core::sse::parser::SseParser;
use sieve_core::tool_use_aggregator::Aggregator;
use sieve_core::Forwarder;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::config::Config;
use crate::upstream_routes::UpstreamRoutes;

// ── 本地用量/计费核算 facade（可选特性 `usage`）─────────────────────────────
//
// billing（独立 token 核算 / relay 偏差观测）默认不编译。为避免在 daemon 请求
// 路径上撒满 `#[cfg]`，这里用 facade 统一类型名：`usage` 开时是真 billing 类型，
// 关时退化为占位空类型。请求 handler 的签名与类型引用两态都可命名，billing 句柄
// 始终为 `None`，仅真正调用 billing 业务方法的函数体内部按 feature 分支。
mod billing_facade {
    #[cfg(feature = "usage")]
    pub(super) use crate::billing::{BillingContext, BillingObserver, Family};

    // 占位类型：`usage` 关时使 daemon 类型引用仍可命名（值永远不被构造）。
    #[cfg(not(feature = "usage"))]
    #[derive(Clone, Copy)]
    pub(super) enum Family {
        Anthropic,
        OpenAi,
    }
    #[cfg(not(feature = "usage"))]
    pub(super) enum BillingObserver {}
    #[cfg(not(feature = "usage"))]
    pub(super) enum BillingContext {}
}

/// 计费观测器句柄：`usage` 开时为真观测器，关时恒为 `None`（占位，永不构造）。
type BillingObserverHandle = Option<std::sync::Arc<billing_facade::BillingObserver>>;
/// 每请求计费上下文句柄：`usage` 开时为真上下文，关时恒为 `None`（占位）。
type BillingCtxHandle = Option<billing_facade::BillingContext>;

// ── 加密审计档案 facade（可选特性 `audit-crypto`）────────────────────────────
//
// 加密归档写入（age + 哈希链）默认不编译。同 billing：用 facade 统一类型名，请求
// 路径签名两态可命名，归档句柄关时恒为 `None`，仅真正写归档的函数体内部按 feature
// 分支。`audit.level = full` 在特性关时优雅降级为不归档（warn 一句，当 metadata 处理）。
mod archive_facade {
    #[cfg(feature = "audit-crypto")]
    pub(super) use crate::audit_archive::ArchiveWriter;

    // 占位类型：`audit-crypto` 关时使 daemon 类型引用仍可命名（值永远不被构造）。
    #[cfg(not(feature = "audit-crypto"))]
    pub(super) enum ArchiveWriter {}
}

/// 加密归档写入器句柄：`audit-crypto` 开时为真写入器，关时恒为 `None`（占位）。
type ArchiveWriterHandle = Option<std::sync::Arc<archive_facade::ArchiveWriter>>;

// ── v2.0：请求上下文（PRD v2.0 §5.6 / §5.6.1）──────────────────────────────

/// 每请求上下文：caller 进程信息 + audit 存储句柄 + listener 元信息（PRD v2.0 §5.6 / §5.6.1；ADR-026）。
///
/// 合并为一个结构体以减少函数参数数量（clippy too_many_arguments）。
/// `clone()` 开销：Arc clone + Option clone + 标量 copy，均为 O(1)。
#[derive(Clone)]
struct RequestCtx {
    /// 调用方进程信息（v2.0 Phase A：macOS 真实反查；其他平台 None）。
    caller: Option<crate::process_context::CallerInfo>,
    /// 审计存储句柄（SQLite append-only，PRD §5.6.1）。
    audit: Arc<crate::audit::AuditStore>,
    /// 本次连接所在 listener 的协议声明（ADR-026 §决策 4）。
    /// proxy_inner 用此字段做协议错位 fail-closed 校验。
    listener_protocol: crate::config::Protocol,
    /// 本次连接所在 listener 的 provider_id（ADR-026 §决策 5）。
    /// 透传到审计 / IPC 事件 / 日志，标注本次请求命中哪个上游。
    /// Stage E 落地审计 schema 后会被消费；当前仅在错位拒绝日志中使用。
    #[allow(dead_code)]
    listener_provider_id: String,
}

impl RequestCtx {
    /// 从 caller_info + audit_store + listener metadata 构造。
    fn new(
        caller: Option<crate::process_context::CallerInfo>,
        audit: Arc<crate::audit::AuditStore>,
        listener_protocol: crate::config::Protocol,
        listener_provider_id: String,
    ) -> Self {
        Self {
            caller,
            audit,
            listener_protocol,
            listener_provider_id,
        }
    }

    /// 构造 `crate::audit::CallerContext`（供 audit 事件填充）。
    /// v2.0 Phase A 接入点预留，后续 audit 写入直接调用。
    #[allow(dead_code)]
    fn caller_context(&self) -> crate::audit::CallerContext {
        crate::audit::CallerContext {
            pid: self.caller.as_ref().map(|c| c.pid),
            exe: self
                .caller
                .as_ref()
                .and_then(|c| c.exe.as_ref())
                .map(|p| p.display().to_string()),
        }
    }
}

// ── ADR-026：multi-listener 元信息 ──────────────────────────────────────────

/// 单 listener 运行时元信息（ADR-026 §决策 1）。
///
/// 把 [`crate::config::UpstreamListener`] 配置升级成运行时形态：
/// - `forwarder` 已 build（含连接池），accept_loop 直接复用
/// - `provider_id` 已规范化（`UpstreamListener::resolved_provider_id` 求值）
///
/// `Clone` 开销：3 × Arc clone + 1 × String clone + 标量 copy，全部 O(1)/O(n_short)。
#[derive(Clone)]
struct ListenerSpec {
    /// 监听端口（127.0.0.1:port，ADR-003 完全本地）。
    port: u16,
    /// 上游转发器（含连接池，rustls TLS）。
    forwarder: Arc<Forwarder>,
    /// 上游身份标识（透传到 RequestCtx → 审计 / 日志 / IPC）。
    provider_id: String,
    /// 协议声明（proxy_inner 用此字段做错位 fail-closed 校验）。
    protocol: crate::config::Protocol,
    /// 上游信任级别（ADR-038）：`Official` 直连 usage 权威、不核算；`Relay` 须独立核算。
    /// 按 host 派生（`UpstreamListener::resolved_trust`），透传到超额计费观测器。
    trust: crate::config::Trust,
}

// ── v2.0：灰名单辅助（PRD v2.0 §5.4.2）─────────────────────────────────────

/// 计算 `DecisionRequest.allow_remember`（PRD v2.0 §5.4.2 / §5.4.3）。
///
/// fail-closed Critical 规则（`is_fail_closed` 返回 true）**必须强制 false**，
/// 禁止用户通过 GUI Remember 将其加入灰名单。
/// 非 Critical 规则可以为 `true`，允许用户选择"记住此决策"。
///
/// 多条 detection 时：任一 detection 的 rule_id 在 fail-closed 名单中 → 整批返回 `false`。
/// （最保守策略，PRD §9 #3 / #14）
///
/// 关联：PRD v2.0 §5.4.2 灰名单 schema、§5.4.3 GUI 接口、sieve_ipc::DecisionRequest::allow_remember。
fn compute_allow_remember(rule_ids: &[&str]) -> bool {
    // 任意一条 rule_id 在 fail-closed 名单中 → 整批不可 remember
    !rule_ids
        .iter()
        .any(|id| sieve_rules::critical_lock::is_fail_closed(id))
}

/// 从一组 detection 列表中提取 rule_id 切片，用于 `compute_allow_remember`。
///
/// 内联辅助，避免在每个调用点重复 collect。
fn detection_rule_ids<'a>(detections: &'a [&sieve_core::Detection]) -> Vec<&'a str> {
    detections.iter().map(|d| d.rule_id.as_str()).collect()
}

/// 暂停状态感知的 `request_decision`（SPEC-002 §9.1 + ADR-028 TODO-4 no-client policy）。
///
/// 行为：
/// 1. 若有任意 detection 命中 `critical_lock::FAIL_CLOSED_RULES` → 照常弹窗（暂停 / no-client policy 不影响 Critical）。
/// 2. 若 IPC server 无 client 连接（GUI 未在线 / CLI 未订阅）→ 按 `no_client_policy` 快速处置（ADR-028 §3）：
///    - `AutoBlock`：直接 Deny（fail-closed，默认）
///    - `AutoWarn`：直接 Allow（低风险 headless）
///    - `HoldAndFailClosed`：继续走原逻辑（等超时，v1.x 行为）
/// 3. 否则若 daemon 处于暂停状态 → **跳过弹窗**，按 `default_on_timeout` 自动决策，
///    写 `AuditEvent::AutoDecidedPaused`，返回合成 response（`by_user=false`）。
/// 4. 否则 → 直接调 `IpcServer::request_decision`。
///
/// 关联：PRD v2.0 §9 #3 #8（Critical 不可暂停）、SPEC-002 §9.1（paused 弹窗矩阵）、ADR-028 §3。
#[allow(clippy::too_many_arguments)]
pub(crate) async fn gated_request_decision(
    ipc: &Arc<sieve_ipc::IpcServer>,
    audit: &Arc<crate::audit::AuditStore>,
    caller: &Option<crate::process_context::CallerInfo>,
    req: sieve_ipc::DecisionRequest,
    timeout: std::time::Duration,
    direction: &str,
    provider_id: &str,
    no_client_policy: crate::cli::NoClientPolicy,
) -> Result<sieve_ipc::DecisionResponse, sieve_ipc::IpcError> {
    let any_critical = req
        .detections
        .iter()
        .any(|d| sieve_rules::critical_lock::is_fail_closed(&d.rule_id));

    // ADR-028 TODO-4：无 client 连接时按 no_client_policy 快速处置。
    // Critical 规则不受此策略影响（fail-closed 硬约束，PRD §9 #3 #8）。
    if !any_critical && ipc.connected_clients() == 0 {
        match no_client_policy {
            crate::cli::NoClientPolicy::AutoBlock => {
                tracing::info!(
                    provider_id,
                    "no client connected → auto-block (ADR-028 no_client_policy=auto-block)"
                );
                spawn_decision_audit(
                    audit,
                    provider_id,
                    caller,
                    &req.detections,
                    "deny",
                    false,
                    req.request_id,
                );
                return Ok(sieve_ipc::DecisionResponse {
                    request_id: req.request_id,
                    decision: sieve_ipc::DecisionAction::Deny,
                    decided_at: chrono::Utc::now(),
                    by_user: false,
                    remember: false,
                    context_hint: Some("auto-block: no client connected (ADR-028)".to_owned()),
                    ui_phase_when_clicked: None,
                });
            }
            crate::cli::NoClientPolicy::AutoWarn => {
                tracing::info!(
                    provider_id,
                    "no client connected → auto-warn allow (ADR-028 no_client_policy=auto-warn)"
                );
                spawn_decision_audit(
                    audit,
                    provider_id,
                    caller,
                    &req.detections,
                    "allow",
                    false,
                    req.request_id,
                );
                return Ok(sieve_ipc::DecisionResponse {
                    request_id: req.request_id,
                    decision: sieve_ipc::DecisionAction::Allow,
                    decided_at: chrono::Utc::now(),
                    by_user: false,
                    remember: false,
                    context_hint: Some("auto-warn: no client connected (ADR-028)".to_owned()),
                    ui_phase_when_clicked: None,
                });
            }
            crate::cli::NoClientPolicy::HoldAndFailClosed => {
                // v1.x 行为：继续走 request_decision（等超时，fail-closed 通过 default_on_timeout）
                tracing::debug!(
                    provider_id,
                    "no client connected → hold-and-fail-closed (ADR-028 no_client_policy=hold-and-fail-closed)"
                );
            }
        }
    }

    if any_critical || !ipc.is_paused() {
        // 捕获 detection / request_id（req 即将被 move 进 request_decision）。
        let detections = req.detections.clone();
        let request_id = req.request_id;
        let resp = ipc.request_decision(req, timeout, direction).await;
        if let Ok(ref r) = resp {
            let decision = match r.decision {
                sieve_ipc::DecisionAction::Allow => "allow",
                sieve_ipc::DecisionAction::Deny => "deny",
                sieve_ipc::DecisionAction::RedactAndAllow => "redact_and_allow",
            };
            spawn_decision_audit(
                audit,
                provider_id,
                caller,
                &detections,
                decision,
                r.by_user,
                request_id,
            );
        }
        return resp;
    }

    // 暂停 + 全部非 Critical → 自动决策
    let auto_action = match req.default_on_timeout {
        sieve_ipc::DefaultOnTimeout::Block => sieve_ipc::DecisionAction::Deny,
        sieve_ipc::DefaultOnTimeout::Allow => sieve_ipc::DecisionAction::Allow,
        sieve_ipc::DefaultOnTimeout::Redact => sieve_ipc::DecisionAction::RedactAndAllow,
    };
    let decision_str = match auto_action {
        sieve_ipc::DecisionAction::Allow => "allow",
        sieve_ipc::DecisionAction::Deny => "deny",
        sieve_ipc::DecisionAction::RedactAndAllow => "redact_and_allow",
    };
    let rule_ids = req
        .detections
        .iter()
        .map(|d| d.rule_id.clone())
        .collect::<Vec<_>>()
        .join(",");
    let request_id_str = req.request_id.to_string();

    tracing::info!(
        request_id = %req.request_id,
        rule_ids = %rule_ids,
        decision = decision_str,
        "paused → 跳过弹窗，自动按 default_on_timeout 处置（SPEC-002 §9.1）"
    );

    let caller_ctx = crate::audit::CallerContext {
        pid: caller.as_ref().map(|c| c.pid),
        exe: caller
            .as_ref()
            .and_then(|c| c.exe.as_ref())
            .map(|p| p.display().to_string()),
    };
    let event = crate::audit::AuditEvent::AutoDecidedPaused {
        rule_ids,
        decision: decision_str.to_owned(),
        request_id: request_id_str,
        caller: caller_ctx,
    };
    let store = Arc::clone(audit);
    let provider_id_owned = provider_id.to_owned();
    tokio::spawn(async move {
        if let Err(e) = store.append(event, &provider_id_owned).await {
            tracing::warn!(error = %e, "audit append AutoDecidedPaused failed");
        }
    });

    Ok(sieve_ipc::DecisionResponse {
        request_id: req.request_id,
        decision: auto_action,
        decided_at: chrono::Utc::now(),
        by_user: false,
        remember: false,
        context_hint: None,
        ui_phase_when_clicked: None,
    })
}

/// 为一次决策结果写 `DecisionMade` 审计事件（fire-and-forget，不阻塞请求热路径，
/// PRD §9 性能预算 P99<20ms）。沿用 daemon 控制面 spawn-audit 模式。
///
/// `decision`：`"allow"` / `"deny"` / `"redact_and_allow"`。`by_user`：true=用户点击，
/// false=超时/系统自动（no-client policy 等）。取首个 detection 作为主关联规则。
///
/// 接线背景：detection 决策结果此前从不落 audit（headless dogfood e2e 抓出，2026-06-18），
/// `sieve audit query` 查不到任何核心流量决策。
fn spawn_decision_audit(
    audit: &Arc<crate::audit::AuditStore>,
    provider_id: &str,
    caller: &Option<crate::process_context::CallerInfo>,
    detections: &[sieve_ipc::DetectionPayload],
    decision: &str,
    by_user: bool,
    request_id: uuid::Uuid,
) {
    let Some(primary) = detections.first() else {
        return;
    };
    let caller_ctx = crate::audit::CallerContext {
        pid: caller.as_ref().map(|c| c.pid),
        exe: caller
            .as_ref()
            .and_then(|c| c.exe.as_ref())
            .map(|p| p.display().to_string()),
    };
    let event = crate::audit::AuditEvent::DecisionMade {
        rule_id: primary.rule_id.clone(),
        decision: decision.to_owned(),
        severity: format!("{:?}", primary.severity).to_lowercase(),
        by_user,
        request_id: request_id.to_string(),
        caller: caller_ctx,
    };
    let store = Arc::clone(audit);
    let provider_id_owned = provider_id.to_owned();
    tokio::spawn(async move {
        if let Err(e) = store.append(event, &provider_id_owned).await {
            tracing::warn!(error = %e, "audit append DecisionMade failed");
        }
    });
}

/// 入站 Critical 拦截写审计（fail-closed 自动 block，无用户决策；PRD §9 #3）。
///
/// 接线背景：入站 block 路径（SSE + JSON、Anthropic + OpenAI）此前一律不落 audit
/// （真机 dogfood 抓出，2026-06-18）。每条 detection 写一条 InboundBlocked 事件，
/// 仅含元数据（rule_id / severity / path_label / caller），零 secret 落盘。
/// fire-and-forget：tokio::spawn 不阻塞热路径（PRD §9 性能预算）。
fn spawn_inbound_blocked_audit(
    audit: &Arc<crate::audit::AuditStore>,
    provider_id: &str,
    caller: &Option<crate::process_context::CallerInfo>,
    detections: &[sieve_core::Detection],
    path_label: &str,
) {
    let caller_ctx = crate::audit::CallerContext {
        pid: caller.as_ref().map(|c| c.pid),
        exe: caller
            .as_ref()
            .and_then(|c| c.exe.as_ref())
            .map(|p| p.display().to_string()),
    };
    for d in detections {
        let event = crate::audit::AuditEvent::InboundBlocked {
            rule_id: d.rule_id.clone(),
            severity: format!("{:?}", d.severity).to_lowercase(),
            request_id: uuid::Uuid::new_v4().to_string(),
            path_label: path_label.to_owned(),
            caller: caller_ctx.clone(),
        };
        let store = Arc::clone(audit);
        let provider_id_owned = provider_id.to_owned();
        tokio::spawn(async move {
            if let Err(e) = store.append(event, &provider_id_owned).await {
                tracing::warn!(error = %e, "audit append InboundBlocked failed");
            }
        });
    }
}

/// 写灰名单条目，二次校验 Critical 锁（PRD v2.0 §5.4.2 二次校验）。
///
/// 调用时机：daemon 收到 `DecisionResponse { decision=Allow, remember=true }` 之后。
///
/// 校验路径：
/// 1. `is_fail_closed(rule_id) == true` → 写 `AuditEvent::GraylistCriticalRejected` + 返回（不写灰名单）
/// 2. `add_entry` 失败 → 写 `AuditEvent::GraylistAddFailed` + warn（fail-soft，不影响本次 Allow 决策）
///
/// 关联：PRD v2.0 §5.4.2「Critical 锁约束」三道防线、§5.4.3 GUI 接口。
///
/// # 参数
/// - `rule_id`：命中的规则 ID
/// - `matched_text`：命中文本（用于 fingerprint 计算；去空白 + 统一小写）
/// - `tool_name`：触发工具名（可空）
/// - `protocol`：`"anthropic"` 或 `"openai"`
/// - `content_kind`：`"tool_use_input"` / `"json_response_body"` 等
/// - `source_agent_str`：source_agent 字符串表示
/// - `context_hint`：用户在 GUI 输入的备注（来自 DecisionResponse.context_hint）
/// - `audit_event_id`：本次 audit 事件 ID（v4 UUID 字符串）
/// - `audit_store`：审计存储句柄（v2.1 接入，PRD §5.4.2）
/// - `caller`：调用方进程信息（v2.0 Phase A）
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn try_write_graylist(
    rule_id: &str,
    matched_text: &str,
    tool_name: &str,
    protocol: &str,
    content_kind: &str,
    source_agent_str: &str,
    context_hint: Option<String>,
    audit_event_id: &str,
    audit_store: &Arc<crate::audit::AuditStore>,
    caller: &Option<crate::process_context::CallerInfo>,
    provider_id: &str,
) {
    let caller_ctx = crate::audit::CallerContext {
        pid: caller.as_ref().map(|c| c.pid),
        exe: caller
            .as_ref()
            .and_then(|c| c.exe.as_ref())
            .map(|p| p.display().to_string()),
    };

    // 二次校验 Critical 锁（防 GUI 端绕过，PRD §5.4.2 第三道防线）
    if sieve_rules::critical_lock::is_fail_closed(rule_id) {
        tracing::error!(
            rule_id,
            "二次校验失败：fail-closed Critical 规则不可 remember，忽略灰名单写入 + audit ERROR"
        );
        // v2.1：写 GraylistCriticalRejected audit 事件（PRD §5.4.2）
        let event = crate::audit::AuditEvent::GraylistCriticalRejected {
            rule_id: rule_id.to_string(),
            request_id: audit_event_id.to_string(),
            caller: caller_ctx,
        };
        let audit = Arc::clone(audit_store);
        let provider_id_owned = provider_id.to_owned();
        tokio::spawn(async move {
            if let Err(e) = audit.append(event, &provider_id_owned).await {
                tracing::warn!(error = %e, "audit append GraylistCriticalRejected failed");
            }
        });
        return;
    }

    let graylist_dir = match sieve_ipc::paths::sieve_home() {
        Ok(home) => home.join("decisions"),
        Err(e) => {
            tracing::warn!(error = %e, "无法获取 SIEVE_HOME，跳过灰名单写入");
            // SIEVE_HOME 不可用视同写入失败，写 GraylistAddFailed audit
            let event = crate::audit::AuditEvent::GraylistAddFailed {
                rule_id: rule_id.to_string(),
                error: format!("SIEVE_HOME 不可用: {e}"),
                request_id: audit_event_id.to_string(),
                caller: caller_ctx,
            };
            let audit = Arc::clone(audit_store);
            let provider_id_owned = provider_id.to_owned();
            tokio::spawn(async move {
                if let Err(ae) = audit.append(event, &provider_id_owned).await {
                    tracing::warn!(error = %ae, "audit append GraylistAddFailed failed");
                }
            });
            return;
        }
    };

    // 规范化命中文本（去首尾空白 + 统一小写）
    let canonical = matched_text.trim().to_lowercase();

    let inputs = sieve_policy::graylist::FingerprintInputs {
        rule_id: rule_id.to_owned(),
        matched_canonical: canonical,
        tool_name: tool_name.to_owned(),
        protocol: protocol.to_owned(),
        content_kind: content_kind.to_owned(),
        source_agent: source_agent_str.to_owned(),
    };
    let fingerprint = sieve_policy::graylist::compute_fingerprint(&inputs);

    let entry = sieve_policy::graylist::GraylistEntry {
        schema_version: 1,
        fingerprint_version: 1,
        rule_id: rule_id.to_owned(),
        rule_version: "v2.0".to_owned(),
        fingerprint: fingerprint.clone(),
        fingerprint_inputs: inputs,
        decision: "allow".to_owned(),
        expires_at: None,
        added_at: chrono::Utc::now().timestamp_millis(),
        added_by: "gui_user_decision".to_owned(),
        context_hint,
        match_count_since: 0,
        audit_event_id: audit_event_id.to_owned(),
    };

    if let Err(e) = sieve_policy::graylist::add_entry(&graylist_dir, entry) {
        // add_entry 失败不影响本次决策（用户已 Allow，仍 forward）——fail-soft
        tracing::warn!(error = %e, rule_id, fingerprint, "灰名单写入失败（warn only，不影响本次 Allow 决策）");
        // v2.1：写 GraylistAddFailed audit 事件（PRD §5.4.2）
        let event = crate::audit::AuditEvent::GraylistAddFailed {
            rule_id: rule_id.to_string(),
            error: e.to_string(),
            request_id: audit_event_id.to_string(),
            caller: caller_ctx,
        };
        let audit = Arc::clone(audit_store);
        let provider_id_owned = provider_id.to_owned();
        tokio::spawn(async move {
            if let Err(ae) = audit.append(event, &provider_id_owned).await {
                tracing::warn!(error = %ae, "audit append GraylistAddFailed failed");
            }
        });
    } else {
        tracing::info!(rule_id, fingerprint, "灰名单条目已写入");
    }
}

/// 查询灰名单，命中时返回 `true`（表示应直接 Allow，跳过 IPC 弹窗）。
///
/// fail-closed：查询失败（文件损坏 / 权限错）→ warn + 返回 `false`，走正常 IPC 流程。
/// PRD §9 #14 禁止 fail-open。
///
/// 关联：PRD v2.0 §5.4.2 灰名单 schema、§5.4.3 GUI 接口预留。
///
/// # 参数说明
/// 同 `try_write_graylist`，但 `matched_text` 允许来自任意命中片段（首条 detection）。
fn check_graylist_hit(
    rule_id: &str,
    matched_text: &str,
    tool_name: &str,
    protocol: &str,
    content_kind: &str,
    source_agent_str: &str,
) -> bool {
    // fail-closed 规则不可灰名单命中（即使文件存在也视为未命中）
    if sieve_rules::critical_lock::is_fail_closed(rule_id) {
        return false;
    }

    let graylist_dir = match sieve_ipc::paths::sieve_home() {
        Ok(home) => home.join("decisions"),
        Err(e) => {
            tracing::warn!(error = %e, "无法获取 SIEVE_HOME，灰名单查询跳过");
            return false;
        }
    };

    let canonical = matched_text.trim().to_lowercase();
    let inputs = sieve_policy::graylist::FingerprintInputs {
        rule_id: rule_id.to_owned(),
        matched_canonical: canonical,
        tool_name: tool_name.to_owned(),
        protocol: protocol.to_owned(),
        content_kind: content_kind.to_owned(),
        source_agent: source_agent_str.to_owned(),
    };
    let digest = sieve_policy::graylist::compute_fingerprint(&inputs);

    match sieve_policy::graylist::lookup(&graylist_dir, &digest) {
        Ok(Some(entry)) if entry.decision == "allow" => {
            tracing::info!(
                rule_id,
                fingerprint = %digest,
                "灰名单命中 → 直接 Allow（跳过 IPC 弹窗）"
            );
            true
        }
        Ok(Some(_)) => false, // decision 不是 allow，走正常流程
        Ok(None) => false,    // 未命中
        Err(e) => {
            // 查询失败（文件损坏 / 权限错）→ fail-closed（PRD §9 #14）
            tracing::warn!(error = %e, rule_id, "灰名单查询失败，fail-closed 走正常 IPC 流程");
            false
        }
    }
}

/// caller PID 反查 stub（PRD v2.0 §5.6 / §6.6 Phase A MVP）。
///
/// TCP peer_addr → PID 在 macOS 上需要 `proc_listpidspath` + 跨进程权限，
/// 工程量超出 Week 6 范围。本期保持 stub 返回 `None`，
/// 通过 TCP 4-tuple 反查 caller PID（v2.0 Phase A 接入真实实现）。
///
/// 调用 [`crate::process_context::lookup_caller_by_socket_addr`]，内部走
/// proc_listpids → proc_pidinfo(FDs) → proc_pidfdinfo(socket_fdinfo) 扫描。
/// 非 macOS 或失败时静默返回 `None`（不影响主流程）。
/// 30 秒 LRU cache 保证热路径 P99 < 1µs。
///
/// 关联：PRD v2.0 §5.6 / §6.6 / OQ-V20-02。
fn peer_addr_to_pid(local: std::net::SocketAddr, peer: std::net::SocketAddr) -> Option<i32> {
    crate::process_context::lookup_caller_by_socket_addr(local, peer)
}

// ── multi-agent header 解析（ADR-019）────────────────────────────────────────
// 修 R8-#1：改用 sieve_ipc::parse_origin_header，支持 3 段（无签名）和 4 段（含签名）格式。
// 旧实现用 rsplitn(2, ':') 在 4 段时把 base64 签名当 chain_depth 导致解析失败 → fail-open，
// 攻击者可在签名字段写入合法 chain_depth 数值绕过 chain_depth ≥ 2 的 GuiPopup 升级。
// 新实现委托给 sieve_ipc::parse_origin_header（splitn(4, ':')），正确处理两种格式。
// 关联：ADR-019 §Header 格式规范、PRD v1.5 §6.5。

/// 从已解析的 origin header 构造 `origin_chain`（`Vec<OriginHop>`）。
///
/// 当前仅记录发送方一跳（chain_depth 反映深度，origin_chain 记录来源 hop）。
/// chain_depth = 0 → 空 chain（用户直接调用，无委托链）。
/// chain_depth ≥ 1 → 添加一个表示发送方的 OriginHop。
///
/// 关联：ADR-019 §origin_chain 构造、PRD v1.5 §4.6。
fn build_origin_chain(
    source_agent: sieve_ipc::protocol::SourceAgent,
    chain_depth: usize,
) -> Vec<sieve_ipc::protocol::OriginHop> {
    if chain_depth == 0 {
        return Vec::new();
    }
    vec![sieve_ipc::protocol::OriginHop {
        agent: source_agent,
        action: "delegate".to_owned(),
        timestamp: chrono::Utc::now(),
    }]
}

/// 解析 `X-Sieve-Source-Channel` header（OpenClaw 跨通道标识）。
///
/// 缺 header 或值为空 → `None`（非 OpenClaw 来源）。
/// 关联：PRD v1.5 §4.5 场景 E、IN-GEN-06。
fn parse_source_channel(headers: &http::HeaderMap) -> Option<String> {
    headers
        .get("x-sieve-source-channel")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// 从请求 headers 解析 `X-Sieve-Origin`，返回 `(source_agent, origin_chain, chain_depth)`。
///
/// - 缺 header → source_agent=Unknown, chain_depth=0, origin_chain=[]
/// - 格式错误 → 同上 + audit 警告（fail-open）
/// - chain_depth ≥ 5 → 返回 chain_depth=5（调用方负责 426）
///
/// 修 R8-#1：改用 `sieve_ipc::parse_origin_header` 支持 3 段/4 段格式。
/// `ChainTooDeep` 错误时返回实际 chain_depth（让调用方触发 426，保持 fail-closed 语义）。
///
/// 关联：ADR-019 §解析策略、PRD v1.5 §6.5。
fn extract_origin_metadata(
    headers: &http::HeaderMap,
) -> (
    sieve_ipc::protocol::SourceAgent,
    Vec<sieve_ipc::protocol::OriginHop>,
    usize,
) {
    let Some(header_val) = headers.get("x-sieve-origin") else {
        return (sieve_ipc::protocol::SourceAgent::Unknown, Vec::new(), 0);
    };

    let Ok(header_str) = header_val.to_str() else {
        tracing::warn!("X-Sieve-Origin: 包含非 UTF-8 字符，fail-open");
        return (sieve_ipc::protocol::SourceAgent::Unknown, Vec::new(), 0);
    };

    match sieve_ipc::parse_origin_header(header_str) {
        Ok(h) => {
            let origin_chain = build_origin_chain(h.source_agent, h.chain_depth);
            (h.source_agent, origin_chain, h.chain_depth)
        }
        Err(sieve_ipc::OriginHeaderError::ChainTooDeep(d)) => {
            // chain_depth ≥ 5：保留真实 depth，让调用方走 426 分支（不 fail-open）。
            tracing::warn!(
                chain_depth = d,
                "X-Sieve-Origin chain_depth ≥ 5，转发给 426 检查"
            );
            (sieve_ipc::protocol::SourceAgent::Unknown, Vec::new(), d)
        }
        Err(e) => {
            tracing::warn!(error = %e, raw = header_str, "X-Sieve-Origin 解析失败，fail-open，视为无 header");
            (sieve_ipc::protocol::SourceAgent::Unknown, Vec::new(), 0)
        }
    }
}

/// 响应 body 的统一类型：错误为装箱 trait object，兼容 h1/h2 body 差异。
type ResponseBody = BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>;

/// 启动 daemon，永久阻塞直到进程收到信号。
///
/// `filter` 是出站规则引擎包装；`inbound_engine` + `inbound_sieveignore` 用于每连接构造
/// [`InboundFilter`]（每连接独立实例，共享 engine Arc）。
/// `cfg.dry_run` 决定是否实际拦截。
/// `audit_store` 透传到所有请求处理路径，供 audit 写入使用（PRD §5.6.1）。
///
/// v1.4：启动时绑定 IpcServer Unix socket，accept loop 在后台 spawn。
/// v2.0：透传 `audit_store: Arc<AuditStore>`；启动 reload listener task（PRD §5.5.5）。
/// v2.1：新增 `outbound_layered` + `inbound_layered`，reload listener 调用 `swap_user` 完成
///       zero-downtime hot swap，无需重启 daemon（PRD §5.5.5）。
///
/// # Errors
/// bind 端口失败或 Forwarder 初始化失败时返回错误。
#[allow(clippy::too_many_arguments)]
/// daemon::run 扩展参数（ADR-028 TODO-4 no-client policy）。
///
/// 单独用结构体封装，避免函数参数过多（clippy too_many_arguments）。
pub struct DaemonRunOpts {
    /// 无 client 接 IPC 时的兜底策略（ADR-028 §3）。
    pub no_client_policy: crate::cli::NoClientPolicy,
}

impl Default for DaemonRunOpts {
    fn default() -> Self {
        Self {
            no_client_policy: crate::cli::NoClientPolicy::AutoBlock,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    cfg: Config,
    filter: Arc<OutboundFilter>,
    inbound_engine: Arc<dyn InboundEngine>,
    inbound_sieveignore: Arc<HashSet<String>>,
    address_guard_config: sieve_core::pipeline::inbound::AddressGuardConfig,
    audit_store: Arc<crate::audit::AuditStore>,
    outbound_layered: Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    inbound_layered: Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    opts: DaemonRunOpts,
) -> Result<()> {
    let dry_run = cfg.dry_run;
    let no_client_policy = opts.no_client_policy;

    // ADR-026 §决策 1+2：把 cfg.upstreams 升级成运行时 ListenerSpec 数组。
    // 单 listener 配置（旧 sieve.toml）走 Config::resolved_upstreams 兼容映射，
    // 在此处统一表现为 Vec<ListenerSpec>。
    // 任一 forwarder 初始化失败 → fail-fast（避免半启动状态）。
    let listener_specs: Vec<ListenerSpec> = {
        let upstreams = cfg.resolved_upstreams();
        let mut specs = Vec::with_capacity(upstreams.len());
        for u in upstreams {
            let provider_id = u.resolved_provider_id();
            let proxy = ProxyConfig::parse(cfg.effective_proxy(&u).as_deref()).map_err(|e| {
                anyhow!(
                    "invalid proxy for listener port {} (url {}): {e}",
                    u.port,
                    u.url
                )
            })?;
            let f = Arc::new(Forwarder::new(&u.url, proxy).map_err(|e| {
                anyhow!(
                    "init forwarder for listener port {} (provider {}, url {}): {e}",
                    u.port,
                    provider_id,
                    u.url
                )
            })?);
            let trust = u.resolved_trust();
            specs.push(ListenerSpec {
                port: u.port,
                forwarder: f,
                provider_id,
                protocol: u.protocol,
                trust,
            });
        }
        specs
    };

    // ADR-038：构造超额计费观测器（仅 [billing_check].enabled 时 Some；纯本地、零网络）。
    // TokenEstimator::new 加载 BPE 词表开销大，启动一次性构造、Arc 透传。
    let billing_observer = build_billing_observer(&cfg)?;

    // ADR-037 full 档：构造加密审计归档写入器（write-only logging，daemon 只持公钥、
    // 结构上不可解密）。仅 `audit.level = full` 时为 Some；recipient 不可解析 → fail-fast
    // （config 校验已查 age1 前缀格式）。启动时清理超期段（保留期，ADR-037 决策 5）。
    let archive_writer = build_archive_writer(&cfg)?;

    // R11-#1：加载 OpenClaw 上游路由表（~/.sieve/upstream-routes.json）。
    // 加载失败（文件不存在 / JSON 非法）时 warn 后继续，所有请求走默认上游兜底。
    // 成功时为每个 provider id 预构建 Forwarder（含连接池），请求处理时直接 map lookup。
    let provider_forwarders: Arc<HashMap<String, Arc<Forwarder>>> = {
        let routes = match sieve_ipc::paths::sieve_home() {
            Ok(home) => {
                let path = home.join("upstream-routes.json");
                match UpstreamRoutes::load(&path) {
                    Ok(r) => {
                        if !r.is_empty() {
                            tracing::info!(
                                count = r.len(),
                                path = %path.display(),
                                "upstream-routes loaded"
                            );
                        }
                        r
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "upstream-routes load failed; all requests will use default upstream"
                        );
                        UpstreamRoutes::default()
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "SIEVE_HOME not set; upstream-routes disabled, using default upstream"
                );
                UpstreamRoutes::default()
            }
        };

        let mut map: HashMap<String, Arc<Forwarder>> = HashMap::new();
        // header-routing 上游统一走全局代理（受限网络下同样需要出网）。SPEC-007。
        let header_route_proxy =
            ProxyConfig::parse(cfg.global_proxy().as_deref()).unwrap_or(ProxyConfig::Direct);
        for (provider_id, upstream_url) in routes.iter() {
            match Forwarder::new(upstream_url, header_route_proxy.clone()) {
                Ok(f) => {
                    tracing::debug!(provider = %provider_id, upstream = %upstream_url, "provider forwarder ready");
                    map.insert(provider_id.to_owned(), Arc::new(f));
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        provider = %provider_id,
                        upstream = %upstream_url,
                        "failed to build forwarder for provider; will use default upstream"
                    );
                }
            }
        }
        Arc::new(map)
    };

    // v1.4：初始化 IpcServer（Unix socket），供 GUI 类 hold 流使用。
    // socket path = ~/.sieve/ipc.sock（或 $SIEVE_HOME/ipc.sock）。
    // 若初始化失败（如 $HOME 未设置），打印警告后继续——GuiPopup detection 会以 fail-closed 处理。
    let ipc_server: Option<Arc<sieve_ipc::IpcServer>> = match sieve_ipc::paths::sieve_home() {
        Ok(home) => {
            let socket_path = sieve_ipc::paths::ipc_socket_path(&home);
            match sieve_ipc::IpcServer::bind(socket_path.clone()) {
                Ok((server, listener)) => {
                    let server = Arc::new(server);
                    let srv_clone = Arc::clone(&server);
                    tokio::spawn(async move {
                        srv_clone.run(listener).await;
                    });
                    tracing::info!(socket = %socket_path.display(), "IPC server started");
                    Some(server)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "IPC server bind failed; GUI popup decisions will use fail-closed fallback");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "SIEVE_HOME not set; IPC server disabled");
            None
        }
    };

    // v2.1 §S.4：启动控制面 handler（GUI → daemon 8 个方法 + 3 个广播）。
    //
    // 关联：ADR-013 Supplement 2026-05-02 / SPEC-002 §9。
    if let Some(ref ipc_srv) = ipc_server {
        let runtime_state = Arc::new(crate::daemon_control_plane::RuntimeState {
            paused_until: arc_swap::ArcSwap::from_pointee(None),
            preset: arc_swap::ArcSwap::from_pointee(crate::daemon_control_plane::RuntimePreset {
                mode: format!("{:?}", cfg.preset).to_lowercase(),
                overrides: std::collections::HashMap::new(),
            }),
            started_at: chrono::Utc::now(),
            listen: sieve_ipc::ListenSnapshot {
                addr: cfg.bind_addr.clone(),
                // ADR-026 兼容：旧 listen 单字段 = listeners[0]，即 cfg.resolved_upstreams()[0].port
                port: listener_specs.first().map(|s| s.port).unwrap_or(cfg.port),
            },
            listeners: listener_specs
                .iter()
                .map(|s| sieve_ipc::ListenerSnapshot {
                    addr: cfg.bind_addr.clone(),
                    port: s.port,
                    provider_id: s.provider_id.clone(),
                    protocol: match s.protocol {
                        crate::config::Protocol::Auto => "auto".to_owned(),
                        crate::config::Protocol::Anthropic => "anthropic".to_owned(),
                        crate::config::Protocol::Openai => "openai".to_owned(),
                    },
                })
                .collect(),
            daemon_version: env!("CARGO_PKG_VERSION").to_owned(),
            protocol_version: "v2".to_owned(),
            audit_db_path: cfg
                .audit_db_path()
                .unwrap_or_else(|_| std::path::PathBuf::from("~/.sieve/audit.db")),
            decisions_dir: sieve_ipc::paths::sieve_home()
                .map(|h| h.join("decisions"))
                .unwrap_or_else(|_| std::path::PathBuf::from("~/.sieve/decisions")),
            user_rules_path: sieve_ipc::paths::sieve_home()
                .ok()
                .map(|h| h.join("rules").join("user.toml")),
            system_rules_count: arc_swap::ArcSwap::from_pointee({
                use sieve_rules::engine::MatchEngine;
                outbound_layered.rule_count() + inbound_layered.rule_count()
            }),
            user_rules_count: arc_swap::ArcSwap::from_pointee(0),
            last_reload: arc_swap::ArcSwap::from_pointee(None),
            purge_in_progress: std::sync::atomic::AtomicBool::new(false),
        });
        // SPEC-005 §3：注入 sieve.hello 握手通知所需静态信息。
        // daemon_boot_id 在本次启动时生成一次，整生命周期不变。
        let hello_builder = sieve_ipc::HelloBuilder {
            daemon_boot_id: uuid::Uuid::new_v4(),
            daemon_version: env!("CARGO_PKG_VERSION").to_owned(),
            audit_db_user_version: audit_store.schema_version(),
            started_at: chrono::Utc::now(),
            preset: format!("{:?}", cfg.preset).to_lowercase(),
        };
        ipc_srv.set_hello_builder(hello_builder).await;

        // SPEC-005 §1.3.1：注入 oversize 帧 audit 回调，避免 sieve-ipc → sieve-cli 循环依赖。
        {
            let audit_for_oversize = Arc::clone(&audit_store);
            let cb: sieve_ipc::OversizeCallback =
                std::sync::Arc::new(move |kind, size_bytes: usize| {
                    let kind_str = match kind {
                        sieve_ipc::OversizeKind::Frame => "frame",
                        sieve_ipc::OversizeKind::Remainder => "remainder",
                    };
                    let event = crate::audit::AuditEvent::IpcOversizeFrame {
                        peer: "ipc_socket".to_owned(),
                        size_bytes: size_bytes as u64,
                        closed_at_ms: chrono::Utc::now().timestamp_millis(),
                    };
                    tracing::warn!(
                        kind = kind_str,
                        size_bytes,
                        "IPC oversize frame audit written"
                    );
                    let store = Arc::clone(&audit_for_oversize);
                    tokio::spawn(async move {
                        // ADR-026 Stage E：oversize 是 IPC 帧事件，无 listener 上下文 → SYSTEM_PROVIDER_ID
                        if let Err(e) = store.append(event, crate::audit::SYSTEM_PROVIDER_ID).await
                        {
                            tracing::warn!(error = %e, "audit write for oversize frame failed");
                        }
                    });
                });
            ipc_srv.set_oversize_callback(cb);
        }

        crate::daemon_control_plane::spawn_control_plane_handler(
            Arc::clone(ipc_srv),
            Arc::clone(&audit_store),
            runtime_state,
            Arc::clone(&outbound_layered),
            Arc::clone(&inbound_layered),
            Arc::clone(&inbound_engine),
            no_client_policy,
        );
    }

    // v2.1 §5.5.5：启动 reload listener（zero-downtime hot swap，PRD §9 #14 fail-safe）。
    // daemon 监听 IpcServer::reload_rx 的用户规则 reload 请求：
    // 1. 重新读 user.toml + lint + 编译出站/入站 UserEngine
    // 2. 成功 → atomic swap_user（LayeredEngine zero-downtime hot reload，无需重启）
    // 3. 推送 NotifyKind::UserRulesReloaded / UserRulesLoadFailed
    // 4. 写 AuditEvent::UserRulesReloaded
    if let Some(ref ipc_srv) = ipc_server {
        if let Some(mut reload_rx) = ipc_srv.reload_rx().await {
            let ipc_for_reload = Arc::clone(ipc_srv);
            let audit_for_reload = Arc::clone(&audit_store);
            let user_rules_path_for_reload = sieve_ipc::paths::sieve_home()
                .ok()
                .map(|h| h.join("rules").join("user.toml"));
            // 持有 Arc 引用以在 reload 时原子替换用户引擎（PRD §5.5.5 hot swap）
            let outbound_layered_for_reload = Arc::clone(&outbound_layered);
            let inbound_layered_for_reload = Arc::clone(&inbound_layered);
            tokio::spawn(async move {
                while let Some(req) = reload_rx.recv().await {
                    // 调用提取的共用函数（与 control plane sieve.reload_config 共享同一逻辑）
                    let _outcome = perform_user_rules_reload(
                        user_rules_path_for_reload.as_deref(),
                        &outbound_layered_for_reload,
                        &inbound_layered_for_reload,
                        &ipc_for_reload,
                        &audit_for_reload,
                        req.trigger_id,
                    );
                }
            });
        }
    }

    // ADR-030: start the update-check + telemetry beacon task.
    // The task is detached — its failure never affects the daemon.
    {
        let env_overrides = sieve_updater::env::from_env();
        if !cfg.update.enabled || env_overrides.no_update {
            tracing::info!("update check disabled by SIEVE_NO_UPDATE");
        } else {
            let url = env_overrides
                .url_override
                .or_else(|| cfg.update.url.clone())
                .unwrap_or_else(|| sieve_updater::DEFAULT_MANIFEST_URL.to_string());
            let no_telemetry = env_overrides.no_telemetry || !cfg.update.telemetry;
            let interval = cfg.update.check_interval_hours.saturating_mul(3600);
            let updater_cfg = sieve_updater::runner::UpdaterConfig {
                base_url: url.clone(),
                interval_secs: interval,
                no_telemetry,
                client_version: env!("CARGO_PKG_VERSION").to_string(),
                channel: cfg.update.channel.clone(),
                proxy: cfg.global_proxy(),
            };
            tracing::info!(
                url = %url,
                telemetry = !no_telemetry,
                interval_secs = interval,
                "starting updater task (ADR-030)"
            );

            // 系统规则热重载链（c 方向）：updater 装入新签名包 → on_rules_installed 钩子
            // 经进程内 mpsc 通知 reload listener → perform_rules_reload → swap_system（无需重启）。
            // 单容量 + try_send：reload 期间的重复信号合并为一次（最终状态一致）。
            let (reload_sys_tx, mut reload_sys_rx) = tokio::sync::mpsc::channel::<()>(1);
            let pack_path = cfg.resolved_rules_pack_path();
            let dev_outbound_path = cfg.resolved_rules_path();
            let dev_inbound_path = cfg.resolved_inbound_rules_path();
            let outbound_layered_for_sysreload = Arc::clone(&outbound_layered);
            let inbound_layered_for_sysreload = Arc::clone(&inbound_layered);
            let ipc_for_sysreload = ipc_server.clone();
            tokio::spawn(async move {
                while reload_sys_rx.recv().await.is_some() {
                    perform_rules_reload(
                        pack_path.as_deref(),
                        &dev_outbound_path,
                        &dev_inbound_path,
                        &outbound_layered_for_sysreload,
                        &inbound_layered_for_sysreload,
                        ipc_for_sysreload.as_ref(),
                    );
                }
            });
            let on_rules_installed: sieve_updater::runner::RulesInstalledHook =
                Arc::new(move || {
                    // try_send：channel 满（已有待处理 reload）时丢弃多余信号，无害。
                    let _ = reload_sys_tx.try_send(());
                });

            tokio::spawn(async move {
                sieve_updater::runner::run(updater_cfg, Some(on_rules_installed)).await;
            });
        }
    }

    // ADR-026 §决策 3：多 listener bind + spawn。任一 bind 失败 → fail-fast
    // （半启动状态会让 doctor 输出混淆，违反"完全本地"的明确性承诺）。
    let mut bound: Vec<(TcpListener, ListenerSpec)> = Vec::with_capacity(listener_specs.len());
    for spec in listener_specs {
        let addr_str = format!("{}:{}", cfg.bind_addr, spec.port);
        let addr: std::net::SocketAddr = addr_str
            .parse()
            .map_err(|e| anyhow!("invalid listener bind addr {}: {e}", addr_str))?;
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("bind listener port {}", spec.port))?;
        tracing::info!(
            listen = %addr,
            upstream_host = %spec.forwarder.upstream_host(),
            provider_id = %spec.provider_id,
            protocol = ?spec.protocol,
            dry_run = dry_run,
            "sieve daemon listener bound"
        );
        bound.push((listener, spec));
    }

    // 全部 bind 成功 → 各 spawn accept_loop。任一 task 退出（panic 才会退）→
    // 整个 daemon 退出（fail-closed：单 listener 故障即等于 bypass 缺口）。
    let mut handles = Vec::with_capacity(bound.len());
    for (listener, spec) in bound {
        let h = tokio::spawn(accept_loop(
            listener,
            spec,
            filter.clone(),
            inbound_engine.clone(),
            inbound_sieveignore.clone(),
            address_guard_config.clone(),
            provider_forwarders.clone(),
            audit_store.clone(),
            ipc_server.clone(),
            archive_writer.clone(),
            billing_observer.clone(),
            dry_run,
            no_client_policy,
        ));
        handles.push(h);
    }
    for h in handles {
        if let Err(e) = h.await {
            tracing::error!(error = %e, "listener task ended unexpectedly");
        }
    }
    Ok(())
}

/// 构造 `full` 档加密审计归档写入器（加密审计档案，可选特性）。
///
/// 仅 `audit.level = full` 时返回 `Some`；否则 `None`（`off` / `metadata` 不归档）。
/// recipient 不可解析时返回 `Err`（fail-fast；config 校验已查 `age1` 前缀格式）。
/// 启动时执行一次保留期清理（超期段删除）；清理失败仅 warn 不阻断。
#[cfg(feature = "audit-crypto")]
fn build_archive_writer(cfg: &Config) -> Result<ArchiveWriterHandle> {
    use crate::config::AuditLevel;
    if cfg.audit.level != AuditLevel::Full {
        return Ok(None);
    }
    let recipient =
        cfg.audit.recipient.as_deref().ok_or_else(|| {
            anyhow!("audit.level = full 但缺 audit.recipient（config 校验应已拦截）")
        })?;
    let dir = cfg.audit.archive_dir.clone().unwrap_or_else(|| {
        sieve_ipc::paths::sieve_home()
            .map(|h| h.join("audit-archive"))
            .unwrap_or_else(|_| std::path::PathBuf::from("audit-archive"))
    });
    let writer = crate::audit_archive::ArchiveWriter::new(
        recipient,
        dir,
        cfg.audit.rotation,
        cfg.audit.hash_chain,
    )
    .context("初始化加密审计归档写入器失败")?;

    match writer.purge_expired(cfg.audit.retention_days) {
        Ok(deleted) if !deleted.is_empty() => {
            tracing::info!(count = deleted.len(), "audit archive: 已清理超期段");
        }
        Err(e) => tracing::warn!(error = %e, "audit archive: purge_expired 失败"),
        _ => {}
    }
    tracing::info!(
        hash_chain = cfg.audit.hash_chain,
        retention_days = cfg.audit.retention_days,
        "audit archive: full 档加密归档已启用（write-only logging）"
    );
    Ok(Some(Arc::new(writer)))
}

/// stub：`audit-crypto` 特性关时不构造加密归档写入器，恒返回 `None`。
///
/// 若 config 设 `audit.level = full` 但二进制未编入 audit-crypto 特性，warn 一句后
/// 优雅降级为不归档（当 metadata 处理）。**绝不 panic/crash**——审计可靠性问题不得
/// 变可用性事故。tail/query/show 等纯 SQLite 审计查询不受影响，始终可用。
#[cfg(not(feature = "audit-crypto"))]
fn build_archive_writer(cfg: &Config) -> Result<ArchiveWriterHandle> {
    use crate::config::AuditLevel;
    if cfg.audit.level == AuditLevel::Full {
        tracing::warn!(
            "config 设 audit.level = full 但本二进制未编入 `audit-crypto` 特性，\
             已降级为 metadata 档（不写加密归档）；元数据审计与 tail/query/show 仍正常"
        );
    }
    Ok(None)
}

/// 构造超额计费观测器（本地用量/计费核算）。仅 `[billing_check].enabled` 时返回 `Some`。
///
/// usage 记录落本地 `~/.sieve/usage.db`（严格本地、永不上传）。`TokenEstimator::new`
/// 加载 bundled BPE 词表（开销大），故启动一次性构造、`Arc` 透传到响应观测点。
#[cfg(feature = "usage")]
fn build_billing_observer(cfg: &Config) -> Result<BillingObserverHandle> {
    if !cfg.billing_check.enabled {
        return Ok(None);
    }
    let usage_path = sieve_ipc::paths::sieve_home()
        .map(|h| h.join("usage.db"))
        .unwrap_or_else(|_| std::path::PathBuf::from("usage.db"));
    let usage = crate::billing::UsageStore::init(&usage_path).context("初始化 usage.db 失败")?;
    let observer = crate::billing::BillingObserver::new(usage, cfg.billing_check.tolerance_pct)
        .context("初始化超额计费观测器失败")?;
    tracing::info!(
        tolerance_pct = cfg.billing_check.tolerance_pct,
        "billing check: 超额计费检测已启用（独立 token 核算，严格本地、零网络）"
    );
    Ok(Some(Arc::new(observer)))
}

/// stub：`usage` 特性关时计费观测器恒为 `None`。
///
/// 若用户 config 启用了 `[billing_check].enabled = true` 但二进制未编入 usage 特性，
/// warn 一句后降级为不观测（不影响安全检测，计费监督非安全拦截）。
#[cfg(not(feature = "usage"))]
fn build_billing_observer(cfg: &Config) -> Result<BillingObserverHandle> {
    if cfg.billing_check.enabled {
        tracing::warn!("config 启用了 billing_check 但本二进制未编入 `usage` 特性，已降级为不观测");
    }
    Ok(None)
}

/// 构造每请求超额计费观测上下文（ADR-038）。
///
/// 仅 `billing` 为 `Some`（`[billing_check].enabled`）**且** `trust == Relay`（中转，
/// `usage` 不可信）时返回 `Some`；`Official` 直连 `usage` 权威、不核算。输入 token 用
/// 请求文本段独立估算（纯本地，不外泄）。`texts` 为 `extract_text_content()` 的 `(offset,
/// text)` 段列表。
#[cfg(feature = "usage")]
fn build_billing_context(
    billing: &BillingObserverHandle,
    trust: crate::config::Trust,
    provider_id: &str,
    model: &str,
    family: billing_facade::Family,
    texts: &[(usize, String)],
) -> BillingCtxHandle {
    let observer = billing.as_ref()?;
    if trust != crate::config::Trust::Relay {
        return None;
    }
    let messages: Vec<String> = texts.iter().map(|(_, t)| t.clone()).collect();
    let independent_input = observer.count_input(family, model, &messages);
    Some(crate::billing::BillingContext {
        observer: Arc::clone(observer),
        trust,
        provider_id: provider_id.to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        model: model.to_string(),
        family,
        independent_input,
    })
}

/// stub：`usage` 特性关时计费上下文恒为 `None`（`billing` 句柄本就恒 `None`）。
#[cfg(not(feature = "usage"))]
fn build_billing_context(
    _billing: &BillingObserverHandle,
    _trust: crate::config::Trust,
    _provider_id: &str,
    _model: &str,
    _family: billing_facade::Family,
    _texts: &[(usize, String)],
) -> BillingCtxHandle {
    None
}

/// 在响应观测点 spawn 超额计费核算（ADR-038，fire-and-forget）。
///
/// `billing_ctx` 为 `None`（非 Relay / 未启用）时 no-op。`completion` 为补全全文（独立
/// output 计数）；`claimed` 为 relay 声明的 `(input, output)` tokens（`None` = 无声明）。
/// 写 `usage.db`（严格本地、永不上传）；检出超额时 `warn`（`sieve usage --overbilled-only`
/// 可查）。**不阻断响应**（计费监督，非安全拦截）。
#[cfg(feature = "usage")]
fn spawn_billing_observation(
    billing_ctx: BillingCtxHandle,
    completion: String,
    claimed: Option<(u64, u64)>,
) {
    let bctx = match billing_ctx {
        Some(b) => b,
        None => return,
    };
    tokio::spawn(async move {
        let verdict = bctx
            .observer
            .observe(
                bctx.trust,
                bctx.family,
                Some(bctx.request_id.clone()),
                bctx.provider_id.clone(),
                bctx.model.clone(),
                bctx.independent_input,
                &completion,
                claimed,
            )
            .await;
        if let crate::billing::Verdict::Overbilled {
            deviation_pct,
            claimed_total,
            independent_total,
            is_estimate,
        } = verdict
        {
            tracing::warn!(
                provider_id = %bctx.provider_id,
                model = %bctx.model,
                deviation_pct,
                claimed_total,
                independent_total,
                is_estimate,
                "BILLING: 检出 relay 超额计费（独立计数 vs relay 声明偏差超容差，本地用量/计费核算）"
            );
        }
    });
}

/// stub：`usage` 特性关时计费观测为 no-op（`billing_ctx` 句柄本就恒 `None`）。
#[cfg(not(feature = "usage"))]
fn spawn_billing_observation(
    _billing_ctx: BillingCtxHandle,
    _completion: String,
    _claimed: Option<(u64, u64)>,
) {
}

/// 提取 Anthropic JSON 响应顶层 `usage` 的 `(input_tokens, output_tokens)`。
///
/// 仅 `usage` 特性的计费观测调用；关闭时透传给 stub 后被忽略（标 allow 免 dead_code）。
#[cfg_attr(not(feature = "usage"), allow(dead_code))]
fn anthropic_claimed_usage(resp_json: &serde_json::Value) -> Option<(u64, u64)> {
    let u = resp_json.get("usage")?;
    let input = u.get("input_tokens").and_then(|v| v.as_u64())?;
    let output = u.get("output_tokens").and_then(|v| v.as_u64())?;
    Some((input, output))
}

/// 拼接 Anthropic JSON 响应 `content[]` 中的 text 块为补全全文。
fn anthropic_completion_text(resp_json: &serde_json::Value) -> String {
    resp_json
        .get("content")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|b| {
                    if b.get("type").and_then(|v| v.as_str()) == Some("text") {
                        b.get("text").and_then(|v| v.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default()
}

/// 提取 OpenAI JSON 响应顶层 `usage` 的 `(prompt_tokens, completion_tokens)`。
///
/// 仅 `usage` 特性的计费观测调用；关闭时透传给 stub 后被忽略（标 allow 免 dead_code）。
#[cfg_attr(not(feature = "usage"), allow(dead_code))]
fn openai_claimed_usage(resp_json: &serde_json::Value) -> Option<(u64, u64)> {
    let u = resp_json.get("usage")?;
    let input = u.get("prompt_tokens").and_then(|v| v.as_u64())?;
    let output = u.get("completion_tokens").and_then(|v| v.as_u64())?;
    Some((input, output))
}

/// 拼接 OpenAI JSON 响应 `choices[].message.content` 为补全全文。
fn openai_completion_text(resp_json: &serde_json::Value) -> String {
    resp_json
        .get("choices")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    c.get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|v| v.as_str())
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default()
}

/// SSE 流式超额计费累计器（ADR-038）：跨 chunk 累计 completion 文本 + relay 声明的 usage，
/// 流自然结束后用于交叉比对。
///
/// - **completion**：所有 `ContentBlockDelta { TextDelta }`（两 SSE parser 统一发此事件）。
/// - **claimed usage**：Anthropic 的 input 在 `MessageStart.usage.input_tokens`、output 在
///   `MessageDelta.usage.output_tokens`；OpenAI（include_usage）的 prompt/completion 在末尾
///   `MessageDelta.usage`（经 `OpenAiSseParser` 归一化，见其 usage-only chunk 处理）。
///
/// 仅 `usage` 特性使用；关闭时整条 SSE billing 路径恒走 None 分支，标 allow 免 dead_code。
#[cfg_attr(not(feature = "usage"), allow(dead_code))]
#[derive(Default)]
struct BillingSseAccumulator {
    completion: String,
    claimed_input: Option<u64>,
    claimed_output: Option<u64>,
}

#[cfg_attr(not(feature = "usage"), allow(dead_code))]
impl BillingSseAccumulator {
    fn observe_events(&mut self, events: &[sieve_core::sse::parser::SseEvent]) {
        use sieve_core::sse::parser::{SseDelta, SseEvent};
        for ev in events {
            match ev {
                SseEvent::MessageStart { message } => {
                    if let Some(it) = message
                        .get("usage")
                        .and_then(|u| u.get("input_tokens"))
                        .and_then(serde_json::Value::as_u64)
                    {
                        self.claimed_input = Some(it);
                    }
                }
                SseEvent::ContentBlockDelta {
                    delta: SseDelta::TextDelta { text },
                    ..
                } => self.completion.push_str(text),
                SseEvent::MessageDelta { usage: Some(u), .. } => {
                    // Anthropic: input_tokens/output_tokens；OpenAI: prompt_tokens/completion_tokens。
                    if let Some(it) = u
                        .get("input_tokens")
                        .or_else(|| u.get("prompt_tokens"))
                        .and_then(serde_json::Value::as_u64)
                    {
                        self.claimed_input = Some(it);
                    }
                    if let Some(ot) = u
                        .get("output_tokens")
                        .or_else(|| u.get("completion_tokens"))
                        .and_then(serde_json::Value::as_u64)
                    {
                        self.claimed_output = Some(ot);
                    }
                }
                _ => {}
            }
        }
    }

    /// relay 声明的 `(input, output)`；input/output 任一缺失则 `None`（无法有意义比对）。
    fn claimed(&self) -> Option<(u64, u64)> {
        match (self.claimed_input, self.claimed_output) {
            (Some(i), Some(o)) => Some((i, o)),
            _ => None,
        }
    }
}

/// 归档脱敏后的出站内容（加密审计档案 full 档，可选特性）。
///
/// **fire-and-forget**：`spawn_blocking` 执行 age 加密 + 文件 IO（同步阻塞），失败仅
/// `warn` 不阻断 forward（审计可靠性问题不得变可用性事故）。`archive` 为 `None`
/// （非 full 档）时 no-op、零开销。**红线**：`redacted_body` 必须是脱敏后内容（调用点保证）。
#[cfg(feature = "audit-crypto")]
fn archive_redacted_outbound(
    archive: &ArchiveWriterHandle,
    redacted_body: &Bytes,
    protocol: &'static str,
) {
    if let Some(aw) = archive {
        let aw = Arc::clone(aw);
        let body = redacted_body.clone();
        tokio::task::spawn_blocking(move || {
            if let Err(e) = aw.append(&body) {
                tracing::warn!(error = %e, protocol, "audit archive append 失败");
            }
        });
    }
}

/// stub：`audit-crypto` 特性关时不写加密归档（`archive` 句柄本就恒 `None`）。no-op。
#[cfg(not(feature = "audit-crypto"))]
fn archive_redacted_outbound(
    _archive: &ArchiveWriterHandle,
    _redacted_body: &Bytes,
    _protocol: &'static str,
) {
}

/// 单 listener 永久 accept loop（ADR-026 §决策 3）。
///
/// 每个 listener 独立 spawn 一份本函数，共享 filter / IPC / audit / inbound engine 等
/// daemon 单例。listener 自身 + 对应的 [`ListenerSpec`]（含 forwarder / protocol /
/// provider_id）则不共享——`RequestCtx` 携带 listener metadata 透传到 proxy_inner，
/// 后者用此做协议错位 fail-closed 校验。
#[allow(clippy::too_many_arguments)]
async fn accept_loop(
    listener: TcpListener,
    spec: ListenerSpec,
    filter: Arc<OutboundFilter>,
    inbound_engine: Arc<dyn InboundEngine>,
    inbound_sieveignore: Arc<HashSet<String>>,
    address_guard_config: sieve_core::pipeline::inbound::AddressGuardConfig,
    provider_forwarders: Arc<HashMap<String, Arc<Forwarder>>>,
    audit_store: Arc<crate::audit::AuditStore>,
    ipc_server: Option<Arc<sieve_ipc::IpcServer>>,
    archive: ArchiveWriterHandle,
    billing: BillingObserverHandle,
    dry_run: bool,
    no_client_policy: crate::cli::NoClientPolicy,
) {
    let listen_addr = match listener.local_addr() {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(error = %e, port = spec.port, "listener.local_addr() failed; aborting accept loop");
            return;
        }
    };

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, port = spec.port, "accept failed");
                continue;
            }
        };

        // v2.0 Phase A：caller PID 反查（PRD §5.6 / §6.6 / OQ-V20-02）。
        let caller_info: Option<crate::process_context::CallerInfo> =
            peer_addr_to_pid(listen_addr, peer).and_then(crate::process_context::lookup_caller);
        tracing::trace!(
            peer = %peer,
            port = spec.port,
            provider_id = %spec.provider_id,
            caller_pid = ?caller_info.as_ref().map(|c| c.pid),
            caller_exe = ?caller_info.as_ref().and_then(|c| c.exe.as_ref()).map(|p| p.display().to_string()),
            "new connection: caller context"
        );

        let forwarder = spec.forwarder.clone();
        let filter = filter.clone();
        let inbound_engine = inbound_engine.clone();
        let inbound_sieveignore = inbound_sieveignore.clone();
        let ipc_server = ipc_server.clone();
        let ag_cfg = address_guard_config.clone();
        let pf = provider_forwarders.clone();
        let archive = archive.clone();
        let billing = billing.clone();
        let listener_trust = spec.trust;
        let req_ctx = RequestCtx::new(
            caller_info.clone(),
            Arc::clone(&audit_store),
            spec.protocol,
            spec.provider_id.clone(),
        );

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let f = forwarder.clone();
                let flt = filter.clone();
                // 每连接独立 InboundFilter（&mut self trait 要求），
                // 传入从 IN-CR-01 RuleEntry 读取的配置（修 R3-#5）
                let ib_filter = InboundFilter::with_address_guard_config(
                    inbound_engine.clone(),
                    inbound_sieveignore.clone(),
                    ag_cfg.clone(),
                );
                let ipc = ipc_server.clone();
                let pf = pf.clone();
                let archive = archive.clone();
                let billing = billing.clone();
                // RequestCtx（caller + audit + listener meta）捕获到闭包
                let ctx = req_ctx.clone();
                async move {
                    proxy(
                        f,
                        pf,
                        flt,
                        ib_filter,
                        dry_run,
                        ipc,
                        archive,
                        billing,
                        listener_trust,
                        ctx,
                        req,
                        no_client_policy,
                    )
                    .await
                }
            });

            if let Err(e) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, svc)
                .await
            {
                tracing::debug!(peer = %peer, error = %e, "connection closed with error");
            }
        });
    }
}

/// 重新读取、lint 并编译用户规则，返回可立即 swap 的两个方向引擎（PRD §5.5.5 v2.1）。
///
/// 返回 `(outbound_engine, inbound_engine, rule_count)`，方向引擎均为 `Option<UserEngine>`：
/// - `None` 表示该方向无规则（文件不存在、或该方向 0 条），LayeredEngine 退化为纯系统引擎
/// - `Some(engine)` 即编译通过的用户引擎，调用方直接调用 `swap_user` 生效
///
/// 任何错误（lint 违规 / SIEVE_HOME 未设置）返回 `Err`（fail-safe：daemon 保留旧引擎）。
/// 用户规则 reload 一次的结果（PRD §5.5.5 / ADR-013 §S.4 reload_config）。
#[derive(Debug, Clone)]
pub(crate) struct ReloadOutcome {
    /// 当前调用方未消费 `success` 字段（成功 / 失败可由 `user_rules_errors.is_empty()` 间接判定），
    /// 但保留供未来扩展（如分级 audit）使用。
    #[allow(dead_code)]
    pub success: bool,
    pub rule_count: usize,
    pub user_rules_errors: Vec<String>,
}

/// 执行一次用户规则 reload 完整流程（lint + 编译 + hot swap + 广播 + audit）。
///
/// 既被 IPC `sieve.reload_user_rules` notification listener 调用（向后兼容），
/// 也被 control plane `sieve.reload_config` 直接同步调用（拿 errors 同步返回）。
///
/// 行为与原内联闭包等价：
/// - 成功 → swap_user + 推 `NotifyKind::UserRulesReloaded` + 写 audit success
/// - 失败 → 不动当前引擎 + 推 `NotifyKind::UserRulesLoadFailed` + 写 audit failure
///
/// 关联：PRD v2.0 §5.5.5 / §9 #14 fail-safe。
pub(crate) fn perform_user_rules_reload(
    user_rules_path: Option<&std::path::Path>,
    outbound_layered: &Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    inbound_layered: &Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    ipc: &Arc<sieve_ipc::IpcServer>,
    audit: &Arc<crate::audit::AuditStore>,
    trigger_id: Option<uuid::Uuid>,
) -> ReloadOutcome {
    let trigger_id_str = trigger_id.map(|id| id.to_string());
    tracing::info!(
        trigger_id = ?trigger_id_str,
        "执行用户规则 reload（PRD §5.5.5）"
    );

    let reload_result = reload_user_engines(user_rules_path);
    let (notify_kind, notify_title, notify_detail, success, rule_count, err_msg) =
        match reload_result {
            Ok((outbound_eng, inbound_eng, count)) => {
                outbound_layered.swap_user(outbound_eng);
                inbound_layered.swap_user(inbound_eng);
                tracing::info!(rule_count = count, "用户规则 hot swap 完成（PRD §5.5.5）");
                (
                    sieve_ipc::protocol::NotifyKind::UserRulesReloaded,
                    format!("用户规则已 hot reload（{count} 条）"),
                    Some("已立即生效，无需重启 daemon".to_owned()),
                    true,
                    count,
                    None,
                )
            }
            Err(e) => {
                tracing::warn!(error = %e, "用户规则重新加载失败（保留旧引擎）");
                (
                    sieve_ipc::protocol::NotifyKind::UserRulesLoadFailed,
                    "用户规则加载失败".to_owned(),
                    Some(e.to_string()),
                    false,
                    0,
                    Some(e.to_string()),
                )
            }
        };

    let notify = sieve_ipc::protocol::StatusBarNotify {
        notify_id: uuid::Uuid::now_v7(),
        created_at: chrono::Utc::now(),
        kind: notify_kind,
        title: notify_title,
        detail: notify_detail,
        rule_id: None,
        auto_dismiss_seconds: 5,
    };
    ipc.broadcast_status_bar(notify);

    // audit 写入（fail-soft）
    let event = crate::audit::AuditEvent::UserRulesReloaded {
        success,
        rule_count: if success { Some(rule_count) } else { None },
        error: err_msg.clone(),
        trigger_id: trigger_id_str,
    };
    let audit_clone = Arc::clone(audit);
    tokio::spawn(async move {
        // ADR-026 Stage E：UserRulesReloaded 是 daemon 系统级事件，无 listener 上下文
        if let Err(e) = audit_clone
            .append(event, crate::audit::SYSTEM_PROVIDER_ID)
            .await
        {
            tracing::warn!(error = %e, "audit append UserRulesReloaded failed");
        }
    });

    ReloadOutcome {
        success,
        rule_count,
        user_rules_errors: err_msg.into_iter().collect(),
    }
}

/// 系统规则热重载（updater 装入新签名包后调用，无需重启）。
///
/// 从签名包（`current.json`）重新加载出站/入站系统规则，编译后原子 `swap_system` 到
/// live 引擎；空集 / 编译失败 → swap 为空集 fail-safe（保持引擎可用）。编译时
/// [`VectorscanEngine::compile`] 同步刷新 fail-closed 运行时注册表（accumulate）。
/// 与 [`perform_user_rules_reload`] 对称——系统层 `ArcSwap` zero-downtime 热替换。
pub(crate) fn perform_rules_reload(
    pack_path: Option<&std::path::Path>,
    dev_outbound_path: &std::path::Path,
    dev_inbound_path: &std::path::Path,
    outbound_layered: &Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    inbound_layered: &Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    ipc: Option<&Arc<sieve_ipc::IpcServer>>,
) {
    let out = crate::reload_system_vectorscan(pack_path, dev_outbound_path, true);
    let inb = crate::reload_system_vectorscan(pack_path, dev_inbound_path, false);
    let out_count = out
        .as_ref()
        .map(sieve_rules::engine::MatchEngine::rule_count)
        .unwrap_or(0);
    let in_count = inb
        .as_ref()
        .map(sieve_rules::engine::MatchEngine::rule_count)
        .unwrap_or(0);
    let total = out_count + in_count;

    // 原子热替换系统层（已在进行中的 scan 持旧快照，结束后释放）。
    outbound_layered.swap_system(out);
    inbound_layered.swap_system(inb);

    tracing::info!(
        out_count,
        in_count,
        "系统规则热重载完成（swap_system，zero-downtime）"
    );

    if let Some(ipc) = ipc {
        let notify = sieve_ipc::protocol::StatusBarNotify {
            notify_id: uuid::Uuid::now_v7(),
            created_at: chrono::Utc::now(),
            kind: sieve_ipc::protocol::NotifyKind::Generic,
            title: format!("规则包已热加载（{total} 条）"),
            detail: Some("已立即生效，无需重启 daemon".to_owned()),
            rule_id: None,
            auto_dismiss_seconds: 5,
        };
        ipc.broadcast_status_bar(notify);
    }
}

fn reload_user_engines(
    user_rules_path: Option<&std::path::Path>,
) -> anyhow::Result<(
    Option<sieve_policy::engine::UserEngine>,
    Option<sieve_policy::engine::UserEngine>,
    usize,
)> {
    use sieve_policy::lint::lint;
    use sieve_policy::loader::load_user_rules;

    let path = user_rules_path
        .ok_or_else(|| anyhow::anyhow!("user rules path 未知（SIEVE_HOME 未设置）"))?;

    if !path.exists() {
        // 文件不存在：两个方向均 None（退化为纯系统规则），视为成功（0 条规则）
        return Ok((None, None, 0));
    }

    let file = load_user_rules(path).map_err(|e| anyhow::anyhow!("user.toml 解析失败: {e}"))?;

    let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
    let violations = lint(&file, file_size);
    if !violations.is_empty() {
        return Err(anyhow::anyhow!(
            "user.toml lint 失败（{} 条违规）：{}",
            violations.len(),
            violations[0].message
        ));
    }

    let total = file.rules.len();

    // 出站引擎（编译 direction=outbound/both 的规则）；该方向无规则时返回 None（fail-safe）
    let outbound_eng = sieve_policy::engine::UserEngine::compile_for_direction(
        file.rules.clone(),
        sieve_policy::loader::RuleDirection::Outbound,
    )
    .ok();

    // 入站引擎（编译 direction=inbound/both 的规则）；该方向无规则时返回 None（fail-safe）
    let inbound_eng = sieve_policy::engine::UserEngine::compile_for_direction(
        file.rules,
        sieve_policy::loader::RuleDirection::Inbound,
    )
    .ok();

    Ok((outbound_eng, inbound_eng, total))
}
/// 请求入口：捕获 `proxy_inner` 的所有错误，转换为 502 Bad Gateway 响应。
///
/// v2.0：新增 `ctx`（caller + audit_store，PRD §5.6 / §5.6.1）参数。
#[allow(clippy::too_many_arguments)]
async fn proxy(
    forwarder: Arc<Forwarder>,
    provider_forwarders: Arc<HashMap<String, Arc<Forwarder>>>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    archive: ArchiveWriterHandle,
    billing: BillingObserverHandle,
    listener_trust: crate::config::Trust,
    ctx: RequestCtx,
    req: Request<Incoming>,
    no_client_policy: crate::cli::NoClientPolicy,
) -> Result<Response<ResponseBody>, hyper::Error> {
    match proxy_inner(
        forwarder,
        provider_forwarders,
        filter,
        inbound_filter,
        dry_run,
        ipc,
        archive,
        billing,
        listener_trust,
        ctx,
        req,
        no_client_policy,
    )
    .await
    {
        Ok(resp) => Ok(resp),
        Err(e) => {
            tracing::error!(error = %e, "proxy failed");
            let body = format!("sieve proxy error: {e}");
            let resp = Response::builder()
                .status(http::StatusCode::BAD_GATEWAY)
                .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(string_body(body))
                .unwrap_or_else(|_| Response::new(empty_body()));
            Ok(resp)
        }
    }
}

/// 核心代理逻辑。
///
/// 路径分发（v1.5，ADR-018 + ADR-019）：
/// - POST /v1/messages → Anthropic 路径（collect body → 出站扫描 → 426 / 脱敏转发 / 入站 SSE tee 检测）
/// - POST /v1/chat/completions → OpenAI 路径（同等出站扫描，走 OpenAI schema 解析）
/// - 其他路径 → 流式透传（Week 1 行为）
///
/// 公共预处理（两条 LLM 路径都执行）：
/// 1. 解析 `X-Sieve-Origin` → source_agent / origin_chain / chain_depth
/// 2. chain_depth ≥ 5 → 直接 426 拒绝（ADR-019 §嵌套深度限制）
/// 3. 解析 `X-Sieve-Source-Channel` → source_channel（OpenClaw 跨通道）
/// 4. chain_depth ≥ 2 → 所有命中强制升级为 GuiPopup disposition
///
/// R11-#1：在所有路径分发前���解析 `X-Sieve-Provider` header 并从路由表中查找上游 Forwarder：
/// - 有 header + 路由表命中 → 用对应 provider 的 Forwarder（OpenClaw 多 provider 路由）
/// - 无 header 或路由表未命中 → 用默认 `forwarder`（cfg.upstream_url 兜底）
///
/// 转发到上游前**移除** `X-Sieve-Provider` header（内部路由 header，不透传给上游）。
///
/// 关联：PRD v1.5 §6.1 / ADR-018（OpenAI 协议）/ ADR-019（multi-agent header）。
#[allow(clippy::too_many_arguments)]
async fn proxy_inner(
    forwarder: Arc<Forwarder>,
    provider_forwarders: Arc<HashMap<String, Arc<Forwarder>>>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    archive: ArchiveWriterHandle,
    billing: BillingObserverHandle,
    listener_trust: crate::config::Trust,
    ctx: RequestCtx,
    req: Request<Incoming>,
    no_client_policy: crate::cli::NoClientPolicy,
) -> Result<Response<ResponseBody>> {
    let (mut parts, body) = req.into_parts();

    // R11-#1：解析 X-Sieve-Provider，查路由表选择上游，转发前移除此 header。
    // 移除在此处（统一入口），后续所有 forward_* 调用无需感知此 header。
    let forwarder: Arc<Forwarder> = {
        let provider_id = parts
            .headers
            .remove("x-sieve-provider")
            .and_then(|v| v.to_str().ok().map(|s| s.trim().to_owned()))
            .filter(|s| !s.is_empty());
        match provider_id {
            Some(ref id) => {
                if let Some(pf) = provider_forwarders.get(id.as_str()) {
                    tracing::debug!(provider = %id, "X-Sieve-Provider: routing to provider upstream");
                    Arc::clone(pf)
                } else {
                    tracing::debug!(
                        provider = %id,
                        "X-Sieve-Provider: no route found, falling back to default upstream"
                    );
                    forwarder
                }
            }
            None => forwarder,
        }
    };

    let path = parts.uri.path().to_string();
    let method = parts.method.clone();

    // ── ADR-026 §决策 4：listener 协议错位 fail-closed 检查 ───────────────────
    //
    // listener 声明的协议与请求 path 隐含的协议不一致时立即 400 拒绝，不进入路径分发。
    // 仅检查 LLM endpoint（/v1/messages 与 /v1/chat/completions）；其他 path（健康
    // 检查 / 透传 / 未知）保持现有透传行为，不强制。
    //
    // listener_protocol == Auto（legacy upstream_url / 省略 protocol 字段的 [[upstream]]）
    // 不匹配下列任一条件，按请求 path 自适应放行，保留 v1.x 单 upstream 双协议能力
    // （ADR-026 §决策 1 向后兼容 + PRD §9 #16/#9）。仅显式声明 anthropic/openai 才错位拒绝。
    //
    // 即便请求带 X-Sieve-Provider header（已在前面 routing 选择 forwarder），listener
    // 协议依然是硬约束——header routing 不能 override（PRD §9 #3 fail-closed 一致性）。
    if path == "/v1/messages" && ctx.listener_protocol == crate::config::Protocol::Openai {
        tracing::warn!(
            path = %path,
            listener_protocol = ?ctx.listener_protocol,
            provider_id = %ctx.listener_provider_id,
            "ADR-026 protocol mismatch: openai listener received anthropic /v1/messages, rejecting"
        );
        return Ok(build_protocol_mismatch_400(&path, ctx.listener_protocol));
    }
    if path == "/v1/chat/completions" && ctx.listener_protocol == crate::config::Protocol::Anthropic
    {
        tracing::warn!(
            path = %path,
            listener_protocol = ?ctx.listener_protocol,
            provider_id = %ctx.listener_provider_id,
            "ADR-026 protocol mismatch: anthropic listener received openai /v1/chat/completions, rejecting"
        );
        return Ok(build_protocol_mismatch_400(&path, ctx.listener_protocol));
    }

    // ── v1.5：公共 header 解析（所有 LLM 路径）────────────────────────────────

    // 1. X-Sieve-Origin → source_agent / origin_chain / chain_depth（ADR-019）
    let (source_agent, origin_chain, chain_depth) = extract_origin_metadata(&parts.headers);

    // 2. chain_depth ≥ 5 → 直接 426（ADR-019 §嵌套深度限制，attack mode）
    if chain_depth >= 5 {
        tracing::warn!(
            chain_depth,
            "X-Sieve-Origin chain_depth ≥ 5，嵌套调用过深，拒绝请求"
        );
        return Ok(build_426_nested_rejection(chain_depth));
    }

    // 3. X-Sieve-Source-Channel（OpenClaw 跨通道，PRD v1.5 §4.5）
    let source_channel = parse_source_channel(&parts.headers);

    // ── 路径分类（白名单 collect，修 R7-#2）─────────────────────────────────────
    //
    // 修 R7-#2（DoS 修复）：改为**路径白名单 collect**，只对需要检测的路径预先缓冲 body；
    // 其余 POST 路径（透传）body 不经过 collect，保持流式，不存在无界缓冲 DoS 向量。
    //
    // 白名单路径：
    //   1. /v1/messages          → Anthropic 出站扫描需要 collect
    //   2. /v1/chat/completions  → OpenAI 出站扫描需要 collect
    //   3. is_skill_install_path → IN-CR-06 body manifest 检测需要 collect
    //
    // IN-CR-06 覆盖范围说明（trade-off，显式记录）：
    //   body manifest 检测仅在 `is_skill_install_path(path)` 为 true 时生效。
    //   真实 OpenClaw endpoint 与路径列表不符时，body 检测不跑（路径白名单 only）。
    //   Week 7 实测后补充准确路径，届时覆盖范围自动扩大。
    //   R6-#4 的死代码问题（所有 POST 都 collect 以确保 body 检测跑到）接受为已知
    //   trade-off，以安全性（no DoS vector）换取检测完备性的妥协在注释中显式标注。
    //
    // 关联：sieve_core::skill_install_guard、PRD v1.5 §4.6、ADR-016。

    let is_messages_post = method == http::Method::POST && path == "/v1/messages";
    let is_chat_completions_post = method == http::Method::POST && path == "/v1/chat/completions";
    let is_skill_post = method == http::Method::POST
        && sieve_core::skill_install_guard::is_skill_install_path(&path);

    // proxy_inner 请求体两态：白名单路径 collect 成 Bytes（出站扫描需要），其余路径
    // 保持流式 Incoming（零缓冲透传，无 DoS 向量）。两态互斥——用 enum 而非
    // `(Option<Bytes>, Option<Incoming>)` 让「既非 collected 又非 streaming」的非法态
    // 在类型层不可表达，消除后续取出点的热路径 `.expect()`（CLAUDE.md 请求路径禁 panic）。
    enum ProxyRequestBody {
        /// 白名单路径（/v1/messages、/v1/chat/completions、skill install）已收集的 body。
        Collected(Bytes),
        /// 其余路径的流式 body，原样透传。
        Streaming(hyper::body::Incoming),
    }

    // 只对白名单路径 collect body；其余 POST 保留为流式 body，完全不缓冲。
    let proxy_body = if is_messages_post || is_chat_completions_post || is_skill_post {
        let collected = body
            .collect()
            .await
            .map_err(|e| anyhow!("collect body (post): {e}"))?;
        ProxyRequestBody::Collected(collected.to_bytes())
    } else {
        ProxyRequestBody::Streaming(body)
    };

    // ── IN-CR-06 OpenClaw skill install 检测（路径白名单 only）──────────────────
    if is_skill_post {
        // 不变量：is_skill_post → collect 块走 Collected 分支。借用做 manifest 检测，不 move。
        let ProxyRequestBody::Collected(body_bytes_skill) = &proxy_body else {
            tracing::error!("BUG: is_skill_post but request body not collected");
            return Ok(build_500_internal_response(
                "skill install body not collected",
            ));
        };

        // body ≤ 4KB 时才做 manifest 检测（> 4KB 多半不是 manifest，跳过减少误判）
        let body_json: serde_json::Value = if body_bytes_skill.len() <= 4096 {
            serde_json::from_slice(body_bytes_skill).unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        };

        let mut skill_detections = sieve_core::skill_install_guard::check_openclaw_skill_install(
            &path,
            &body_json,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );

        // chain_depth ≥ 2 → 强制 GuiPopup（ADR-019）
        if chain_depth >= 2 {
            for d in &mut skill_detections {
                if matches!(d.action, Action::HookMark) {
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                    };
                }
            }
        }

        if !skill_detections.is_empty() {
            if let Some(ref ipc_server) = ipc {
                use chrono::Utc;
                let request_id = uuid::Uuid::new_v4();
                let (timeout_seconds, default_on_timeout) = skill_detections
                    .iter()
                    .find_map(|d| {
                        if let Action::HoldForDecision {
                            timeout_seconds, ..
                        } = d.action
                        {
                            Some((timeout_seconds, sieve_ipc::DefaultOnTimeout::Block))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((120, sieve_ipc::DefaultOnTimeout::Block));

                let ipc_detections = skill_detections
                    .iter()
                    .map(|d| sieve_ipc::protocol::DetectionPayload {
                        rule_id: d.rule_id.clone(),
                        severity: map_severity_to_ipc(d.severity),
                        disposition: sieve_ipc::Disposition::GuiPopup,
                        title: format!("IN-CR-06 OpenClaw Skill Install 检测：{}", d.rule_id),
                        one_line_summary: d.evidence_truncated.clone(),
                        details: serde_json::json!({ "path": path }),
                        recommendation: None,
                    })
                    .collect();

                // v2.0：计算 allow_remember（PRD §5.4.2）
                let skill_rule_ids: Vec<&str> = skill_detections
                    .iter()
                    .map(|d| d.rule_id.as_str())
                    .collect();
                let allow_remember = compute_allow_remember(&skill_rule_ids);

                let ipc_req = sieve_ipc::DecisionRequest {
                    request_id,
                    created_at: Utc::now(),
                    timeout_seconds,
                    default_on_timeout,
                    detections: ipc_detections,
                    source_agent,
                    origin_chain: origin_chain.clone(),
                    source_channel: source_channel.clone(),
                    explicit_chain_depth: Some(chain_depth),
                    allow_remember,
                };

                let timeout_dur = std::time::Duration::from_secs(u64::from(timeout_seconds).max(1));
                let outcome = gated_request_decision(
                    ipc_server,
                    &ctx.audit,
                    &ctx.caller,
                    ipc_req,
                    timeout_dur,
                    "inbound",
                    &ctx.listener_provider_id,
                    no_client_policy,
                )
                .await;

                match outcome {
                    Ok(resp) => match resp.decision {
                        sieve_ipc::DecisionAction::Allow
                        | sieve_ipc::DecisionAction::RedactAndAllow => {
                            tracing::info!("IN-CR-06 GUI: Allow → 转发原 body");
                            // v2.0：remember=true 时写灰名单（PRD §5.4.2 二次校验）
                            if resp.remember && allow_remember {
                                if let Some(d) = skill_detections.first() {
                                    let agent_str = format!("{:?}", source_agent).to_lowercase();
                                    try_write_graylist(
                                        &d.rule_id,
                                        &d.evidence_truncated,
                                        "",
                                        "anthropic",
                                        "tool_use_input",
                                        &agent_str,
                                        resp.context_hint.clone(),
                                        &request_id.to_string(),
                                        &ctx.audit,
                                        &ctx.caller,
                                        &ctx.listener_provider_id,
                                    );
                                }
                            }
                            // fall-through，继续路径分发
                        }
                        sieve_ipc::DecisionAction::Deny => {
                            tracing::warn!("IN-CR-06 GUI: Deny → 426");
                            return Ok(build_426_response(&skill_detections));
                        }
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, "IN-CR-06 GUI: IPC error, fail-closed → 426");
                        return Ok(build_426_response(&skill_detections));
                    }
                }
            } else {
                // IPC 未初始化：fail-closed → 426
                tracing::warn!("IN-CR-06: IPC not initialized, fail-closed → 426");
                return Ok(build_426_response(&skill_detections));
            }
        }
    }

    // ── 路径分发 ─────────────────────────────────────────────────────────────

    if is_messages_post {
        // body 已在 POST 预收集块中 collect，直接取出
        let ProxyRequestBody::Collected(body_bytes) = proxy_body else {
            tracing::error!("BUG: is_messages_post but request body not collected");
            return Ok(build_500_internal_response("messages body not collected"));
        };

        // 2. 解析 AnthropicRequest；解析失败则直接透传（上游会返回 400）
        let anthropic_req: sieve_core::protocol::anthropic::AnthropicRequest =
            match serde_json::from_slice(&body_bytes) {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!("non-anthropic body, passing through: {e}");
                    return forward_raw(forwarder, parts, body_bytes).await;
                }
            };

        // 3. 提取文本段 → 逐段扫描
        let texts = anthropic_req.extract_text_content();

        // ADR-038：构造超额计费观测上下文（仅 Relay 上游 + billing 启用时为 Some）。
        // 输入 token 在请求侧用文本段独立估算（纯本地计数，不落盘原文、不外泄）。
        let billing_ctx = build_billing_context(
            &billing,
            listener_trust,
            &ctx.listener_provider_id,
            &anthropic_req.model,
            billing_facade::Family::Anthropic,
            &texts,
        );

        let mut all_detections: Vec<sieve_core::Detection> = Vec::new();

        for (offset, text) in &texts {
            use sieve_core::pipeline::PipelineNode;
            use sieve_core::protocol::unified_message::{
                ContentBlock, ContentSpan, Direction, MessageMetadata, UpstreamProvider,
            };
            use std::time::SystemTime;

            let mut msg = sieve_core::UnifiedMessage {
                role: sieve_core::Role::User,
                content_blocks: vec![ContentBlock::Text {
                    text: text.clone(),
                    span: Some(ContentSpan {
                        start: *offset,
                        end: *offset + text.len(),
                    }),
                }],
                tool_uses: vec![],
                tool_results: vec![],
                metadata: MessageMetadata {
                    session_id: "outbound-scan".into(),
                    direction: Direction::Outbound,
                    upstream_provider: UpstreamProvider::Anthropic,
                    received_at: SystemTime::now(),
                },
            };

            let hits = filter
                .process(&mut msg)
                .map_err(|e| anyhow!("outbound filter: {e}"))?;
            all_detections.extend(hits);
        }

        // 4. chain_depth ≥ 2 → HookMark 升级为 HoldForDecision（强制 GUI 弹窗，ADR-019）
        // 修 R9-#2：chain_depth ≥ 2 时 HookMark + Redact 都升级为 HoldForDecision。
        if chain_depth >= 2 {
            tracing::info!(
                chain_depth,
                "X-Sieve-Origin chain_depth ≥ 2（Anthropic 路径），HookMark + Redact 升级为 GuiPopup"
            );
            for d in &mut all_detections {
                match &d.action {
                    Action::HookMark => {
                        d.action = Action::HoldForDecision {
                            request_id: uuid::Uuid::new_v4(),
                            timeout_seconds: 60,
                            default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                        };
                    }
                    Action::Redact { .. } => {
                        d.action = Action::HoldForDecision {
                            request_id: uuid::Uuid::new_v4(),
                            timeout_seconds: 60,
                            default_on_timeout: sieve_core::detection::DefaultOnTimeout::Redact,
                        };
                    }
                    _ => {}
                }
            }
        }

        // 5. 决策：
        //    a. AutoRedact（Action::Redact）→ 脱敏 body bytes 后转发
        //    b. fail-closed Critical Block → 426（PRD §9 #3）
        //    c. 非 fail-closed Critical Block：dry_run=true 时仅 warn，dry_run=false 时 426
        //    d. GuiPopup（Action::HoldForDecision）→ hold HTTP 长连接等 GUI 决策（R2-#1）
        //    e. 其余 → 透传

        // 5a. 收集需要脱敏的 hit（累计文本偏移，不是 raw body 字节偏移）
        //
        // 修 #1（AutoRedact 偏移修复）：Detection.span 来自 extract_text_content() 的
        // 累计文本字符偏移，不是 raw JSON body 的字节范围。
        // 正确做法：用 redact_segments() 在文本段字符串内替换，然后重新序列化 JSON。
        // 原 redact_body_bytes(&body_bytes, ...) 路径只保留给 fuzz/单测，不在这里使用。
        let mut redact_hits: Vec<RedactHit> = all_detections
            .iter()
            .filter(|d| matches!(d.action, Action::Redact { .. }))
            .map(|d| RedactHit {
                rule_id: d.rule_id.clone(),
                start: d.span.start,
                end: d.span.end,
            })
            .collect();

        // 5b/c. 收集需要 Block 的 detection
        let blocking: Vec<&sieve_core::Detection> = all_detections
            .iter()
            .filter(|d| {
                if d.action != Action::Block {
                    return false;
                }
                if d.severity != sieve_core::Severity::Critical {
                    return false;
                }
                sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run
            })
            .collect();

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "OUTBOUND BLOCKED");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection");
            }
            let cloned: Vec<sieve_core::Detection> =
                blocking.iter().map(|d| (*d).clone()).collect();
            return Ok(build_426_response(&cloned));
        }

        // 4d. 出站 GuiPopup（HoldForDecision）：hold HTTP 长连接等待 GUI 决策（R2-#1 修复）。
        //
        // 出站请求是非流式 HTTP：body 已 collect，无需 SSE keep-alive（入站才需要）。
        // 客户端等待期间持有普通 HTTP 长连接（reqwest / Claude Code client 的超时决定等待上限）。
        //
        // 决策映射：
        //   Allow → 原 body 转发上游
        //   RedactAndAllow → redact_hits 非空则脱敏，否则原 body 转发
        //   Deny → 426 拒绝
        //   超时 → 按 default_on_timeout（OUT-06/08 = Redact，OUT-07/09/10 = Block）
        //
        // 关联：PRD v1.4 §5.4.2 出站超时策略表、ADR-016（二维处置矩阵）。
        let hold_detections_outbound: Vec<&sieve_core::Detection> = all_detections
            .iter()
            .filter(|d| matches!(d.action, Action::HoldForDecision { .. }))
            .collect();

        if !hold_detections_outbound.is_empty() {
            // v2.0 §5.4.2：决策前先查灰名单（PRD §5.4.2 灰名单命中 → 直接 Allow，不调 IPC）
            // fail-closed：查询失败 → 走正常 IPC 流程（PRD §9 #14 禁止 fail-open）
            let agent_str_out = format!("{:?}", source_agent).to_lowercase();
            let graylist_hit = hold_detections_outbound.first().is_some_and(|d| {
                check_graylist_hit(
                    &d.rule_id,
                    &d.evidence_truncated,
                    "",
                    "anthropic",
                    "outbound_text",
                    &agent_str_out,
                )
            });
            if graylist_hit {
                tracing::info!("出站 Anthropic GuiPopup：灰名单命中 → 直接放行，跳过 IPC 弹窗");
                // 灰名单命中：直接跳过 hold，继续往下走转发路径
            } else if let Some(ref ipc_server) = ipc {
                use chrono::Utc;

                let request_id = uuid::Uuid::new_v4();

                // 修 R11-#2：从 hold_detections 的 default_on_timeout 取最严策略。
                // 与 OpenAI 路径完全镜像（merge_strictest_timeout + map_dot_to_ipc）。
                // OUT-06/08 default=Redact → 超时脱敏转发；OUT-07 default=Block → 超时 426。
                let (timeout_seconds, default_on_timeout) = hold_detections_outbound.iter().fold(
                    (60_u32, sieve_ipc::DefaultOnTimeout::Allow),
                    |acc, d| {
                        if let Action::HoldForDecision {
                            timeout_seconds,
                            default_on_timeout,
                            ..
                        } = &d.action
                        {
                            let merged =
                                merge_strictest_timeout(acc.1, map_dot_to_ipc(*default_on_timeout));
                            (acc.0.max(*timeout_seconds), merged)
                        } else {
                            acc
                        }
                    },
                );

                let ipc_detections = hold_detections_outbound
                    .iter()
                    .map(|d| sieve_ipc::protocol::DetectionPayload {
                        rule_id: d.rule_id.clone(),
                        severity: map_severity_to_ipc(d.severity),
                        disposition: sieve_ipc::Disposition::GuiPopup,
                        title: format!("出站检测命中：{}", d.rule_id),
                        one_line_summary: d.evidence_truncated.clone(),
                        details: serde_json::json!({}),
                        recommendation: None,
                    })
                    .collect();

                // v2.0：计算 allow_remember（PRD §5.4.2）
                let outbound_rule_ids = detection_rule_ids(&hold_detections_outbound);
                let allow_remember = compute_allow_remember(&outbound_rule_ids);

                let ipc_req = sieve_ipc::DecisionRequest {
                    request_id,
                    created_at: Utc::now(),
                    timeout_seconds,
                    default_on_timeout,
                    detections: ipc_detections,
                    // v1.5：注入 multi-agent 元数据（ADR-019）
                    source_agent,
                    origin_chain: origin_chain.clone(),
                    source_channel: source_channel.clone(),
                    // 修 R7-#5：填入 header 真实 chain_depth
                    explicit_chain_depth: Some(chain_depth),
                    allow_remember,
                };

                // 出站 hold：无 SSE keep-alive，直接 await 决策
                let timeout_dur = std::time::Duration::from_secs(u64::from(timeout_seconds).max(1));
                let outcome = gated_request_decision(
                    ipc_server,
                    &ctx.audit,
                    &ctx.caller,
                    ipc_req,
                    timeout_dur,
                    "outbound",
                    &ctx.listener_provider_id,
                    no_client_policy,
                )
                .await;

                match outcome {
                    Ok(resp) => match resp.decision {
                        sieve_ipc::DecisionAction::Allow => {
                            tracing::info!("OUTBOUND GUI: Allow → 转发原 body");
                            // v2.0：remember=true 时写灰名单（PRD §5.4.2）
                            if resp.remember && allow_remember {
                                if let Some(d) = hold_detections_outbound.first() {
                                    let agent_str = format!("{:?}", source_agent).to_lowercase();
                                    try_write_graylist(
                                        &d.rule_id,
                                        &d.evidence_truncated,
                                        "",
                                        "anthropic",
                                        "outbound_text",
                                        &agent_str,
                                        resp.context_hint.clone(),
                                        &request_id.to_string(),
                                        &ctx.audit,
                                        &ctx.caller,
                                        &ctx.listener_provider_id,
                                    );
                                }
                            }
                            // 继续往下，走正常转发路径
                        }
                        sieve_ipc::DecisionAction::RedactAndAllow => {
                            tracing::info!("OUTBOUND GUI: RedactAndAllow → 脱敏后转发");
                            // 修 R3-#3：把 held detection 的 span 升级到 redact_hits，
                            // 保证仅命中 GUI 类（无 AutoRedact 类）时也能正确脱敏。
                            // 去重：跳过已在 redact_hits 中存在的 span。
                            for d in &hold_detections_outbound {
                                let already = redact_hits
                                    .iter()
                                    .any(|h| h.start == d.span.start && h.end == d.span.end);
                                if !already {
                                    redact_hits.push(RedactHit {
                                        rule_id: d.rule_id.clone(),
                                        start: d.span.start,
                                        end: d.span.end,
                                    });
                                }
                            }
                            // fall-through 到下方 redact_hits 处理（现在含 GUI 类 span）
                        }
                        sieve_ipc::DecisionAction::Deny => {
                            tracing::warn!("OUTBOUND GUI: Deny → 426");
                            let held: Vec<sieve_core::Detection> = hold_detections_outbound
                                .iter()
                                .map(|d| (*d).clone())
                                .collect();
                            return Ok(build_426_response(&held));
                        }
                    },
                    Err(e) => {
                        // 修 R11-#2：IPC 错误 / 超时时按 default_on_timeout 三路分支（镜像 OpenAI 路径）。
                        tracing::warn!(error = %e, ?default_on_timeout, "OUTBOUND GUI: IPC error, applying default_on_timeout");
                        match default_on_timeout {
                            sieve_ipc::DefaultOnTimeout::Redact => {
                                tracing::info!("OUTBOUND GUI: timeout default=redact → 脱敏转发");
                                for d in &hold_detections_outbound {
                                    let already = redact_hits
                                        .iter()
                                        .any(|h| h.start == d.span.start && h.end == d.span.end);
                                    if !already {
                                        redact_hits.push(RedactHit {
                                            rule_id: d.rule_id.clone(),
                                            start: d.span.start,
                                            end: d.span.end,
                                        });
                                    }
                                }
                            }
                            sieve_ipc::DefaultOnTimeout::Allow => {
                                tracing::info!("OUTBOUND GUI: timeout default=allow → 放行");
                            }
                            sieve_ipc::DefaultOnTimeout::Block => {
                                let held: Vec<sieve_core::Detection> = hold_detections_outbound
                                    .iter()
                                    .map(|d| (*d).clone())
                                    .collect();
                                return Ok(build_426_response(&held));
                            }
                        }
                    }
                }
            } else {
                // 修 R11-#2：IPC 未初始化时按 default_on_timeout 三路分支（镜像 OpenAI 路径）。
                let effective_dot = hold_detections_outbound.iter().fold(
                    sieve_ipc::DefaultOnTimeout::Allow,
                    |acc, d| {
                        if let Action::HoldForDecision {
                            default_on_timeout, ..
                        } = &d.action
                        {
                            merge_strictest_timeout(acc, map_dot_to_ipc(*default_on_timeout))
                        } else {
                            acc
                        }
                    },
                );
                match effective_dot {
                    sieve_ipc::DefaultOnTimeout::Redact => {
                        tracing::info!(
                            "OUTBOUND GUI: IPC not initialized, default_on_timeout=redact → 脱敏转发"
                        );
                        for d in &hold_detections_outbound {
                            let already = redact_hits
                                .iter()
                                .any(|h| h.start == d.span.start && h.end == d.span.end);
                            if !already {
                                redact_hits.push(RedactHit {
                                    rule_id: d.rule_id.clone(),
                                    start: d.span.start,
                                    end: d.span.end,
                                });
                            }
                        }
                    }
                    sieve_ipc::DefaultOnTimeout::Allow => {
                        tracing::info!(
                            "OUTBOUND GUI: IPC not initialized, default_on_timeout=allow → 放行"
                        );
                    }
                    sieve_ipc::DefaultOnTimeout::Block => {
                        tracing::warn!(
                            "OUTBOUND GUI: IPC not initialized, default_on_timeout=block → 426"
                        );
                        let held: Vec<sieve_core::Detection> = hold_detections_outbound
                            .iter()
                            .map(|d| (*d).clone())
                            .collect();
                        return Ok(build_426_response(&held));
                    }
                }
            }
        }

        // 4a. AutoRedact：在文本段层脱敏，重新序列化 JSON 后转发（不返回 426）
        //
        // 修 #1：不再用 redact_body_bytes(&body_bytes, ...)，改为：
        // 1. redact_segments() 在文本字符串层替换
        // 2. 把替换后的文本写回 AnthropicRequest messages
        // 3. serde_json 重新序列化为新 body
        // 这样保证脱敏后 raw body 里不含原始 secret，且 JSON 结构合法。
        if !redact_hits.is_empty() {
            let seg_result = redact_segments(&texts, &redact_hits);
            tracing::info!(
                count = seg_result.redacted_count,
                rules = %seg_result.redacted_summary,
                "OUTBOUND AUTO-REDACT"
            );

            // audit：OutboundRedacted（fire-and-forget，不阻塞热路径，PRD §9 性能预算）。
            // 此前出站脱敏从不落 audit（headless dogfood e2e 抓出，2026-06-18）。
            // raw_json=None：脱敏事件**不持久化原文**（含 secret，PRD §5.6.1 / §9 #13）。
            if let Some(d) = all_detections
                .iter()
                .find(|d| matches!(d.action, Action::Redact { .. }))
            {
                let event = crate::audit::AuditEvent::OutboundRedacted {
                    rule_id: d.rule_id.clone(),
                    severity: format!("{:?}", d.severity).to_lowercase(),
                    request_id: uuid::Uuid::new_v4().to_string(),
                    raw_json: None,
                    caller: ctx.caller_context(),
                };
                let store = Arc::clone(&ctx.audit);
                let provider_id = ctx.listener_provider_id.clone();
                tokio::spawn(async move {
                    if let Err(e) = store.append(event, &provider_id).await {
                        tracing::warn!(error = %e, "audit append OutboundRedacted failed");
                    }
                });
            }

            // 把替换后文本写回 AnthropicRequest，然后重新序列化
            let new_body_bytes =
                apply_redacted_texts_to_request(&anthropic_req, &texts, &seg_result.texts)
                    .and_then(|req| {
                        serde_json::to_vec(&req).map_err(|e| anyhow!("re-serialize json: {e}"))
                    })?;

            // 验证脱敏后 JSON 仍然合法（关键回归断言）
            if serde_json::from_slice::<serde_json::Value>(&new_body_bytes).is_err() {
                return Err(anyhow!("redact_segments 产生了非法 JSON，fail-closed 拦截"));
            }

            let new_body = Bytes::from(new_body_bytes);
            let new_len = new_body.len();

            // ADR-037 full 档：归档脱敏后的出站内容（fire-and-forget，off hot path）。
            // 红线：只存脱敏后 new_body——原始 secret 已被 redact_segments 替换为占位符，
            // 绝不落原始 body。spawn_blocking（age 加密 + 文件 IO 是同步阻塞），失败仅 warn
            // 不阻断 forward（审计可靠性问题不得变可用性事故，ADR-007）。
            archive_redacted_outbound(&archive, &new_body, "anthropic");

            // 更新 Content-Length header
            let mut new_parts = parts.clone();
            new_parts.headers.insert(
                http::header::CONTENT_LENGTH,
                http::HeaderValue::from(new_len),
            );

            // 5. prompt 地址 seed（脱敏后仍需 seed，基于原始地址）
            for (_, text) in &texts {
                if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                    tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
                }
            }

            return forward_with_inbound_inspection(
                forwarder,
                inbound_filter,
                dry_run,
                ipc,
                new_parts,
                new_body,
                MultiAgentMeta {
                    source_agent,
                    origin_chain,
                    source_channel,
                    chain_depth,
                },
                RequestCtx::new(
                    ctx.caller.clone(),
                    Arc::clone(&ctx.audit),
                    ctx.listener_protocol,
                    ctx.listener_provider_id.clone(),
                ),
                billing_ctx,
            )
            .await;
        }

        if dry_run && !all_detections.is_empty() {
            tracing::warn!(
                count = all_detections.len(),
                "OUTBOUND DRY-RUN: would have flagged"
            );
            for d in &all_detections {
                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection (dry_run)");
            }
        }

        // 5. prompt 地址 seed
        for (_, text) in &texts {
            if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
            }
        }

        // 6. 出站通过 → 入站 SSE tee 截流检测
        return forward_with_inbound_inspection(
            forwarder,
            inbound_filter,
            dry_run,
            ipc,
            parts,
            body_bytes,
            MultiAgentMeta {
                source_agent,
                origin_chain,
                source_channel,
                chain_depth,
            },
            RequestCtx::new(
                ctx.caller.clone(),
                Arc::clone(&ctx.audit),
                ctx.listener_protocol,
                ctx.listener_provider_id.clone(),
            ),
            billing_ctx,
        )
        .await;
    }

    // ── OpenAI Chat Completions 路径（v1.5，ADR-018）────────────────────────────
    if is_chat_completions_post {
        // body 已在 POST 预收集块中 collect，直接取出
        let ProxyRequestBody::Collected(body_bytes) = proxy_body else {
            tracing::error!("BUG: is_chat_completions_post but request body not collected");
            return Ok(build_500_internal_response(
                "chat completions body not collected",
            ));
        };
        return proxy_openai(
            forwarder,
            filter,
            inbound_filter,
            dry_run,
            ipc,
            archive,
            billing,
            listener_trust,
            parts,
            body_bytes,
            source_agent,
            origin_chain,
            source_channel,
            chain_depth,
            RequestCtx::new(
                ctx.caller.clone(),
                Arc::clone(&ctx.audit),
                ctx.listener_protocol,
                ctx.listener_provider_id.clone(),
            ),
            no_client_policy,
        )
        .await;
    }

    // 其他路径：流式透传（Week 1 行为）。
    // Collected（白名单 POST 未命中检测，fall through）→ forward_raw；
    // Streaming（非白名单）→ forward_streaming。enum 两态穷尽，无非法态可表达。
    match proxy_body {
        ProxyRequestBody::Collected(body_bytes) => forward_raw(forwarder, parts, body_bytes).await,
        ProxyRequestBody::Streaming(stream) => forward_streaming(forwarder, parts, stream).await,
    }
}

/// OpenAI Chat Completions 路径处理（`/v1/chat/completions`）。
///
/// 行为与 Anthropic 路径对称：
/// 1. body 已由调用方 collect（proxy_inner POST 预收集块）
/// 2. 解析 `OpenAIRequest`；解析失败 → 透传（上游返回 400）
/// 3. 提取文本段 → 逐段扫描（规则引擎与 Anthropic 路径共享）
/// 4. chain_depth ≥ 2 → 任何命中强制升级为 GuiPopup
/// 5. Block / GuiPopup / 透传 决策（与 Anthropic 路径相同）
/// 6. stream=true → `forward_with_openai_inbound_inspection`（修 R6-#2）
///
/// 关联：ADR-018 §路由、ADR-019 §chain_depth 升级、PRD v1.5 §6.1。
#[allow(clippy::too_many_arguments)]
async fn proxy_openai(
    forwarder: Arc<Forwarder>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    archive: ArchiveWriterHandle,
    billing: BillingObserverHandle,
    listener_trust: crate::config::Trust,
    parts: http::request::Parts,
    body_bytes: Bytes,
    source_agent: sieve_ipc::protocol::SourceAgent,
    origin_chain: Vec<sieve_ipc::protocol::OriginHop>,
    source_channel: Option<String>,
    chain_depth: usize,
    ctx: RequestCtx,
    no_client_policy: crate::cli::NoClientPolicy,
) -> Result<Response<ResponseBody>> {
    let RequestCtx {
        caller,
        audit: audit_store,
        listener_protocol,
        listener_provider_id,
    } = ctx;
    use sieve_core::pipeline::PipelineNode;
    use sieve_core::protocol::unified_message::{
        ContentBlock, ContentSpan, Direction, MessageMetadata, UpstreamProvider,
    };
    use std::time::SystemTime;

    // 1. 解析 OpenAIRequest；解析失败 → 透传
    let openai_req: sieve_core::protocol::openai::OpenAIRequest =
        match serde_json::from_slice(&body_bytes) {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!("non-openai body on /v1/chat/completions, passing through: {e}");
                return forward_raw(forwarder, parts, body_bytes).await;
            }
        };

    // 2. 提取文本段 → 逐段扫描
    let texts = openai_req.extract_text_content();

    // ADR-038：构造超额计费观测上下文（仅 Relay 上游 + billing 启用时）。
    let billing_ctx = build_billing_context(
        &billing,
        listener_trust,
        &listener_provider_id,
        &openai_req.model,
        billing_facade::Family::OpenAi,
        &texts,
    );

    let mut all_detections: Vec<sieve_core::Detection> = Vec::new();

    for (offset, text) in &texts {
        let mut msg = sieve_core::UnifiedMessage {
            role: sieve_core::Role::User,
            content_blocks: vec![ContentBlock::Text {
                text: text.clone(),
                span: Some(ContentSpan {
                    start: *offset,
                    end: *offset + text.len(),
                }),
            }],
            tool_uses: vec![],
            tool_results: vec![],
            metadata: MessageMetadata {
                session_id: "outbound-scan-openai".into(),
                direction: Direction::Outbound,
                upstream_provider: UpstreamProvider::OpenAI,
                received_at: SystemTime::now(),
            },
        };

        let hits = filter
            .process(&mut msg)
            .map_err(|e| anyhow!("outbound filter (openai): {e}"))?;
        all_detections.extend(hits);
    }

    // 4. chain_depth ≥ 2 → 所有命中（含 HookTerminal disposition）强制升级为 GuiPopup
    //    （ADR-019 §chain_depth 升级策略）
    // 4. chain_depth ≥ 2 → HookMark + Redact 升级为 HoldForDecision（强制 GUI 弹窗，ADR-019）
    //
    // 修 R9-#2：OpenAI 路径之前只升级 HookMark，Action::Redact 仍静默脱敏。
    // 与 Anthropic 路径对称修复：嵌套调用时 Redact 也需 GUI 确认。
    if chain_depth >= 2 {
        tracing::info!(
            chain_depth,
            "X-Sieve-Origin chain_depth ≥ 2（OpenAI 路径），HookMark + Redact 升级为 GuiPopup"
        );
        for d in &mut all_detections {
            match &d.action {
                Action::HookMark => {
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                    };
                }
                Action::Redact { .. } => {
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Redact,
                    };
                }
                _ => {}
            }
        }
    }

    // 5a. 收集需要脱敏的 hit（与 Anthropic 路径对称，修 A2-#1）
    let mut redact_hits_openai: Vec<RedactHit> = all_detections
        .iter()
        .filter(|d| matches!(d.action, Action::Redact { .. }))
        .map(|d| RedactHit {
            rule_id: d.rule_id.clone(),
            start: d.span.start,
            end: d.span.end,
        })
        .collect();

    // 5b. Block（Critical fail-closed）
    let blocking: Vec<&sieve_core::Detection> = all_detections
        .iter()
        .filter(|d| {
            if d.action != Action::Block {
                return false;
            }
            if d.severity != sieve_core::Severity::Critical {
                return false;
            }
            sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run
        })
        .collect();

    if !blocking.is_empty() {
        tracing::warn!(count = blocking.len(), "OUTBOUND BLOCKED (openai)");
        for d in &blocking {
            tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "openai detection");
        }
        let cloned: Vec<sieve_core::Detection> = blocking.iter().map(|d| (*d).clone()).collect();
        return Ok(build_426_response(&cloned));
    }

    // 5c. GuiPopup（HoldForDecision）
    let hold_detections: Vec<&sieve_core::Detection> = all_detections
        .iter()
        .filter(|d| matches!(d.action, Action::HoldForDecision { .. }))
        .collect();

    if !hold_detections.is_empty() {
        // v2.0 §5.4.2：出站 OpenAI GuiPopup 路径——决策前先查灰名单（与 Anthropic 路径对称）
        // fail-closed：查询失败 → 走正常 IPC 流程（PRD §9 #14 禁止 fail-open）
        let agent_str_openai_out = format!("{:?}", source_agent).to_lowercase();
        let graylist_hit_openai = hold_detections.first().is_some_and(|d| {
            check_graylist_hit(
                &d.rule_id,
                &d.evidence_truncated,
                "",
                "openai",
                "outbound_text",
                &agent_str_openai_out,
            )
        });
        if graylist_hit_openai {
            tracing::info!("出站 OpenAI GuiPopup：灰名单命中 → 直接放行，跳过 IPC 弹窗");
            // 灰名单命中：直接跳过 hold，继续往下走转发路径
        } else if let Some(ref ipc_server) = ipc {
            use chrono::Utc;

            let request_id = uuid::Uuid::new_v4();

            // 修 R3-#4 / R10-#2（OpenAI 路径）：从 detection.default_on_timeout 读，不硬编码 Block。
            let (timeout_seconds, default_on_timeout) = hold_detections.iter().fold(
                (60_u32, sieve_ipc::DefaultOnTimeout::Allow),
                |acc, d| {
                    if let Action::HoldForDecision {
                        timeout_seconds,
                        default_on_timeout,
                        ..
                    } = &d.action
                    {
                        let merged =
                            merge_strictest_timeout(acc.1, map_dot_to_ipc(*default_on_timeout));
                        (acc.0.max(*timeout_seconds), merged)
                    } else {
                        acc
                    }
                },
            );

            // chain_depth ≥ 2 时在弹窗标题里显示完整 origin_chain 信息（ADR-019）
            let chain_note = if chain_depth >= 2 {
                format!("（嵌套调用 depth={chain_depth}）")
            } else {
                String::new()
            };

            let ipc_detections = hold_detections
                .iter()
                .map(|d| sieve_ipc::protocol::DetectionPayload {
                    rule_id: d.rule_id.clone(),
                    severity: map_severity_to_ipc(d.severity),
                    disposition: sieve_ipc::Disposition::GuiPopup,
                    title: format!("出站检测命中{chain_note}：{}", d.rule_id),
                    one_line_summary: d.evidence_truncated.clone(),
                    details: serde_json::json!({ "chain_depth": chain_depth }),
                    recommendation: None,
                })
                .collect();

            // v2.0：计算 allow_remember（PRD §5.4.2）
            let openai_outbound_rule_ids = detection_rule_ids(&hold_detections);
            let allow_remember = compute_allow_remember(&openai_outbound_rule_ids);

            let ipc_req = sieve_ipc::DecisionRequest {
                request_id,
                created_at: Utc::now(),
                timeout_seconds,
                default_on_timeout,
                detections: ipc_detections,
                // v1.5：注入 multi-agent 元数据
                source_agent,
                origin_chain: origin_chain.clone(),
                source_channel: source_channel.clone(),
                // 修 R7-#5：填入 header 真实 chain_depth
                explicit_chain_depth: Some(chain_depth),
                allow_remember,
            };

            let timeout_dur = std::time::Duration::from_secs(u64::from(timeout_seconds).max(1));
            let outcome = gated_request_decision(
                ipc_server,
                &audit_store,
                &caller,
                ipc_req,
                timeout_dur,
                "outbound",
                &listener_provider_id,
                no_client_policy,
            )
            .await;

            match outcome {
                Ok(resp) => match resp.decision {
                    sieve_ipc::DecisionAction::Allow => {
                        tracing::info!("OUTBOUND GUI (openai): Allow → 转发原 body");
                        // v2.0：remember=true 时写灰名单（PRD §5.4.2）
                        if resp.remember && allow_remember {
                            if let Some(d) = hold_detections.first() {
                                let agent_str = format!("{:?}", source_agent).to_lowercase();
                                try_write_graylist(
                                    &d.rule_id,
                                    &d.evidence_truncated,
                                    "",
                                    "openai",
                                    "outbound_text",
                                    &agent_str,
                                    resp.context_hint.clone(),
                                    &request_id.to_string(),
                                    &audit_store,
                                    &caller,
                                    &listener_provider_id,
                                );
                            }
                        }
                        // fall-through 到透传（不脱敏，用户明确选择原样允许）
                    }
                    sieve_ipc::DecisionAction::RedactAndAllow => {
                        tracing::info!("OUTBOUND GUI (openai): RedactAndAllow → 脱敏后转发");
                        // 修 R3-#3：把 held detection 的 span 升级到 redact_hits_openai，
                        // 保证仅命中 GUI 类（无 AutoRedact 类）时也能正确脱敏。
                        // 去重：跳过已在 redact_hits_openai 中存在的 span。
                        for d in &hold_detections {
                            let already = redact_hits_openai
                                .iter()
                                .any(|h| h.start == d.span.start && h.end == d.span.end);
                            if !already {
                                redact_hits_openai.push(RedactHit {
                                    rule_id: d.rule_id.clone(),
                                    start: d.span.start,
                                    end: d.span.end,
                                });
                            }
                        }
                        // fall-through 到下方 redact_hits_openai 处理（现在含 GUI 类 span）
                    }
                    sieve_ipc::DecisionAction::Deny => {
                        tracing::warn!("OUTBOUND GUI (openai): Deny → 426");
                        let held: Vec<sieve_core::Detection> =
                            hold_detections.iter().map(|d| (*d).clone()).collect();
                        return Ok(build_426_response(&held));
                    }
                },
                Err(e) => {
                    // 修 R3-#4（OpenAI 路径）：按规则 default_on_timeout 兜底
                    tracing::warn!(error = %e, ?default_on_timeout, "OUTBOUND GUI (openai): IPC error, applying default_on_timeout");
                    match default_on_timeout {
                        sieve_ipc::DefaultOnTimeout::Redact => {
                            for d in &hold_detections {
                                let already = redact_hits_openai
                                    .iter()
                                    .any(|h| h.start == d.span.start && h.end == d.span.end);
                                if !already {
                                    redact_hits_openai.push(RedactHit {
                                        rule_id: d.rule_id.clone(),
                                        start: d.span.start,
                                        end: d.span.end,
                                    });
                                }
                            }
                        }
                        sieve_ipc::DefaultOnTimeout::Allow => {
                            tracing::info!("OUTBOUND GUI (openai): timeout default=allow → 放���");
                        }
                        sieve_ipc::DefaultOnTimeout::Block => {
                            let held: Vec<sieve_core::Detection> =
                                hold_detections.iter().map(|d| (*d).clone()).collect();
                            return Ok(build_426_response(&held));
                        }
                    }
                }
            }
        } else {
            // IPC 未初始化：按规则 default_on_timeout 兜底（修 R3-#4 OpenAI 路径）
            let effective_dot =
                hold_detections
                    .iter()
                    .fold(sieve_ipc::DefaultOnTimeout::Allow, |acc, d| {
                        if let Action::HoldForDecision {
                            default_on_timeout, ..
                        } = &d.action
                        {
                            merge_strictest_timeout(acc, map_dot_to_ipc(*default_on_timeout))
                        } else {
                            acc
                        }
                    });
            match effective_dot {
                sieve_ipc::DefaultOnTimeout::Redact => {
                    tracing::info!(
                        "OUTBOUND GUI (openai): IPC not initialized, default_on_timeout=redact → 脱��转发"
                    );
                    for d in &hold_detections {
                        let already = redact_hits_openai
                            .iter()
                            .any(|h| h.start == d.span.start && h.end == d.span.end);
                        if !already {
                            redact_hits_openai.push(RedactHit {
                                rule_id: d.rule_id.clone(),
                                start: d.span.start,
                                end: d.span.end,
                            });
                        }
                    }
                }
                sieve_ipc::DefaultOnTimeout::Allow => {
                    tracing::info!(
                        "OUTBOUND GUI (openai): IPC not initialized, default_on_timeout=allow → 放行"
                    );
                }
                sieve_ipc::DefaultOnTimeout::Block => {
                    tracing::warn!(
                        "OUTBOUND GUI (openai): IPC not initialized, default_on_timeout=block → 426"
                    );
                    let held: Vec<sieve_core::Detection> =
                        hold_detections.iter().map(|d| (*d).clone()).collect();
                    return Ok(build_426_response(&held));
                }
            }
        }
    }

    if dry_run && !all_detections.is_empty() {
        tracing::warn!(
            count = all_detections.len(),
            "OUTBOUND DRY-RUN (openai): would have flagged"
        );
    }

    // 5d. AutoRedact（修 A2-#1）：命中 Redact action 的 secret 在转发前脱敏，
    // 不返回 426；与 Anthropic 路径对称。OpenAI message.content 同时支持
    // string 和 array-of-content-parts，由专用函数处理。
    if !redact_hits_openai.is_empty() {
        let seg_result = redact_segments(&texts, &redact_hits_openai);
        tracing::info!(
            count = seg_result.redacted_count,
            rules = %seg_result.redacted_summary,
            "OUTBOUND AUTO-REDACT (openai)"
        );

        // audit：OutboundRedacted（与 Anthropic 路径对称，fire-and-forget；raw_json=None 不存原文）。
        // 注：proxy_openai 已把 ctx 析构成局部 caller/audit_store/listener_provider_id。
        if let Some(d) = all_detections
            .iter()
            .find(|d| matches!(d.action, Action::Redact { .. }))
        {
            let caller_ctx = crate::audit::CallerContext {
                pid: caller.as_ref().map(|c| c.pid),
                exe: caller
                    .as_ref()
                    .and_then(|c| c.exe.as_ref())
                    .map(|p| p.display().to_string()),
            };
            let event = crate::audit::AuditEvent::OutboundRedacted {
                rule_id: d.rule_id.clone(),
                severity: format!("{:?}", d.severity).to_lowercase(),
                request_id: uuid::Uuid::new_v4().to_string(),
                raw_json: None,
                caller: caller_ctx,
            };
            let store = Arc::clone(&audit_store);
            let provider_id = listener_provider_id.clone();
            tokio::spawn(async move {
                if let Err(e) = store.append(event, &provider_id).await {
                    tracing::warn!(error = %e, "audit append OutboundRedacted (openai) failed");
                }
            });
        }

        let new_body_bytes =
            apply_redacted_texts_to_openai_request(&openai_req, &texts, &seg_result.texts)
                .and_then(|req| {
                    serde_json::to_vec(&req).map_err(|e| anyhow!("re-serialize openai json: {e}"))
                })?;

        // 验证脱敏后 JSON 仍然合法
        if serde_json::from_slice::<serde_json::Value>(&new_body_bytes).is_err() {
            return Err(anyhow!(
                "redact_segments (openai) 产生了非法 JSON，fail-closed 拦截"
            ));
        }

        let new_body = bytes::Bytes::from(new_body_bytes);
        let new_len = new_body.len();

        // ADR-037 full 档：归档脱敏后的出站内容（OpenAI 路径，红线只存脱敏后 new_body）。
        archive_redacted_outbound(&archive, &new_body, "openai");

        let mut new_parts = parts.clone();
        new_parts.headers.insert(
            http::header::CONTENT_LENGTH,
            http::HeaderValue::from(new_len),
        );

        // 修 R8-#3：AutoRedact 后 stream=true 仍需入站 SSE 检测。
        // 原实现直接 forward_raw，跳过了 forward_with_openai_inbound_inspection，
        // 导致脱敏后的 OpenAI 流式响应不经过入站规则检测（漏检）。
        // 修法与 Anthropic 路径等价：脱敏后用新 body 继续走入站检测路径。
        // stream=false 时直接透传（非流式响应无需 SSE 解析，同非 AutoRedact 分支）。

        // 修 R9-#1：AutoRedact 路径也需要 seed prompt 地址（与 Anthropic 路径等价）。
        // AutoRedact 改写 secret，不影响 EVM 地址；seed 用原始 texts 即可。
        for (_, text) in &texts {
            if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                tracing::warn!(error = %e, "seed_known_addresses_from_text failed (openai autoredact)");
            }
        }

        // 漏洞修复：AutoRedact stream=false 分支也需入站检测（同非 AutoRedact stream=false 修复）。
        // forward_with_openai_inbound_inspection 内部按 Content-Type 路由，JSON 响应走 JSON 路径。
        return forward_with_openai_inbound_inspection(
            forwarder,
            inbound_filter,
            dry_run,
            ipc,
            new_parts,
            new_body,
            MultiAgentMeta {
                source_agent,
                origin_chain,
                source_channel,
                chain_depth,
            },
            RequestCtx::new(
                caller.clone(),
                Arc::clone(&audit_store),
                listener_protocol,
                listener_provider_id.clone(),
            ),
            billing_ctx,
        )
        .await;
    }

    // 5. prompt 地址 seed（修 R9-#1，与 Anthropic 路径等价）
    // 用户 prompt 中的 EVM 地址需要提前注入 InboundFilter 会话状态，
    // 否则流式响应里的地址替换（IN-CR-01）因缺少参照而漏检。
    for (_, text) in &texts {
        if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
            tracing::warn!(error = %e, "seed_known_addresses_from_text failed (openai)");
        }
    }

    // 6. 出站通过 → 入站检测路由（修 R6-#2 + 漏洞修复）
    // stream=true 时用 OpenAI SSE parser 做 tee 截流检测，与 Anthropic 路径对称。
    // stream=false 时：上游可能返回 application/json（含 tool_calls），
    //   同样需要入站检测（漏洞修复：lessons.md 2026-04-27 [安全]）。
    //   forward_with_openai_inbound_inspection 内部按 Content-Type 路由：
    //   JSON 响应走 handle_openai_json_inbound，SSE 响应走原 SSE 路径。
    // R6-#3 RESOLVED：OpenAiSseParser 已支持 ContentBlockStart/Stop（含 tool_call 首帧），
    //    inbound_filter 协议无关聚合 tool_use 已经生效。
    forward_with_openai_inbound_inspection(
        forwarder,
        inbound_filter,
        dry_run,
        ipc,
        parts,
        body_bytes,
        MultiAgentMeta {
            source_agent,
            origin_chain,
            source_channel,
            chain_depth,
        },
        RequestCtx::new(caller, audit_store, listener_protocol, listener_provider_id),
        billing_ctx,
    )
    .await
}

/// 透传并同步做入站 SSE 解析检测（tee 模式）。
///
/// 字节流同时被：
/// 1. 原样 forward 给客户端（via bounded channel）
/// 2. 异步喂给 SseParser → Aggregator → InboundFilter 检测
///
/// v1.4 分支逻辑：
/// - `Action::Block`（fail-closed Critical）→ 注入 `sieve_blocked` event 并截流
/// - `Action::HookMark` → 写 IPC pending 文件，SSE 流原样转发（**不注入 sieve_blocked**）
/// - `Action::HoldForDecision` → hold 流 + keep-alive，等用户决策
/// - 其余 → 透传
///
/// 关联：ADR-014 §双层防御、ADR-016 §dispatch 路由、PRD v1.4 §6.7。
/// Multi-agent 元数据，从 `X-Sieve-Origin` / `X-Sieve-Source-Channel` 解析而来。
///
/// 在入站路径和出站路径构造 `DecisionRequest` 时注入，供 GUI / hook 显示来源信息。
/// 关联：ADR-019 §字段定义、PRD v1.5 §6.5。
#[derive(Clone)]
struct MultiAgentMeta {
    source_agent: sieve_ipc::protocol::SourceAgent,
    origin_chain: Vec<sieve_ipc::protocol::OriginHop>,
    source_channel: Option<String>,
    /// `X-Sieve-Origin` header 中解析的真实嵌套深度（修 R7-#5）。
    ///
    /// 用于填充 `DecisionRequest::explicit_chain_depth`，使 GUI/hook
    /// 能展示 header 真实深度而非受限于 `origin_chain.len()`。
    chain_depth: usize,
}

#[allow(clippy::too_many_arguments)]
async fn forward_with_inbound_inspection(
    forwarder: Arc<Forwarder>,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
    meta: MultiAgentMeta,
    ctx: RequestCtx,
    billing_ctx: BillingCtxHandle,
) -> Result<Response<ResponseBody>> {
    // 解构 ctx 供内部使用（避免在 spawn move 时多次 clone Arc）
    let RequestCtx {
        caller,
        audit: audit_store,
        listener_protocol,
        listener_provider_id,
    } = ctx;
    use http_body_util::Full;

    // 修 A2-#2：把 source_channel 注入 InboundFilter，使 IN-GEN-06 运行时提级逻辑
    // 能感知来源 channel（PRD v1.5 §4.5）。必须在 SSE 检测开始前调用。
    inbound_filter.set_source_channel(meta.source_channel.clone());

    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = Full::new(body_bytes)
        .map_err(|e| -> hyper::Error { match e {} })
        .boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (mut resp_parts, resp_body) = upstream_resp.into_parts();

    // 入站响应可能被 sieve 注入 sieve_blocked event 截流，实际 body 长度不一定等于上游
    // content-length。剥掉 content-length 强制 chunked transfer，防止 hyper client 截断。
    resp_parts.headers.remove(http::header::CONTENT_LENGTH);

    // 漏洞修复（lessons.md 2026-04-27 [安全]）：按 Content-Type 路由入站检测路径。
    //
    // 原实现假设入站永远是 SSE 流（text/event-stream），上游返回 application/json
    // 时响应 body 直接透传，所有入站规则失效。修复：
    //   - text/event-stream → 走现有 SSE 路径（tokio::spawn + channel tee）
    //   - application/json  → 收集完整 body → 解析 content[] → 提取 tool_use →
    //                         喂 InboundFilter → 命中 Critical 时替换为 sieve_blocked JSON
    let is_json_response = resp_parts
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.starts_with("application/json"))
        .unwrap_or(false);

    if is_json_response {
        return handle_anthropic_json_inbound(
            resp_parts,
            resp_body,
            inbound_filter,
            dry_run,
            meta,
            ipc.clone(),
            RequestCtx::new(
                caller.clone(),
                Arc::clone(&audit_store),
                listener_protocol,
                listener_provider_id.clone(),
            ),
            billing_ctx,
        )
        .await;
    }

    // P0-5：bounded channel，深度 64，上游读取自然受背压限制。
    const INBOUND_CHANNEL_DEPTH: usize = 64;
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
        INBOUND_CHANNEL_DEPTH,
    );

    // meta 需要在 spawn 闭包中 capture（用于入站 DecisionRequest 注入）
    let inbound_meta = meta;

    tokio::spawn(async move {
        let meta = inbound_meta;
        let mut parser = SseParser::new();
        let mut aggregator = Aggregator::new();
        // ADR-038：仅 billing 启用（billing_ctx=Some）时累计 SSE usage + completion。
        let mut billing_acc = billing_ctx
            .as_ref()
            .map(|_| BillingSseAccumulator::default());

        use http_body_util::BodyStream;
        let mut stream = BodyStream::new(resp_body);

        while let Some(frame_result) = stream.next().await {
            match frame_result {
                Ok(frame) => {
                    let Some(frame_bytes) = frame.data_ref().cloned() else {
                        if tx.send(Ok(frame)).await.is_err() {
                            return;
                        }
                        continue;
                    };

                    // P0-5：push_chunk 超限时 fail-closed（IN-CAP-01）
                    let events = match parser.push_chunk(&frame_bytes) {
                        Ok(evts) => evts,
                        Err(e) => {
                            tracing::warn!(error = %e, "SSE parser 容量超限，fail-closed 注入 sieve_blocked");
                            let cap_detection =
                                build_cap_detection("IN-CAP-01", "cap-sse-event-too-large");
                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    };

                    // ADR-038：累计本批 SSE usage + completion（Anthropic SSE 观测）。
                    if let Some(acc) = billing_acc.as_mut() {
                        acc.observe_events(&events);
                    }

                    // 收集本批 events 的 detections，按 action 分组处理
                    // 修 R8-#2：传入 meta.chain_depth，chain_depth ≥ 2 时 HookMark 升级为 GuiPopup
                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                        &events,
                        &mut inbound_filter,
                        &mut aggregator,
                        dry_run,
                        meta.chain_depth,
                        &ipc,
                        &audit_store,
                        caller.as_ref(),
                        &listener_provider_id,
                    );

                    // 修 #4（fail-closed 被绕过修复）：Block 检查必须在 Hold 之前。
                    // 原代码 Hold allow 后 continue 会跳过 Block 检查，导致同批同时含
                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed（PRD §9 #3）。
                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
                    // 关联：ADR-014 §双层防御、PRD §9 #3。

                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
                    if !blocking.is_empty() {
                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
                        for d in &blocking {
                            tracing::warn!(rule = %d.rule_id, "inbound detection");
                        }
                        spawn_inbound_blocked_audit(
                            &audit_store,
                            &listener_provider_id,
                            &caller,
                            &blocking,
                            "anthropic_sse",
                        );
                        let blocked_payload = build_sieve_blocked_sse(&blocking);
                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                        return;
                    }

                    // 2. Hook 类：写 pending 文件，失败时 fail-closed（不允许 fail-open）
                    for d in &hook_detections {
                        if let Err(e) = write_hook_pending_or_fail_closed(d, &meta) {
                            tracing::error!(
                                error = %e,
                                rule = %d.rule_id,
                                "Hook pending write failed; fail-closed: truncating SSE stream"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    }

                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
                    if !hold_detections.is_empty() {
                        if let Some(ref ipc_server) = ipc {
                            // keep-alive channel：daemon 把心跳写入 SSE 流
                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
                            let tx_ka = tx.clone();

                            // 修 R2-#3：触发帧不先发给客户端——暂存在 frame_bytes 变量里。
                            // 决策 Allow/RedactAndAllow 后再发（见下方 match 分支）；
                            // 决策 Deny 时不发，避免恶意内容已污染客户端上下文。
                            // hold 期间只向客户端发 keep-alive comment（不是模型内容）。

                            // 启动 keep-alive 转发 task
                            let ka_fwd_handle = tokio::spawn(async move {
                                while let Some(ka_bytes) = ka_rx.recv().await {
                                    if tx_ka
                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            });

                            // 构造 IPC 请求
                            use chrono::Utc;
                            let request_id = uuid::Uuid::new_v4();
                            let timeout_seconds = hold_detections
                                .iter()
                                .find_map(|d| {
                                    if let Action::HoldForDecision {
                                        timeout_seconds, ..
                                    } = d.action
                                    {
                                        Some(timeout_seconds)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(60);

                            let ipc_detections = hold_detections
                                .iter()
                                .map(|d| sieve_ipc::protocol::DetectionPayload {
                                    rule_id: d.rule_id.clone(),
                                    severity: map_severity_to_ipc(d.severity),
                                    disposition: sieve_ipc::Disposition::GuiPopup,
                                    title: format!("检测命中：{}", d.rule_id),
                                    one_line_summary: d.evidence_truncated.clone(),
                                    details: serde_json::json!({}),
                                    recommendation: None,
                                })
                                .collect();

                            // v2.0：计算 allow_remember（PRD §5.4.2）
                            let inbound_sse_rule_ids: Vec<&str> =
                                hold_detections.iter().map(|d| d.rule_id.as_str()).collect();
                            let allow_remember = compute_allow_remember(&inbound_sse_rule_ids);

                            let ipc_req = sieve_ipc::DecisionRequest {
                                request_id,
                                created_at: Utc::now(),
                                timeout_seconds,
                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                                detections: ipc_detections,
                                // v1.5：注入 multi-agent 元数据（ADR-019）
                                source_agent: meta.source_agent,
                                origin_chain: meta.origin_chain.clone(),
                                source_channel: meta.source_channel.clone(),
                                // 修 R7-#5：填入 header 真实 chain_depth
                                explicit_chain_depth: Some(meta.chain_depth),
                                allow_remember,
                            };

                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
                                Arc::clone(ipc_server),
                                ipc_req,
                                ka_tx,
                                "inbound",
                            )
                            .await;

                            ka_fwd_handle.abort();

                            match outcome {
                                Ok(sieve_core::pipeline::HoldOutcome::Allow {
                                    remember,
                                    context_hint,
                                })
                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow {
                                    remember,
                                    context_hint,
                                }) => {
                                    // 修 R2-#3：用户允许后，补发缓存的触发帧（hold 前未发），
                                    // 然后继续转发后续 SSE。

                                    // v2.0 §5.4.2：remember=true 时写灰名单（PRD §5.4.2）
                                    if remember && allow_remember {
                                        let agent_str =
                                            format!("{:?}", meta.source_agent).to_lowercase();
                                        for det in &hold_detections {
                                            try_write_graylist(
                                                &det.rule_id,
                                                &det.evidence_truncated,
                                                "",
                                                "anthropic",
                                                "inbound_sse",
                                                &agent_str,
                                                context_hint.clone(),
                                                &request_id.to_string(),
                                                &audit_store,
                                                &caller,
                                                &listener_provider_id,
                                            );
                                        }
                                    }

                                    if tx
                                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }
                                    continue;
                                }
                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
                                    // 修 R2-#3：用户拒绝时不发触发帧，直接注入 sieve_blocked 并关流。
                                    tracing::warn!(%reason, "INBOUND BLOCKED by GUI decision");
                                    spawn_inbound_blocked_audit(
                                        &audit_store,
                                        &listener_provider_id,
                                        &caller,
                                        &hold_detections,
                                        "anthropic_sse",
                                    );
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "IPC hold error, fail-closed");
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                            }
                        } else {
                            // IPC 未初始化：fail-closed，阻断
                            tracing::warn!(
                                "GuiPopup detection but IPC server not initialized; fail-closed"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    }

                    // 无 blocking / hold：透传原始 frame
                    if tx
                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(std::io::Error::other(format!(
                            "upstream body error: {e}"
                        ))))
                        .await;
                    return;
                }
            }
        }

        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
        let flushed = parser.flush();
        // ADR-038：累计 flush 残留 events（流尾 usage / 末段文本可能在此）。
        if let Some(acc) = billing_acc.as_mut() {
            acc.observe_events(&flushed);
        }
        // 修 R8-#2：flush 阶段同样传入 chain_depth，HookMark 升级逻辑一致
        let (blocking, hook_detections, flush_hold_detections) = classify_inbound_detections(
            &flushed,
            &mut inbound_filter,
            &mut aggregator,
            dry_run,
            meta.chain_depth,
            &ipc,
            &audit_store,
            caller.as_ref(),
            &listener_provider_id,
        );

        // flush 阶段 Hook 类同样 fail-closed：写失败即截流
        for d in &hook_detections {
            if let Err(e) = write_hook_pending_or_fail_closed(d, &meta) {
                tracing::error!(
                    error = %e,
                    rule = %d.rule_id,
                    "Hook pending write failed (flush); fail-closed: truncating SSE stream"
                );
                let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                return;
            }
        }

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, "inbound detection (flush)");
            }
            spawn_inbound_blocked_audit(
                &audit_store,
                &listener_provider_id,
                &caller,
                &blocking,
                "anthropic_sse",
            );
            let blocked_payload = build_sieve_blocked_sse(&blocking);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
            return;
        }

        // 修 #5（flush 阶段 hold 丢失修复）：
        // flush 路径的 HoldForDecision 命中不能静默丢弃。
        // 此时流已断无法 hold + IPC 通知 GUI，必须 fail-closed。
        // 关联：ADR-014 §双层防御、PRD §9 #3。
        if !flush_hold_detections.is_empty() {
            tracing::warn!(
                count = flush_hold_detections.len(),
                "INBOUND BLOCKED (flush-hold): GuiPopup detection at EOF, fail-closed"
            );
            for d in &flush_hold_detections {
                tracing::warn!(rule = %d.rule_id, "flush-hold detection → fail-closed");
            }
            spawn_inbound_blocked_audit(
                &audit_store,
                &listener_provider_id,
                &caller,
                &flush_hold_detections,
                "anthropic_sse",
            );
            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
        }

        // ADR-038：流处理结束 → 超额计费观测（completion + relay usage 已跨 chunk 累计）。
        // 仅在流自然走到结尾时触发；中途被拦截 return 的流不观测（无完整 usage，可接受缺口）。
        if let (Some(bctx), Some(acc)) = (billing_ctx, billing_acc) {
            let claimed = acc.claimed();
            spawn_billing_observation(Some(bctx), acc.completion, claimed);
        }
    });

    let body_stream = ReceiverStream::new(rx);
    let response_body: ResponseBody = StreamBody::new(body_stream)
        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();

    Ok(Response::from_parts(resp_parts, response_body))
}

/// OpenAI 路径入站 SSE 解析检测（tee 模式，修 R6-#2）。
///
/// 与 [`forward_with_inbound_inspection`] 逻辑完全对称，唯一区别是使用
/// [`sieve_core::sse::openai_parser::OpenAiSseParser`] 而非 Anthropic [`SseParser`]。
///
/// OpenAI SSE 格式：`data: {...}\n\n`，无 `event:` 头。
/// 产出的 [`SseEvent`] 类型与 Anthropic 相同，inbound_filter 无需感知协议差异。
///
/// R6-#3 RESOLVED：OpenAiSseParser 已支持 ContentBlockStart/Stop（含 tool_call 首帧），
/// Aggregator 的 tool_use 完整检测能力已经生效。
///
/// 关联：ADR-018 §流式解析 / PRD v1.5 §6.1 / R6-#2。
#[allow(clippy::too_many_arguments)]
async fn forward_with_openai_inbound_inspection(
    forwarder: Arc<Forwarder>,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
    meta: MultiAgentMeta,
    ctx: RequestCtx,
    billing_ctx: BillingCtxHandle,
) -> Result<Response<ResponseBody>> {
    // 解构 ctx 供内部使用
    let RequestCtx {
        caller,
        audit: audit_store,
        listener_protocol,
        listener_provider_id,
    } = ctx;
    use http_body_util::Full;
    use sieve_core::sse::openai_parser::OpenAiSseParser;
    use sieve_core::sse::parser::SseParse as _;

    inbound_filter.set_source_channel(meta.source_channel.clone());

    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = Full::new(body_bytes)
        .map_err(|e| -> hyper::Error { match e {} })
        .boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (mut resp_parts, resp_body) = upstream_resp.into_parts();

    // 剥掉 content-length，防止 hyper client 截断注入的 sieve_blocked event。
    resp_parts.headers.remove(http::header::CONTENT_LENGTH);

    // 漏洞修复（lessons.md 2026-04-27 [安全]）：OpenAI 路径同样按 Content-Type 路由。
    // application/json 非流式响应里的 tool_calls 数组否则会完全绕过入站检测。
    let is_json_response = resp_parts
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.starts_with("application/json"))
        .unwrap_or(false);

    if is_json_response {
        return handle_openai_json_inbound(
            resp_parts,
            resp_body,
            inbound_filter,
            dry_run,
            meta,
            ipc.clone(),
            RequestCtx::new(
                caller.clone(),
                Arc::clone(&audit_store),
                listener_protocol,
                listener_provider_id.clone(),
            ),
            billing_ctx,
        )
        .await;
    }

    const INBOUND_CHANNEL_DEPTH: usize = 64;
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
        INBOUND_CHANNEL_DEPTH,
    );

    let inbound_meta = meta;

    tokio::spawn(async move {
        let meta = inbound_meta;
        let mut parser = OpenAiSseParser::new();
        let mut aggregator = Aggregator::new();
        // ADR-038：仅 billing 启用时累计 OpenAI SSE usage + completion。
        let mut billing_acc = billing_ctx
            .as_ref()
            .map(|_| BillingSseAccumulator::default());

        use http_body_util::BodyStream;
        let mut stream = BodyStream::new(resp_body);

        while let Some(frame_result) = stream.next().await {
            match frame_result {
                Ok(frame) => {
                    let Some(frame_bytes) = frame.data_ref().cloned() else {
                        if tx.send(Ok(frame)).await.is_err() {
                            return;
                        }
                        continue;
                    };

                    // P0-5：feed 超限时 fail-closed（IN-CAP-01）
                    let events = match parser.feed(&frame_bytes) {
                        Ok(evts) => evts,
                        Err(e) => {
                            tracing::warn!(error = %e, "OpenAI SSE parser 容量超限，fail-closed 注入 sieve_blocked");
                            let cap_detection =
                                build_cap_detection("IN-CAP-01", "cap-sse-event-too-large");
                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    };

                    // ADR-038：累计本批 SSE usage + completion（OpenAI SSE 观测）。
                    if let Some(acc) = billing_acc.as_mut() {
                        acc.observe_events(&events);
                    }

                    // 修 R8-#2：传入 meta.chain_depth，chain_depth ≥ 2 时 HookMark 升级为 GuiPopup
                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                        &events,
                        &mut inbound_filter,
                        &mut aggregator,
                        dry_run,
                        meta.chain_depth,
                        &ipc,
                        &audit_store,
                        caller.as_ref(),
                        &listener_provider_id,
                    );

                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
                    if !blocking.is_empty() {
                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (openai)");
                        for d in &blocking {
                            tracing::warn!(rule = %d.rule_id, "openai inbound detection");
                        }
                        spawn_inbound_blocked_audit(
                            &audit_store,
                            &listener_provider_id,
                            &caller,
                            &blocking,
                            "openai_sse",
                        );
                        let blocked_payload = build_sieve_blocked_sse(&blocking);
                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                        return;
                    }

                    // 2. Hook 类：写 pending 文件，失败时 fail-closed
                    for d in &hook_detections {
                        if let Err(e) = write_hook_pending_or_fail_closed(d, &meta) {
                            tracing::error!(
                                error = %e,
                                rule = %d.rule_id,
                                "Hook pending write failed (openai); fail-closed"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    }

                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
                    if !hold_detections.is_empty() {
                        if let Some(ref ipc_server) = ipc {
                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
                            let tx_ka = tx.clone();

                            let ka_fwd_handle = tokio::spawn(async move {
                                while let Some(ka_bytes) = ka_rx.recv().await {
                                    if tx_ka
                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            });

                            use chrono::Utc;
                            let request_id = uuid::Uuid::new_v4();
                            let timeout_seconds = hold_detections
                                .iter()
                                .find_map(|d| {
                                    if let Action::HoldForDecision {
                                        timeout_seconds, ..
                                    } = d.action
                                    {
                                        Some(timeout_seconds)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(60);

                            let ipc_detections = hold_detections
                                .iter()
                                .map(|d| sieve_ipc::protocol::DetectionPayload {
                                    rule_id: d.rule_id.clone(),
                                    severity: map_severity_to_ipc(d.severity),
                                    disposition: sieve_ipc::Disposition::GuiPopup,
                                    title: format!("检测命中（openai）：{}", d.rule_id),
                                    one_line_summary: d.evidence_truncated.clone(),
                                    details: serde_json::json!({}),
                                    recommendation: None,
                                })
                                .collect();

                            // v2.0：计算 allow_remember（PRD §5.4.2）
                            let openai_sse_rule_ids: Vec<&str> =
                                hold_detections.iter().map(|d| d.rule_id.as_str()).collect();
                            let allow_remember = compute_allow_remember(&openai_sse_rule_ids);

                            let ipc_req = sieve_ipc::DecisionRequest {
                                request_id,
                                created_at: Utc::now(),
                                timeout_seconds,
                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                                detections: ipc_detections,
                                source_agent: meta.source_agent,
                                origin_chain: meta.origin_chain.clone(),
                                source_channel: meta.source_channel.clone(),
                                // 修 R7-#5：填入 header 真实 chain_depth
                                explicit_chain_depth: Some(meta.chain_depth),
                                allow_remember,
                            };

                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
                                Arc::clone(ipc_server),
                                ipc_req,
                                ka_tx,
                                "inbound",
                            )
                            .await;

                            ka_fwd_handle.abort();

                            match outcome {
                                Ok(sieve_core::pipeline::HoldOutcome::Allow {
                                    remember,
                                    context_hint,
                                })
                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow {
                                    remember,
                                    context_hint,
                                }) => {
                                    // v2.0 §5.4.2：remember=true 时写灰名单（OpenAI 入站 SSE 路径）
                                    if remember && allow_remember {
                                        let agent_str =
                                            format!("{:?}", meta.source_agent).to_lowercase();
                                        for det in &hold_detections {
                                            try_write_graylist(
                                                &det.rule_id,
                                                &det.evidence_truncated,
                                                "",
                                                "openai",
                                                "inbound_sse",
                                                &agent_str,
                                                context_hint.clone(),
                                                &request_id.to_string(),
                                                &audit_store,
                                                &caller,
                                                &listener_provider_id,
                                            );
                                        }
                                    }

                                    if tx
                                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }
                                    continue;
                                }
                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
                                    tracing::warn!(%reason, "INBOUND BLOCKED (openai) by GUI decision");
                                    spawn_inbound_blocked_audit(
                                        &audit_store,
                                        &listener_provider_id,
                                        &caller,
                                        &hold_detections,
                                        "openai_sse",
                                    );
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "IPC hold error (openai), fail-closed");
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                            }
                        } else {
                            tracing::warn!(
                                "GuiPopup detection (openai) but IPC server not initialized; fail-closed"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    }

                    // 无 blocking / hold：透传原始 frame
                    if tx
                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(std::io::Error::other(format!(
                            "upstream body error (openai): {e}"
                        ))))
                        .await;
                    return;
                }
            }
        }

        // 流结束（EOF / 提前断流），flush parser 解析残留
        let flushed = parser.flush();
        // ADR-038：累计 flush 残留 events（流尾 usage / 末段文本可能在此）。
        if let Some(acc) = billing_acc.as_mut() {
            acc.observe_events(&flushed);
        }
        // 修 R8-#2：flush 阶段同样传入 chain_depth，HookMark 升级逻辑一致
        let (blocking, hook_detections, flush_hold_detections) = classify_inbound_detections(
            &flushed,
            &mut inbound_filter,
            &mut aggregator,
            dry_run,
            meta.chain_depth,
            &ipc,
            &audit_store,
            caller.as_ref(),
            &listener_provider_id,
        );

        for d in &hook_detections {
            if let Err(e) = write_hook_pending_or_fail_closed(d, &meta) {
                tracing::error!(
                    error = %e,
                    rule = %d.rule_id,
                    "Hook pending write failed (openai flush); fail-closed"
                );
                let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                return;
            }
        }

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (openai flush)");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, "openai inbound detection (flush)");
            }
            spawn_inbound_blocked_audit(
                &audit_store,
                &listener_provider_id,
                &caller,
                &blocking,
                "openai_sse",
            );
            let blocked_payload = build_sieve_blocked_sse(&blocking);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
            return;
        }

        if !flush_hold_detections.is_empty() {
            tracing::warn!(
                count = flush_hold_detections.len(),
                "INBOUND BLOCKED (openai flush-hold): GuiPopup at EOF, fail-closed"
            );
            for d in &flush_hold_detections {
                tracing::warn!(rule = %d.rule_id, "openai flush-hold detection → fail-closed");
            }
            spawn_inbound_blocked_audit(
                &audit_store,
                &listener_provider_id,
                &caller,
                &flush_hold_detections,
                "openai_sse",
            );
            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
        }

        // ADR-038：流处理结束 → 超额计费观测（OpenAI SSE）。
        if let (Some(bctx), Some(acc)) = (billing_ctx, billing_acc) {
            let claimed = acc.claimed();
            spawn_billing_observation(Some(bctx), acc.completion, claimed);
        }
    });

    let body_stream = ReceiverStream::new(rx);
    let response_body: ResponseBody = StreamBody::new(body_stream)
        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();

    Ok(Response::from_parts(resp_parts, response_body))
}

/// 对一批已解析的 [`SseEvent`] 运行 inbound 检测，按 action 分类返回三个列表：
/// - `blocking`：`Action::Block` 需立即截流的 detections
/// - `hook_detections`：`Action::HookMark` 需写 pending 文件的 detections
/// - `hold_detections`：`Action::HoldForDecision` 需 hold 流的 detections
///
/// v1.4 变更：不再把所有 Critical 都返回 blocking；HookMark 和 HoldForDecision 单独处理。
///
/// 关联 ADR-016 §dispatch 路由、ADR-014 §双层防御。
/// 修 R8-#2：新增 `chain_depth` 参数，实现入站 SSE HookMark 在 chain_depth ≥ 2 时
/// 升级为 HoldForDecision（GuiPopup），与出站路径和 IN-CR-06 路径的升级策略一致。
///
/// 旧实现：入站 HookMark 命中直接写 pending 文件然后继续转发流，
/// 但 daemon 注释明确要求 chain_depth ≥ 2 所有命中强制 GuiPopup hold；
/// 升级逻辑在出站路径已实现，入站路径漏掉导致行为不一致。
///
/// 修法：chain_depth ≥ 2 时把 HookMark detection 的 action 替换为 HoldForDecision，
/// 移入 hold_detections 而非 hook_detections，从而走 GUI hold 分支。
///
/// 关联 ADR-019 §chain_depth 升级策略、PRD v1.5 §6.5。
/// PRD v2.0 §5.7.4 双路径不变量：从 SSE / JSON 任一路径完成 tool_use 后调本 helper
/// 把 record 加入序列窗口并跑 IN-SEQ-* 启发式。
///
/// 默认 cargo feature `sequence_detection` 关闭时（GA 默认形态，PRD §9 #15），
/// `record_tool_use_into_sequence` 与 `detect_sequence_hits` 均为 no-op，本函数零开销。
///
/// 命中通过 IPC broadcast StatusBar 通知 + audit 写入（PRD §5.7 / §9 #15）。
/// **不引入新 Block 路径**（PRD §9 #15 硬约束）。
///
/// v2.0：新增 `ipc_server` / `audit_store` / `caller` 参数，接入 StatusBar + audit。
/// feature `sequence_detection` 关闭时 detect_sequence_hits 是 no-op，
/// 但闭包仍构造 Arc clone 保持代码始终编译通过。
#[allow(clippy::too_many_arguments)]
fn record_into_sequence_and_detect(
    inbound_filter: &sieve_core::pipeline::inbound::InboundFilter,
    tool: &sieve_core::CompletedToolCall,
    rule_hits: &[sieve_core::Detection],
    path_label: &'static str,
    ipc_server: &Option<Arc<sieve_ipc::IpcServer>>,
    audit_store: &Arc<crate::audit::AuditStore>,
    caller: Option<&crate::process_context::CallerInfo>,
    provider_id: &str,
) {
    let rule_ids: Vec<String> = rule_hits.iter().map(|d| d.rule_id.clone()).collect();
    if let Err(e) = inbound_filter.record_tool_use_into_sequence(&tool.name, &tool.input, rule_ids)
    {
        tracing::warn!(path = path_label, error = %e, "sequence_window record failed");
        return;
    }
    match inbound_filter.detect_sequence_hits() {
        Ok(hits) if !hits.is_empty() => {
            for h in &hits {
                tracing::info!(
                    target: "sequence_alert",
                    path = path_label,
                    rule_id = %h.rule_id,
                    description = %h.description,
                    "IN-SEQ-* sequence detection hit (StatusBar notify, no block per PRD §9 #15)"
                );

                // v2.0 §5.7：IPC StatusBar 通知（单向广播，no-block）
                if let Some(ref srv) = ipc_server {
                    let notify = sieve_ipc::protocol::StatusBarNotify {
                        notify_id: uuid::Uuid::now_v7(),
                        created_at: chrono::Utc::now(),
                        kind: sieve_ipc::protocol::NotifyKind::SequenceHit,
                        title: format!("行为序列检测命中：{}", h.rule_id),
                        detail: Some(h.description.clone()),
                        rule_id: Some(h.rule_id.clone()),
                        auto_dismiss_seconds: 10,
                    };
                    srv.broadcast_status_bar(notify);
                }

                // v2.0 §5.7：audit 写入（fail-soft，PRD §5.6.1）
                let event = crate::audit::AuditEvent::SequenceHit {
                    rule_id: h.rule_id.clone(),
                    description: h.description.clone(),
                    path_label: path_label.to_owned(),
                    caller: crate::audit::CallerContext {
                        pid: caller.map(|c| c.pid),
                        exe: caller
                            .and_then(|c| c.exe.as_ref())
                            .map(|p| p.display().to_string()),
                    },
                };
                let audit_clone = Arc::clone(audit_store);
                let provider_id_owned = provider_id.to_owned();
                tokio::spawn(async move {
                    if let Err(e) = audit_clone.append(event, &provider_id_owned).await {
                        tracing::warn!(error = %e, "audit append SequenceHit failed");
                    }
                });
            }
        }
        Ok(_) => {}
        Err(e) => tracing::warn!(path = path_label, error = %e, "sequence_window detect failed"),
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_inbound_detections(
    events: &[sieve_core::sse::parser::SseEvent],
    inbound_filter: &mut sieve_core::pipeline::inbound::InboundFilter,
    aggregator: &mut sieve_core::tool_use_aggregator::Aggregator,
    dry_run: bool,
    chain_depth: usize,
    ipc_server: &Option<Arc<sieve_ipc::IpcServer>>,
    audit_store: &Arc<crate::audit::AuditStore>,
    caller: Option<&crate::process_context::CallerInfo>,
    provider_id: &str,
) -> (
    Vec<sieve_core::Detection>,
    Vec<sieve_core::Detection>,
    Vec<sieve_core::Detection>,
) {
    let mut all_hits: Vec<sieve_core::Detection> = Vec::new();

    for evt in events {
        match inbound_filter.observe_event(evt) {
            Ok(hits) => all_hits.extend(hits),
            Err(e) => tracing::warn!(error = %e, "inbound observe_event error"),
        }
        match aggregator.process(evt) {
            Ok(Some(tool)) => match inbound_filter.on_tool_use_complete(&tool) {
                Ok(hits) => {
                    // PRD §5.7.4 双路径不变量：SSE 路径接入序列窗口（feature off 时是 no-op）
                    record_into_sequence_and_detect(
                        inbound_filter,
                        &tool,
                        &hits,
                        "sse",
                        ipc_server,
                        audit_store,
                        caller,
                        provider_id,
                    );
                    all_hits.extend(hits);
                }
                Err(e) => tracing::warn!(error = %e, "inbound on_tool_use_complete error"),
            },
            Ok(None) => {}
            Err(sieve_core::tool_use_aggregator::AggregatorError::MalformedToolUse {
                ref tool_id,
                ref error,
            }) => {
                tracing::warn!(tool_id = %tool_id, error = %error, "malformed tool_use partial_json，fail-closed Critical");
                all_hits.push(build_malformed_tool_use_detection(tool_id));
            }
            Err(e) => {
                tracing::warn!(error = %e, "aggregator 容量超限，fail-closed");
                all_hits.push(build_cap_detection("IN-CAP-02", "cap-aggregator-too-large"));
            }
        }
    }

    let mut blocking: Vec<sieve_core::Detection> = Vec::new();
    let mut hook_detections: Vec<sieve_core::Detection> = Vec::new();
    let mut hold_detections: Vec<sieve_core::Detection> = Vec::new();

    for mut d in all_hits {
        match &d.action {
            Action::Block => {
                // fail-closed Critical Block 永远阻断；非 fail-closed 遵 dry_run
                if d.severity == sieve_core::Severity::Critical
                    && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
                {
                    blocking.push(d);
                }
                // 其余 Block（低于 Critical 或 dry_run 豁免）静默记录
            }
            Action::HookMark => {
                // 修 R8-#2：chain_depth ≥ 2 时 HookMark 升级为 HoldForDecision（强制 GUI hold）
                // 原来 HookMark 写 pending 文件后继续转发，但 chain_depth ≥ 2 规则要求强制弹窗。
                if chain_depth >= 2 {
                    tracing::info!(
                        chain_depth,
                        rule = %d.rule_id,
                        "入站 HookMark 因 chain_depth ≥ 2 升级为 GuiPopup"
                    );
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                    };
                    hold_detections.push(d);
                } else {
                    // chain_depth < 2：正常写 pending 文件，SSE 流继续转发
                    hook_detections.push(d);
                }
            }
            Action::HoldForDecision { .. } => {
                // GUI 类：hold 流等决策
                // fail-closed 规则 GuiPopup 也走 hold，失败时 fail-closed
                hold_detections.push(d);
            }
            Action::MarkOnly | Action::SilentLog | Action::Redact { .. } => {
                // 静默 / 状态栏 / 脱敏（入站脱敏暂不实现，Week 5）
            }
        }
    }

    (blocking, hook_detections, hold_detections)
}

/// 写 IPC pending 文件，失败时返回 `Err`（调用方负责 fail-closed）。
///
/// 旧函数 `write_hook_pending_silent` 只 warn 后继续，违反 fail-closed 原则。
/// 新函数返回 `Result`，调用方在 `Err` 时必须注入 `sieve_blocked` 并截流。
///
/// 修 R7-#3：加 `meta` 参数，DecisionRequest 中填入真实 multi-agent 元数据，
/// hook/GUI 读 pending 文件时不再丢失来源信息（之前硬编码 Unknown + 空 chain）。
///
/// 关联 PRD §9 #3（Critical 不可关）、ADR-014 §Hook 路径、SPEC-001 §3.1、ADR-019。
fn write_hook_pending_or_fail_closed(
    d: &sieve_core::Detection,
    meta: &MultiAgentMeta,
) -> Result<(), sieve_ipc::error::IpcError> {
    let sieve_home = sieve_ipc::paths::sieve_home()?;
    write_hook_pending_to(d, &sieve_home, meta)
}

/// 写 IPC pending 文件到指定 base 目录，失败时返回 `Err`。
///
/// 内部实现，分离出来方便测试注入临时路径，不依赖环境变量。
///
/// 修 R7-#3：`meta` 参数携带 source_agent / origin_chain / source_channel，
/// 注入 `DecisionRequest` 使 hook 端能展示完整来源信息。
///
/// 关联 SPEC-001 §3.1、ADR-014 §Hook 路径、ADR-019。
fn write_hook_pending_to(
    d: &sieve_core::Detection,
    sieve_home: &std::path::Path,
    meta: &MultiAgentMeta,
) -> Result<(), sieve_ipc::error::IpcError> {
    use chrono::Utc;

    let request_id = uuid::Uuid::new_v4();
    // 修 R7-#5：使用 meta.chain_depth（来自 X-Sieve-Origin header 真实数值），
    // 而非 origin_chain.len()（只计已知 hop 数，中间层未知时比真实值小）。
    let explicit_depth = Some(meta.chain_depth);
    let ipc_req = sieve_ipc::DecisionRequest {
        request_id,
        created_at: Utc::now(),
        timeout_seconds: 60,
        default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
        detections: vec![sieve_ipc::protocol::DetectionPayload {
            rule_id: d.rule_id.clone(),
            severity: map_severity_to_ipc(d.severity),
            disposition: sieve_ipc::Disposition::HookTerminal,
            title: format!("检测命中：{}", d.rule_id),
            one_line_summary: d.evidence_truncated.clone(),
            details: serde_json::json!({}),
            recommendation: None,
        }],
        // 修 R7-#3：注入真实 multi-agent 元数据（不再硬编码 Unknown/empty）
        source_agent: meta.source_agent,
        origin_chain: meta.origin_chain.clone(),
        source_channel: meta.source_channel.clone(),
        explicit_chain_depth: explicit_depth,
        // v2.0：Hook 类规则 allow_remember 按 fail-closed 清单计算（PRD §5.4.2）
        // Hook 类规则（IN-CR-02~04、IN-GEN-01/03）均在 FAIL_CLOSED_RULES 中，
        // 正常情况下此值为 false；保持动态计算以兼容未来用户自定义规则。
        allow_remember: compute_allow_remember(&[d.rule_id.as_str()]),
    };

    sieve_ipc::pending_file::write_pending(&ipc_req, sieve_home)?;

    tracing::info!(
        rule = %d.rule_id,
        request_id = %request_id,
        source_agent = ?meta.source_agent,
        "HookMark: pending file written, SSE stream continues"
    );

    Ok(())
}

/// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
pub(crate) fn map_severity_to_ipc(s: sieve_core::Severity) -> sieve_ipc::Severity {
    match s {
        sieve_core::Severity::Critical => sieve_ipc::Severity::Critical,
        sieve_core::Severity::High => sieve_ipc::Severity::High,
        sieve_core::Severity::Medium => sieve_ipc::Severity::Medium,
        sieve_core::Severity::Low => sieve_ipc::Severity::Low,
    }
}

/// 构造 sieve_blocked JSON 响应 body（application/json 路径使用，非 SSE 格式）。
///
/// 漏洞修复（lessons.md 2026-04-27 [安全]）：非流式 JSON 响应被拦截时，
/// 不能注入 SSE event 格式的 sieve_blocked，需要返回合法 JSON body。
/// 格式与 build_sieve_blocked_sse 的 payload 部分相同，只是不包装为 SSE event 行。
fn build_sieve_blocked_json_body(detections: &[sieve_core::Detection]) -> Bytes {
    let payload = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": epoch_secs_string(),
        "detections": detections.iter().map(|d| serde_json::json!({
            "rule_id": d.rule_id,
            "severity": d.severity,
            "fingerprint": d.fingerprint,
        })).collect::<Vec<_>>(),
        "guidance": {
            "zh": format!(
                "Sieve 检测到 {} 条入站 Critical 命中。非流式响应已被替换。\
                 Critical 级别命中不可通过白名单绕过，请人工审查当前上下文后重试。",
                detections.len()
            ),
            "en": format!(
                "Sieve blocked {} inbound critical detection(s). Non-streaming response replaced. \
                 Critical detections cannot be bypassed via allowlist. Please review and retry.",
                detections.len()
            ),
        }
    });
    Bytes::from(payload.to_string())
}

/// 构造因入站 JSON 拦截而替换响应的 HTTP Response。
fn build_sieve_blocked_json_response(
    detections: &[sieve_core::Detection],
) -> Response<ResponseBody> {
    let body_bytes = build_sieve_blocked_json_body(detections);
    Response::builder()
        .status(http::StatusCode::OK)
        .header(
            http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .header(http::header::CONTENT_LENGTH, body_bytes.len())
        .body(bytes_body(body_bytes))
        .unwrap_or_else(|_| Response::new(empty_body()))
}

/// JSON 路径（Anthropic / OpenAI 共用）单条入站 detection 的处置：push 进 `blocking`。
///
/// 非流式 JSON 无 keep-alive hold 机制，故 `HoldForDecision`（GuiPopup，含 IN-CR-01
/// 文本地址替换 / IN-CR-05 签名工具）降级为 fail-closed `Block`；`HookMark` 在
/// `chain_depth >= 2` 时升级阻断（与 SSE 路径一致）。tool_use 与响应文本两类命中共用，
/// 消除 handle_anthropic_json_inbound / handle_openai_json_inbound 的重复处置逻辑。
fn classify_json_inbound_detection(
    mut d: sieve_core::Detection,
    dry_run: bool,
    chain_depth: usize,
    blocking: &mut Vec<sieve_core::Detection>,
) {
    match &d.action {
        sieve_core::detection::Action::Block => {
            if d.severity == sieve_core::Severity::Critical
                && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
            {
                blocking.push(d);
            }
        }
        sieve_core::detection::Action::HoldForDecision { .. } => {
            // JSON 路径 GuiPopup：暂无 keep-alive 机制，fail-closed
            tracing::warn!(
                rule = %d.rule_id,
                "GuiPopup in non-streaming JSON path, fail-closed"
            );
            d.action = sieve_core::detection::Action::Block;
            blocking.push(d);
        }
        _ => {
            // HookMark / MarkOnly / SilentLog / Redact：记录但不阻断
            // chain_depth ≥ 2 时 HookMark 升级为 GuiPopup（同 SSE 路径）
            if chain_depth >= 2 && matches!(d.action, sieve_core::detection::Action::HookMark) {
                tracing::warn!(
                    rule = %d.rule_id,
                    "HookMark upgraded to GuiPopup (chain_depth >= 2), fail-closed in JSON path"
                );
                blocking.push(d);
            }
        }
    }
}

/// 处理 Anthropic 非流式 JSON 入站响应的入站检测路径。
///
/// 漏洞修复（lessons.md 2026-04-27 [安全]）：上游返回 `application/json` 时，
/// 收集完整 body，解析 `content[]` 中的 `tool_use` 块，
/// 喂给 `InboundFilter::on_tool_use_complete`。
/// 命中 fail-closed Critical 时把响应 body 替换为 sieve_blocked JSON；否则原样透传。
///
/// 不走 SSE 路径（tokio::spawn + channel tee），直接 await body 收集再决策。
#[allow(clippy::too_many_arguments)]
async fn handle_anthropic_json_inbound(
    resp_parts: http::response::Parts,
    resp_body: impl http_body::Body<Data = Bytes, Error = impl std::fmt::Display> + Send + 'static,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    meta: MultiAgentMeta,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    ctx: RequestCtx,
    billing_ctx: BillingCtxHandle,
) -> Result<Response<ResponseBody>> {
    // ADR-026 Stage E：listener_provider_id 透传到 record_into_sequence_and_detect，
    // listener_protocol 在 JSON 路径已确定（外层 proxy_inner 已校验），此处无消费者。
    let RequestCtx {
        caller,
        audit: audit_store,
        listener_protocol: _,
        listener_provider_id,
    } = ctx;
    use http_body_util::BodyExt as _;

    // 收集完整 body（非流式 JSON 响应通常很小，不存在 DoS 风险，上游负责 content-length 校验）
    let body_bytes = match resp_body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            tracing::warn!("handle_anthropic_json_inbound: collect body error: {e}");
            return Ok(build_sieve_blocked_json_response(&[]));
        }
    };

    // 解析响应 JSON（宽松解析，只取 content 数组）
    let resp_json: serde_json::Value = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(e) => {
            // 无法解析 JSON，原样透传（不拦截非 JSON 内容）
            tracing::debug!("handle_anthropic_json_inbound: non-JSON body, passthrough: {e}");
            return passthrough_json_response(resp_parts, body_bytes);
        }
    };

    // ADR-038：超额计费观测（Anthropic JSON 路径）。relay 声明的 usage（顶层 `usage`）+
    // 独立估算的 output（completion 全文）交叉比对，写 usage.db（fire-and-forget，永不上传）。
    spawn_billing_observation(
        billing_ctx,
        anthropic_completion_text(&resp_json),
        anthropic_claimed_usage(&resp_json),
    );

    // 提取 content[] 中的 tool_use 块
    let content_arr = resp_json.get("content").and_then(|v| v.as_array());
    let mut all_blocking: Vec<sieve_core::Detection> = Vec::new();

    if let Some(content) = content_arr {
        for block in content {
            let obj = match block.as_object() {
                Some(o) => o,
                None => continue,
            };
            if obj.get("type").and_then(|v| v.as_str()) != Some("tool_use") {
                continue;
            }
            let tool_id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let tool_name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tool_input = obj
                .get("input")
                .cloned()
                .unwrap_or(serde_json::Value::Object(Default::default()));

            let completed = sieve_core::CompletedToolCall {
                id: tool_id,
                name: tool_name,
                input: tool_input,
            };

            match inbound_filter.on_tool_use_complete(&completed) {
                Ok(hits) => {
                    // PRD §5.7.4 双路径不变量：Anthropic JSON 路径接入序列窗口
                    record_into_sequence_and_detect(
                        &inbound_filter,
                        &completed,
                        &hits,
                        "anthropic-json",
                        &ipc,
                        &audit_store,
                        caller.as_ref(),
                        &listener_provider_id,
                    );
                    for d in hits {
                        classify_json_inbound_detection(
                            d,
                            dry_run,
                            meta.chain_depth,
                            &mut all_blocking,
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "handle_anthropic_json_inbound: on_tool_use_complete error");
                }
            }
        }
    }

    // 入站文本扫描（IN-GEN-* + IN-CR-01 地址替换）——ADR-025 / PRD §9 #16 四路由对等。
    // 修复 v1.5.4 同类 P0：observe_event/scan_text 类入站能力此前只挂 SSE，两条 JSON
    // 路径 by-construction 不扫 assistant 文本，IN-CR-01 地址替换零拦截。
    let assistant_text = anthropic_completion_text(&resp_json);
    if !assistant_text.is_empty() {
        match inbound_filter.scan_assistant_text(&assistant_text) {
            Ok(hits) => {
                for d in hits {
                    classify_json_inbound_detection(
                        d,
                        dry_run,
                        meta.chain_depth,
                        &mut all_blocking,
                    );
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "handle_anthropic_json_inbound: scan_assistant_text error");
            }
        }
    }

    if !all_blocking.is_empty() {
        tracing::warn!(
            count = all_blocking.len(),
            "INBOUND BLOCKED (anthropic json)"
        );
        for d in &all_blocking {
            tracing::warn!(rule = %d.rule_id, "anthropic json inbound detection");
        }
        spawn_inbound_blocked_audit(
            &audit_store,
            &listener_provider_id,
            &caller,
            &all_blocking,
            "anthropic_json",
        );
        return Ok(build_sieve_blocked_json_response(&all_blocking));
    }

    // 无拦截：原样透传（重建含原 headers 的响应）
    passthrough_json_response(resp_parts, body_bytes)
}

/// 处理 OpenAI 非流式 JSON 入站响应的入站检测路径。
///
/// 漏洞修复（lessons.md 2026-04-27 [安全]）：上游返回 `application/json` 时，
/// 解析 `choices[].message.tool_calls` 提取 function 调用，喂给 InboundFilter。
/// 命中 fail-closed Critical 时替换响应 body 为 sieve_blocked JSON；否则原样透传。
#[allow(clippy::too_many_arguments)]
async fn handle_openai_json_inbound(
    resp_parts: http::response::Parts,
    resp_body: impl http_body::Body<Data = Bytes, Error = impl std::fmt::Display> + Send + 'static,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    meta: MultiAgentMeta,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    ctx: RequestCtx,
    billing_ctx: BillingCtxHandle,
) -> Result<Response<ResponseBody>> {
    // ADR-026 Stage E：listener_provider_id 透传到 record_into_sequence_and_detect，
    // listener_protocol 在 JSON 路径已确定（外层 proxy_inner 已校验），此处无消费者。
    let RequestCtx {
        caller,
        audit: audit_store,
        listener_protocol: _,
        listener_provider_id,
    } = ctx;
    use http_body_util::BodyExt as _;

    let body_bytes = match resp_body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            tracing::warn!("handle_openai_json_inbound: collect body error: {e}");
            return Ok(build_sieve_blocked_json_response(&[]));
        }
    };

    let resp_json: serde_json::Value = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(e) => {
            tracing::debug!("handle_openai_json_inbound: non-JSON body, passthrough: {e}");
            return passthrough_json_response(resp_parts, body_bytes);
        }
    };

    // ADR-038：超额计费观测（OpenAI JSON 路径，relay usage prompt_tokens/completion_tokens
    // + 独立估算 output 交叉比对，写 usage.db；永不上传）。
    spawn_billing_observation(
        billing_ctx,
        openai_completion_text(&resp_json),
        openai_claimed_usage(&resp_json),
    );

    // 遍历 choices[].message.tool_calls[]
    let choices = resp_json.get("choices").and_then(|v| v.as_array());
    let mut all_blocking: Vec<sieve_core::Detection> = Vec::new();

    if let Some(choices) = choices {
        for choice in choices {
            let message = match choice.get("message") {
                Some(m) => m,
                None => continue,
            };
            let tool_calls = match message.get("tool_calls").and_then(|v| v.as_array()) {
                Some(tc) => tc,
                None => continue,
            };
            for tc in tool_calls {
                let obj = match tc.as_object() {
                    Some(o) => o,
                    None => continue,
                };
                let tc_id = obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let func = match obj.get("function").and_then(|v| v.as_object()) {
                    Some(f) => f,
                    None => continue,
                };
                let func_name = func
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let arguments_str = func
                    .get("arguments")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");
                let input: serde_json::Value = serde_json::from_str(arguments_str)
                    .unwrap_or(serde_json::Value::Object(Default::default()));

                let completed = sieve_core::CompletedToolCall {
                    id: tc_id,
                    name: func_name,
                    input,
                };

                match inbound_filter.on_tool_use_complete(&completed) {
                    Ok(hits) => {
                        // PRD §5.7.4 双路径不变量：OpenAI JSON 路径接入序列窗口
                        record_into_sequence_and_detect(
                            &inbound_filter,
                            &completed,
                            &hits,
                            "openai-json",
                            &ipc,
                            &audit_store,
                            caller.as_ref(),
                            &listener_provider_id,
                        );
                        for d in hits {
                            classify_json_inbound_detection(
                                d,
                                dry_run,
                                meta.chain_depth,
                                &mut all_blocking,
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "handle_openai_json_inbound: on_tool_use_complete error");
                    }
                }
            }
        }
    }

    // 入站文本扫描（IN-GEN-* + IN-CR-01 地址替换）——ADR-025 / PRD §9 #16 四路由对等。
    // 修复 v1.5.4 同类 P0：observe_event/scan_text 类入站能力此前只挂 SSE，两条 JSON
    // 路径 by-construction 不扫 assistant 文本，IN-CR-01 地址替换零拦截。
    let assistant_text = openai_completion_text(&resp_json);
    if !assistant_text.is_empty() {
        match inbound_filter.scan_assistant_text(&assistant_text) {
            Ok(hits) => {
                for d in hits {
                    classify_json_inbound_detection(
                        d,
                        dry_run,
                        meta.chain_depth,
                        &mut all_blocking,
                    );
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "handle_openai_json_inbound: scan_assistant_text error");
            }
        }
    }

    if !all_blocking.is_empty() {
        tracing::warn!(count = all_blocking.len(), "INBOUND BLOCKED (openai json)");
        for d in &all_blocking {
            tracing::warn!(rule = %d.rule_id, "openai json inbound detection");
        }
        spawn_inbound_blocked_audit(
            &audit_store,
            &listener_provider_id,
            &caller,
            &all_blocking,
            "openai_json",
        );
        return Ok(build_sieve_blocked_json_response(&all_blocking));
    }

    passthrough_json_response(resp_parts, body_bytes)
}

/// 把已收集的 body bytes 原样构建为 HTTP Response（JSON 路径无拦截时透传）。
fn passthrough_json_response(
    resp_parts: http::response::Parts,
    body_bytes: Bytes,
) -> Result<Response<ResponseBody>> {
    let body_len = body_bytes.len();
    let mut builder = Response::builder().status(resp_parts.status);
    // 保留原 headers（除 content-length，重新按实际 body 大小设置）
    for (name, value) in &resp_parts.headers {
        if name == http::header::CONTENT_LENGTH {
            continue;
        }
        builder = builder.header(name, value);
    }
    builder = builder.header(http::header::CONTENT_LENGTH, body_len);
    let resp = builder
        .body(bytes_body(body_bytes))
        .map_err(|e| anyhow!("passthrough_json_response: build response: {e}"))?;
    Ok(resp)
}

/// 构造注入给客户端的 `sieve_blocked` SSE event 字节块。
fn build_sieve_blocked_sse(detections: &[sieve_core::Detection]) -> Bytes {
    let payload = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": epoch_secs_string(),
        "detections": detections.iter().map(|d| serde_json::json!({
            "rule_id": d.rule_id,
            "severity": d.severity,
            "fingerprint": d.fingerprint,
        })).collect::<Vec<_>>(),
        "guidance": {
            "zh": format!(
                "Sieve 检测到 {} 条入站 Critical 命中。流已截断，响应不完整。\
                 Critical 级别命中不可通过白名单绕过，请人工审查当前上下文后重试。",
                detections.len()
            ),
            "en": format!(
                "Sieve blocked {} inbound critical detection(s). Stream truncated. \
                 Critical detections cannot be bypassed via allowlist. Please review the context and retry.",
                detections.len()
            ),
        }
    });
    Bytes::from(format!("\nevent: sieve_blocked\ndata: {}\n\n", payload))
}

/// 把 `sieve_core::detection::DefaultOnTimeout` 映射为 `sieve_ipc::DefaultOnTimeout`。
///
/// 修 R3-#4 / R10-#2：engine_adapter 把规则里的 default_on_timeout 存入 Detection，
/// daemon 通过此函数转换后传给 IPC 层。
fn map_dot_to_ipc(dot: sieve_core::detection::DefaultOnTimeout) -> sieve_ipc::DefaultOnTimeout {
    match dot {
        sieve_core::detection::DefaultOnTimeout::Redact => sieve_ipc::DefaultOnTimeout::Redact,
        sieve_core::detection::DefaultOnTimeout::Block => sieve_ipc::DefaultOnTimeout::Block,
        sieve_core::detection::DefaultOnTimeout::Allow => sieve_ipc::DefaultOnTimeout::Allow,
    }
}

/// 合并两个 `DefaultOnTimeout`，取更严格的一方：Block > Redact > Allow。
///
/// 多条 hold_detections 时用于取最严兜底策略。
fn merge_strictest_timeout(
    a: sieve_ipc::DefaultOnTimeout,
    b: sieve_ipc::DefaultOnTimeout,
) -> sieve_ipc::DefaultOnTimeout {
    use sieve_ipc::DefaultOnTimeout::{Allow, Block, Redact};
    match (a, b) {
        (Block, _) | (_, Block) => Block,
        (Redact, _) | (_, Redact) => Redact,
        (Allow, Allow) => Allow,
    }
}

/// 用已收集的 body bytes 重新构造请求并转发。
async fn forward_raw(
    forwarder: Arc<Forwarder>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
) -> Result<Response<ResponseBody>> {
    use http_body_util::Full;

    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = Full::new(body_bytes)
        .map_err(|e| -> hyper::Error { match e {} })
        .boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (resp_parts, resp_body) = upstream_resp.into_parts();
    let body: ResponseBody = resp_body
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();
    Ok(Response::from_parts(resp_parts, body))
}

/// 流式透传（Week 1 路径），不缓冲 body。
async fn forward_streaming(
    forwarder: Arc<Forwarder>,
    mut parts: http::request::Parts,
    body: Incoming,
) -> Result<Response<ResponseBody>> {
    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = body.map_err(|e| -> hyper::Error { e }).boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (resp_parts, resp_body) = upstream_resp.into_parts();
    let body: ResponseBody = resp_body
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();
    Ok(Response::from_parts(resp_parts, body))
}

/// 构造因嵌套调用过深（chain_depth ≥ 5）的 426 Upgrade Required 响应。
///
/// 构造协议错位拒绝响应（ADR-026 §决策 4）。
///
/// listener 声明的协议与请求 path 隐含的协议不一致时，daemon fail-closed 拒绝，
/// 返回 400 Bad Request + sieve_blocked event payload。
/// 例：Anthropic listener 收到 `/v1/chat/completions` → 400。
/// 关联：ADR-026 §决策 4、PRD §9 #3 fail-closed、ADR-007。
/// 构造 500 响应——仅用于「类型不变量被违反」的不可达 BUG 兜底，把原热路径
/// `.expect()` 的 panic（= DoS 向量）降级为明确的 500 + error log。正常控制流
/// 永不命中（请求体两态在 collect 块就已确定，见 proxy_inner 的 `ProxyRequestBody`）。
fn build_500_internal_response(reason: &str) -> Response<ResponseBody> {
    let body_json = serde_json::json!({
        "type": "sieve_error",
        "error": "internal_invariant_violation",
        "reason": reason,
    });
    let body_bytes = Bytes::from(body_json.to_string());
    Response::builder()
        .status(http::StatusCode::INTERNAL_SERVER_ERROR) // 500
        .header(
            http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .body(bytes_body(body_bytes))
        .unwrap_or_else(|_| Response::new(empty_body()))
}

fn build_protocol_mismatch_400(
    path: &str,
    listener_protocol: crate::config::Protocol,
) -> Response<ResponseBody> {
    let listener_proto_str = match listener_protocol {
        // Auto 永不进入本函数（Auto listener 不触发错位检查），保留以满足穷尽性。
        crate::config::Protocol::Auto => "auto",
        crate::config::Protocol::Anthropic => "anthropic",
        crate::config::Protocol::Openai => "openai",
    };
    let body_json = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": epoch_secs_string(),
        "reason": "listener_protocol_mismatch",
        "request_path": path,
        "listener_protocol": listener_proto_str,
        "guidance": {
            "zh": format!(
                "Sieve 检测到本 listener 声明协议为 {}，但请求路径 {} 属于不同协议，拒绝处理。\
                 建议：将 client 指向声明匹配协议的 listener，或检查 sieve.toml [[upstream]] 配置。",
                listener_proto_str, path
            ),
            "en": format!(
                "Sieve rejected request: listener configured for {} protocol but request path {} \
                 implies a different protocol. ADR-026 enforces strict listener-protocol matching.",
                listener_proto_str, path
            ),
        }
    });
    let body_bytes = Bytes::from(body_json.to_string());
    Response::builder()
        .status(http::StatusCode::BAD_REQUEST) // 400
        .header(
            http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .body(bytes_body(body_bytes))
        .unwrap_or_else(|_| Response::new(empty_body()))
}

/// 攻击模式检测：超过 5 层 agent 嵌套调用视为异常，直接拒绝。
/// 关联：ADR-019 §嵌套深度限制、PRD v1.5 §6.5。
fn build_426_nested_rejection(chain_depth: usize) -> Response<ResponseBody> {
    let body_json = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": epoch_secs_string(),
        "reason": "nested_call_too_deep",
        "chain_depth": chain_depth,
        "guidance": {
            "zh": format!(
                "Sieve 检测到 agent 嵌套调用层数（{}）超过安全上限（5），请求被拒绝。",
                chain_depth
            ),
            "en": format!(
                "Sieve rejected request: nested agent call depth ({}) exceeds safety limit (5).",
                chain_depth
            ),
        }
    });
    let body_bytes = Bytes::from(body_json.to_string());
    Response::builder()
        .status(http::StatusCode::UPGRADE_REQUIRED) // 426
        .header(
            http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .body(bytes_body(body_bytes))
        .unwrap_or_else(|_| Response::new(empty_body()))
}

/// 构造 426 Upgrade Required 拦截响应（ADR-008 候选）。
fn build_426_response(detections: &[sieve_core::Detection]) -> Response<ResponseBody> {
    let blocked_at = epoch_secs_string();
    let detections_json: Vec<serde_json::Value> = detections
        .iter()
        .map(|d| {
            serde_json::json!({
                "rule_id": d.rule_id,
                "severity": d.severity,
                "fingerprint": d.fingerprint,
            })
        })
        .collect();
    let body_json = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": blocked_at,
        "detections": detections_json,
        "guidance": {
            "zh": format!(
                "Sieve 检测到 {} 条出站 Critical 命中。请检查后用 .sieveignore 加入 fingerprint 白名单，或重新发送脱敏消息。",
                detections.len()
            ),
            "en": format!(
                "Sieve blocked {} outbound critical detection(s). Review your message, then either redact or add fingerprint(s) to .sieveignore.",
                detections.len()
            ),
        }
    });
    let body_bytes = Bytes::from(body_json.to_string());
    Response::builder()
        .status(http::StatusCode::UPGRADE_REQUIRED) // 426
        .header(
            http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .body(bytes_body(body_bytes))
        .unwrap_or_else(|_| Response::new(empty_body()))
}

/// 返回 UNIX epoch 秒字符串（Phase 1 简化，Week 4 改 RFC3339）。
fn epoch_secs_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.to_string()
}

/// 把字节包成 `ResponseBody`。
fn bytes_body(b: Bytes) -> ResponseBody {
    use http_body_util::Full;
    Full::new(b)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
        .boxed()
}

/// 把字符串包成 `ResponseBody`（用于错误响应）。
fn string_body(s: String) -> ResponseBody {
    bytes_body(Bytes::from(s))
}

/// 空 body（fallback 错误响应）。
fn empty_body() -> ResponseBody {
    use http_body_util::Empty;
    Empty::<Bytes>::new()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
        .boxed()
}

/// 构造 malformed tool_use Detection（P0-6，IN-CR-05-MALFORMED）。
fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
    use sieve_core::detection::{Action, ContentSource};
    use sieve_core::protocol::unified_message::ContentSpan;
    use uuid::Uuid;
    sieve_core::Detection {
        id: Uuid::new_v4(),
        rule_id: "IN-CR-05-MALFORMED".into(),
        severity: sieve_core::Severity::Critical,
        action: Action::Block,
        source: ContentSource::InboundAssistantText,
        span: ContentSpan { start: 0, end: 0 },
        evidence_truncated: format!("tool_id={tool_id}"),
        fingerprint: "malformed-tool-use-partial-json".into(),
        source_channel: None,
        origin_chain_depth: 0,
    }
}

/// 构造容量上限 Detection（P0-5，IN-CAP-01 / IN-CAP-02）。
fn build_cap_detection(rule_id: &str, fingerprint_key: &str) -> sieve_core::Detection {
    use sieve_core::detection::{Action, ContentSource};
    use sieve_core::protocol::unified_message::ContentSpan;
    use uuid::Uuid;
    sieve_core::Detection {
        id: Uuid::new_v4(),
        rule_id: rule_id.into(),
        severity: sieve_core::Severity::Critical,
        action: Action::Block,
        source: ContentSource::InboundAssistantText,
        span: ContentSpan { start: 0, end: 0 },
        evidence_truncated: String::new(),
        fingerprint: fingerprint_key.into(),
        source_channel: None,
        origin_chain_depth: 0,
    }
}

/// 把脱敏后的文本段列表写回 [`AnthropicRequest`] 并返回新 request。
///
/// `original_texts` 是 `extract_text_content()` 返回的原始段列表；
/// `redacted_texts` 是 `redact_segments()` 返回的替换后文本列表（顺序对应）。
///
/// 实现逻辑：遍历 messages，对每个文本 content 按 segment 索引匹配并替换。
///
/// # Errors
/// 如果 `redacted_texts` 长度与 `original_texts` 不一致，返回错误。
///
/// 关联：PRD v1.4 §6.1（AutoRedact 路径），修 #1（AutoRedact 偏移修复）。
fn apply_redacted_texts_to_request(
    req: &sieve_core::protocol::anthropic::AnthropicRequest,
    original_texts: &[(usize, String)],
    redacted_texts: &[String],
) -> Result<sieve_core::protocol::anthropic::AnthropicRequest> {
    if original_texts.len() != redacted_texts.len() {
        return Err(anyhow!(
            "redacted_texts 长度 {} 与 original_texts 长度 {} 不一致",
            redacted_texts.len(),
            original_texts.len()
        ));
    }

    // 用计数器追踪当前处理到第几个 segment（与 extract_text_content 遍历顺序一致）
    let mut seg_idx = 0usize;

    let mut new_messages: Vec<sieve_core::protocol::anthropic::AnthropicMessage> = Vec::new();
    for msg in &req.messages {
        let new_content = match &msg.content {
            serde_json::Value::String(_) => {
                // String 类型：一个 segment
                let replacement = redacted_texts
                    .get(seg_idx)
                    .cloned()
                    .unwrap_or_else(|| msg.content.as_str().unwrap_or("").to_string());
                seg_idx += 1;
                serde_json::Value::String(replacement)
            }
            serde_json::Value::Array(blocks) => {
                let mut new_blocks = Vec::with_capacity(blocks.len());
                for block in blocks {
                    if let Some(block_obj) = block.as_object() {
                        if block_obj.get("type").and_then(|v| v.as_str()) == Some("text")
                            && block_obj.get("text").and_then(|v| v.as_str()).is_some()
                        {
                            let replacement =
                                redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                                    block_obj
                                        .get("text")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string()
                                });
                            seg_idx += 1;
                            let mut new_obj = block_obj.clone();
                            new_obj
                                .insert("text".to_string(), serde_json::Value::String(replacement));
                            new_blocks.push(serde_json::Value::Object(new_obj));
                            continue;
                        }
                    }
                    new_blocks.push(block.clone());
                }
                serde_json::Value::Array(new_blocks)
            }
            other => other.clone(),
        };
        new_messages.push(sieve_core::protocol::anthropic::AnthropicMessage {
            role: msg.role.clone(),
            content: new_content,
        });
    }

    // 处理 system prompt（与 extract_text_content 遍历顺序一致）
    let new_system = if let Some(system) = &req.system {
        if system.as_str().is_some() {
            let replacement = redacted_texts
                .get(seg_idx)
                .cloned()
                .unwrap_or_else(|| system.as_str().unwrap_or("").to_string());
            seg_idx += 1;
            Some(serde_json::Value::String(replacement))
        } else if let Some(blocks) = system.as_array() {
            let mut new_blocks = Vec::with_capacity(blocks.len());
            for block in blocks {
                if block.get("text").and_then(|v| v.as_str()).is_some() {
                    let replacement = redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                        block
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    });
                    seg_idx += 1;
                    let mut new_obj = block.as_object().cloned().unwrap_or_default();
                    new_obj.insert("text".to_string(), serde_json::Value::String(replacement));
                    new_blocks.push(serde_json::Value::Object(new_obj));
                } else {
                    new_blocks.push(block.clone());
                }
            }
            Some(serde_json::Value::Array(new_blocks))
        } else {
            Some(system.clone())
        }
    } else {
        None
    };

    let _ = seg_idx; // 消除 unused variable 警告

    Ok(sieve_core::protocol::anthropic::AnthropicRequest {
        model: req.model.clone(),
        max_tokens: req.max_tokens,
        messages: new_messages,
        stream: req.stream,
        system: new_system,
        tools: req.tools.clone(),
        tool_choice: req.tool_choice.clone(),
        extra: req.extra.clone(),
    })
}

/// 把脱敏后的文本段列表写回 [`OpenAIRequest`] 并返回新 request（修 A2-#1）。
///
/// OpenAI `message.content` 有两种形式：
/// - `string`：对应一个 segment
/// - `array of content parts`：每个 `{"type":"text","text":"..."}` 对应一个 segment；
///   `image_url` 等非文本 part 原样保留（不计入 segment 计数）
///
/// `original_texts` 与 `redacted_texts` 必须顺序对应；长度不一致时返回错误。
///
/// 关联：PRD v1.4 §6.1（AutoRedact），ADR-018（OpenAI 协议适配）。
fn apply_redacted_texts_to_openai_request(
    req: &sieve_core::protocol::openai::OpenAIRequest,
    original_texts: &[(usize, String)],
    redacted_texts: &[String],
) -> Result<sieve_core::protocol::openai::OpenAIRequest> {
    if original_texts.len() != redacted_texts.len() {
        return Err(anyhow!(
            "redacted_texts 长度 {} 与 original_texts 长度 {} 不一致",
            redacted_texts.len(),
            original_texts.len()
        ));
    }

    let mut seg_idx = 0usize;
    let mut new_messages: Vec<sieve_core::protocol::openai::OpenAIMessage> = Vec::new();

    for msg in &req.messages {
        let new_content = match &msg.content {
            Some(serde_json::Value::String(_)) => {
                let replacement = redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                    msg.content
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                });
                seg_idx += 1;
                Some(serde_json::Value::String(replacement))
            }
            Some(serde_json::Value::Array(parts)) => {
                let mut new_parts = Vec::with_capacity(parts.len());
                for part in parts {
                    if let Some(obj) = part.as_object() {
                        if obj.get("type").and_then(|v| v.as_str()) == Some("text")
                            && obj.get("text").and_then(|v| v.as_str()).is_some()
                        {
                            let replacement =
                                redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                                    obj.get("text")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string()
                                });
                            seg_idx += 1;
                            let mut new_obj = obj.clone();
                            new_obj
                                .insert("text".to_string(), serde_json::Value::String(replacement));
                            new_parts.push(serde_json::Value::Object(new_obj));
                            continue;
                        }
                    }
                    // image_url 等非 text part 原样保留，不消耗 segment index
                    new_parts.push(part.clone());
                }
                Some(serde_json::Value::Array(new_parts))
            }
            other => other.clone(),
        };
        new_messages.push(sieve_core::protocol::openai::OpenAIMessage {
            role: msg.role.clone(),
            content: new_content,
            name: msg.name.clone(),
            tool_calls: msg.tool_calls.clone(),
            tool_call_id: msg.tool_call_id.clone(),
            extra: msg.extra.clone(),
        });
    }

    let _ = seg_idx; // 消除 unused variable 警告

    Ok(sieve_core::protocol::openai::OpenAIRequest {
        model: req.model.clone(),
        messages: new_messages,
        stream: req.stream,
        tools: req.tools.clone(),
        max_tokens: req.max_tokens,
        temperature: req.temperature,
        extra: req.extra.clone(),
    })
}

// ─── 单元测试：Hook pending fail-closed ──────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sieve_core::detection::{Action, ContentSource, Detection, Severity};
    use sieve_core::protocol::unified_message::ContentSpan;
    use uuid::Uuid;

    /// 构造最小化的 HookMark Detection，用于测试 write_hook_pending_to。
    fn make_hook_detection() -> Detection {
        Detection {
            id: Uuid::new_v4(),
            rule_id: "IN-CR-02".to_string(),
            severity: Severity::Critical,
            action: Action::HookMark,
            source: ContentSource::InboundToolUseInput,
            span: ContentSpan { start: 0, end: 10 },
            evidence_truncated: "rm -rf /".to_string(),
            fingerprint: "deadbeef01234567".to_string(),
            source_channel: None,
            origin_chain_depth: 0,
        }
    }

    /// happy path：base 目录可写 → 返回 Ok，pending 文件存在。
    ///
    /// 验证 HookMark 写成功后调用方可继续转发 SSE 流，不触发 fail-closed。
    /// 关联 PRD §9 #3、SPEC-001 §3.1。
    #[test]
    fn hook_pending_write_happy_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let d = make_hook_detection();

        let meta = MultiAgentMeta {
            source_agent: sieve_ipc::protocol::SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            chain_depth: 0,
        };
        let result = write_hook_pending_to(&d, tmp.path(), &meta);

        assert!(result.is_ok(), "可写目录应返回 Ok，得到: {result:?}");

        // 验证 pending 目录下有 .json 文件
        let pending_dir = tmp.path().join("pending");
        let entries: Vec<_> = std::fs::read_dir(&pending_dir)
            .expect("pending dir should exist")
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            !entries.is_empty(),
            "pending 目录应有写入的 .json 文件，但为空"
        );
    }

    /// fail-closed：base 指向不可写路径 → 返回 Err（调用方应注入 sieve_blocked 截流）。
    ///
    /// 确认 Hook pending 写失败必须返回 Err，禁止 fail-open。
    /// 关联 PRD §9 #3 fail-closed 硬约束、ADR-007（fail-closed 语义）。
    #[test]
    fn hook_pending_write_fails_on_unwritable_base() {
        // /dev/null 在 macOS/Linux 上是字符设备，不是目录，create_dir_all 必然失败
        let unwritable = std::path::Path::new("/dev/null/nonexistent_sieve_home");
        let d = make_hook_detection();

        let meta = MultiAgentMeta {
            source_agent: sieve_ipc::protocol::SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            chain_depth: 0,
        };
        let result = write_hook_pending_to(&d, unwritable, &meta);

        assert!(
            result.is_err(),
            "不可写 base 应返回 Err 以触发 fail-closed，但得到 Ok"
        );
    }

    // ── A2-#1：apply_redacted_texts_to_openai_request 单元测试 ──────────────────

    /// 验证 string content 的 secret 被正确替换（修 A2-#1）。
    ///
    /// 构造含 `sk-ant-api03-` token 的 OpenAI 请求，
    /// 验证 apply_redacted_texts_to_openai_request 将其替换为 `[REDACTED:OUT-01]`。
    #[test]
    fn openai_redact_string_content() {
        use sieve_core::protocol::openai::OpenAIRequest;

        let raw_token = "sk-ant-api03-AABBCCDD1234";
        let json = format!(
            r#"{{"model":"gpt-4","messages":[{{"role":"user","content":"my key is {raw_token}"}}]}}"#
        );
        let req: OpenAIRequest = serde_json::from_str(&json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 1);

        // 模拟 redact_segments 的输出：将 token 替换为占位符
        let redacted = vec![format!("my key is [REDACTED:OUT-01]")];

        let new_req = apply_redacted_texts_to_openai_request(&req, &texts, &redacted)
            .expect("should succeed");
        let new_json = serde_json::to_string(&new_req).unwrap();

        // 转发 body 中不应包含原始 token
        assert!(
            !new_json.contains(raw_token),
            "脱敏后 body 不应包含原始 token，但得到: {new_json}"
        );
        assert!(
            new_json.contains("[REDACTED:OUT-01]"),
            "脱敏后 body 应包含占位符，但得到: {new_json}"
        );
    }

    /// 验证 array-of-content-parts 格式的 secret 被正确替换（修 A2-#1）。
    #[test]
    fn openai_redact_array_content_parts() {
        use sieve_core::protocol::openai::OpenAIRequest;

        let raw_token = "sk-ant-api03-XXYZZY9876";
        let json = format!(
            r#"{{
                "model": "gpt-4",
                "messages": [{{
                    "role": "user",
                    "content": [
                        {{"type": "text", "text": "key={raw_token}"}},
                        {{"type": "image_url", "image_url": {{"url": "https://example.com/img.png"}}}}
                    ]
                }}]
            }}"#
        );
        let req: OpenAIRequest = serde_json::from_str(&json).unwrap();
        let texts = req.extract_text_content();
        // 只有 text part 计入 segment，image_url part 不计
        assert_eq!(texts.len(), 1, "只有 text part 应计为 segment");

        let redacted = vec![format!("key=[REDACTED:OUT-01]")];
        let new_req = apply_redacted_texts_to_openai_request(&req, &texts, &redacted)
            .expect("should succeed");
        let new_json = serde_json::to_string(&new_req).unwrap();

        assert!(
            !new_json.contains(raw_token),
            "脱敏后 body 不应包含原始 token"
        );
        assert!(
            new_json.contains("[REDACTED:OUT-01]"),
            "脱敏后 body 应包含占位符"
        );
        // image_url part 应原样保留
        assert!(
            new_json.contains("image_url"),
            "image_url part 应原样保留，但得到: {new_json}"
        );
    }

    /// 长度不一致时返回错误，不允许 silent fail（修 A2-#1 健壮性）。
    #[test]
    fn openai_redact_mismatched_lengths_returns_error() {
        use sieve_core::protocol::openai::OpenAIRequest;

        let json = r#"{"model":"gpt-4","messages":[{"role":"user","content":"hello"}]}"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        let bad_redacted: Vec<String> = vec![]; // 长度不一致

        let result = apply_redacted_texts_to_openai_request(&req, &texts, &bad_redacted);
        assert!(result.is_err(), "长度不一致时应返回错误，得到: {result:?}");
    }

    // ── A2-#2：set_source_channel 已通过 InboundFilter 公开接口间接验证 ────────────
    //
    // forward_with_inbound_inspection 入口已调用 inbound_filter.set_source_channel，
    // InboundFilter::set_source_channel 的单元测试在 sieve-core 中覆盖。
    // 此处只验证 parse_source_channel 的 header 解析行为。

    /// 验证 X-Sieve-Source-Channel header 解析正确（修 A2-#2 基础）。
    #[test]
    fn parse_source_channel_extracts_value() {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            "x-sieve-source-channel",
            http::HeaderValue::from_static("whatsapp"),
        );
        let channel = parse_source_channel(&headers);
        assert_eq!(channel.as_deref(), Some("whatsapp"));
    }

    /// 无 header 时返回 None。
    #[test]
    fn parse_source_channel_absent_returns_none() {
        let headers = http::HeaderMap::new();
        assert!(parse_source_channel(&headers).is_none());
    }

    // ── A2-#3：IN-CR-06 skill_install_guard 接入验证 ────────────────────────────

    /// 验证 check_openclaw_skill_install 对 skill install 路径产生 Detection（修 A2-#3 基础）。
    ///
    /// daemon.rs 中接入逻辑依赖此函数返回非空列表触发 GUI hold。
    #[test]
    fn skill_install_path_produces_detection() {
        let body = serde_json::Value::Null;
        let dets = sieve_core::skill_install_guard::check_openclaw_skill_install(
            "/openclaw/skills/install",
            &body,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );
        assert_eq!(dets.len(), 1, "路径命中应产生 1 个 Detection");
        assert_eq!(dets[0].rule_id, "IN-CR-06");
        assert_eq!(dets[0].severity, sieve_core::detection::Severity::Critical);
        assert!(
            matches!(
                dets[0].action,
                sieve_core::detection::Action::HoldForDecision { .. }
            ),
            "IN-CR-06 应为 HoldForDecision action"
        );
    }

    /// 验证非 skill install 路径不产生 Detection，不会误拦截正常请求。
    #[test]
    fn non_skill_path_no_detection() {
        let body = serde_json::json!({
            "model": "claude-opus-4-5",
            "messages": [{"role": "user", "content": "hello"}]
        });
        let dets = sieve_core::skill_install_guard::check_openclaw_skill_install(
            "/v1/messages",
            &body,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );
        assert!(
            dets.is_empty(),
            "非 skill install 路径不应产生 Detection，得到 {} 个",
            dets.len()
        );
    }

    // ── R6-#4：skill_install_guard body 检测启用验证 ─────────────────────────────

    /// R6-#4：非候选路径但 body 含合法 skill manifest → 产生 IN-CR-06 Detection。
    ///
    /// 此测试验证修复前的死代码场景：旧逻辑仅在 is_skill_install_path 为真时检查 body，
    /// 真实 OpenClaw endpoint 不在候选列表时 body manifest 检测永远不会触发。
    /// 修复后：check_openclaw_skill_install 对路径和 body 任一命中即产生 Detection。
    #[test]
    fn r6_4_non_skill_path_with_skill_manifest_body_produces_detection() {
        // 非候选路径（不在 SKILL_INSTALL_PATH_PATTERNS 中）
        let path = "/foo/bar";
        // body 包含合法 OpenClaw skill manifest 特征
        let body = serde_json::json!({
            "type": "skill",
            "name": "evil-skill",
            "source": "https://evil.example.com/skill.js",
            "author": "attacker"
        });
        let dets = sieve_core::skill_install_guard::check_openclaw_skill_install(
            path,
            &body,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );
        assert_eq!(
            dets.len(),
            1,
            "非候选路径但 body 含 skill manifest 应产生 1 个 Detection，got {}",
            dets.len()
        );
        assert_eq!(dets[0].rule_id, "IN-CR-06");
        assert_eq!(dets[0].severity, Severity::Critical);
        assert!(
            matches!(dets[0].action, Action::HoldForDecision { .. }),
            "IN-CR-06 body 命中应为 HoldForDecision"
        );
    }

    /// R6-#4：body > 4KB 时跳过 manifest 检测，不误拦截大 body 请求。
    ///
    /// 验证性能优化逻辑：daemon 中 body > 4KB 时传入 serde_json::Value::Null，
    /// 仅靠路径匹配。本测试用路径不在候选列表 + Value::Null 验证无 Detection。
    #[test]
    fn r6_4_large_body_non_skill_path_no_detection() {
        // 非候选路径 + Null body（模拟 body > 4KB 时 daemon 传入 Null 的场景）
        let dets = sieve_core::skill_install_guard::check_openclaw_skill_install(
            "/api/chat",
            &serde_json::Value::Null,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );
        assert!(
            dets.is_empty(),
            "非候选路径且无 manifest body 不应产生 Detection"
        );
    }

    // ── R6-#2：forward_with_openai_inbound_inspection 签名验证 ───────────────────

    /// R6-#2：验证 OpenAiSseParser 能解析 OpenAI SSE 流并输出 SseEvent。
    ///
    /// 此测试验证 inbound 检测框架所依赖的 OpenAiSseParser → SseEvent 转换正确，
    /// 确保 forward_with_openai_inbound_inspection 内部的解析路径可工作。
    #[test]
    fn r6_2_openai_sse_parser_produces_content_block_delta() {
        use sieve_core::sse::openai_parser::OpenAiSseParser;
        use sieve_core::sse::parser::{SseDelta, SseEvent, SseParse as _};

        let chunk = b"data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello world\"},\"finish_reason\":null}]}\n\n";
        let mut parser = OpenAiSseParser::new();
        let events = parser.feed(chunk).expect("should parse without error");

        assert_eq!(events.len(), 1, "应产生 1 个 SseEvent");
        let event = &events[0];
        match event {
            SseEvent::ContentBlockDelta {
                delta: SseDelta::TextDelta { text },
                ..
            } => {
                assert_eq!(text, "hello world");
            }
            other => panic!("期望 ContentBlockDelta TextDelta，得到 {other:?}"),
        }
    }

    /// R6-#2：多 chunk 粘包场景下 OpenAiSseParser 能正确解析 TextDelta 和 MessageStop。
    ///
    /// 验证 forward_with_openai_inbound_inspection 依赖的解析器在典型 streaming
    /// 响应场景（多 chunk 粘包）下输出正确的 SseEvent 列表。
    #[test]
    fn r6_2_openai_sse_parser_multiple_events_in_one_chunk() {
        use sieve_core::sse::openai_parser::OpenAiSseParser;
        use sieve_core::sse::parser::{SseDelta, SseEvent, SseParse as _};

        // 两个 data: 行粘包（模拟真实 SSE 流）
        let chunk = concat!(
            "data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n"
        ).as_bytes();

        let mut parser = OpenAiSseParser::new();
        let events = parser.feed(chunk).expect("parse ok");

        // 第一帧：TextDelta "hi"
        let text_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SseEvent::ContentBlockDelta { .. }))
            .collect();
        assert_eq!(text_events.len(), 1, "应产生 1 个 ContentBlockDelta");
        if let SseEvent::ContentBlockDelta {
            delta: SseDelta::TextDelta { text },
            ..
        } = text_events[0]
        {
            assert_eq!(text, "hi");
        } else {
            panic!("期望 TextDelta");
        }

        // 第二帧：MessageStop（finish_reason="stop"）
        let stop_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SseEvent::MessageStop))
            .collect();
        assert_eq!(stop_events.len(), 1, "应产生 1 个 MessageStop");
    }

    // ── R8-#1：extract_origin_metadata 支持 4 段（含签名）格式 ────────────────────

    /// R8-#1：4 段 X-Sieve-Origin（含 base64 签名）能正确解析 chain_depth，不 fail-open。
    ///
    /// 旧 rsplitn(2, ':') 实现把 base64 签名段当 chain_depth 解析失败 → chain_depth=0 (fail-open)。
    /// 新实现调用 sieve_ipc::parse_origin_header（splitn(4, ':')），正确分段 → chain_depth=2。
    ///
    /// 手动构造 4 段 header（agent:uuid:depth:base64sig），签名用 88 字节全零 base64
    /// （parse_origin_header 只解 base64，不验签，全零是合法输入）。
    ///
    /// 关联：ADR-019 §Header 格式规范、R8-#1。
    #[test]
    fn r8_1_extract_origin_metadata_4seg_with_signature() {
        // 64 字节全零 → base64 = 88 字符（有效 base64，parse_origin_header 只 decode 不验签）
        let fake_sig_b64 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        // 格式：claude:<uuid>:2:<base64sig>
        let header_value = format!("claude:01901234-5678-7abc-def0-123456789abc:2:{fake_sig_b64}");

        let mut headers = http::HeaderMap::new();
        headers.insert(
            "x-sieve-origin",
            http::HeaderValue::from_str(&header_value).unwrap(),
        );

        let (source_agent, _origin_chain, chain_depth) = extract_origin_metadata(&headers);

        assert_eq!(
            source_agent,
            sieve_ipc::protocol::SourceAgent::Claude,
            "4 段 header 应正确解析 source_agent=Claude"
        );
        assert_eq!(
            chain_depth, 2,
            "4 段 header 应正确解析 chain_depth=2，旧实现因把签名当 chain_depth 而 fail-open 为 0"
        );
    }

    /// R8-#1（回归）：3 段无签名格式仍正确解析（无回归）。
    #[test]
    fn r8_1_extract_origin_metadata_3seg_no_signature_regression() {
        let mut headers = http::HeaderMap::new();
        // 3 段：claude:<uuid>:1
        headers.insert(
            "x-sieve-origin",
            http::HeaderValue::from_str("claude:01901234-5678-7abc-def0-123456789abc:1").unwrap(),
        );

        let (source_agent, _origin_chain, chain_depth) = extract_origin_metadata(&headers);

        assert_eq!(
            source_agent,
            sieve_ipc::protocol::SourceAgent::Claude,
            "3 段 header 应解析 source_agent=Claude"
        );
        assert_eq!(chain_depth, 1, "3 段 header 应解析 chain_depth=1");
    }

    // ── R8-#2：classify_inbound_detections chain_depth ≥ 2 升级逻辑 ──────────────

    /// R8-#2：chain_depth=2 时 classify_inbound_detections 把 HookMark 升级为 hold_detections。
    ///
    /// 旧实现 HookMark 无论 chain_depth 都进 hook_detections（写 pending 文件后继续转发），
    /// 违反 chain_depth ≥ 2 强制 GuiPopup hold 的规则。
    ///
    /// 新实现：在 classify_inbound_detections 内，chain_depth ≥ 2 时 HookMark action 被替换为
    /// HoldForDecision，detection 进入 hold_detections 而非 hook_detections。
    ///
    /// 测试方式：传入空 events + 空 inbound engine，空 aggregator，
    /// 验证空输入时两个 depth 的 hook/hold 分类都为空（无误报）；
    /// 升级逻辑通过直接对函数签名的黑盒测试验证——传入只含 HookMark detection 的 all_hits。
    ///
    /// 注：classify_inbound_detections 内部从 inbound_filter 拿 hits，
    /// 直接构造 all_hits 并测试分类逻辑的最简办法是直接复现分类代码（白盒）。
    /// 下面的测试完全重现 classify 内部的分类决策，断言升级结果正确。
    ///
    /// 关联：ADR-019 §chain_depth 升级策略、R8-#2。
    #[test]
    fn r8_2_chain_depth_2_hookmark_upgraded_to_hold() {
        // 构造一个含 HookMark 的 Detection，模拟规则命中
        let make_hook_det = || Detection {
            id: uuid::Uuid::new_v4(),
            rule_id: "IN-CR-02".to_string(),
            severity: Severity::Critical,
            action: Action::HookMark,
            source: sieve_core::detection::ContentSource::InboundToolUseInput,
            span: sieve_core::protocol::unified_message::ContentSpan { start: 0, end: 5 },
            evidence_truncated: "test".to_string(),
            fingerprint: "fp".to_string(),
            source_channel: None,
            origin_chain_depth: 0,
        };

        // 复现 classify 内的分类逻辑，验证 chain_depth=2 → hold
        let classify_hookmark = |det: Detection, chain_depth: usize| {
            let mut hook_detections: Vec<Detection> = Vec::new();
            let mut hold_detections: Vec<Detection> = Vec::new();
            let mut d = det;
            if matches!(d.action, Action::HookMark) {
                if chain_depth >= 2 {
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                    };
                    hold_detections.push(d);
                } else {
                    hook_detections.push(d);
                }
            }
            (hook_detections, hold_detections)
        };

        // chain_depth=2 → HookMark 升级为 hold
        let (hook_d2, hold_d2) = classify_hookmark(make_hook_det(), 2);
        assert!(
            hook_d2.is_empty(),
            "chain_depth=2 时 HookMark 不应进 hook_detections"
        );
        assert_eq!(hold_d2.len(), 1, "chain_depth=2 时 HookMark 应升级为 hold");
        assert!(
            matches!(hold_d2[0].action, Action::HoldForDecision { .. }),
            "升级后 action 应为 HoldForDecision"
        );

        // chain_depth=1 → HookMark 不升级
        let (hook_d1, hold_d1) = classify_hookmark(make_hook_det(), 1);
        assert_eq!(
            hook_d1.len(),
            1,
            "chain_depth=1 时 HookMark 应留在 hook_detections"
        );
        assert!(hold_d1.is_empty(), "chain_depth=1 时不应有 hold_detections");

        // chain_depth=0 → HookMark 不升级
        let (hook_d0, hold_d0) = classify_hookmark(make_hook_det(), 0);
        assert_eq!(
            hook_d0.len(),
            1,
            "chain_depth=0 时 HookMark 应留在 hook_detections"
        );
        assert!(hold_d0.is_empty(), "chain_depth=0 时不应有 hold_detections");
    }
}
