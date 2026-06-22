use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use arc_swap::ArcSwap;
use chrono::{DateTime, Utc};
use tokio::io::AsyncWriteExt;

use crate::frame_reader::FrameReader;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// oversize 帧事件类型（SPEC-005 §1.3.1）。
///
/// 注入 audit 回调时用于区分超限来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OversizeKind {
    /// 单帧 > 1 MiB（含 \n）。
    Frame,
    /// partial remainder > 1 MiB 且无 newline。
    Remainder,
}

/// oversize 帧 audit 回调函数类型。
///
/// daemon 层注入此 callback，用于将 oversize 事件写入 audit SQLite 而不引入
/// sieve-ipc → sieve-cli 的循环依赖。
pub type OversizeCallback = Arc<dyn Fn(OversizeKind, usize) + Send + Sync>;

use crate::{
    error::{rpc_codes, IpcError},
    protocol::{
        CancelReason, DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout,
        EvaluateRequest, EvaluateResult, HealthRequest, HealthResult, JudgeToolCallRequest,
        JudgeToolCallResult, ListGraylistRequest, ListGraylistResult, ListRulesResult,
        PausedChangedNotify, PresetChangedNotify, PurgeHistoryRequest, PurgeHistoryResult,
        ReloadConfigRequest, ReloadConfigResult, ReloadUserRules, RemoveGraylistRequest,
        RemoveGraylistResult, RequestDecisionCanceledNotify, SetPausedRequest, SetPausedResult,
        SetPresetOverridesRequest, SetPresetOverridesResult, SetPresetRequest, SetPresetResult,
        StatusBarNotify,
    },
};

/// pending map：request_id → oneshot 发送端，等待 GUI 回复。
type PendingMap = Arc<Mutex<HashMap<Uuid, oneshot::Sender<DecisionResponse>>>>;

/// daemon 启动时注入 IpcServer 的握手信息，用于构造 `sieve.hello` 通知。
///
/// 静态字段（整生命周期不变）存此处；动态字段（paused/uptime）在 `handle_connection`
/// 中按时刻计算。关联：SPEC-005 §3。
#[derive(Debug, Clone)]
pub struct HelloBuilder {
    /// daemon 本次启动时生成的唯一 UUID（整生命周期不变）。
    pub daemon_boot_id: Uuid,
    /// daemon 版本（来自 `env!("CARGO_PKG_VERSION")`）。
    pub daemon_version: String,
    /// audit.db PRAGMA user_version（schema 版本号）。
    pub audit_db_user_version: u32,
    /// daemon 启动时刻（UTC），用于计算 uptime_seconds。
    pub started_at: DateTime<Utc>,
    /// 当前 preset 名称（如 `"default"` / `"paranoid"` / `"custom"`）。
    /// handle_connection 时读快照，之后 preset 变更会广播 PresetChangedNotify。
    pub preset: String,
}

/// 控制面错误（IPC handler 内部用，序列化为 JSON-RPC ErrorObject）。
///
/// 关联：ADR-013 §S.2 错误码段。
#[derive(Debug, Clone)]
pub struct ControlError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl ControlError {
    pub fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(rpc_codes::INVALID_PARAMS, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(rpc_codes::INTERNAL_ERROR, message)
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(rpc_codes::PAYLOAD_TOO_LARGE, message)
    }

    pub fn unknown_fingerprint(fingerprint: impl Into<String>) -> Self {
        let fp = fingerprint.into();
        Self::new(
            rpc_codes::UNKNOWN_FINGERPRINT,
            format!("unknown fingerprint: {fp}"),
        )
    }

    /// SPEC-005 §11A `-32006 rules_loading`：规则引擎尚未完成初始化。Since v2.0。
    pub fn rules_loading() -> Self {
        Self::new(rpc_codes::RULES_LOADING, "rules_loading")
    }

    /// SPEC-005 §11B `-32007 purge_in_progress`：并发 purge 防护。Since v2.0。
    pub fn purge_in_progress() -> Self {
        Self::new(rpc_codes::PURGE_IN_PROGRESS, "purge_in_progress")
    }
}

/// 变更类请求（set_paused / set_preset / set_preset_overrides）的 fan-out 计划。
///
/// daemon 处理完 mutating request 后，通过 reply channel 返回结果 **同时** 携带此结构，
/// IPC server 层在写 result 之前先 fan-out 所有通知（SPEC-005 §10.0.1）。
///
/// 避免在 daemon 层直接 `try_send` 广播（可能因通道满而丢失），改为在 IPC server 层
/// `send`（await）逐一写入，保证 result 发出前所有 GUI 都已入队。
#[derive(Debug)]
pub enum BroadcastPlan {
    /// 广播 `sieve.paused_changed` 通知。
    PausedChanged(PausedChangedNotify),
    /// 广播 `sieve.preset_changed` 通知。
    PresetChanged(PresetChangedNotify),
}

/// 控制面请求（GUI → daemon，由 IPC server 反序列化后通过 mpsc 发到 daemon）。
///
/// 每条请求携带 `oneshot::Sender` 用于回执（daemon 处理完写入），
/// IPC server 收到回执后序列化为 JSON-RPC response 写回 GUI socket。
///
/// 关联：ADR-013 Supplement 2026-05-02 §S.4。
pub enum ControlPlaneRequest {
    /// SPEC-005 §10.0.1：reply 携带 BroadcastPlan，IPC server 先 fan-out 再写 result。
    SetPaused {
        params: SetPausedRequest,
        /// 触发本次变更的原始 GUI 请求 ID（SPEC-005 §10.0.2）。
        ///
        /// 从 JSON-RPC `id` 字段解析（GUI 发送的字符串 UUID）；
        /// 非 UUID 格式（如整数 id）时为 `None`。handler 应将此值填入
        /// `PausedChangedNotify.origin_request_id`，使 GUI 能识别"本地回声"。
        origin_request_id: Option<Uuid>,
        reply: oneshot::Sender<Result<(SetPausedResult, Option<BroadcastPlan>), ControlError>>,
    },
    /// SPEC-005 §10.0.1：reply 携带 BroadcastPlan，IPC server 先 fan-out 再写 result。
    SetPreset {
        params: SetPresetRequest,
        /// 触发本次变更的原始 GUI 请求 ID（SPEC-005 §10.0.2）。同 SetPaused 语义。
        origin_request_id: Option<Uuid>,
        reply: oneshot::Sender<Result<(SetPresetResult, Option<BroadcastPlan>), ControlError>>,
    },
    /// SPEC-005 §10.0.1：reply 携带 BroadcastPlan，IPC server 先 fan-out 再写 result。
    SetPresetOverrides {
        params: SetPresetOverridesRequest,
        /// 触发本次变更的原始 GUI 请求 ID（SPEC-005 §10.0.2）。同 SetPaused 语义。
        origin_request_id: Option<Uuid>,
        reply: oneshot::Sender<
            Result<(SetPresetOverridesResult, Option<BroadcastPlan>), ControlError>,
        >,
    },
    ReloadConfig {
        params: ReloadConfigRequest,
        reply: oneshot::Sender<Result<ReloadConfigResult, ControlError>>,
    },
    Health {
        params: HealthRequest,
        reply: oneshot::Sender<Result<HealthResult, ControlError>>,
    },
    Evaluate {
        params: EvaluateRequest,
        reply: oneshot::Sender<Result<EvaluateResult, ControlError>>,
    },
    ListGraylist {
        params: ListGraylistRequest,
        reply: oneshot::Sender<Result<ListGraylistResult, ControlError>>,
    },
    RemoveGraylist {
        params: RemoveGraylistRequest,
        reply: oneshot::Sender<Result<RemoveGraylistResult, ControlError>>,
    },
    /// SPEC-005 §11A `sieve.list_rules`（v2.0+ 兼容扩展）。
    ListRules {
        reply: oneshot::Sender<Result<ListRulesResult, ControlError>>,
    },
    /// SPEC-005 §11B `sieve.purge_history`（v2.0+ 兼容扩展）。
    PurgeHistory {
        params: PurgeHistoryRequest,
        reply: oneshot::Sender<Result<PurgeHistoryResult, ControlError>>,
    },
    /// SPEC-005 §11C `sieve.judge_tool_call`（Since v2.x 向后兼容扩展）。
    ///
    /// daemon 侧 handler 可能阻塞等 GUI 弹窗（30–120s），**必须 spawn-per-judge 并发处理**，
    /// 不可在串行控制面 loop 内联 await，否则阻塞所有其他控制面请求。
    JudgeToolCall {
        params: JudgeToolCallRequest,
        reply: oneshot::Sender<Result<JudgeToolCallResult, ControlError>>,
    },
}

