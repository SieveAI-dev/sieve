//! 控制面 IPC handler。
//!
//! 接收 [`sieve_ipc::ControlPlaneRequest`]，处理后通过 oneshot 回执给 IPC server，
//! 并按需写 audit + 广播 daemon → GUI 通知（preset_changed / paused_changed）。
//!
//! 设计要点：
//! - **fail-soft 审计**：所有 audit 写入 spawn task 异步执行，hot path 不阻塞。
//! - **Critical 锁防线二**：`set_preset_overrides` 路径在写入前对每条 rule_id 调用
//!   [`sieve_rules::critical_lock::is_fail_closed`]，命中则记入 rejected + 审计 `kind=critical_lock_blocked`。
//! - **fail-closed paused**：暂停状态影响范围**永远不包含 Critical 锁规则的 disposition**。
//!   `applies_to` 列表在 [`paused_applies_to`] 中固定声明。
//! - **paused 的 hot path 消费**：handle_set_paused 双写到 `RuntimeState.paused_until`（GUI 快照）
//!   与 `IpcServer.paused_until`（hot path 共享）。daemon::gated_request_decision 在每次
//!   `request_decision` 前调 `IpcServer::is_paused()`，命中且无 critical 检测时跳过弹窗、
//!   按 `default_on_timeout` 自动决策、写 `AuditEvent::AutoDecidedPaused`。
//!
//! 关联：防线二 / SPEC-002 §9。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use arc_swap::ArcSwap;
use chrono::{DateTime, Utc};
use sieve_core::pipeline::inbound::InboundEngine;
use sieve_core::tool_use_aggregator::CompletedToolCall;
use sieve_core::ContentSource;
use sieve_ipc::{
    AuditDbSnapshot, BroadcastPlan, ControlError, ControlPlaneRequest, EvaluateMatch,
    EvaluateResult, GraylistEntrySummary, GraylistSnapshot, HealthResult, IpcServer, IpcSnapshot,
    JudgeToolCallRequest, JudgeToolCallResult, ListGraylistResult, ListRulesResult, ListenSnapshot,
    PausedChangedNotify, PresetChangedNotify, PresetOverride, PresetSnapshot, PurgeHistoryResult,
    RejectedOverride, ReloadConfigResult, RemoveGraylistResult, RuleSummary, RulesSnapshot,
    SetPausedResult, SetPresetOverridesResult, SetPresetResult,
};
use uuid::Uuid;

use crate::audit::{AuditEvent, AuditStore};
use crate::cli::NoClientPolicy;

/// daemon 运行时可变状态（control plane 与 hot path 共享）。
///
/// 所有字段用 [`ArcSwap`] / 原子原语保护，hot path 走 lock-free read。
pub struct RuntimeState {
    /// 暂停截止时间。`None` = 未暂停。
    pub paused_until: ArcSwap<Option<DateTime<Utc>>>,
    /// 当前生效的 preset（mode + custom overrides）。
    pub preset: ArcSwap<RuntimePreset>,
    /// daemon 启动时间（不可变）。
    pub started_at: DateTime<Utc>,
    /// 监听端口与地址（不可变）。
    ///
    /// daemon 可同时绑定多个 listener。本字段保留为 `listeners[0]`
    /// 别名向后兼容旧 GUI 客户端（仅读 health.listen 单值）；新代码应读 `listeners` 数组。
    pub listen: ListenSnapshot,
    /// 多 listener 快照数组（Stage F）。
    /// daemon::run 启动时按 `cfg.resolved_upstreams()` 顺序填充，不可变。
    pub listeners: Vec<sieve_ipc::ListenerSnapshot>,
    /// daemon 版本号（不可变）。
    pub daemon_version: String,
    /// IPC 协议版本号（不可变）。
    pub protocol_version: String,
    /// audit.db 路径（不可变）。
    pub audit_db_path: PathBuf,
    /// 灰名单目录（不可变）。
    pub decisions_dir: PathBuf,
    /// 用户规则文件路径（不可变；reload_config 共享同一路径）。
    pub user_rules_path: Option<PathBuf>,
    /// 系统规则数（reload 后刷新）。
    pub system_rules_count: ArcSwap<usize>,
    /// 用户规则数（reload 后刷新）。
    pub user_rules_count: ArcSwap<usize>,
    /// 上次 reload 时间。
    pub last_reload: ArcSwap<Option<DateTime<Utc>>>,
    /// `sieve.purge_history` 并发防护标志（SPEC-005 §11B）。
    ///
    /// CAS 操作：true 表示 purge 正在进行，此时第二个请求立即返回 `-32007 purge_in_progress`。
    pub purge_in_progress: AtomicBool,
}

/// 运行时 preset 快照（mode + 逐规则覆盖）。
#[derive(Debug, Clone)]
pub struct RuntimePreset {
    pub mode: String,
    pub overrides: HashMap<String, PresetOverride>,
}

impl RuntimePreset {
    /// 默认 preset（mode=standard + 空 overrides）。
    /// 主要供 daemon::run 初始化失败兜底 + 单元测试构造。
    /// SPEC-005 §5.6：v2 preset mode 用 `standard`（v1 旧值 `default` 已重命名）。
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn default_mode() -> Self {
        Self {
            mode: "standard".to_owned(),
            overrides: HashMap::new(),
        }
    }
}

impl RuntimeState {
    /// 当前暂停截止时间（health 快照用）。
    ///
    /// hot path 消费走 [`sieve_ipc::IpcServer::is_paused`] —— RuntimeState
    /// 持有的本字段是 GUI snapshot 用的镜像，由 [`handle_set_paused`] 双写保持一致。
    /// 关联：SPEC-002 §9.1 / daemon::gated_request_decision。
    pub fn paused_now(&self) -> Option<DateTime<Utc>> {
        let snapshot = self.paused_until.load();
        match snapshot.as_ref().as_ref() {
            Some(until) => {
                if *until > Utc::now() {
                    Some(*until)
                } else {
                    // 自动恢复：写回 None（避免后续重复检查）。
                    self.paused_until.store(Arc::new(None));
                    None
                }
            }
            None => None,
        }
    }
}