/// 控制面 channel 容量。
///
/// 容量 32：预期单 GUI 突发并发 ≤ 5（设置面板批量改 + health 轮询），32 留足余量。
const CONTROL_CHANNEL_CAPACITY: usize = 32;

/// reload 通知 channel 容量。
///
/// 容量 16：daemon 通常实时消费（spawn task 监听），16 条积压足够应对短暂卡顿；
/// 积压超限说明 daemon reload handler 异常，此时丢弃最新通知优于阻塞 IPC server。
const RELOAD_CHANNEL_CAPACITY: usize = 16;

/// 心跳间隔（秒）。SPEC-005 §4 要求 25 秒内无出站消息时发送 sieve.heartbeat。
const HEARTBEAT_INTERVAL_SECS: u64 = 25;

/// accept 错误退避时长。非连接级 accept 错误（典型 EMFILE/ENFILE fd 耗尽）后等待此
/// 时长再重试，给系统回收 fd 的窗口，同时把错误日志限到 ≤10/s、避免 busy-loop 烧 CPU。
const ACCEPT_ERROR_BACKOFF: Duration = Duration::from_millis(100);

/// GUI 客户端的写通道列表：支持多个并发 GUI 连接（fan-out 广播）。
///
/// 每个 GUI 连接注册一个独立 `mpsc::Sender<String>`；broadcast 时顺序投递。
/// 通道容量设为 32，满了视为短暂背压（保留 sender）而非断线。
/// 写失败（`TrySendError::Closed`）时立即从 Vec 移除（lazy 清理，无需显式注销）。
///
/// 关联：PRD v2.1 §5.4.3（多 GUI 客户端支持）、ADR-013。
type GuiWriters = Arc<std::sync::Mutex<Vec<mpsc::Sender<String>>>>;

/// IPC 服务端，监听 Unix socket，维护与 GUI 的长连接并推送决策请求。
///
/// # 连接语义（v2.1 多 GUI 客户端）
///
/// - GUI 启动后主动连接此 socket，保持长连接。
/// - 支持多个并发 GUI 连接（如 sieve-gui-macos + sieve doctor 同时运行）。
/// - GUI 断线后 `gui_writers` 中对应 sender 在下次 broadcast 时自动清理。
/// - `request_decision` 仍发往第一个已连接 GUI（决策请求有状态，不 fan-out）。
///
/// # 双向通信模型
///
/// ```text
/// [主代理]  ─request_decision JSON-RPC request─▶  [GUI]
/// [主代理]  ◀─decision_response JSON-RPC response─  [GUI]
/// [主代理]  ─sieve.notify_status_bar notification─▶  [GUI×N]  （fan-out 单向）
/// [rules edit]  ─sieve.reload_user_rules notification─▶  [主代理 IpcServer]  （单向）
/// ```
///
/// 每个方向在同一条 Unix socket 连接上用换行分隔的 JSON-RPC 帧传输。
/// `handle_connection` 负责从 GUI 读取响应帧并派发到 `pending` map；
/// `request_decision` 通过 `gui_writers[0]` mpsc 通道写入请求帧。
///
/// # 单向通知（v2.1）
///
/// - `broadcast_status_bar`：daemon 向**所有**已连接 GUI 广播状态栏通知。
///   `TrySendError::Closed` 的 sender 立即从 Vec 移除（lazy 清理）；
///   `TrySendError::Full` 视为短暂背压，保留 sender 不断线。
///   失败（无客户端 / socket 写错）静默丢弃 + debug 日志，**daemon 主流程不阻塞**。
///   关联：PRD v2.0 §5.7、PRD v2.1 §5.4.3、ADR-013。
/// - `reload_rx`：daemon 通过此 channel 接收来自 `sieve rules edit` 的 reload 通知。
///   关联：PRD v2.0 §5.5.5、ADR-013。
///
/// 关联：ADR-013 §3（JSON-RPC over Unix socket）、ADR-014 §5（GUI 路径）。
pub struct IpcServer {
    socket_path: PathBuf,
    pending: PendingMap,
    /// 当前已连接的所有 GUI 客户端写通道；无 GUI 时为空 Vec。
    ///
    /// 使用 `std::sync::Mutex`（非 tokio）：broadcast_status_bar 持锁时间极短
    /// （drain + try_send，无 await），不会跨 await 点持锁，std Mutex 足够。
    gui_writers: GuiWriters,
    /// reload 通知发送端（接收端通过 `reload_rx()` 暴露给 daemon）。
    reload_tx: mpsc::Sender<ReloadUserRules>,
    /// reload 通知接收端（`Option` 包装以支持 `take` 一次性移交给 daemon task）。
    reload_rx: Arc<Mutex<Option<mpsc::Receiver<ReloadUserRules>>>>,
    /// 控制面请求发送端（GUI 控制面 8 个方法的反序列化结果汇聚到此 channel）。
    control_tx: mpsc::Sender<ControlPlaneRequest>,
    /// 控制面接收端（daemon 通过 `control_rx()` 取出，单独 task 消费）。
    control_rx: Arc<Mutex<Option<mpsc::Receiver<ControlPlaneRequest>>>>,
    /// 暂停截止时间（hot path 直接读，避免跨 crate 取 RuntimeState）。
    ///
    /// 关联：ADR-013 §S.4 set_paused / SPEC-002 §9.1。
    /// `None` = 未暂停；`Some(t)` 且 `t > now` = 暂停中。
    /// daemon 控制面 handler 通过 [`Self::set_paused_until`] 同步。
    paused_until: Arc<ArcSwap<Option<DateTime<Utc>>>>,
    /// sieve.hello 握手通知所需静态信息（SPEC-005 §3）。
    ///
    /// daemon 启动后通过 [`Self::set_hello_builder`] 注入；连接前为 `None`。
    /// 若未设置则跳过握手（保持向后兼容，测试场景可不设置）。
    hello_builder: Arc<Mutex<Option<HelloBuilder>>>,
    /// oversize 帧 audit 回调（SPEC-005 §1.3.1）。
    ///
    /// daemon 层通过 [`Self::set_oversize_callback`] 注入，用于写 `AuditEvent::IpcOversizeFrame`
    /// 而不引入 sieve-ipc → sieve-cli 循环依赖。未注入时只打 warn! log。
    oversize_callback: Arc<std::sync::Mutex<Option<OversizeCallback>>>,
}