/// 暂停期间受影响的 disposition 集合（**永不包含 Critical 锁路径**）。
///
/// 关联：SPEC-002 §9.1。
fn paused_applies_to() -> Vec<String> {
    vec![
        "AutoRedact".to_owned(),
        "StatusBar".to_owned(),
        "Ask:non_critical".to_owned(),
    ]
}

/// 启动控制面 handler task。
///
/// 调用方：[`crate::daemon::run`] 在 IpcServer 启动后立即调用。task 内部循环
/// 消费 `control_rx()`，对每条请求 dispatch 到对应 handler。
///
/// 参数：
/// - `ipc`：IpcServer，用于广播通知 + 获取 inflight 状态
/// - `audit_store`：审计写入
/// - `state`：运行时状态（paused / preset / 启动元信息）
/// - `outbound_layered`/`inbound_layered`：evaluate 沙箱用
#[allow(clippy::too_many_arguments)]
pub fn spawn_control_plane_handler(
    ipc: Arc<IpcServer>,
    audit_store: Arc<AuditStore>,
    state: Arc<RuntimeState>,
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
    // judge_tool_call 用：与真实入站路径同款检测引擎 + 无 client 处置策略。
    inbound_engine: Arc<dyn InboundEngine>,
    no_client_policy: NoClientPolicy,
) {
    let ipc_for_task = Arc::clone(&ipc);
    tokio::spawn(async move {
        // 取出 control_rx（一次性）
        let mut control_rx = match ipc_for_task.control_rx().await {
            Some(rx) => rx,
            None => {
                tracing::warn!("control_rx already taken; control-plane handler exits");
                return;
            }
        };
        tracing::info!("control-plane handler started");
        while let Some(req) = control_rx.recv().await {
            dispatch_request(
                req,
                &ipc_for_task,
                &audit_store,
                &state,
                &outbound_layered,
                &inbound_layered,
                &inbound_engine,
                no_client_policy,
            )
            .await;
        }
        tracing::info!("control-plane handler exiting (channel closed)");
    });
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_request(
    req: ControlPlaneRequest,
    ipc: &Arc<IpcServer>,
    audit: &Arc<AuditStore>,
    state: &Arc<RuntimeState>,
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
    inbound_engine: &Arc<dyn InboundEngine>,
    no_client_policy: NoClientPolicy,
) {
    match req {
        ControlPlaneRequest::SetPaused {
            params,
            origin_request_id,
            reply,
        } => {
            // SPEC-005 §10.0.1：IPC server 层收到 (result, BroadcastPlan) 后先 fan-out 再写 result。
            let result = handle_set_paused(params, origin_request_id, audit, state).await;
            // 同步给 IpcServer hot path（set_paused_until 必须在 reply.send 之前调用）。
            if let Ok((ref r, _)) = result {
                ipc.set_paused_until(r.paused_until);
            }
            let _ = reply.send(result);
        }
        ControlPlaneRequest::SetPreset {
            params,
            origin_request_id,
            reply,
        } => {
            // SPEC-005 §10.0.1：同上。
            let result = handle_set_preset(params, origin_request_id, ipc, audit, state).await;
            let _ = reply.send(result);
        }
        ControlPlaneRequest::SetPresetOverrides {
            params,
            origin_request_id,
            reply,
        } => {
            // SPEC-005 §10.0.1：同上。
            let result =
                handle_set_preset_overrides(params, origin_request_id, ipc, audit, state).await;
            let _ = reply.send(result);
        }
        ControlPlaneRequest::ReloadConfig { params: _, reply } => {
            let result =
                handle_reload_config(ipc, audit, state, outbound_layered, inbound_layered).await;
            let _ = reply.send(result);
        }
        ControlPlaneRequest::Health { params: _, reply } => {
            let result = handle_health(ipc, state).await;
            let _ = reply.send(result);
        }
        ControlPlaneRequest::Evaluate { params, reply } => {
            let result = handle_evaluate(params, outbound_layered, inbound_layered).await;
            let _ = reply.send(result);
        }
        ControlPlaneRequest::ListGraylist { params, reply } => {
            let result = handle_list_graylist(params, state).await;
            let _ = reply.send(result);
        }
        ControlPlaneRequest::RemoveGraylist { params, reply } => {
            let result = handle_remove_graylist(params, audit, state).await;
            let _ = reply.send(result);
        }
        // SPEC-005 §11A：只读，不走串行化队列（§10.0.1 只读请求并发例外）。
        ControlPlaneRequest::ListRules { reply } => {
            let result = handle_list_rules(outbound_layered, inbound_layered).await;
            let _ = reply.send(result);
        }
        // SPEC-005 §11B：写操作，通过互斥 flag 防并发。
        ControlPlaneRequest::PurgeHistory { params, reply } => {
            let result = handle_purge_history(params, audit, state).await;
            let _ = reply.send(result);
        }
        // SPEC-005 §11C：sieve.judge_tool_call（Since v2.x）。
        // ⚠️ 命中 Critical 时 handle_judge_tool_call 会阻塞等 GUI 弹窗（30–120s）。
        // 控制面是单 task 串行消费，**必须 spawn-per-judge**，否则一个待确认的工具调用
        // 会卡死 health / set_paused 等所有其他控制面请求。reply 在 spawn 内回执。
        ControlPlaneRequest::JudgeToolCall { params, reply } => {
            let ipc = Arc::clone(ipc);
            let audit = Arc::clone(audit);
            let engine = Arc::clone(inbound_engine);
            tokio::spawn(async move {
                let result =
                    handle_judge_tool_call(params, &ipc, &audit, &engine, no_client_policy).await;
                let _ = reply.send(result);
            });
        }
    }
}

// ─────────────────────────── handlers ───────────────────────────────

async fn handle_set_paused(
    params: sieve_ipc::SetPausedRequest,
    origin_request_id: Option<Uuid>,
    audit: &Arc<AuditStore>,
    state: &Arc<RuntimeState>,
) -> Result<(SetPausedResult, Option<BroadcastPlan>), ControlError> {
    if params.minutes > 60 {
        return Err(ControlError::invalid_params(
            "minutes must be in [0, 60]; daemon refuses unbounded pause",
        ));
    }

    let (paused, until) = if params.minutes == 0 {
        state.paused_until.store(Arc::new(None));
        (false, None)
    } else {
        let until = Utc::now() + chrono::Duration::minutes(i64::from(params.minutes));
        state.paused_until.store(Arc::new(Some(until)));
        (true, Some(until))
    };

    // audit
    let event = AuditEvent::PausedSet {
        until: until.map(|t| t.to_rfc3339()),
        source: "gui".to_owned(),
    };
    spawn_audit(audit, event);

    // SPEC-005 §10.0.1：广播由 IPC server 层在写 result 之前执行（BroadcastPlan 回传）。
    // SPEC-005 §10.0.2：origin_request_id 透传，GUI 可据此识别本地回声。
    let broadcast = BroadcastPlan::PausedChanged(PausedChangedNotify {
        paused,
        paused_until: until,
        reason: "user_request".to_owned(),
        applies_to: paused_applies_to(),
        // SPEC-005 §10.2 required；client 触发的 set_paused，与上方 PausedSet 审计 source 一致。
        source: "gui".to_owned(),
        origin_request_id,
    });

    Ok((
        SetPausedResult {
            paused,
            paused_until: until,
            applies_to: paused_applies_to(),
        },
        Some(broadcast),
    ))
}

async fn handle_set_preset(
    params: sieve_ipc::SetPresetRequest,
    origin_request_id: Option<Uuid>,
    ipc: &Arc<IpcServer>,
    audit: &Arc<AuditStore>,
    state: &Arc<RuntimeState>,
) -> Result<(SetPresetResult, Option<BroadcastPlan>), ControlError> {
    let mode = params.mode.to_lowercase();
    // SPEC-005 §5.6：v1 旧值 `default` 在 v2 重命名为 `standard`；兼容旧 client 发来的 `default`。
    let mode = if mode == "default" {
        "standard".to_owned()
    } else {
        mode
    };
    if !matches!(mode.as_str(), "strict" | "standard" | "relaxed" | "custom") {
        return Err(ControlError::invalid_params(format!(
            "unknown preset mode: {}",
            params.mode
        )));
    }

    let prev = state.preset.load_full();
    let new_preset = RuntimePreset {
        mode: mode.clone(),
        // 切换 mode 时清空 custom overrides；切回 custom 由 set_preset_overrides 重建。
        overrides: if mode == "custom" {
            prev.overrides.clone()
        } else {
            HashMap::new()
        },
    };
    state.preset.store(Arc::new(new_preset.clone()));

    let now = Utc::now();
    let event = AuditEvent::PresetChanged {
        from_mode: prev.mode.clone(),
        to_mode: new_preset.mode.clone(),
        source: "gui".to_owned(),
    };
    spawn_audit(audit, event);

    // SPEC-005 §10.0.1：广播由 IPC server 层在写 result 之前执行（BroadcastPlan 回传）。
    // SPEC-005 §10.0.2：origin_request_id 透传，GUI 可据此识别本地回声。
    let broadcast = BroadcastPlan::PresetChanged(PresetChangedNotify {
        mode: new_preset.mode.clone(),
        overrides: new_preset.overrides.clone(),
        changed_at: now,
        source: "gui".to_owned(),
        origin_request_id,
    });

    // set_preset 不需要同步 ipc（只有 paused_until 需要），但 hello_builder.preset 快照是启动时刻的，
    // 实时 preset 由 PresetChangedNotify fan-out 驱动。
    let _ = ipc; // 保留参数以防未来需要（如 reload_config 共用此函数）

    Ok((SetPresetResult { applied_at: now }, Some(broadcast)))
}

async fn handle_set_preset_overrides(
    params: sieve_ipc::SetPresetOverridesRequest,
    origin_request_id: Option<Uuid>,
    ipc: &Arc<IpcServer>,
    audit: &Arc<AuditStore>,
    state: &Arc<RuntimeState>,
) -> Result<(SetPresetOverridesResult, Option<BroadcastPlan>), ControlError> {
    if params.overrides.is_empty() {
        return Err(ControlError::invalid_params(
            "overrides map cannot be empty",
        ));
    }

    let mut applied: Vec<String> = Vec::new();
    let mut rejected: Vec<RejectedOverride> = Vec::new();
    let mut new_overrides: HashMap<String, PresetOverride> = state.preset.load().overrides.clone();

    for (rule_id, ov) in params.overrides {
        // 防线二：critical_lock 强校验
        if sieve_rules::critical_lock::is_fail_closed(&rule_id) {
            rejected.push(RejectedOverride {
                rule_id: rule_id.clone(),
                reason: "critical_lock".to_owned(),
            });
            // 审计 critical_lock_blocked
            spawn_audit(
                audit,
                AuditEvent::CriticalLockBlocked {
                    rule_id: rule_id.clone(),
                    source: "ipc_set_overrides".to_owned(),
                },
            );
            spawn_audit(
                audit,
                AuditEvent::PresetOverrideRejected {
                    rule_id,
                    reason: "critical_lock".to_owned(),
                    source: "gui".to_owned(),
                },
            );
            continue;
        }

        // 字段值合法性
        if !matches!(ov.default_on_timeout.as_str(), "block" | "allow" | "redact") {
            rejected.push(RejectedOverride {
                rule_id: rule_id.clone(),
                reason: "invalid_value".to_owned(),
            });
            spawn_audit(
                audit,
                AuditEvent::PresetOverrideRejected {
                    rule_id,
                    reason: "invalid_value".to_owned(),
                    source: "gui".to_owned(),
                },
            );
            continue;
        }
        if !(5..=600).contains(&ov.timeout_seconds) {
            rejected.push(RejectedOverride {
                rule_id: rule_id.clone(),
                reason: "invalid_value".to_owned(),
            });
            spawn_audit(
                audit,
                AuditEvent::PresetOverrideRejected {
                    rule_id,
                    reason: "invalid_value".to_owned(),
                    source: "gui".to_owned(),
                },
            );
            continue;
        }

        spawn_audit(
            audit,
            AuditEvent::PresetOverrideApplied {
                rule_id: rule_id.clone(),
                timeout_seconds: ov.timeout_seconds,
                default_on_timeout: ov.default_on_timeout.clone(),
                source: "gui".to_owned(),
            },
        );
        new_overrides.insert(rule_id.clone(), ov);
        applied.push(rule_id);
    }

    // 自动切到 custom mode 并存储新 overrides
    let prev = state.preset.load_full();
    let new_preset = RuntimePreset {
        mode: "custom".to_owned(),
        overrides: new_overrides.clone(),
    };
    state.preset.store(Arc::new(new_preset.clone()));

    if prev.mode != "custom" {
        spawn_audit(
            audit,
            AuditEvent::PresetChanged {
                from_mode: prev.mode.clone(),
                to_mode: "custom".to_owned(),
                source: "gui".to_owned(),
            },
        );
    }

    // SPEC-005 §10.0.1：广播由 IPC server 层在写 result 之前执行（BroadcastPlan 回传）。
    // SPEC-005 §10.0.2：origin_request_id 透传，GUI 可据此识别本地回声。
    let broadcast = BroadcastPlan::PresetChanged(PresetChangedNotify {
        mode: new_preset.mode,
        overrides: new_overrides,
        changed_at: Utc::now(),
        source: "gui".to_owned(),
        origin_request_id,
    });

    let _ = ipc; // 保留参数（ipc 在此函数不再直接 broadcast）

    Ok((
        SetPresetOverridesResult { applied, rejected },
        Some(broadcast),
    ))
}

async fn handle_reload_config(
    ipc: &Arc<IpcServer>,
    audit: &Arc<AuditStore>,
    state: &Arc<RuntimeState>,
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
) -> Result<ReloadConfigResult, ControlError> {
    // 直接调 daemon::perform_user_rules_reload（与 IPC reload listener 共用同一逻辑），
    // 同步拿到 user_rules_errors 写回 GUI。
    let outcome = crate::daemon::perform_user_rules_reload(
        state.user_rules_path.as_deref(),
        outbound_layered,
        inbound_layered,
        ipc,
        audit,
        Some(Uuid::now_v7()),
    );

    let now = Utc::now();
    state.last_reload.store(Arc::new(Some(now)));
    state.user_rules_count.store(Arc::new(outcome.rule_count));

    spawn_audit(
        audit,
        AuditEvent::ConfigReloaded {
            user_rules_errors_count: outcome.user_rules_errors.len(),
            source: "gui".to_owned(),
        },
    );

    Ok(ReloadConfigResult {
        reloaded_at: now,
        system_rules_count: (**state.system_rules_count.load()) as u32,
        user_rules_count: outcome.rule_count as u32,
        user_rules_errors: outcome.user_rules_errors,
    })
}

async fn handle_health(
    ipc: &Arc<IpcServer>,
    state: &Arc<RuntimeState>,
) -> Result<HealthResult, ControlError> {
    let preset_snapshot = state.preset.load_full();
    let paused_until_val = state.paused_now();
    let paused_bool = paused_until_val.is_some();

    let audit_db_path = state.audit_db_path.clone();
    let audit_db_size = std::fs::metadata(&audit_db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let graylist_count = sieve_policy::graylist::list_entries(&state.decisions_dir)
        .map(|v| v.len())
        .unwrap_or(0);

    let now = Utc::now();
    let uptime = (now - state.started_at).num_seconds().max(0) as u64;

    Ok(HealthResult {
        daemon_version: state.daemon_version.clone(),
        protocol_version: state.protocol_version.clone(),
        started_at: state.started_at,
        uptime_seconds: uptime,
        preset: PresetSnapshot {
            mode: preset_snapshot.mode.clone(),
            overrides: preset_snapshot.overrides.clone(),
        },
        paused: paused_bool,
        paused_until: paused_until_val,
        listen: state.listen.clone(),
        listeners: state.listeners.clone(),
        audit_db: AuditDbSnapshot {
            path: audit_db_path.display().to_string(),
            size_bytes: audit_db_size,
            schema_version: 2,
            // events_total/today 需要查 audit.db；为避免锁竞争与多语句开销，
            // health 接口只返回近似值；GUI 可直接读 audit.db 拿精确计数。
            events_total: 0,
            events_today: 0,
        },
        rules: RulesSnapshot {
            system_count: (**state.system_rules_count.load()) as u32,
            user_count: (**state.user_rules_count.load()) as u32,
            last_reload: **state.last_reload.load(),
        },
        graylist: GraylistSnapshot {
            active_count: graylist_count as u32,
        },
        ipc: IpcSnapshot {
            connected_clients: ipc.connected_clients() as u32,
            total_decisions_inflight: ipc.inflight_decisions().await as u32,
        },
    })
}

async fn handle_evaluate(
    params: sieve_ipc::EvaluateRequest,
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
) -> Result<EvaluateResult, ControlError> {
    use sieve_ipc::{EvaluateContentKind, EvaluateDirection};
    use sieve_rules::engine::{ContentKind, Direction, MatchEngine, Protocol, ScanRequest};

    let direction = match params.direction {
        EvaluateDirection::Outbound => Direction::Outbound,
        EvaluateDirection::Inbound => Direction::Inbound,
    };
    let content_kind = match params.content_kind {
        EvaluateContentKind::RawText => ContentKind::RequestBody,
        EvaluateContentKind::ToolUseInput => ContentKind::ToolUseInput,
        EvaluateContentKind::ModelResponse => ContentKind::JsonResponseBody,
    };
    let req = ScanRequest {
        bytes: params.payload.as_bytes(),
        direction,
        protocol: Protocol::Anthropic,
        content_kind,
        tool_name: None,
        source_agent: Some(match &params.source_agent {
            sieve_ipc::SourceAgent::Claude => "claude",
            sieve_ipc::SourceAgent::OpenClaw => "openclaw",
            sieve_ipc::SourceAgent::Hermes => "hermes",
            sieve_ipc::SourceAgent::Unknown => "unknown",
        }),
        caller_exe: None,
    };

    let engine: &dyn MatchEngine = match direction {
        Direction::Outbound => &**outbound_layered,
        Direction::Inbound => &**inbound_layered,
    };

    let report = engine
        .scan_with_context(req)
        .map_err(|e| ControlError::internal(format!("sandbox scan failed: {e}")))?;

    let matches: Vec<EvaluateMatch> = report
        .hits
        .iter()
        .map(|hit| {
            let critical_locked = sieve_rules::critical_lock::is_fail_closed(&hit.rule_id);
            // critical_lock 规则只回类别摘要，不回原 payload 片段（数据保护）。
            let summary = if critical_locked {
                format!("(critical_locked) rule={}", hit.rule_id)
            } else {
                let snippet_end = hit.end.min(hit.start + 32).min(params.payload.len());
                let snippet_start = hit.start.min(snippet_end);
                let snippet = params.payload.get(snippet_start..snippet_end).unwrap_or("");
                format!(
                    "matched {} bytes at {}..{}: {:?}",
                    hit.end.saturating_sub(hit.start),
                    hit.start,
                    hit.end,
                    snippet
                )
            };
            let rule_kind = if hit.rule_id.starts_with("user:") {
                "user".to_owned()
            } else {
                "system".to_owned()
            };
            EvaluateMatch {
                rule_id: hit.rule_id.clone(),
                rule_kind,
                severity: if critical_locked {
                    "critical".to_owned()
                } else {
                    "unknown".to_owned()
                },
                disposition: sieve_ipc::Disposition::StatusBar,
                matched_pattern_summary: summary,
                fields_triggered: Vec::new(),
                would_decision: if critical_locked {
                    "deny".to_owned()
                } else {
                    "allow".to_owned()
                },
                would_recommendation: None,
            }
        })
        .collect();

    Ok(EvaluateResult {
        evaluated_at: Utc::now(),
        matches,
        no_match: Vec::new(),
    })
}

async fn handle_list_graylist(
    params: sieve_ipc::ListGraylistRequest,
    state: &Arc<RuntimeState>,
) -> Result<ListGraylistResult, ControlError> {
    let entries = sieve_policy::graylist::list_entries(&state.decisions_dir)
        .map_err(|e| ControlError::internal(format!("list graylist: {e}")))?;

    let limit = params.limit.unwrap_or(50) as usize;

    // 简单分页：cursor 为下一页起始 added_at 时间戳（毫秒）字符串。
    let start_idx = match params.cursor.as_deref() {
        None => 0,
        Some(cursor) => entries
            .iter()
            .position(|e| e.added_at.to_string() == cursor)
            .map(|i| i + 1)
            .unwrap_or(0),
    };

    let slice: Vec<&sieve_policy::graylist::GraylistEntry> =
        entries.iter().skip(start_idx).take(limit).collect();

    let next_cursor = if start_idx + slice.len() < entries.len() {
        slice.last().map(|e| e.added_at.to_string())
    } else {
        None
    };

    let summaries: Vec<GraylistEntrySummary> = slice
        .iter()
        .map(|e| GraylistEntrySummary {
            fingerprint: e.fingerprint.clone(),
            rule_id: e.rule_id.clone(),
            rule_kind: if e.rule_id.starts_with("user:") {
                "user".to_owned()
            } else {
                "system".to_owned()
            },
            added_at: e.added_at,
            added_by: e.added_by.clone(),
            context_hint: e.context_hint.clone(),
            match_count_since: e.match_count_since,
            expires_at: e.expires_at.map(|t| t.timestamp_millis()),
        })
        .collect();

    Ok(ListGraylistResult {
        entries: summaries,
        next_cursor,
    })
}

async fn handle_remove_graylist(
    params: sieve_ipc::RemoveGraylistRequest,
    audit: &Arc<AuditStore>,
    state: &Arc<RuntimeState>,
) -> Result<RemoveGraylistResult, ControlError> {
    // 先查询拿 rule_id（写 audit 用）
    let entry = sieve_policy::graylist::lookup(&state.decisions_dir, &params.fingerprint)
        .map_err(|e| ControlError::internal(format!("graylist lookup: {e}")))?
        .ok_or_else(|| ControlError::unknown_fingerprint(params.fingerprint.clone()))?;

    let removed = sieve_policy::graylist::remove_entry(&state.decisions_dir, &params.fingerprint)
        .map_err(|e| ControlError::internal(format!("graylist remove: {e}")))?;

    if !removed {
        return Err(ControlError::unknown_fingerprint(params.fingerprint));
    }

    let audit_event_id = Uuid::now_v7().to_string();
    spawn_audit(
        audit,
        AuditEvent::GraylistRemoved {
            fingerprint: params.fingerprint.clone(),
            rule_id: entry.rule_id,
            removed_by: "gui_user_action".to_owned(),
        },
    );

    Ok(RemoveGraylistResult {
        removed: true,
        audit_event_id,
    })
}

// ─────────────────────────── v2.0+ 兼容扩展 handlers ───────────────────────

/// SPEC-005 §11A `sieve.list_rules`（v2.0+ 兼容扩展）。
///
/// 只读操作，不经过串行化队列（§10.0.1 只读请求并发例外）。
/// 从 inbound + outbound 两个 LayeredEngine 各取系统规则；用户规则当前未在 list 中
/// 区分方向（用 rule_id `user:` 前缀 + UserEngine source_rules 暂不对外暴露）。
///
/// **注意**：`RuleEntry.id` 是规则 ID，无独立 `title` 字段，此处用 `description`
/// 作为 title 的 fallback（SPEC-005 §11A `title` 说明："UI 显示标题"）。
async fn handle_list_rules(
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
) -> Result<ListRulesResult, ControlError> {
    use sieve_rules::manifest::{DefaultOnTimeout, Disposition, Severity};

    let mut rules: Vec<RuleSummary> = Vec::new();

    // ── 系统规则：outbound ──────────────────────────────────────────────────
    let outbound_sys = outbound_layered.system_rules_snapshot();
    for entry in &outbound_sys {
        let disp = entry.effective_disposition();
        let critical_lock = sieve_rules::critical_lock::is_fail_closed(&entry.id);
        let (default_on_timeout, timeout_seconds) = if disp == Disposition::GuiPopup {
            let dot = match entry.default_on_timeout {
                DefaultOnTimeout::Block => Some("block".to_owned()),
                DefaultOnTimeout::Allow => Some("allow".to_owned()),
                DefaultOnTimeout::Redact => Some("redact".to_owned()),
            };
            (dot, entry.timeout_seconds)
        } else {
            (None, None)
        };
        rules.push(RuleSummary {
            rule_id: entry.id.clone(),
            title: entry.description.clone(),
            severity: match entry.severity {
                Severity::Low => "low".to_owned(),
                Severity::Medium => "medium".to_owned(),
                Severity::High => "high".to_owned(),
                Severity::Critical => "critical".to_owned(),
            },
            direction: "outbound".to_owned(),
            disposition: match disp {
                Disposition::GuiPopup => "gui_popup".to_owned(),
                Disposition::AutoRedact => "auto_redact".to_owned(),
                Disposition::StatusBar => "status_bar".to_owned(),
                Disposition::HookTerminal => "hook_terminal".to_owned(),
            },
            default_on_timeout,
            timeout_seconds,
            critical_lock,
            enabled: true, // 系统规则始终启用（未启用的不会被加载）
            rule_kind: "system".to_owned(),
            description: Some(entry.description.clone()),
        });
    }

    // ── 系统规则：inbound ───────────────────────────────────────────────────
    let inbound_sys = inbound_layered.system_rules_snapshot();
    for entry in &inbound_sys {
        let disp = entry.effective_disposition();
        let critical_lock = sieve_rules::critical_lock::is_fail_closed(&entry.id);
        let (default_on_timeout, timeout_seconds) = if disp == Disposition::GuiPopup {
            let dot = match entry.default_on_timeout {
                DefaultOnTimeout::Block => Some("block".to_owned()),
                DefaultOnTimeout::Allow => Some("allow".to_owned()),
                DefaultOnTimeout::Redact => Some("redact".to_owned()),
            };
            (dot, entry.timeout_seconds)
        } else {
            (None, None)
        };
        rules.push(RuleSummary {
            rule_id: entry.id.clone(),
            title: entry.description.clone(),
            severity: match entry.severity {
                Severity::Low => "low".to_owned(),
                Severity::Medium => "medium".to_owned(),
                Severity::High => "high".to_owned(),
                Severity::Critical => "critical".to_owned(),
            },
            direction: "inbound".to_owned(),
            disposition: match disp {
                Disposition::GuiPopup => "gui_popup".to_owned(),
                Disposition::AutoRedact => "auto_redact".to_owned(),
                Disposition::StatusBar => "status_bar".to_owned(),
                Disposition::HookTerminal => "hook_terminal".to_owned(),
            },
            default_on_timeout,
            timeout_seconds,
            critical_lock,
            enabled: true,
            rule_kind: "system".to_owned(),
            description: Some(entry.description.clone()),
        });
    }

    // 规则数为 0 极罕见（daemon 刚启动，引擎未初始化）。
    // 当前实现在 daemon 启动时同步初始化引擎，此分支在实践中不会触发；
    // 保留作为防御性检查，符合 SPEC §11A 错误码 -32006 定义。
    {
        use sieve_rules::engine::MatchEngine;
        if rules.is_empty() && outbound_layered.rule_count() + inbound_layered.rule_count() == 0 {
            return Err(ControlError::rules_loading());
        }
    }

    Ok(ListRulesResult { rules })
}

/// SPEC-005 §11B `sieve.purge_history`（v2.0+ 兼容扩展）。
///
/// 删除所有 audit events（不 DROP TABLE）。互斥防并发：并发调用时第二个立即返回 -32007。
async fn handle_purge_history(
    params: sieve_ipc::PurgeHistoryRequest,
    audit: &Arc<AuditStore>,
    state: &Arc<RuntimeState>,
) -> Result<PurgeHistoryResult, ControlError> {
    // CAS：false → true（防并发）
    if state
        .purge_in_progress
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err(ControlError::purge_in_progress());
    }

    // 确保函数退出时（无论成功或失败）都重置标志。
    struct PurgeGuard<'a>(&'a AtomicBool);
    impl<'a> Drop for PurgeGuard<'a> {
        fn drop(&mut self) {
            self.0.store(false, Ordering::Release);
        }
    }
    let _guard = PurgeGuard(&state.purge_in_progress);

    // 写 purge_started 审计事件（confirmed_at 来自 GUI）
    spawn_audit(
        audit,
        AuditEvent::PurgeHistoryStarted {
            confirmed_at_ms: params.confirmed_at,
        },
    );

    // 执行删除
    let rows_deleted = audit
        .delete_all_events()
        .await
        .map_err(|e| ControlError::internal(format!("purge_history delete failed: {e}")))?;

    let purged_now = chrono::Utc::now();

    // 写 purge_completed 审计事件（内部记录用 epoch ms）
    spawn_audit(
        audit,
        AuditEvent::PurgeHistoryCompleted {
            rows_deleted,
            purged_at_ms: purged_now.timestamp_millis(),
        },
    );

    // wire 响应用 ISO8601 Timestamp（SPEC-005 §11B）
    Ok(PurgeHistoryResult {
        purged_at: purged_now,
        rows_deleted,
    })
}