impl IpcServer {
    /// 绑定 Unix socket 并返回服务端实例。
    ///
    /// socket_path 已存在时先删除旧文件（daemon 重启场景）。
    pub fn bind(socket_path: PathBuf) -> Result<(Self, UnixListener), IpcError> {
        // 旧 socket 文件存在则先删除，否则 bind 会失败。
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }
        let listener = UnixListener::bind(&socket_path)?;
        // SPEC-005 §1.1：socket 文件权限必须为 0600，防止其他用户访问。
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
        }
        let (reload_tx, reload_rx) = mpsc::channel::<ReloadUserRules>(RELOAD_CHANNEL_CAPACITY);
        let (control_tx, control_rx) =
            mpsc::channel::<ControlPlaneRequest>(CONTROL_CHANNEL_CAPACITY);
        let server = Self {
            socket_path,
            pending: Arc::new(Mutex::new(HashMap::new())),
            gui_writers: Arc::new(std::sync::Mutex::new(Vec::new())),
            reload_tx,
            reload_rx: Arc::new(Mutex::new(Some(reload_rx))),
            control_tx,
            control_rx: Arc::new(Mutex::new(Some(control_rx))),
            paused_until: Arc::new(ArcSwap::from_pointee(None)),
            hello_builder: Arc::new(Mutex::new(None)),
            oversize_callback: Arc::new(std::sync::Mutex::new(None)),
        };
        Ok((server, listener))
    }

    /// 取出控制面接收端（只能调用一次，之后返回 None）。
    ///
    /// daemon 应在启动时调用，spawn task 监听控制面请求：
    ///
    /// ```ignore
    /// if let Some(mut rx) = server.control_rx().await {
    ///     tokio::spawn(async move {
    ///         while let Some(req) = rx.recv().await {
    ///             match req {
    ///                 ControlPlaneRequest::Health { params, reply } => { ... }
    ///                 // ...
    ///             }
    ///         }
    ///     });
    /// }
    /// ```
    pub async fn control_rx(&self) -> Option<mpsc::Receiver<ControlPlaneRequest>> {
        self.control_rx.lock().await.take()
    }

    /// 内部触发用户规则 reload（控制面 `sieve.reload_config` 复用此通道）。
    ///
    /// 与外部 `send_reload_user_rules_oneshot` 等价，但走进程内 mpsc 而非 socket。
    /// 关联：ADR-013 Supplement §S.4 sieve.reload_config。
    pub async fn trigger_user_rules_reload(
        &self,
        trigger: ReloadUserRules,
    ) -> Result<(), IpcError> {
        self.reload_tx
            .send(trigger)
            .await
            .map_err(|_| IpcError::FileLock("reload channel closed".to_owned()))
    }

    /// 注入 sieve.hello 握手通知所需静态信息（SPEC-005 §3）。
    ///
    /// daemon 启动时调用一次；此后每个新连接的 `handle_connection` 会读取此快照
    /// 并在连接建立后发送第一条消息 `sieve.hello`。
    pub async fn set_hello_builder(&self, builder: HelloBuilder) {
        *self.hello_builder.lock().await = Some(builder);
    }

    /// 注入 oversize 帧 audit 回调（SPEC-005 §1.3.1）。
    ///
    /// daemon 启动时调用一次；之后每个连接遇到 oversize frame/remainder 时都会调用此 callback，
    /// 用于写 `AuditEvent::IpcOversizeFrame` 到 SQLite。未注入时只打 warn! log。
    ///
    /// sieve-ipc 不依赖 sieve-cli，通过此 callback 反转依赖。
    pub fn set_oversize_callback(&self, cb: OversizeCallback) {
        if let Ok(mut guard) = self.oversize_callback.lock() {
            *guard = Some(cb);
        }
    }

    /// 设置 / 清除暂停截止时间。
    ///
    /// `None` = 立即恢复；`Some(t)` = 暂停至 `t`（UTC）。daemon 控制面 handler 在
    /// [`crate::ControlPlaneRequest::SetPaused`] 处理后调用此方法同步给 hot path。
    pub fn set_paused_until(&self, until: Option<DateTime<Utc>>) {
        self.paused_until.store(Arc::new(until));
    }

    /// 查询当前是否处于暂停状态（hot path lock-free 读）。
    ///
    /// 自动恢复：若 `paused_until` 已过期则原地清空并返回 false。
    /// 关联：SPEC-002 §9.1（暂停期间非 Critical 弹窗跳过）。
    pub fn is_paused(&self) -> bool {
        let snap = self.paused_until.load();
        match snap.as_ref().as_ref() {
            Some(until) => {
                if *until > Utc::now() {
                    true
                } else {
                    // 自动恢复——避免下次重复检查
                    self.paused_until.store(Arc::new(None));
                    false
                }
            }
            None => false,
        }
    }

    /// 当前 paused_until 快照（health 报告用）。
    pub fn paused_until_snapshot(&self) -> Option<DateTime<Utc>> {
        let snap = self.paused_until.load();
        snap.as_ref().as_ref().copied().filter(|t| *t > Utc::now())
    }

    /// 当前已连接的 GUI 客户端数量（health snapshot 用）。
    pub fn connected_clients(&self) -> usize {
        self.gui_writers.lock().map(|w| w.len()).unwrap_or(0)
    }

    /// 当前 inflight 的决策请求数量（health snapshot 用）。
    pub async fn inflight_decisions(&self) -> usize {
        self.pending.lock().await.len()
    }

    /// 取出 reload 通知接收端（只能调用一次，之后返回 None）。
    ///
    /// daemon 应在启动时调用一次，spawn task 监听此 channel：
    ///
    /// ```ignore
    /// if let Some(mut rx) = server.reload_rx().await {
    ///     tokio::spawn(async move {
    ///         while let Some(reload) = rx.recv().await {
    ///             // 重新加载 user.toml …
    ///         }
    ///     });
    /// }
    /// ```
    ///
    /// 关联：PRD v2.0 §5.5.5、ADR-013。
    pub async fn reload_rx(&self) -> Option<mpsc::Receiver<ReloadUserRules>> {
        self.reload_rx.lock().await.take()
    }

    /// 运行 accept 循环，处理来自 GUI 和 sieve-hook/rules 进程的连接。
    ///
    /// # v2.1 多 GUI 客户端支持
    ///
    /// 每个新连接都先尝试**长连接握手**：如果客户端在 200ms 内发来首行消息，
    /// 则检查是否为已知短连接通知（`sieve.reload_user_rules`）并走 `handle_notification`；
    /// 否则视为 GUI 长连接，注册到 `gui_writers` Vec 并走 `handle_connection`。
    ///
    /// 没有首行消息（超时）的连接直接视为 GUI 长连接（GUI 不主动发握手帧）。
    ///
    /// 来自 `sieve rules edit` 的 reload 通知（短连接）通过 `reload_tx` 分发到
    /// `reload_rx` channel，daemon 通过 `reload_rx()` 取出接收端监听。
    pub async fn run(&self, listener: UnixListener) {
        info!(socket = %self.socket_path.display(), "IPC server listening");
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let pending = Arc::clone(&self.pending);
                    let gui_writers = Arc::clone(&self.gui_writers);
                    let reload_tx = self.reload_tx.clone();
                    let control_tx = self.control_tx.clone();
                    // 握手信息快照（SPEC-005 §3）。
                    let hello_builder = self.hello_builder.lock().await.clone();
                    let paused_until = Arc::clone(&self.paused_until);
                    // oversize callback 快照（SPEC-005 §1.3.1）。
                    let oversize_callback =
                        self.oversize_callback.lock().ok().and_then(|g| g.clone());

                    // 为新连接创建 mpsc 通道：发送端注册到 gui_writers，接收端传给 handle_connection。
                    // 同时 clone 一份发送端给 handle_connection 用，让控制面响应能路由回当前连接。
                    // oneshot client（如 reload）连接短暂，断开后 try_send 自动清理其 sender。
                    let (tx, rx) = mpsc::channel::<String>(32);
                    let conn_tx = tx.clone();
                    {
                        // 毒化恢复：持锁线程 panic 不破坏 Vec 结构，into_inner 取出数据继续
                        // 注册（审查 §6——原 .expect 在毒化时 panic = DoS）。
                        let mut writers = gui_writers
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        writers.push(tx);
                        let count = writers.len();
                        if count == 1 {
                            info!("first GUI client connected; gui_writers count = 1");
                        } else {
                            info!(count, "additional GUI client connected");
                        }
                    }

                    let gui_writers_for_ctx = Arc::clone(&gui_writers);
                    tokio::spawn(async move {
                        let ctx = ConnectionContext {
                            pending,
                            write_tx: conn_tx,
                            write_rx: rx,
                            reload_tx,
                            control_tx,
                            hello_builder,
                            paused_until,
                            oversize_callback,
                            gui_writers: gui_writers_for_ctx,
                        };
                        if let Err(e) = handle_connection(stream, ctx).await {
                            error!("IPC connection error: {e}");
                        }
                        // 连接断开：gui_writers 中对应的 sender 已 drop（rx drop 时发送端关闭），
                        // 下次 broadcast_status_bar 的 try_send 会检测到 Closed 并自动清理。
                        // 此处只记录日志；不需要显式从 Vec 删除（lazy 清理策略）。
                        debug!("GUI connection task exited; dead sender will be cleaned on next broadcast");
                    });
                }
                Err(e) => {
                    // accept 错误绝不击穿整个控制面 daemon（fail-closed 安全代理的可用性
                    // 单点）：单次瞬态错误——对端在 accept 完成前断开、或 EMFILE/ENFILE fd
                    // 耗尽——不应让 GUI 弹窗 / hook pending / reload 全部永久失效。参照 hyper：
                    // 连接级错误立即重试，其余退避后重试避免 busy-loop。真正不可恢复的
                    // listener 损坏交由 launchd KeepAlive 重启进程，而非让 accept 循环自杀。
                    if is_connection_error(&e) {
                        debug!("IPC accept transient connection error, retrying: {e}");
                    } else {
                        error!("IPC accept error, backing off {ACCEPT_ERROR_BACKOFF:?}: {e}");
                        tokio::time::sleep(ACCEPT_ERROR_BACKOFF).await;
                    }
                }
            }
        }
    }

    /// 向**所有**已连接的 GUI 广播 StatusBarNotify（fan-out 单向，不等回复）。
    ///
    /// # 行为
    ///
    /// - 无 GUI 连接时静默丢弃 + debug 日志，**daemon 主流程不阻塞**。
    /// - 使用 `try_send`（非阻塞）：
    ///   - `TrySendError::Closed`：GUI 已断线，立即从 Vec 移除该 sender（lazy 清理）。
    ///   - `TrySendError::Full`：GUI 写通道短暂背压，保留 sender，记录 debug 日志。
    /// - 持锁时间极短（drain + try_send 均不 await），不会跨 await 点持 std Mutex。
    ///
    /// 关联：PRD v2.0 §5.7（行为序列 StatusBar 通知）+ PRD v2.1 §5.4.3（多 GUI 客户端）+ ADR-013。
    pub fn broadcast_status_bar(&self, notify: StatusBarNotify) {
        let label = format!("status_bar notify_id={}", notify.notify_id);
        self.broadcast_method("sieve.notify_status_bar", &notify, &label);
    }

    /// 向**所有**已连接的 GUI 广播 preset 变更通知。
    ///
    /// 关联：ADR-013 Supplement §S.3 / SPEC-002 §9.2。
    pub fn broadcast_preset_changed(&self, notify: PresetChangedNotify) {
        let label = format!("preset_changed mode={}", notify.mode);
        self.broadcast_method("sieve.preset_changed", &notify, &label);
    }

    /// 向**所有**已连接的 GUI 广播 paused 状态变更通知。
    ///
    /// 关联：ADR-013 Supplement §S.3 / SPEC-002 §9.1。
    pub fn broadcast_paused_changed(&self, notify: PausedChangedNotify) {
        let label = format!("paused_changed paused={}", notify.paused);
        self.broadcast_method("sieve.paused_changed", &notify, &label);
    }

    /// 向**所有**已连接的 GUI 广播 request_decision 取消通知。
    ///
    /// 关联：ADR-013 Supplement §S.3 / SPEC-002 §9.3 / §9.4。
    pub fn broadcast_request_decision_canceled(&self, notify: RequestDecisionCanceledNotify) {
        let label = format!("request_decision_canceled request_id={}", notify.request_id);
        self.broadcast_method("sieve.request_decision_canceled", &notify, &label);
    }

    /// 通用 fan-out 广播（所有 broadcast_* 方法的共用实现）。
    ///
    /// 行为与 broadcast_status_bar 历史实现一致：
    /// - 无 GUI 连接时静默丢弃 + debug 日志。
    /// - `try_send`：Closed → 移除（lazy 清理）；Full → 保留（背压）。
    /// - 持 `std::sync::Mutex` 锁短暂、无 await。
    fn broadcast_method<T: serde::Serialize>(&self, method: &str, params: &T, label: &str) {
        let notification = crate::protocol::jsonrpc::Request {
            jsonrpc: "2.0".to_owned(),
            method: method.to_owned(),
            params: match serde_json::to_value(params) {
                Ok(v) => Some(v),
                Err(e) => {
                    warn!(method, label, "failed to serialize broadcast params: {e}");
                    return;
                }
            },
            id: None,
        };

        let mut payload = match serde_json::to_string(&notification) {
            Ok(s) => s,
            Err(e) => {
                warn!(method, "failed to serialize JSON-RPC frame: {e}");
                return;
            }
        };
        payload.push('\n');

        // 毒化恢复：持锁线程 panic 不破坏 Vec 结构，into_inner 取出数据继续（审查 §6）。
        let mut writers = self
            .gui_writers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if writers.is_empty() {
            debug!(
                method,
                label, "broadcast: no GUI clients connected; dropping"
            );
            return;
        }

        let total = writers.len();
        let mut alive: Vec<mpsc::Sender<String>> = Vec::with_capacity(total);
        let mut sent = 0usize;
        let mut removed = 0usize;

        for sender in writers.drain(..) {
            match sender.try_send(payload.clone()) {
                Ok(()) => {
                    sent += 1;
                    alive.push(sender);
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    debug!(method, label, "GUI writer channel full; retaining sender");
                    alive.push(sender);
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    debug!(
                        method,
                        label, "GUI client sender closed; removing from gui_writers"
                    );
                    removed += 1;
                }
            }
        }
        *writers = alive;

        if removed > 0 {
            debug!(
                method,
                label,
                sent,
                removed,
                alive = writers.len(),
                "broadcast: cleaned up dead GUI clients"
            );
        } else {
            debug!(
                method,
                label, sent, "broadcast: delivered to all GUI clients"
            );
        }
    }

    /// 向已连接的 GUI 发送决策请求，等待响应或超时。
    ///
    /// `direction` 参数：`"inbound"` / `"outbound"`，调用方根据流量方向传入。
    /// wire DTO 层（SPEC-005 §6.0）用此值填充 `sieve.request_decision` 的 `direction` 字段。
    ///
    /// # 行为
    ///
    /// - 如果没有 GUI 客户端连接：**立即 fallback**，不等超时。
    ///   （等超时无意义——没人能决策。）
    /// - 如果 GUI 写通道已满（背压）或 GUI 进程崩溃（`try_send` 返回 Full/Closed）：
    ///   **立即 fallback**，不阻塞 SSE pipeline。**不用 `send().await`**——队列满会
    ///   把 hot path 卡死直到 timeout 到期，期间整个 SSE 连接 hold 住，对用户体验
    ///   而言相当于 daemon 死锁（known-issues-v1.4 P2-R10-#4）。
    /// - 如果 GUI 在 `timeout` 内回复：返回 GUI 的决策。
    /// - 如果超时：按 `default_on_timeout` 构造兜底响应，并从 pending map 清理。
    pub async fn request_decision(
        &self,
        req: DecisionRequest,
        timeout: Duration,
        direction: &str,
    ) -> Result<DecisionResponse, IpcError> {
        let request_id = req.request_id;
        let default_on_timeout = req.default_on_timeout;

        // 1. 检查 GUI 是否已连接（取第一个 sender 用于决策请求，决策请求不 fan-out）。
        let sender = {
            // 毒化恢复：持锁线程 panic 不破坏 Vec 结构，into_inner 取出数据继续（审查 §6）。
            let writers = self
                .gui_writers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            writers.first().cloned()
        };

        let Some(sender) = sender else {
            // 没有 GUI——立即 fallback，不消耗超时时间。
            debug!(%request_id, "no GUI client connected; immediate fallback");
            return Ok(make_timeout_fallback(request_id, default_on_timeout));
        };

        // 2. 注册 oneshot channel，等待 GUI 回复。
        let (tx, rx) = oneshot::channel::<DecisionResponse>();
        {
            let mut map = self.pending.lock().await;
            map.insert(request_id, tx);
        }

        // 3. 通过 wire DTO 适配层序列化请求（SPEC-005 §6.0 / §6.1 / §6.1.1 / §6.1.2）。
        //    P1-5 + P2-2 + P2-4：使用 RequestDecisionWireKind 替换内部 struct 直接序列化。
        let wire = crate::wire::RequestDecisionWireKind::from_request(&req, direction);
        let wire_params = wire.to_value()?;
        let rpc_req = crate::protocol::jsonrpc::Request::call(
            "sieve.request_decision",
            wire_params,
            serde_json::Value::String(request_id.to_string()),
        );
        let mut payload = serde_json::to_string(&rpc_req)?;
        payload.push('\n');

        // try_send 而非 send().await：避免 mpsc 队列满时阻塞 hot path（P2-R10-#4）。
        // Full → 背压；Closed → 客户端已断。两者都立即降级，不让 SSE pipeline 等。
        if let Err(e) = sender.try_send(payload) {
            self.pending.lock().await.remove(&request_id);
            match e {
                mpsc::error::TrySendError::Full(_) => {
                    warn!(%request_id, "GUI writer channel full (backpressure); immediate fallback");
                }
                mpsc::error::TrySendError::Closed(_) => {
                    warn!(%request_id, "GUI writer channel closed; immediate fallback");
                }
            }
            return Ok(make_timeout_fallback(request_id, default_on_timeout));
        }

        // 4. 等待 GUI 回复或超时。
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(_)) => {
                // oneshot sender 已丢弃（handle_connection 因断线退出），走超时兜底。
                warn!(%request_id, "decision sender dropped (GUI disconnected); fallback");
                // 不广播 cancel（GUI 已断，没人能收到通知；其他可能存在的 GUI 实例
                // 也不会收到，因 pending 已清空）。
                Ok(make_timeout_fallback(request_id, default_on_timeout))
            }
            Err(_elapsed) => {
                // 超时，清理 pending map + 广播取消通知给所有 GUI（SPEC-002 §9.3）。
                self.pending.lock().await.remove(&request_id);
                warn!(%request_id, "decision timeout");
                let auto_decision = match default_on_timeout {
                    DefaultOnTimeout::Block => DecisionAction::Deny,
                    DefaultOnTimeout::Allow => DecisionAction::Allow,
                    DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
                };
                self.broadcast_request_decision_canceled(RequestDecisionCanceledNotify {
                    request_id,
                    reason: CancelReason::Timeout,
                    auto_decision,
                });
                Ok(make_timeout_fallback(request_id, default_on_timeout))
            }
        }
    }

    /// 供测试使用：直接注入一个决策响应，模拟 GUI 回调。
    pub async fn inject_decision(&self, resp: DecisionResponse) {
        let mut map = self.pending.lock().await;
        if let Some(tx) = map.remove(&resp.request_id) {
            let _ = tx.send(resp);
        }
    }
}