/// SPEC-005 §11C `sieve.judge_tool_call` handler。
///
/// client（agent 的 PreToolUse hook）把即将执行的结构化工具调用喂来，daemon 用
/// **与真实入站 SSE 路径同款的检测引擎**（[`InboundEngine::check_tool_use`]，扫 tool_name +
/// 全文扫 tool_input）判危：
/// - 命中 fail-closed Critical 规则 → 复用 [`crate::daemon::gated_request_decision`]
///   走 GUI 弹窗确认 + 审计，按用户决策回 allow/deny。
/// - 无 Critical 命中 → allow（v1 范围：非 Critical 工具检测放行，不为每次工具调用弹窗）。
///
/// **fail-closed**：引擎错误 / GUI 拒绝 / 超时（gated_request_decision 对 Critical 按
/// `default_on_timeout=Block` 强制拒绝）→ deny。client 端到自身 deadline 仍须独立兜底。
async fn handle_judge_tool_call(
    params: JudgeToolCallRequest,
    ipc: &Arc<IpcServer>,
    audit: &Arc<AuditStore>,
    inbound_engine: &Arc<dyn InboundEngine>,
    no_client_policy: NoClientPolicy,
) -> Result<JudgeToolCallResult, ControlError> {
    // 1. 构造 CompletedToolCall 喂引擎（与 SSE 入站工具检测同一 trait 方法，行为一致）。
    let tool = CompletedToolCall {
        id: params.tool_use_id.clone(),
        name: params.tool_name.clone(),
        input: params.tool_input.clone(),
    };
    let detections = inbound_engine
        .check_tool_use(&tool, ContentSource::InboundToolUseInput)
        .map_err(|e| ControlError::internal(format!("inbound engine scan failed: {e}")))?;

    // 2. 只对 fail-closed Critical 命中强制人工确认（High-Risk Tool Policy Gate）。
    let critical: Vec<&sieve_core::Detection> = detections
        .iter()
        .filter(|d| sieve_rules::critical_lock::is_fail_closed(&d.rule_id))
        .collect();
    if critical.is_empty() {
        return Ok(JudgeToolCallResult {
            verdict: "allow".to_owned(),
            rule_id: None,
            reason: None,
        });
    }

    // 3. 组装 DecisionRequest（仅 Critical 命中），复用统一决策链。
    let primary_rule = critical[0].rule_id.clone();
    let ipc_detections: Vec<sieve_ipc::DetectionPayload> = critical
        .iter()
        .map(|d| sieve_ipc::DetectionPayload {
            rule_id: d.rule_id.clone(),
            severity: crate::daemon::map_severity_to_ipc(d.severity),
            disposition: sieve_ipc::Disposition::GuiPopup,
            title: format!("危险工具调用：{}（{}）", params.tool_name, d.rule_id),
            one_line_summary: d.evidence_truncated.clone(),
            details: serde_json::json!({
                "tool_name": params.tool_name,
                "tool_use_id": params.tool_use_id,
                "cwd": params.cwd,
            }),
            recommendation: Some(serde_json::json!({
                "decision": "deny",
                "confidence": "high",
                "reason": "入站危险工具调用，fail-closed",
            })),
        })
        .collect();

    // timeout：client deadline 与默认 120s 取较小，钳在 [5, 120]。
    let timeout_seconds: u32 = if params.timeout_ms > 0 {
        (params.timeout_ms / 1000).clamp(5, 120)
    } else {
        120
    };

    let req = sieve_ipc::DecisionRequest {
        request_id: Uuid::now_v7(),
        created_at: Utc::now(),
        timeout_seconds,
        default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
        detections: ipc_detections,
        source_agent: params.source_agent,
        origin_chain: Vec::new(),
        source_channel: None,
        explicit_chain_depth: None,
        allow_remember: false,
    };

    let timeout_dur = Duration::from_secs(u64::from(timeout_seconds).max(1));
    // caller=None（hook 来源，暂不附 caller 进程上下文）；provider_id=系统级。
    let outcome = crate::daemon::gated_request_decision(
        ipc,
        audit,
        &None,
        req,
        timeout_dur,
        "inbound",
        crate::audit::SYSTEM_PROVIDER_ID,
        no_client_policy,
    )
    .await;

    match outcome {
        Ok(resp) => match resp.decision {
            sieve_ipc::DecisionAction::Allow | sieve_ipc::DecisionAction::RedactAndAllow => {
                Ok(JudgeToolCallResult {
                    verdict: "allow".to_owned(),
                    rule_id: Some(primary_rule),
                    reason: None,
                })
            }
            sieve_ipc::DecisionAction::Deny => Ok(JudgeToolCallResult {
                verdict: "deny".to_owned(),
                rule_id: Some(primary_rule.clone()),
                reason: Some(format!("用户拒绝危险工具调用（{primary_rule}）")),
            }),
        },
        Err(e) => {
            // fail-closed：决策链错误（IPC 故障等）→ deny。
            tracing::warn!(error = %e, "judge_tool_call decision failed → fail-closed deny");
            Ok(JudgeToolCallResult {
                verdict: "deny".to_owned(),
                rule_id: Some(primary_rule.clone()),
                reason: Some(format!(
                    "daemon 决策失败，fail-closed 拒绝（{primary_rule}）"
                )),
            })
        }
    }
}