/// 处理单个 GUI 长连接（或短连接通知）。
///
/// 同时管理两个方向：
/// - **读方向**：从 GUI 读换行分隔的 JSON-RPC response，派发到 `pending` map；
///   同时识别 `sieve.reload_user_rules` 通知并转发到 `reload_tx`。
/// - **写方向**：从 `write_rx` mpsc 通道读取待发送的帧，写入 GUI socket。
///
/// 任一方向出错（GUI 断线 / 写失败）都会退出；`write_rx` drop 后其对应的
/// `Sender` 在 `gui_writers` Vec 中下次 broadcast 时自动清理（lazy 策略）。
struct ConnectionContext {
    pending: PendingMap,
    write_tx: mpsc::Sender<String>,
    write_rx: mpsc::Receiver<String>,
    reload_tx: mpsc::Sender<ReloadUserRules>,
    control_tx: mpsc::Sender<ControlPlaneRequest>,
    hello_builder: Option<HelloBuilder>,
    paused_until: Arc<ArcSwap<Option<DateTime<Utc>>>>,
    /// oversize 帧 audit 回调（SPEC-005 §1.3.1）。
    oversize_callback: Option<OversizeCallback>,
    /// 所有 GUI 客户端写通道（fan-out 广播用）。
    gui_writers: GuiWriters,
}

async fn handle_connection(stream: UnixStream, ctx: ConnectionContext) -> Result<(), IpcError> {
    let ConnectionContext {
        pending,
        write_tx,
        mut write_rx,
        reload_tx,
        control_tx,
        hello_builder,
        paused_until,
        oversize_callback,
        gui_writers: gui_writers_clone,
    } = ctx;
    info!("GUI client connected");

    let (read_half, mut write_half) = stream.into_split();
    // SPEC-005 §1.3.1：用 FrameReader（read_buf + memchr）替代无界 BufReader::lines()。
    let mut frame_reader = FrameReader::new(read_half);

    // SPEC-005 §3：连接建立后第一条出站消息必须是 sieve.hello notification。
    if let Some(builder) = hello_builder {
        let paused = {
            let snap = paused_until.load();
            match snap.as_ref().as_ref() {
                Some(until) => *until > Utc::now(),
                None => false,
            }
        };
        let uptime_seconds = (Utc::now() - builder.started_at).num_seconds().max(0) as u64;
        let params = crate::protocol::HelloParams {
            protocol_version: "v2".to_owned(),
            daemon_version: builder.daemon_version,
            paused,
            preset: builder.preset,
            uptime_seconds,
            audit_db_user_version: builder.audit_db_user_version,
            daemon_boot_id: builder.daemon_boot_id,
        };
        let notification = crate::protocol::jsonrpc::Request {
            jsonrpc: "2.0".to_owned(),
            method: "sieve.hello".to_owned(),
            params: Some(serde_json::to_value(&params).unwrap_or(serde_json::Value::Null)),
            id: None,
        };
        match serde_json::to_string(&notification) {
            Ok(mut frame) => {
                frame.push('\n');
                if let Err(e) = write_half.write_all(frame.as_bytes()).await {
                    warn!("failed to send sieve.hello: {e}");
                    return Ok(());
                }
                debug!("sent sieve.hello to new GUI client");
            }
            Err(e) => {
                warn!("failed to serialize sieve.hello: {e}");
            }
        }
    }

    // SPEC-005 §4：25 秒内无出站消息时发送 sieve.heartbeat notification；
    // 任何出站帧（包括心跳本身）都重置计时器。
    // 用 `tokio::time::interval` + `reset()` 实现：初始 tick 立即到期，
    // 但我们在 select! 里不会 await 它直到循环的第一次写方向或心跳分支——
    // 改用 `interval_at(Instant::now() + 25s, 25s)` 保证连接建立后 25s 才首发。
    let mut heartbeat_interval = tokio::time::interval_at(
        tokio::time::Instant::now() + Duration::from_secs(HEARTBEAT_INTERVAL_SECS),
        Duration::from_secs(HEARTBEAT_INTERVAL_SECS),
    );
    heartbeat_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            // 读方向：GUI 发来 decision_response 或控制面 request。
            // SPEC-005 §1.3.1：read_frame 返回完整帧（不含尾部 \n）。
            // OversizeFrame / OversizeRemainder → ? 向上传播，handle_connection 返回 Err，
            // run() 的 spawn task 中打 error 日志并关闭连接（MUST 约束）。
            frame_result = frame_reader.read_frame() => {
                match frame_result {
                    Err(e) => {
                        // 超限：SPEC-005 §1.3.1 MUST 关连接；size_bytes 不含 payload。
                        let (kind, size_bytes) = match &e {
                            crate::frame_reader::FrameError::OversizeFrame { size_bytes } => {
                                (OversizeKind::Frame, *size_bytes)
                            }
                            crate::frame_reader::FrameError::OversizeRemainder { size_bytes } => {
                                (OversizeKind::Remainder, *size_bytes)
                            }
                            _ => (OversizeKind::Frame, 0),
                        };
                        warn!(size_bytes, "IPC oversize frame; closing connection");
                        // 若已注入 audit callback，调用写入 AuditEvent::IpcOversizeFrame。
                        if let Some(ref cb) = oversize_callback {
                            cb(kind, size_bytes);
                        }
                        return Err(IpcError::from(e));
                    }
                    Ok(None) => {
                        // GUI 关闭连接（EOF）。
                        info!("GUI client closed connection");
                        break;
                    }
                    Ok(Some(raw_bytes)) => {
                        // §1.3.1 第 5 条：parse_and_dispatch 失败不关连接。
                        let line = match std::str::from_utf8(&raw_bytes) {
                            Ok(s) => s.trim().to_owned(),
                            Err(_) => {
                                warn!("received non-UTF8 IPC frame; ignoring");
                                continue;
                            }
                        };
                        if line.is_empty() {
                            continue;
                        }
                        debug!("received IPC message from GUI");
                        dispatch_message(
                            &line,
                            &pending,
                            &reload_tx,
                            &control_tx,
                            &write_tx,
                            &gui_writers_clone,
                        )
                        .await;
                    }
                }
            }

            // 写方向：主代理 push request_decision 给 GUI（含控制面响应回执）。
            // SPEC-005 §10.0.1：fan-out write 加 2 秒 bounded write timeout；
            // 超时 / EPIPE / ECONNRESET / EBADF 视为失联，退出 handle_connection。
            msg = write_rx.recv() => {
                match msg {
                    None => {
                        // 发送端已丢弃（IpcServer 被 drop），退出。
                        debug!("GUI write channel closed");
                        break;
                    }
                    Some(payload) => {
                        let write_res = tokio::time::timeout(
                            Duration::from_secs(2),
                            write_half.write_all(payload.as_bytes()),
                        )
                        .await;
                        match write_res {
                            Err(_elapsed) => {
                                warn!("GUI write timeout (2s); disconnecting");
                                break;
                            }
                            Ok(Err(e)) => {
                                // EPIPE / ECONNRESET / EBADF 等均视为失联。
                                warn!("failed to write to GUI socket: {e}; disconnecting");
                                break;
                            }
                            Ok(Ok(())) => {
                                // 出站帧重置心跳计时器。
                                heartbeat_interval
                                    .reset_after(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
                            }
                        }
                    }
                }
            }

            // 心跳：25 秒无出站消息时发送 sieve.heartbeat notification（SPEC-005 §4）。
            // 同样包 2 秒 write timeout（SPEC-005 §10.0.1 fan-out timeout 适用所有写路径）。
            _ = heartbeat_interval.tick() => {
                let frame = heartbeat_frame();
                let write_res = tokio::time::timeout(
                    Duration::from_secs(2),
                    write_half.write_all(frame.as_bytes()),
                )
                .await;
                match write_res {
                    Err(_elapsed) => {
                        warn!("GUI heartbeat write timeout (2s); disconnecting");
                        break;
                    }
                    Ok(Err(e)) => {
                        warn!("failed to write heartbeat to GUI socket: {e}; disconnecting");
                        break;
                    }
                    Ok(Ok(())) => {
                        debug!("sent sieve.heartbeat to GUI");
                    }
                }
            }
        }
    }

    // 连接断开：把所有 pending oneshot 全部触发 fallback（drop sender）。
    // 丢弃 sender 会让 rx 收到 Err(RecvError)，request_decision 走 fallback。
    let mut map = pending.lock().await;
    let count = map.len();
    if count > 0 {
        warn!(
            pending_count = count,
            "GUI disconnected with pending requests; dropping all"
        );
        map.clear(); // 清空 map，sender 被 drop，所有等待者收到 Err 并 fallback。
    }
    // _gui_writers 持有的是 Arc<Mutex<Vec<Sender>>>，此处 drop 不影响 Vec 内容。
    // 对应 sender 在下次 broadcast_status_bar 的 try_send 返回 Closed 时自动移除。

    Ok(())
}