// ─────────────────────────── helpers ───────────────────────────────

fn spawn_audit(audit: &Arc<AuditStore>, event: AuditEvent) {
    let store = Arc::clone(audit);
    tokio::spawn(async move {
        // control plane 是 daemon 系统级路径，无 listener 上下文
        if let Err(e) = store.append(event, crate::audit::SYSTEM_PROVIDER_ID).await {
            tracing::warn!(error = %e, "audit append failed (control plane)");
        }
    });
}

#[allow(dead_code)]
fn epoch_ms_from_systime(t: SystemTime) -> i64 {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use sieve_ipc::PresetOverride;

    fn make_test_state() -> Arc<RuntimeState> {
        Arc::new(RuntimeState {
            paused_until: ArcSwap::from_pointee(None),
            preset: ArcSwap::from_pointee(RuntimePreset::default_mode()),
            started_at: Utc::now(),
            listen: ListenSnapshot {
                addr: "127.0.0.1".to_owned(),
                port: 11453,
            },
            // multi-listener 快照数组（测试用单元素）
            listeners: vec![sieve_ipc::ListenerSnapshot {
                addr: "127.0.0.1".to_owned(),
                port: 11453,
                provider_id: "anthropic".to_owned(),
                protocol: "anthropic".to_owned(),
            }],
            daemon_version: "test".to_owned(),
            protocol_version: "v2".to_owned(),
            audit_db_path: PathBuf::from("/tmp/test_audit.db"),
            decisions_dir: PathBuf::from("/tmp/test_decisions"),
            user_rules_path: None,
            system_rules_count: ArcSwap::from_pointee(0),
            user_rules_count: ArcSwap::from_pointee(0),
            last_reload: ArcSwap::from_pointee(None),
            purge_in_progress: AtomicBool::new(false),
        })
    }

    /// Critical 锁防线二：set_preset_overrides 路径必须拒绝 critical_lock 规则。
    ///
    /// 关联：防线二。
    #[tokio::test]
    async fn set_preset_overrides_rejects_critical_lock_rules() {
        // IN-CR-05-EVM 是系统 fail-closed 规则；运行时注册（替代历史硬编码名单）。
        sieve_rules::critical_lock::register_rules(&[sieve_rules::manifest::RuleEntry {
            id: "IN-CR-05-EVM".into(),
            severity: sieve_rules::manifest::Severity::Critical,
            action: sieve_rules::manifest::Action::Block,
            pattern: "x".into(),
            description: "IN-CR-05-EVM".into(),
            entropy_min: None,
            keywords: vec![],
            allowlist_regexes: vec![],
            allowlist_stopwords: vec![],
            disposition: None,
            fail_closed: None,
            timeout_seconds: None,
            default_on_timeout: sieve_rules::manifest::DefaultOnTimeout::Block,
        }]);
        // 构造一个临时 audit + ipc + state（仅用于 audit 写入与 broadcast 的副作用，不真正断言）。
        let dir = tempfile::tempdir().unwrap();
        let audit = Arc::new(AuditStore::init(&dir.path().join("audit.db")).expect("audit init"));
        let socket = dir.path().join("ipc.sock");
        let (ipc, listener) = IpcServer::bind(socket).unwrap();
        let ipc = Arc::new(ipc);
        let ipc_run = Arc::clone(&ipc);
        tokio::spawn(async move { ipc_run.run(listener).await });

        let state = make_test_state();

        // 投喂混合规则：1 条 critical_lock + 1 条非 critical_lock
        let mut overrides = HashMap::new();
        overrides.insert(
            "IN-CR-05-EVM".to_owned(), // critical_lock 内
            PresetOverride {
                timeout_seconds: 30,
                default_on_timeout: "block".to_owned(),
            },
        );
        overrides.insert(
            "IN-GEN-04".to_owned(), // 非 critical_lock
            PresetOverride {
                timeout_seconds: 30,
                default_on_timeout: "block".to_owned(),
            },
        );

        let (result, _broadcast) = handle_set_preset_overrides(
            sieve_ipc::SetPresetOverridesRequest { overrides },
            None,
            &ipc,
            &audit,
            &state,
        )
        .await
        .expect("handler should succeed (partial accept)");

        assert!(
            result
                .rejected
                .iter()
                .any(|r| r.rule_id == "IN-CR-05-EVM" && r.reason == "critical_lock"),
            "IN-CR-05-EVM 必须被 critical_lock 拒绝，rejected={:?}",
            result.rejected
        );
        assert!(
            result.applied.contains(&"IN-GEN-04".to_owned()),
            "IN-GEN-04 应被接受，applied={:?}",
            result.applied
        );
    }

    /// minutes > 60 → invalid_params。
    #[tokio::test]
    async fn set_paused_rejects_unbounded_pause() {
        let dir = tempfile::tempdir().unwrap();
        let audit = Arc::new(AuditStore::init(&dir.path().join("audit.db")).expect("audit init"));
        let socket = dir.path().join("ipc.sock");
        let (ipc, listener) = IpcServer::bind(socket).unwrap();
        let ipc = Arc::new(ipc);
        let ipc_run = Arc::clone(&ipc);
        tokio::spawn(async move { ipc_run.run(listener).await });

        let state = make_test_state();
        let err = handle_set_paused(
            sieve_ipc::SetPausedRequest { minutes: 120 },
            None,
            &audit,
            &state,
        )
        .await
        .unwrap_err();

        assert_eq!(err.code, sieve_ipc::error::rpc_codes::INVALID_PARAMS);
    }
}