/// 解析一行 JSON-RPC 消息，区分 decision response、单向通知、控制面 request 并分别派发。
async fn dispatch_message(
    line: &str,
    pending: &PendingMap,
    reload_tx: &mpsc::Sender<ReloadUserRules>,
    control_tx: &mpsc::Sender<ControlPlaneRequest>,
    write_tx: &mpsc::Sender<String>,
    gui_writers: &GuiWriters,
) {
    // 先尝试解析为通用 JSON Value，从 method 字段判断消息类型。
    // SPEC-005 §1.3.1 §12.2：JSON 解析失败必须返回 -32700 parse_error，不关闭连接。
    // 尽力从原始 bytes 提取 id 字段（parse_error 时 id 可能无法获取，则用 null）。
    let val: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            warn!("failed to parse IPC frame as JSON: {e}");
            // 尝试仅提取 id 字段（容错 partial parse）。
            let fallback_id: serde_json::Value = serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .and_then(|v| v.get("id").cloned())
                .unwrap_or(serde_json::Value::Null);
            write_error_response(
                fallback_id,
                ControlError::new(rpc_codes::PARSE_ERROR, format!("parse error: {e}")),
                write_tx,
            )
            .await;
            return;
        }
    };

    // 有 method 字段 = 通知（Notification）或请求（Request）。
    if let Some(method) = val.get("method").and_then(|v| v.as_str()) {
        let method = method.to_owned();
        let id = val.get("id").cloned();
        let params = val
            .get("params")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        match method.as_str() {
            // ── 单向通知（无 id）────────────────────────────────────
            "sieve.reload_user_rules" => {
                dispatch_notification_line(line, reload_tx).await;
            }
            // ── v2.1 控制面 request（有 id）─────────────────────────
            "sieve.set_paused"
            | "sieve.set_preset"
            | "sieve.set_preset_overrides"
            | "sieve.reload_config"
            | "sieve.health"
            | "sieve.evaluate"
            | "sieve.list_graylist"
            | "sieve.remove_graylist"
            // v2.0+ 兼容扩展（SPEC-005 §11A §11B §11C）
            | "sieve.list_rules"
            | "sieve.purge_history"
            | "sieve.judge_tool_call" => {
                let Some(id) = id else {
                    warn!(method = %method, "control-plane method requires id; treating as notification dropped");
                    return;
                };
                dispatch_control_plane(
                    method.as_str(),
                    params,
                    id,
                    control_tx,
                    write_tx,
                    gui_writers,
                )
                .await;
            }
            other => {
                // 未知 method：有 id 时回 -32601；无 id 时静默忽略。
                if let Some(id) = id {
                    let err = ControlError::new(
                        rpc_codes::METHOD_NOT_FOUND,
                        format!("method not found: {other}"),
                    );
                    write_error_response(id, err, write_tx).await;
                } else {
                    warn!(
                        method = other,
                        "received unknown IPC notification; ignoring"
                    );
                }
            }
        }
        return;
    }

    // 无 method = response（GUI 回复的 decision）。
    let rpc: crate::protocol::jsonrpc::Response = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            warn!("failed to parse IPC response from GUI: {e}");
            return;
        }
    };

    if let Some(err_obj) = &rpc.error {
        error!(
            code = err_obj.code,
            message = %err_obj.message,
            "GUI returned rpc error"
        );
        // SPEC-005 §12.4 / P1-NEW：GUI→daemon error response 按段位处理 pending。
        // -32100~-32199 段：GUI 拒绝/取消某个 pending decision（如 -32100 DecisionRejected）。
        // 应清理对应 pending decision channel，避免泄漏（request_decision 会阻塞直到超时）。
        if (-32199..=-32100).contains(&err_obj.code) {
            // id 字段包含被拒绝的原始 request id（如果 GUI 协议正确实现的话）。
            // 尝试从 rpc.id 解析 Uuid，匹配 pending map。
            let maybe_uuid: Option<uuid::Uuid> =
                rpc.id.as_str().and_then(|s| s.parse().ok()).or_else(|| {
                    rpc.id.as_u64().and(None) // u64 id 不是 Uuid，跳过
                });
            if let Some(request_id) = maybe_uuid {
                let mut map = pending.lock().await;
                if let Some(tx) = map.remove(&request_id) {
                    // 通知等待方：GUI 拒绝/取消，走 fallback（Err -> timeout fallback）。
                    drop(tx); // tx drop → rx.await 返回 Err(RecvError) → fallback
                    warn!(
                        code = err_obj.code,
                        %request_id,
                        "GUI rejected/cancelled pending decision; dropping channel (§12.4)"
                    );
                } else {
                    warn!(
                        code = err_obj.code,
                        %request_id,
                        "GUI returned -32100~99 error but no matching pending request"
                    );
                }
            } else {
                warn!(
                    code = err_obj.code,
                    "GUI returned -32100~99 error without parseable Uuid id; cannot clean pending"
                );
            }
        }
        return;
    }

    if let Some(result) = rpc.result {
        match serde_json::from_value::<DecisionResponse>(result) {
            Ok(resp) => {
                let mut map = pending.lock().await;
                if let Some(tx) = map.remove(&resp.request_id) {
                    let _ = tx.send(resp);
                } else {
                    warn!(
                        request_id = %resp.request_id,
                        "no pending request for this decision"
                    );
                }
            }
            Err(e) => {
                warn!("failed to deserialize DecisionResponse: {e}");
            }
        }
    }
}

/// 控制面 method 路由：反序列化 params → 发 ControlPlaneRequest → 等回执 → 写回 GUI。
///
/// SPEC-005 §10.0.1：set_paused / set_preset / set_preset_overrides 必须先 fan-out 通知
/// 所有 GUI，再返回 result 给请求方。
async fn dispatch_control_plane(
    method: &str,
    params: serde_json::Value,
    id: serde_json::Value,
    control_tx: &mpsc::Sender<ControlPlaneRequest>,
    write_tx: &mpsc::Sender<String>,
    gui_writers: &GuiWriters,
) {
    /// 反序列化 params；缺失时使用 Default 值。
    fn parse_params<T: serde::de::DeserializeOwned + Default>(
        v: serde_json::Value,
    ) -> Result<T, ControlError> {
        if v.is_null() {
            return Ok(T::default());
        }
        serde_json::from_value(v).map_err(|e| ControlError::invalid_params(e.to_string()))
    }

    /// 反序列化 params；缺失时报 invalid_params。
    fn require_params<T: serde::de::DeserializeOwned>(
        v: serde_json::Value,
    ) -> Result<T, ControlError> {
        if v.is_null() {
            return Err(ControlError::invalid_params("params required"));
        }
        serde_json::from_value(v).map_err(|e| ControlError::invalid_params(e.to_string()))
    }

    /// 从 JSON-RPC `id` 字段尝试解析 UUID（SPEC-005 §10.0.2 origin_request_id 透传）。
    ///
    /// GUI 发送的 `id` 通常是 UUID 字符串；整数 `id` 时返回 `None`（无法映射为 UUID）。
    fn extract_origin_uuid(id: &serde_json::Value) -> Option<Uuid> {
        id.as_str()?.parse().ok()
    }

    match method {
        "sieve.set_paused" => {
            let p: SetPausedRequest = match require_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let origin_request_id = extract_origin_uuid(&id);
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::SetPaused {
                    params: p,
                    origin_request_id,
                    reply,
                })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            // SPEC-005 §10.0.1：先 fan-out 再写 result。
            forward_reply_with_broadcast::<SetPausedResult>(id, rx, write_tx, gui_writers).await;
        }
        "sieve.set_preset" => {
            let p: SetPresetRequest = match require_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let origin_request_id = extract_origin_uuid(&id);
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::SetPreset {
                    params: p,
                    origin_request_id,
                    reply,
                })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            // SPEC-005 §10.0.1：先 fan-out 再写 result。
            forward_reply_with_broadcast::<SetPresetResult>(id, rx, write_tx, gui_writers).await;
        }
        "sieve.set_preset_overrides" => {
            let p: SetPresetOverridesRequest = match parse_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let origin_request_id = extract_origin_uuid(&id);
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::SetPresetOverrides {
                    params: p,
                    origin_request_id,
                    reply,
                })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            // SPEC-005 §10.0.1：先 fan-out 再写 result。
            forward_reply_with_broadcast::<SetPresetOverridesResult>(id, rx, write_tx, gui_writers)
                .await;
        }
        "sieve.reload_config" => {
            let p: ReloadConfigRequest = match parse_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::ReloadConfig { params: p, reply })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            forward_reply::<ReloadConfigResult>(id, rx, write_tx).await;
        }
        "sieve.health" => {
            let p: HealthRequest = match parse_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::Health { params: p, reply })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            forward_reply::<HealthResult>(id, rx, write_tx).await;
        }
        "sieve.evaluate" => {
            let p: EvaluateRequest = match require_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            // payload 上限 64KB（ADR-013 §S.4）。
            const PAYLOAD_LIMIT: usize = 64 * 1024;
            if p.payload.len() > PAYLOAD_LIMIT {
                return write_error_response(
                    id,
                    ControlError::payload_too_large(format!(
                        "payload {} bytes exceeds {PAYLOAD_LIMIT} byte limit",
                        p.payload.len()
                    )),
                    write_tx,
                )
                .await;
            }
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::Evaluate { params: p, reply })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            forward_reply::<EvaluateResult>(id, rx, write_tx).await;
        }
        "sieve.list_graylist" => {
            let p: ListGraylistRequest = match parse_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::ListGraylist { params: p, reply })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            forward_reply::<ListGraylistResult>(id, rx, write_tx).await;
        }
        "sieve.remove_graylist" => {
            let p: RemoveGraylistRequest = match require_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::RemoveGraylist { params: p, reply })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            forward_reply::<RemoveGraylistResult>(id, rx, write_tx).await;
        }
        // SPEC-005 §11A：sieve.list_rules（v2.0+ 兼容扩展）。
        "sieve.list_rules" => {
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::ListRules { reply })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            forward_reply::<ListRulesResult>(id, rx, write_tx).await;
        }
        // SPEC-005 §11B：sieve.purge_history（v2.0+ 兼容扩展）。
        "sieve.purge_history" => {
            let p: PurgeHistoryRequest = match require_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::PurgeHistory { params: p, reply })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            forward_reply::<PurgeHistoryResult>(id, rx, write_tx).await;
        }
        // SPEC-005 §11C：sieve.judge_tool_call（Since v2.x 向后兼容扩展）。
        "sieve.judge_tool_call" => {
            let p: JudgeToolCallRequest = match require_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::JudgeToolCall { params: p, reply })
                .await
                .is_err()
            {
                return write_error_response(
                    id,
                    ControlError::internal("daemon control channel closed"),
                    write_tx,
                )
                .await;
            }
            forward_reply::<JudgeToolCallResult>(id, rx, write_tx).await;
        }
        other => {
            // 不会到达此分支（外层 match 已穷举）。
            write_error_response(
                id,
                ControlError::new(
                    rpc_codes::METHOD_NOT_FOUND,
                    format!("control-plane method not implemented: {other}"),
                ),
                write_tx,
            )
            .await;
        }
    }
}

/// 等待 daemon 通过 oneshot 返回回执，序列化为 JSON-RPC response 写回 GUI。
async fn forward_reply<T: serde::Serialize>(
    id: serde_json::Value,
    rx: oneshot::Receiver<Result<T, ControlError>>,
    write_tx: &mpsc::Sender<String>,
) {
    match rx.await {
        Ok(Ok(value)) => match serde_json::to_value(&value) {
            Ok(result) => write_success_response(id, result, write_tx).await,
            Err(e) => {
                write_error_response(
                    id,
                    ControlError::internal(format!("serialize result failed: {e}")),
                    write_tx,
                )
                .await;
            }
        },
        Ok(Err(err)) => write_error_response(id, err, write_tx).await,
        Err(_) => {
            // daemon 端 reply sender 被 drop 而未 send（handler panic / 提前退出）。
            write_error_response(
                id,
                ControlError::internal("daemon dropped reply without responding"),
                write_tx,
            )
            .await;
        }
    }
}

/// SPEC-005 §10.0.1：mutating request（set_paused / set_preset / set_preset_overrides）专用。
///
/// 等待 daemon 回执（携带 `BroadcastPlan`），先 fan-out 所有 GUI 通知（send 等待入队），
/// 再向请求方写 result response。保证 result 发出前所有 GUI 已入队通知。
async fn forward_reply_with_broadcast<T: serde::Serialize>(
    id: serde_json::Value,
    rx: oneshot::Receiver<Result<(T, Option<BroadcastPlan>), ControlError>>,
    write_tx: &mpsc::Sender<String>,
    gui_writers: &GuiWriters,
) {
    match rx.await {
        Ok(Ok((value, broadcast_plan))) => {
            // Step 1：执行 fan-out（先广播，后 result）。
            if let Some(plan) = broadcast_plan {
                let (method, payload) = match plan {
                    BroadcastPlan::PausedChanged(notify) => {
                        ("sieve.paused_changed", serde_json::to_value(&notify).ok())
                    }
                    BroadcastPlan::PresetChanged(notify) => {
                        ("sieve.preset_changed", serde_json::to_value(&notify).ok())
                    }
                };
                if let Some(params) = payload {
                    let notification = crate::protocol::jsonrpc::Request {
                        jsonrpc: "2.0".to_owned(),
                        method: method.to_owned(),
                        params: Some(params),
                        id: None,
                    };
                    if let Ok(mut frame) = serde_json::to_string(&notification) {
                        frame.push('\n');
                        // fan-out：send（await）到所有已连接 GUI。
                        let senders: Vec<mpsc::Sender<String>> =
                            gui_writers.lock().map(|g| g.clone()).unwrap_or_default();
                        for sender in &senders {
                            // send 是 await（非 try_send），保证通知入队后再写 result。
                            // 通道关闭（GUI 已断线）时忽略错误。
                            let _ = sender.send(frame.clone()).await;
                        }
                        debug!(
                            method,
                            gui_count = senders.len(),
                            "fan-out broadcast before result (§10.0.1)"
                        );
                    }
                }
            }
            // Step 2：写 result 给请求方。
            match serde_json::to_value(&value) {
                Ok(result) => write_success_response(id, result, write_tx).await,
                Err(e) => {
                    write_error_response(
                        id,
                        ControlError::internal(format!("serialize result failed: {e}")),
                        write_tx,
                    )
                    .await;
                }
            }
        }
        Ok(Err(err)) => write_error_response(id, err, write_tx).await,
        Err(_) => {
            write_error_response(
                id,
                ControlError::internal("daemon dropped reply without responding"),
                write_tx,
            )
            .await;
        }
    }
}

async fn write_success_response(
    id: serde_json::Value,
    result: serde_json::Value,
    write_tx: &mpsc::Sender<String>,
) {
    let resp = crate::protocol::jsonrpc::Response {
        jsonrpc: "2.0".to_owned(),
        result: Some(result),
        error: None,
        id,
    };
    write_jsonrpc_frame(resp, write_tx).await;
}

async fn write_error_response(
    id: serde_json::Value,
    err: ControlError,
    write_tx: &mpsc::Sender<String>,
) {
    let resp = crate::protocol::jsonrpc::Response {
        jsonrpc: "2.0".to_owned(),
        result: None,
        error: Some(crate::protocol::jsonrpc::ErrorObject {
            code: err.code,
            message: err.message,
            data: err.data,
        }),
        id,
    };
    write_jsonrpc_frame(resp, write_tx).await;
}

async fn write_jsonrpc_frame(
    resp: crate::protocol::jsonrpc::Response,
    write_tx: &mpsc::Sender<String>,
) {
    let mut payload = match serde_json::to_string(&resp) {
        Ok(s) => s,
        Err(e) => {
            warn!("failed to serialize JSON-RPC response: {e}");
            return;
        }
    };
    payload.push('\n');
    if write_tx.send(payload).await.is_err() {
        debug!("write_tx closed; control-plane response dropped");
    }
}

/// 解析 `sieve.reload_user_rules` 通知并发送到 reload channel。
async fn dispatch_notification_line(line: &str, reload_tx: &mpsc::Sender<ReloadUserRules>) {
    let val: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            warn!("failed to parse reload notification: {e}");
            return;
        }
    };

    let params = val
        .get("params")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let reload: ReloadUserRules = if params.is_null() {
        ReloadUserRules::default()
    } else {
        match serde_json::from_value(params) {
            Ok(r) => r,
            Err(e) => {
                warn!("failed to deserialize ReloadUserRules params: {e}");
                ReloadUserRules::default()
            }
        }
    };

    if let Err(_e) = reload_tx.send(reload).await {
        warn!("reload channel closed; dropping reload notification");
    } else {
        debug!("reload_user_rules notification dispatched to daemon");
    }
}

/// 判断 accept() 错误是否为连接级瞬态错误（对端在 accept 完成前断开 / reset / refuse）。
///
/// 这类错误立即重试即可、无需退避；其余错误（典型 EMFILE/ENFILE fd 耗尽）走退避重试。
/// 无论哪类都**不**终止 accept 循环——见 [`IpcServer::run`]。参照 hyper server 的分类。
fn is_connection_error(e: &std::io::Error) -> bool {
    use std::io::ErrorKind;
    matches!(
        e.kind(),
        ErrorKind::ConnectionRefused | ErrorKind::ConnectionAborted | ErrorKind::ConnectionReset
    )
}

/// 构造 `sieve.heartbeat` 通知帧（换行结尾，SPEC-005 §4）。
///
/// 格式：`{"jsonrpc":"2.0","method":"sieve.heartbeat"}\n`，无 params 字段。
fn heartbeat_frame() -> String {
    "{\"jsonrpc\":\"2.0\",\"method\":\"sieve.heartbeat\"}\n".to_owned()
}

fn make_timeout_fallback(
    request_id: Uuid,
    default_on_timeout: DefaultOnTimeout,
) -> DecisionResponse {
    let action = match default_on_timeout {
        DefaultOnTimeout::Block => DecisionAction::Deny,
        DefaultOnTimeout::Allow => DecisionAction::Allow,
        DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
    };
    DecisionResponse {
        request_id,
        decision: action,
        decided_at: Utc::now(),
        by_user: false,
        remember: false,
        context_hint: None,
        ui_phase_when_clicked: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tempfile::tempdir;

    fn dummy_request() -> DecisionRequest {
        DecisionRequest {
            request_id: Uuid::now_v7(),
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![],
            source_agent: Default::default(),
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        }
    }

    /// accept 错误分类（修复 IPC accept 单点：单次 accept 错误绝不 break 整个 accept
    /// 循环——否则瞬态 fd 耗尽即永久击穿控制面 daemon）。连接级瞬态错误（对端在 accept
    /// 完成前断开 / reset / refuse）→ 立即重试，不退避。
    #[test]
    fn connection_level_accept_errors_are_immediate_retry() {
        use std::io::{Error, ErrorKind};
        for kind in [
            ErrorKind::ConnectionRefused,
            ErrorKind::ConnectionAborted,
            ErrorKind::ConnectionReset,
        ] {
            assert!(is_connection_error(&Error::from(kind)), "{kind:?}");
        }
    }

    /// 资源类 / 其他 accept 错误（典型 EMFILE/ENFILE fd 耗尽）不算连接级，走退避
    /// 重试避免 busy-loop——但同样**不会** break 循环。
    #[test]
    fn resource_accept_errors_take_backoff_path() {
        use std::io::{Error, ErrorKind};
        // EMFILE = 24（POSIX 通用），稳定 Rust 无专门 ErrorKind，归为非连接级 → 退避。
        assert!(!is_connection_error(&Error::from_raw_os_error(24)));
        for kind in [ErrorKind::PermissionDenied, ErrorKind::Other] {
            assert!(!is_connection_error(&Error::from(kind)), "{kind:?}");
        }
    }

    /// 回归测试 P2-R10-#4：GUI 写队列已满时 `request_decision` 必须立即 fallback，
    /// 不能用 `send().await` 把 hot path 阻塞到 timeout 到期。
    #[tokio::test]
    async fn request_decision_does_not_block_when_writer_full() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (server, _listener) = IpcServer::bind(socket_path).expect("bind");

        // 容量 1 的 mpsc，预先填满模拟 GUI 写队列背压。
        let (tx, _rx) = mpsc::channel::<String>(1);
        tx.try_send("占位".to_owned()).expect("first slot");
        assert!(
            matches!(
                tx.try_send("满".to_owned()),
                Err(mpsc::error::TrySendError::Full(_))
            ),
            "队列应已满"
        );
        server
            .gui_writers
            .lock()
            .expect("gui_writers lock")
            .push(tx);

        // timeout 给到 60s——若实现回退到 send().await 会卡满 60s，测试会超时。
        let req = dummy_request();
        let start = Instant::now();
        let resp = server
            .request_decision(req, Duration::from_secs(60), "inbound")
            .await
            .expect("request_decision");
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(1),
            "队列满时应立即降级，实际耗时 {elapsed:?}"
        );
        assert_eq!(resp.decision, DecisionAction::Deny);
        assert!(!resp.by_user, "fallback 不应标记为 by_user");
    }

    /// SPEC-005 §3：连接建立后第一条出站消息必须是 sieve.hello，含全部 7 个必填字段。
    #[tokio::test]
    async fn hello_is_first_message_after_connect() {
        use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};
        use tokio::net::UnixStream;

        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (server, listener) = IpcServer::bind(socket_path.clone()).expect("bind");
        let server = Arc::new(server);

        let boot_id = Uuid::now_v7();
        let started_at = Utc::now();
        server
            .set_hello_builder(HelloBuilder {
                daemon_boot_id: boot_id,
                daemon_version: "0.0.0-test".to_owned(),
                audit_db_user_version: 2,
                started_at,
                preset: "default".to_owned(),
            })
            .await;

        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });

        // 给 server 一点时间启动。
        tokio::time::sleep(Duration::from_millis(20)).await;

        // 客户端连接并读第一帧。
        let stream = UnixStream::connect(&socket_path).await.expect("connect");
        let mut lines = TokioBufReader::new(stream).lines();
        let first_line = tokio::time::timeout(Duration::from_secs(2), lines.next_line())
            .await
            .expect("timeout waiting for sieve.hello")
            .expect("io error")
            .expect("connection closed before hello");

        let val: serde_json::Value =
            serde_json::from_str(&first_line).expect("hello must be valid JSON");
        assert_eq!(val["jsonrpc"], "2.0");
        assert_eq!(
            val["method"], "sieve.hello",
            "첫 번째 메시지는 sieve.hello 여야 함"
        );
        let params = val.get("params").expect("hello must have params");
        assert_eq!(params["protocol_version"], "v2");
        assert!(params.get("daemon_version").is_some());
        assert!(params.get("paused").is_some());
        assert!(params.get("preset").is_some());
        assert!(params.get("uptime_seconds").is_some());
        assert!(params.get("audit_db_user_version").is_some());
        assert_eq!(
            params["daemon_boot_id"],
            boot_id.to_string(),
            "daemon_boot_id 必须与注入值一致"
        );
    }

    /// SPEC-005 §4：heartbeat_frame 必须是合法 JSON-RPC 通知 + method = sieve.heartbeat + 无 params。
    #[test]
    fn heartbeat_frame_format() {
        let frame = heartbeat_frame();
        assert!(frame.ends_with('\n'), "heartbeat frame 必须以换行结尾");
        let val: serde_json::Value =
            serde_json::from_str(frame.trim_end()).expect("heartbeat frame 必须是合法 JSON");
        assert_eq!(val["jsonrpc"], "2.0", "jsonrpc 字段必须为 2.0");
        assert_eq!(
            val["method"], "sieve.heartbeat",
            "method 必须为 sieve.heartbeat"
        );
        assert!(val.get("params").is_none(), "heartbeat 不应有 params 字段");
        assert!(val.get("id").is_none(), "heartbeat 是通知，不应有 id 字段");
    }

    /// SPEC-005 §1.1：bind 后 socket 文件权限必须为 0600。
    #[tokio::test]
    async fn bind_sets_socket_permissions_0600() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (_server, _listener) = IpcServer::bind(socket_path.clone()).expect("bind");
        let meta = std::fs::metadata(&socket_path).expect("metadata");
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "socket 文件应为 0600，实际为 {mode:o}");
    }

    /// 审查 §6：gui_writers 锁毒化恢复依赖 `PoisonError::into_inner()` 保留 Vec 数据。
    /// 本测试固化该 idiom 语义——持锁线程 panic 毒化 Mutex 后，`into_inner` 仍取回完整
    /// 数据，守护 478/587/666 三处从 `.expect()`（毒化时 panic = DoS）改为毒化恢复的
    /// 正确性前提。
    #[test]
    fn poisoned_mutex_into_inner_recovers_data() {
        use std::sync::{Arc, Mutex};
        let m: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![7, 8, 9]));
        let m2 = Arc::clone(&m);
        let _ = std::thread::spawn(move || {
            let _g = m2.lock().expect("first lock");
            panic!("poison the mutex on purpose");
        })
        .join();
        assert!(m.lock().is_err(), "持锁线程 panic 后 Mutex 必须处于毒化态");
        let recovered = m.lock().unwrap_or_else(|p| p.into_inner());
        assert_eq!(
            &*recovered,
            &[7, 8, 9],
            "into_inner 必须取回完整 Vec（重构所依赖的恢复语义）"
        );
    }
}
