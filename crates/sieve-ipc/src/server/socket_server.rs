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

/// GUI peer 代码签名核验回调（F1-b，SPEC-005 §6.2.4）。
///
/// daemon 层注入（macOS 上用 Security framework 对连接对端进程做代码签名核验），
/// 参数为连接的 raw fd，返回 `true` = 对端通过核验。与 [`OversizeCallback`] 同理，
/// 通过回调反转依赖，sieve-ipc 不引入 FFI。
///
/// **未注入 = gate 关闭**（源码构建 / dogfood 场景无签名信任锚可用；残余风险见
/// SPEC-005 §6.4）。注入后仅作用于 wire 应答放行 Critical 的路径；`inject_decision`
/// 测试注入路径与 `resolve_decision`（自有 A 方案门禁）不经此 gate。
pub type PeerVerifier = Arc<dyn Fn(std::os::unix::io::RawFd) -> bool + Send + Sync>;

/// 连接级 peer 核验状态：懒执行 + 缓存（每连接至多真验一次）。
///
/// 核验只在该连接首次尝试放行 Critical 时触发，避免对 CLI 短连接 / 心跳连接
/// 做无谓的签名检查。fd 在 handle_connection 存活期间有效（split 两半共享同一 socket）。
struct PeerGate {
    verifier: Option<PeerVerifier>,
    raw_fd: std::os::unix::io::RawFd,
    verdict: std::sync::OnceLock<bool>,
}

impl PeerGate {
    fn new(verifier: Option<PeerVerifier>, raw_fd: std::os::unix::io::RawFd) -> Self {
        Self {
            verifier,
            raw_fd,
            verdict: std::sync::OnceLock::new(),
        }
    }

    /// 本连接是否允许放行 Critical（allow / redact_and_allow）决策应答。
    fn permits_critical_allow(&self) -> bool {
        match &self.verifier {
            None => true, // gate 未启用
            Some(verify) => *self.verdict.get_or_init(|| verify(self.raw_fd)),
        }
    }
}

use crate::{
    error::{rpc_codes, IpcError},
    protocol::{
        CancelReason, DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout,
        EvaluateRequest, EvaluateResult, HealthRequest, HealthResult, JudgeToolCallRequest,
        JudgeToolCallResult, ListGraylistRequest, ListGraylistResult, ListPendingRequest,
        ListPendingResult, ListRulesResult, MergedDecisionResponse, PausedChangedNotify,
        PendingDetectionSummary, PendingSnapshot, PresetChangedNotify, PurgeHistoryRequest,
        PurgeHistoryResult, ReloadConfigRequest, ReloadConfigResult, ReloadUserRules,
        RemoveGraylistRequest, RemoveGraylistResult, RequestDecisionCanceledNotify,
        ResolveDecisionRequest, ResolveDecisionResult, ResolveStatus, SetPausedRequest,
        SetPausedResult, SetPresetOverridesRequest, SetPresetOverridesResult, SetPresetRequest,
        SetPresetResult, Severity, StatusBarNotify,
    },
};

/// 单条待决策在 pending map 中的条目。
///
/// 除 oneshot 应答端外，携带 daemon 侧计算的元数据：`max_severity`（A 方案授权门禁
/// 依据，**不信客户端自报**）与 `snapshot`（`list_pending` 只读投影）。
///
/// 关联：F1-a 决策授权门禁基础、SPEC-005 §11D / §11E。
struct PendingEntry {
    /// GUI / CLI 回复决策的 oneshot 发送端。
    responder: oneshot::Sender<DecisionResponse>,
    /// 本次请求涉及的最高严重等级（daemon 侧从 detections 计算）。
    ///
    /// A 方案：`resolve_decision` 对 `Critical` 类的 `allow` / `redact_and_allow`
    /// 静默改写为 deny，判定据此字段，**不信客户端传来的 severity**。
    max_severity: Severity,
    /// `list_pending` 只读投影（不含 responder / oneshot 运行时对象）。
    snapshot: Arc<PendingSnapshot>,
}

/// pending map：request_id → [`PendingEntry`]，等待 GUI / CLI 回复。
type PendingMap = Arc<Mutex<HashMap<Uuid, PendingEntry>>>;

/// 从 `DecisionRequest` 计算本次请求涉及的最高严重等级。
///
/// 空 detections（异常降级）视为 `Low`（保守：无命中不应触发 Critical 门禁）。
fn compute_max_severity(req: &DecisionRequest) -> Severity {
    req.detections
        .iter()
        .map(|d| d.severity)
        .max_by_key(|s| s.rank())
        .unwrap_or(Severity::Low)
}

/// 从 `DecisionRequest` 裁剪出 `list_pending` 只读投影。
///
/// `age_seconds` 在 daemon 应答 `list_pending` 时按 `created_at` 现算，此处填 0。
fn build_pending_snapshot(
    req: &DecisionRequest,
    max_severity: Severity,
    direction: &str,
    provider_id: Option<&str>,
) -> PendingSnapshot {
    PendingSnapshot {
        request_id: req.request_id,
        max_severity,
        detections: req
            .detections
            .iter()
            .map(|d| PendingDetectionSummary {
                rule_id: d.rule_id.clone(),
                severity: d.severity,
                title: d.title.clone(),
                one_line_summary: d.one_line_summary.clone(),
            })
            .collect(),
        timeout_seconds: req.timeout_seconds,
        default_on_timeout: req.default_on_timeout,
        direction: direction.to_owned(),
        source_agent: req.source_agent,
        provider_id: provider_id.map(str::to_owned),
        created_at: req.created_at,
        age_seconds: 0,
    }
}

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
/// 关联：JSON-RPC 错误码段。
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
/// 关联：IPC 控制面 dispatch 协议。
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
    /// SPEC-005 §11D `sieve.list_pending`（Since v2.x 兼容扩展，只读）。
    ///
    /// headless 枚举当前待决策快照；daemon handler 直接读 pending map，快速无阻塞。
    ListPending {
        params: ListPendingRequest,
        reply: oneshot::Sender<Result<ListPendingResult, ControlError>>,
    },
    /// SPEC-005 §11E `sieve.resolve_decision`（Since v2.x 兼容扩展，A 方案授权）。
    ///
    /// headless 解决单个待决策；daemon handler 按 daemon 侧 `max_severity` 门禁
    /// （Critical 静默 deny），快速无阻塞（不等 GUI）。
    ResolveDecision {
        params: ResolveDecisionRequest,
        reply: oneshot::Sender<Result<ResolveDecisionResult, ControlError>>,
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
/// 关联：多 GUI 客户端支持。
type GuiWriters = Arc<std::sync::Mutex<Vec<mpsc::Sender<String>>>>;

/// IPC 服务端，监听 Unix socket，维护与 GUI 的长连接并推送决策请求。
///
/// # 连接语义（v2.1 多 GUI 客户端）
///
/// - GUI 启动后主动连接此 socket，保持长连接。
/// - 支持多个并发 GUI 连接（如 sieve-gui-macos + sieve doctor 同时运行）。
/// - GUI 断线后 `gui_writers` 中对应 sender 在下次 broadcast 时自动清理。
/// - `request_decision` fan-out 给**所有** live writer，首个 `decision_response` 生效
///   （首答胜出），其余 client 收 `sieve.request_decision_canceled`（`resolved_by_peer`）。
///   历史教训（BUG-L1c）：只投单个 writer 时，「channel 未关但对端已不消费」的僵尸连接
///   会吞掉决策帧——其余 live GUI 永不弹窗，daemon 只能等满 timeout 后 fail-closed。
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
/// `request_decision` 遍历 `gui_writers` 快照，把请求帧 fan-out 投递给所有 live writer
/// （SPEC-005 §1.4：pending 是单 oneshot，首答胜出；解决后广播 `resolved_by_peer` 收口）。
///
/// # 单向通知（v2.1）
///
/// - `broadcast_status_bar`：daemon 向**所有**已连接 GUI 广播状态栏通知。
///   `TrySendError::Closed` 的 sender 立即从 Vec 移除（lazy 清理）；
///   `TrySendError::Full` 视为短暂背压，保留 sender 不断线。
///   失败（无客户端 / socket 写错）静默丢弃 + debug 日志，**daemon 主流程不阻塞**。
///   关联：行为序列 StatusBar 通知 + 多 GUI 客户端支持。
/// - `reload_rx`：daemon 通过此 channel 接收来自 `sieve rules edit` 的 reload 通知。
///   关联：编辑器关闭后 lint + atomic backup + IPC reload 流程。
///
/// 关联：JSON-RPC over Unix socket 传输 + 双层防御 GUI 路径。
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
    /// 关联：set_paused / SPEC-002 §9.1。
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
    /// GUI peer 代码签名核验回调（F1-b）。
    ///
    /// daemon 层通过 [`Self::set_peer_verifier`] 注入；未注入 = gate 关闭。
    peer_verifier: Arc<std::sync::Mutex<Option<PeerVerifier>>>,
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
            peer_verifier: Arc::new(std::sync::Mutex::new(None)),
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
    /// 关联：sieve.reload_config 控制面方法。
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

    /// 注入 GUI peer 代码签名核验回调（F1-b，SPEC-005 §6.2.4）。
    ///
    /// daemon 启动时按配置调用一次；注入后每个连接首次尝试放行 Critical 决策应答时
    /// 触发核验（结果按连接缓存）。核验未通过 → 该应答的 allow / redact_and_allow
    /// 静默改写为 deny（与 `resolve_decision` 的 A 方案同范式）。
    pub fn set_peer_verifier(&self, verifier: PeerVerifier) {
        if let Ok(mut guard) = self.peer_verifier.lock() {
            *guard = Some(verifier);
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
    /// 关联：编辑器关闭后 lint + atomic backup + IPC reload 流程。
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
                    // peer 核验回调快照（F1-b）。
                    let peer_verifier = self.peer_verifier.lock().ok().and_then(|g| g.clone());

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
                            peer_verifier,
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
    /// 关联：行为序列 StatusBar 通知 + 多 GUI 客户端支持。
    pub fn broadcast_status_bar(&self, notify: StatusBarNotify) {
        let label = format!("status_bar notify_id={}", notify.notify_id);
        self.broadcast_method("sieve.notify_status_bar", &notify, &label);
    }

    /// 向**所有**已连接的 GUI 广播 preset 变更通知。
    ///
    /// 关联：SPEC-002 §9.2。
    pub fn broadcast_preset_changed(&self, notify: PresetChangedNotify) {
        let label = format!("preset_changed mode={}", notify.mode);
        self.broadcast_method("sieve.preset_changed", &notify, &label);
    }

    /// 向**所有**已连接的 GUI 广播 paused 状态变更通知。
    ///
    /// 关联：SPEC-002 §9.1。
    pub fn broadcast_paused_changed(&self, notify: PausedChangedNotify) {
        let label = format!("paused_changed paused={}", notify.paused);
        self.broadcast_method("sieve.paused_changed", &notify, &label);
    }

    /// 向**所有**已连接的 GUI 广播 request_decision 取消通知。
    ///
    /// 关联：SPEC-002 §9.3 / §9.4。
    pub fn broadcast_request_decision_canceled(&self, notify: RequestDecisionCanceledNotify) {
        let label = format!("request_decision_canceled request_id={}", notify.request_id);
        self.broadcast_method("sieve.request_decision_canceled", &notify, &label);
    }

    /// 通用 fan-out 广播（所有 broadcast_* 方法的共用实现）。
    ///
    /// 委托给自由函数 [`broadcast_method_to`]——后者不依赖 `&self`，供
    /// [`PendingCleanupGuard`] 在 `request_decision` future 被 cancel 后仍能广播取消通知。
    fn broadcast_method<T: serde::Serialize>(&self, method: &str, params: &T, label: &str) {
        broadcast_method_to(&self.gui_writers, method, params, label);
    }

    /// 向已连接的 GUI 发送决策请求，等待响应或超时。
    ///
    /// `direction` 参数：`"inbound"` / `"outbound"`，调用方根据流量方向传入。
    /// wire DTO 层（SPEC-005 §6.0）用此值填充 `sieve.request_decision` 的 `direction` 字段。
    ///
    /// # 行为
    ///
    /// - 请求帧 **fan-out** 给所有 live writer（SPEC-005 §1.4，BUG-L1c 修复）；
    ///   首个 `decision_response` 生效，解决后广播 `resolved_by_peer` 收口其余 client。
    /// - 如果没有 GUI 客户端连接：**立即 fallback**，不等超时。
    ///   （等超时无意义——没人能决策。）
    /// - 如果所有 writer 都不可投递（`try_send` 全部 Full/Closed）：**立即 fallback**，
    ///   不阻塞 SSE pipeline。**不用 `send().await`**——队列满会把 hot path 卡死直到
    ///   timeout 到期，期间整个 SSE 连接 hold 住，对用户体验而言相当于 daemon 死锁
    ///   （known-issues-v1.4 P2-R10-#4）。
    /// - 如果 GUI 在 `timeout` 内回复：返回 GUI 的决策。
    /// - 如果超时：按 `default_on_timeout` 构造兜底响应，并从 pending map 清理。
    /// - 本 future 被调用方 cancel（HTTP 客户端断连）：[`PendingCleanupGuard`] 兜底
    ///   清理 pending 条目 + 广播 `upstream_disconnected`（BUG-L1b 修复）。
    pub async fn request_decision(
        &self,
        req: DecisionRequest,
        timeout: Duration,
        direction: &str,
        provider_id: Option<&str>,
    ) -> Result<DecisionResponse, IpcError> {
        // SPEC-005 §6.1.1：wire 字段 `timeout_seconds` 必须落在 [30, 120]。client（GUI）对该
        // 字段无下限校验——下发 0 会让倒计时首 tick 即归零（异常/静默关窗），下发越界大值则
        // 进度条失真。所有 request_decision 发送都经过本方法这唯一 choke point，故在此钳制即可
        // 保证 wire 契约对所有构造路径成立（含多 issue 合并取最小值后越界的情形）。
        // 钳制而非拒绝：拒绝会中断决策流→可能 fail-open；钳到 SPEC 下限 30s 保证 GUI 可用。（D-3）
        let mut req = req;
        let clamped_timeout = req.timeout_seconds.clamp(30, 120);
        if clamped_timeout != req.timeout_seconds {
            warn!(
                request_id = %req.request_id,
                original = req.timeout_seconds,
                clamped = clamped_timeout,
                "timeout_seconds out of SPEC-005 §6.1.1 range [30,120]; clamped before wire send"
            );
            req.timeout_seconds = clamped_timeout;
        }

        let request_id = req.request_id;
        let default_on_timeout = req.default_on_timeout;

        // 1. 取所有已连接 GUI writer 的快照。决策请求 **fan-out** 给所有 live writer
        //    （SPEC-005 §1.4）：pending 是单 oneshot，首个 decision_response 生效（首答
        //    胜出——应答路径按 request_id 从 map remove，天然幂等）；解决后广播
        //    request_decision_canceled(resolved_by_peer) 让其余 client 收回决策 UI。
        //
        //    历史教训（两层递进）：
        //    - 旧实现只取 writers.first()，index 0 是 stale sender（Closed）时直接
        //      fallback deny，从不尝试 live writer → 误 fail-closed；
        //    - 改为「发给第一个 try_send 成功的 writer」仍不够（BUG-L1c 真机实证）：
        //      GUI 重启过渡期新旧连接混存，「channel 未关但对端已不消费」的僵尸连接
        //      try_send 照样成功、帧被吞 → live GUI 永不弹窗，daemon 等满 timeout 后
        //      fail-closed。try_send 成功 ≠ 对端会处理，唯一可靠语义是 fan-out。
        let writers_snapshot: Vec<_> = {
            // 毒化恢复：持锁线程 panic 不破坏 Vec 结构，into_inner 取出数据继续。
            let writers = self
                .gui_writers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            writers.clone()
        };

        if writers_snapshot.is_empty() {
            // 没有 GUI——立即 fallback，不消耗超时时间。
            debug!(%request_id, "no GUI client connected; immediate fallback");
            return Ok(make_timeout_fallback(request_id, default_on_timeout));
        }

        // 2. 注册 oneshot channel，等待 GUI 回复。
        //    F1-a：pending entry 携带 daemon 侧计算的 max_severity（A 方案授权门禁依据，
        //    不信客户端自报）+ list_pending 只读投影。
        let max_severity = compute_max_severity(&req);
        let snapshot = Arc::new(build_pending_snapshot(
            &req,
            max_severity,
            direction,
            provider_id,
        ));
        let (tx, rx) = oneshot::channel::<DecisionResponse>();
        {
            let mut map = self.pending.lock().await;
            map.insert(
                request_id,
                PendingEntry {
                    responder: tx,
                    max_severity,
                    snapshot,
                },
            );
        }

        // BUG-L1b：pending 注册后立即武装 RAII 兜底。本 future 被调用方 cancel（HTTP 客户端
        // 断连 → 连接任务 abort）时，下方 timeout 分支永不执行，guard 的 Drop 负责从
        // pending map 移除条目 + 广播 upstream_disconnected 取消通知。所有正常退出路径
        // 显式 disarm。
        let mut cleanup_guard = PendingCleanupGuard::new(
            Arc::clone(&self.pending),
            Arc::clone(&self.gui_writers),
            request_id,
            default_on_timeout,
        );

        // 3. 通过 wire DTO 适配层序列化请求（SPEC-005 §6.0 / §6.1 / §6.1.1 / §6.1.2）。
        //    P1-5 + P2-2 + P2-4：使用 RequestDecisionWireKind 替换内部 struct 直接序列化。
        let wire = crate::wire::RequestDecisionWireKind::from_request(&req, direction, provider_id);
        let wire_params = wire.to_value()?;
        let rpc_req = crate::protocol::jsonrpc::Request::call(
            "sieve.request_decision",
            wire_params,
            serde_json::Value::String(request_id.to_string()),
        );
        let mut payload = serde_json::to_string(&rpc_req)?;
        payload.push('\n');

        // 4. fan-out：遍历 live writer 逐个 try_send，投给**所有**成功的 writer
        //    （BUG-L1c 修复；SPEC-005 §1.4）。try_send 而非 send().await：避免 mpsc
        //    队列满阻塞 hot path。Full = live 但忙 → 本次决策收不到帧（无法应答），
        //    保留 sender 不断线；Closed = 已断 → 记下待清理。
        let mut sent_count = 0usize;
        let mut had_closed = false;
        for sender in &writers_snapshot {
            match sender.try_send(payload.clone()) {
                Ok(()) => sent_count += 1,
                Err(mpsc::error::TrySendError::Full(_)) => {
                    warn!(%request_id, "a GUI writer channel full (backpressure); skipping this writer for fan-out");
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    had_closed = true;
                }
            }
        }

        // 主动清理本轮发现的 stale sender（不再等下次 broadcast lazy 清理，避免后续请求反复
        // 打死同一 sender）。retain 幂等清掉所有 receiver 已 drop 的 sender。
        if had_closed {
            let mut writers = self
                .gui_writers
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            writers.retain(|s| !s.is_closed());
        }

        if sent_count == 0 {
            // 无任何 live writer 可投递（全部 Closed/Full）——fail-closed fallback，保持核心
            // 安全不变量：真无人在线 → 按 default_on_timeout（Block→Deny）兜底。
            cleanup_guard.disarm();
            self.pending.lock().await.remove(&request_id);
            warn!(%request_id, "no usable GUI writer (all closed/full); immediate fallback");
            return Ok(make_timeout_fallback(request_id, default_on_timeout));
        }

        // 5. 等待首个回复或超时。
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(resp)) => {
                // 首答生效（wire 应答 / inject_decision / resolve_decision 殊途同归）。
                // fan-out 收口：广播 resolved_by_peer 让其余 client 收回决策 UI
                // （SPEC-005 §6.3；已答复方对未知 request_id 的 canceled 是 no-op）。
                cleanup_guard.disarm();
                self.broadcast_request_decision_canceled(RequestDecisionCanceledNotify {
                    request_id,
                    reason: CancelReason::ResolvedByPeer,
                    auto_decision: resp.decision,
                });
                Ok(resp)
            }
            Ok(Err(_)) => {
                // oneshot sender 已丢弃：最后一个 GUI 断线（handle_connection 清空 pending）
                // 或某 client 以 §12.4 错误响应拒绝（dispatch_message drop entry）。
                cleanup_guard.disarm();
                warn!(%request_id, "decision sender dropped (client disconnected or rejected); fallback");
                // fan-out 后其余 client 可能仍持有决策 UI——广播 resolved_by_peer 收口
                // （无 live writer 时广播天然 no-op，行为等价旧实现）。
                let fallback = make_timeout_fallback(request_id, default_on_timeout);
                self.broadcast_request_decision_canceled(RequestDecisionCanceledNotify {
                    request_id,
                    reason: CancelReason::ResolvedByPeer,
                    auto_decision: fallback.decision,
                });
                Ok(fallback)
            }
            Err(_elapsed) => {
                // 超时，清理 pending map + 广播取消通知给所有 GUI（SPEC-002 §9.3）。
                cleanup_guard.disarm();
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
    ///
    /// **绕过 wire 应答路径**（不经 socket 帧解析），供服务端单测直接触发决策；
    /// F1-b 的 peer 核验只作用于 wire 投递的应答，本注入路径保持 trusted。
    pub async fn inject_decision(&self, resp: DecisionResponse) {
        let mut map = self.pending.lock().await;
        if let Some(entry) = map.remove(&resp.request_id) {
            let _ = entry.responder.send(resp);
        }
    }

    /// `sieve.list_pending`：当前所有待决策的只读快照（SPEC-005 §11D）。
    ///
    /// `age_seconds` 按各条 `snapshot.created_at` 相对当前时刻现算。空集返回 `[]`。
    /// 过滤（severity / provider_id）在 client 侧做，daemon 保持薄。
    pub async fn list_pending(&self) -> ListPendingResult {
        let map = self.pending.lock().await;
        let now = Utc::now();
        let pending = map
            .values()
            .map(|entry| {
                let mut snap = (*entry.snapshot).clone();
                snap.age_seconds = (now - snap.created_at).num_seconds().max(0) as u64;
                snap
            })
            .collect();
        ListPendingResult { pending }
    }

    /// `sieve.resolve_decision`：headless 解决单个待决策（SPEC-005 §11E，A 方案授权）。
    ///
    /// **A 方案门禁**：若目标 pending 的 `max_severity == Critical`（daemon 侧计算，
    /// 不信客户端），且请求 decision 为 `Allow` / `RedactAndAllow`，则**静默改写为 `Deny`**
    /// ——不回特殊错误、不提示 GUI 路径（不向调用方暴露"存在 GUI 绕过路径"）。
    /// `High` 及以下按传入 decision 处置。
    ///
    /// `remember` 恒为 false（不给 CLI 开永久白名单）。审计由原始 `request_decision`
    /// 调用点在 `responder.send` 后自动记录（决策 audit 零特殊处理）。
    ///
    /// 目标 request_id 不存在（已超时 / 已被 GUI 解决 / id 不存在）→ `NotFound`。
    pub async fn resolve_decision(
        &self,
        request_id: Uuid,
        decision: DecisionAction,
        context_hint: Option<String>,
    ) -> ResolveDecisionResult {
        let entry = {
            let mut map = self.pending.lock().await;
            map.remove(&request_id)
        };
        let Some(entry) = entry else {
            return ResolveDecisionResult {
                status: ResolveStatus::NotFound,
                effective_decision: None,
            };
        };

        // A 方案：Critical 类的 allow / redact_and_allow 静默改写为 deny。
        let effective = if entry.max_severity == Severity::Critical
            && matches!(
                decision,
                DecisionAction::Allow | DecisionAction::RedactAndAllow
            ) {
            DecisionAction::Deny
        } else {
            decision
        };

        let resp = DecisionResponse {
            request_id,
            decision: effective,
            decided_at: Utc::now(),
            by_user: true,
            remember: false,
            context_hint,
            ui_phase_when_clicked: None,
        };
        // responder 已被移出 map，send 让原始 request_decision 的 await 返回，
        // 触发其调用点的决策 audit（含本次静默 deny）。send 失败（await 端已 drop）
        // 时 pending 已被消费，仍视为 resolved（幂等）。
        let _ = entry.responder.send(resp);

        ResolveDecisionResult {
            status: ResolveStatus::Resolved,
            effective_decision: Some(effective),
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
    /// GUI peer 代码签名核验回调（F1-b）；`None` = gate 关闭。
    peer_verifier: Option<PeerVerifier>,
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
        peer_verifier,
    } = ctx;
    info!("GUI client connected");

    // F1-b：into_split 之前取 raw fd（split 后两半共享同一 socket，fd 在连接存活期有效）。
    // 核验懒执行：仅该连接首次尝试放行 Critical 决策应答时触发。
    let peer_gate = {
        use std::os::unix::io::AsRawFd;
        PeerGate::new(peer_verifier, stream.as_raw_fd())
    };

    let (read_half, mut write_half) = stream.into_split();
    // SPEC-005 §1.3.1：用 FrameReader（read_buf + memchr）替代无界 BufReader::lines()。
    let mut frame_reader = FrameReader::new(read_half);

    // SPEC-005 §3：连接建立后第一条出站消息必须是 sieve.hello notification。
    if let Some(builder) = hello_builder {
        // SPEC-005 §3.2（D-5 修复）：握手不能只取 paused 布尔而丢弃截止时间，
        // 否则 client 握手进入暂停态却拿不到 until → 状态降级、菜单栏假装正常。
        // 取过期过滤后的 until 快照，paused 由其 is_some() 派生，二者天然一致。
        let paused_until_value = {
            let snap = paused_until.load();
            snap.as_ref()
                .as_ref()
                .copied()
                .filter(|until| *until > Utc::now())
        };
        let paused = paused_until_value.is_some();
        let uptime_seconds = (Utc::now() - builder.started_at).num_seconds().max(0) as u64;
        let params = crate::protocol::HelloParams {
            protocol_version: "v2".to_owned(),
            daemon_version: builder.daemon_version,
            paused,
            paused_until: paused_until_value,
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
                            &peer_gate,
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

    // 先显式 drop 本连接的 write_rx，使 gui_writers 中本连接的 sender 立即
    // `is_closed()`，否则下方 has_live_writer 会把"正在断开的自己"误判为 live
    // （write_rx 未 drop → sender 仍 open → 单 GUI 断开也不清空 → 误等满 timeout）。
    drop(write_rx);

    // 连接断开：仅当**没有任何 live GUI writer 存活**时才清空 pending（真无人可应答
    // → 立即 fail-closed fallback）。若仍有其他 live client 连着，保留 pending：可能被
    // 其他 GUI 应答，或靠 request_decision 自身 timeout 兜底。
    //
    // 修复多 client 平等场景缺陷：headless CLI 用短连接查 list_pending / resolve_decision /
    // health 后断开，本尾部不得误清空 GUI 的未决 pending（旧逻辑无差别 clear 假设"任何断开
    // 的连接都是持有 pending 的 GUI"，在 CLI + GUI 并存时会把 GUI 决策误 fallback deny）。
    // 单 GUI 场景行为不变：唯一 GUI 断开后无 live writer → 照旧清空立即 fallback。
    let has_live_writer = {
        let writers = gui_writers_clone
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        writers.iter().any(|s| !s.is_closed())
    };
    if !has_live_writer {
        let mut map = pending.lock().await;
        let count = map.len();
        if count > 0 {
            warn!(
                pending_count = count,
                "no live GUI writer after disconnect; dropping all pending (fail-closed fallback)"
            );
            map.clear(); // 清空 map，sender 被 drop，所有等待者收到 Err 并 fallback。
        }
    } else {
        debug!("connection closed but live GUI writer remains; pending retained");
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
    peer_gate: &PeerGate,
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
            // v2.0+ 兼容扩展（SPEC-005 §11A §11B §11C §11D §11E）
            | "sieve.list_rules"
            | "sieve.purge_history"
            | "sieve.judge_tool_call"
            | "sieve.list_pending"
            | "sieve.resolve_decision" => {
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
                if let Some(entry) = map.remove(&request_id) {
                    // 通知等待方：GUI 拒绝/取消，走 fallback（Err -> timeout fallback）。
                    drop(entry); // entry drop → responder drop → rx.await 返回 Err(RecvError) → fallback
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
        let resp = match serde_json::from_value::<DecisionResponse>(result.clone()) {
            Ok(resp) => resp,
            Err(single_error) => match serde_json::from_value::<MergedDecisionResponse>(result) {
                Ok(merged) => merged.into_aggregate(),
                Err(merged_error) => {
                    warn!(
                        "failed to deserialize decision response: single={single_error}; merged={merged_error}"
                    );
                    return;
                }
            },
        };
        let mut map = pending.lock().await;
        if let Some(entry) = map.remove(&resp.request_id) {
            // F1-b（SPEC-005 §6.2.4）：wire 应答放行 Critical 前必须通过 peer 代码签名核验；
            // 未通过 → allow / redact_and_allow 静默改写为 deny（与 resolve_decision 的
            // A 方案同范式，daemon 侧权威 max_severity，不信客户端自报）。
            // inject_decision 注入路径与 resolve_decision 不经此处。
            let resp = if entry.max_severity == Severity::Critical
                && matches!(
                    resp.decision,
                    DecisionAction::Allow | DecisionAction::RedactAndAllow
                )
                && !peer_gate.permits_critical_allow()
            {
                warn!(
                    request_id = %resp.request_id,
                    "GUI peer code-signing verification failed; Critical allow rewritten to deny (F1-b)"
                );
                DecisionResponse {
                    decision: DecisionAction::Deny,
                    remember: false,
                    ..resp
                }
            } else {
                resp
            };
            let _ = entry.responder.send(resp);
        } else {
            warn!(
                request_id = %resp.request_id,
                "no pending request for this decision"
            );
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
            // payload 上限 64KB。
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
        // SPEC-005 §11D：sieve.list_pending（Since v2.x，只读）。
        "sieve.list_pending" => {
            let p: ListPendingRequest = match parse_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::ListPending { params: p, reply })
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
            forward_reply::<ListPendingResult>(id, rx, write_tx).await;
        }
        // SPEC-005 §11E：sieve.resolve_decision（Since v2.x，A 方案授权）。
        "sieve.resolve_decision" => {
            let p: ResolveDecisionRequest = match require_params(params) {
                Ok(p) => p,
                Err(e) => return write_error_response(id, e, write_tx).await,
            };
            let (reply, rx) = oneshot::channel();
            if control_tx
                .send(ControlPlaneRequest::ResolveDecision { params: p, reply })
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
            forward_reply::<ResolveDecisionResult>(id, rx, write_tx).await;
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

/// 通用 fan-out 广播实现（自由函数，不依赖 `IpcServer` 实例）。
///
/// 行为与 broadcast_status_bar 历史实现一致：
/// - 无 GUI 连接时静默丢弃 + debug 日志。
/// - `try_send`：Closed → 移除（lazy 清理）；Full → 保留（背压）。
/// - 持 `std::sync::Mutex` 锁短暂、无 await——**可在 Drop 中安全调用**
///   （[`PendingCleanupGuard`] 依赖此性质在 future 被 cancel 后广播取消通知）。
fn broadcast_method_to<T: serde::Serialize>(
    gui_writers: &GuiWriters,
    method: &str,
    params: &T,
    label: &str,
) {
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
    let mut writers = gui_writers
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

/// 广播 `sieve.request_decision_canceled`（自由函数形态，供 [`PendingCleanupGuard`] 使用）。
fn broadcast_request_decision_canceled_to(
    gui_writers: &GuiWriters,
    notify: RequestDecisionCanceledNotify,
) {
    let label = format!("request_decision_canceled request_id={}", notify.request_id);
    broadcast_method_to(
        gui_writers,
        "sieve.request_decision_canceled",
        &notify,
        &label,
    );
}

/// RAII 兜底（BUG-L1b）：`request_decision` future 被调用方 cancel 时清理 pending 泄漏。
///
/// 场景：`request_decision` 被 HTTP handler await，HTTP 客户端在决策等待期断连 →
/// 连接任务 abort → 本 future 在任意 await 点被 drop → `tokio::time::timeout` 分支
/// 永不执行 → pending map 条目滞留（`sieve decisions list` 出现 age 远超 timeout 的
/// 僵尸条目），且 GUI 收不到 cancel、决策 UI 悬空到自身倒计时结束。
///
/// 本 guard 在 pending 注册后立即武装；所有正常退出路径（首答 / 超时 / 无 writer /
/// sender drop）显式 [`Self::disarm`]。仅当 future 被 cancel（Drop 时仍武装）才触发：
/// 从 pending map 移除条目 + 广播 `upstream_disconnected` 取消通知。
///
/// Drop 中不能 await：优先 `try_lock` 同步清理（future 被 cancel 时通常无锁竞争）；
/// 竞争失败则 spawn 清理任务兜底（仍在 runtime 上下文内——drop 发生在连接任务 abort 时）。
struct PendingCleanupGuard {
    pending: PendingMap,
    gui_writers: GuiWriters,
    request_id: Uuid,
    /// 广播 cancel 时携带的兜底处置（`default_on_timeout` 映射，与超时路径一致）。
    auto_decision: DecisionAction,
    armed: bool,
}

impl PendingCleanupGuard {
    fn new(
        pending: PendingMap,
        gui_writers: GuiWriters,
        request_id: Uuid,
        default_on_timeout: DefaultOnTimeout,
    ) -> Self {
        Self {
            pending,
            gui_writers,
            request_id,
            auto_decision: match default_on_timeout {
                DefaultOnTimeout::Block => DecisionAction::Deny,
                DefaultOnTimeout::Allow => DecisionAction::Allow,
                DefaultOnTimeout::Redact => DecisionAction::RedactAndAllow,
            },
            armed: true,
        }
    }

    /// 正常退出路径调用：pending 已由该路径自行处置，Drop 不再兜底。
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for PendingCleanupGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let notify = RequestDecisionCanceledNotify {
            request_id: self.request_id,
            reason: CancelReason::UpstreamDisconnected,
            auto_decision: self.auto_decision,
        };
        match self.pending.try_lock() {
            Ok(mut map) => {
                // 条目已被他人消费（首答与 cancel 竞态）→ 已处置，无需广播。
                if map.remove(&self.request_id).is_some() {
                    drop(map);
                    warn!(
                        request_id = %self.request_id,
                        "request_decision future canceled (caller dropped); pending cleaned + cancel broadcast"
                    );
                    broadcast_request_decision_canceled_to(&self.gui_writers, notify);
                }
            }
            Err(_) => {
                // 锁竞争：不能在 Drop 里阻塞等锁（tokio Mutex 无同步阻塞路径），
                // spawn 清理任务兜底。future 在 runtime 内被 drop，try_current 必可用；
                // 极端情形（runtime 正在关停）下 spawn 失败 = 进程将亡，泄漏无害。
                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    let pending = Arc::clone(&self.pending);
                    let gui_writers = Arc::clone(&self.gui_writers);
                    let request_id = self.request_id;
                    handle.spawn(async move {
                        if pending.lock().await.remove(&request_id).is_some() {
                            warn!(
                                %request_id,
                                "request_decision future canceled (caller dropped); pending cleaned + cancel broadcast (deferred)"
                            );
                            broadcast_request_decision_canceled_to(&gui_writers, notify);
                        }
                    });
                } else {
                    warn!(
                        request_id = %self.request_id,
                        "request_decision future canceled but no runtime for deferred cleanup"
                    );
                }
            }
        }
    }
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
            .request_decision(req, Duration::from_secs(60), "inbound", None)
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

    /// SPEC-005 §6.1.1：wire `timeout_seconds` 必须在 [30,120]。daemon 发送前在唯一 choke
    /// point 钳制越界值（如 0），保证 GUI 倒计时不被 0 击穿。（D-3）
    #[tokio::test]
    async fn request_decision_clamps_out_of_range_timeout_seconds() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (server, _listener) = IpcServer::bind(socket_path).expect("bind");

        // 注册能收帧的 writer（容量足够，不触发背压降级）。
        let (tx, mut rx) = mpsc::channel::<String>(4);
        server
            .gui_writers
            .lock()
            .expect("gui_writers lock")
            .push(tx);

        // 越界 timeout_seconds=0 → wire 应被钳到 SPEC 下限 30。
        let mut req = dummy_request();
        req.timeout_seconds = 0;

        // 短 oneshot 超时：帧 try_send 后立即走 fallback，不阻塞测试。
        let _ = server
            .request_decision(req, Duration::from_millis(50), "inbound", None)
            .await
            .expect("request_decision");

        // 取出已发送的 wire 帧，断言 timeout_seconds 已钳到 30。
        let payload = rx.try_recv().expect("wire frame should have been sent");
        let v: serde_json::Value =
            serde_json::from_str(payload.trim_end()).expect("wire frame is valid JSON");
        assert_eq!(
            v["params"]["timeout_seconds"], 30,
            "越界 timeout_seconds 应钳到 SPEC-005 §6.1.1 下限 30；wire frame: {payload}"
        );

        // 上界同理：121 → 120。
        let (tx2, mut rx2) = mpsc::channel::<String>(4);
        server.gui_writers.lock().expect("gui_writers lock").clear();
        server
            .gui_writers
            .lock()
            .expect("gui_writers lock")
            .push(tx2);
        let mut req2 = dummy_request();
        req2.timeout_seconds = 121;
        let _ = server
            .request_decision(req2, Duration::from_millis(50), "inbound", None)
            .await
            .expect("request_decision");
        let payload2 = rx2.try_recv().expect("wire frame 2 sent");
        let v2: serde_json::Value =
            serde_json::from_str(payload2.trim_end()).expect("wire frame 2 valid JSON");
        assert_eq!(
            v2["params"]["timeout_seconds"], 120,
            "越界 timeout_seconds 应钳到 SPEC-005 §6.1.1 上限 120；wire frame: {payload2}"
        );
    }

    /// 回归 stale-writer 缺陷：gui_writers 中 index 0 是 stale sender（GUI 断连后
    /// receiver 已 drop，但清理是 lazy 的、滞留首位），index 1 是 live client。旧实现只取
    /// `writers.first()`，try_send 命中 Closed 即直接 fallback deny，从不尝试 live writer →
    /// 误 fail-closed（多 client 平等场景的可用性缺陷）。修复后遍历快照跳过 stale、改用 live
    /// writer，并主动清理死 sender。
    #[tokio::test]
    async fn request_decision_skips_stale_writer_and_uses_live_one() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (server, _listener) = IpcServer::bind(socket_path).expect("bind");
        let server = Arc::new(server);

        // index 0：stale sender——drop rx 使其 Closed。
        let (dead_tx, dead_rx) = mpsc::channel::<String>(4);
        drop(dead_rx);
        // index 1：live writer。
        let (live_tx, mut live_rx) = mpsc::channel::<String>(4);
        {
            let mut w = server.gui_writers.lock().expect("gui_writers lock");
            w.push(dead_tx);
            w.push(live_tx);
        }

        let req = dummy_request();
        let request_id = req.request_id;

        // spawn request_decision（它会 await GUI 回复）。
        let server_for_task = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            server_for_task
                .request_decision(req, Duration::from_secs(60), "inbound", None)
                .await
        });

        // live writer 应在 2s 内收到决策请求帧（修复前：帧打给死的 dead_tx → live_rx 永不到达 → 超时即红）。
        let line = tokio::time::timeout(Duration::from_secs(2), live_rx.recv())
            .await
            .expect("live writer 应收到决策请求（修复前帧打给 stale sender，永不到达）")
            .expect("live_rx 不应关闭");
        let v: serde_json::Value =
            serde_json::from_str(line.trim_end()).expect("wire frame valid JSON");
        assert_eq!(v["method"], "sieve.request_decision", "应是决策请求帧");

        // 注入 GUI 真实决策（Allow, by_user）。
        server
            .inject_decision(DecisionResponse {
                request_id,
                decision: DecisionAction::Allow,
                decided_at: Utc::now(),
                by_user: true,
                remember: false,
                context_hint: None,
                ui_phase_when_clicked: None,
            })
            .await;

        let resp = handle
            .await
            .expect("join request_decision task")
            .expect("request_decision ok");
        assert!(resp.by_user, "应返回 GUI 真实决策而非 fail-closed fallback");
        assert_eq!(
            resp.decision,
            DecisionAction::Allow,
            "应为 live GUI 的 Allow 决策"
        );

        // 死 sender 应被主动清理（不再等下次 broadcast lazy 清理）。
        let remaining = server.gui_writers.lock().expect("gui_writers lock").len();
        assert_eq!(
            remaining, 1,
            "stale sender 应被主动清理，只剩 1 个 live writer"
        );
    }

    /// 安全契约回归（stale-writer 反向）：所有 writer 都 stale（receiver 全 drop）时，必须仍
    /// fail-closed fallback——遍历改造不得放宽「真无人在线 → deny」的核心不变量。
    #[tokio::test]
    async fn request_decision_all_stale_writers_still_fail_closed() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (server, _listener) = IpcServer::bind(socket_path).expect("bind");

        // 两个全 stale 的 sender。
        for _ in 0..2 {
            let (tx, rx) = mpsc::channel::<String>(4);
            drop(rx);
            server
                .gui_writers
                .lock()
                .expect("gui_writers lock")
                .push(tx);
        }

        let req = dummy_request(); // default_on_timeout = Block
        let start = Instant::now();
        let resp = server
            .request_decision(req, Duration::from_secs(60), "inbound", None)
            .await
            .expect("request_decision");
        // 全 stale 应立即降级，不消耗 60s 超时。
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "全 stale 应立即 fallback，实际 {:?}",
            start.elapsed()
        );
        assert_eq!(
            resp.decision,
            DecisionAction::Deny,
            "全 stale 必须 fail-closed deny"
        );
        assert!(!resp.by_user, "fallback 不应标记 by_user");
        // 全部死 sender 应被清理。
        let remaining = server.gui_writers.lock().expect("gui_writers lock").len();
        assert_eq!(remaining, 0, "全 stale sender 应被清空");
    }

    /// BUG-L1c 修复主断言（SPEC-005 §1.4 fan-out）：决策请求必须 fan-out 给**所有** live
    /// writer；首个应答生效；解决后所有 client 收到 resolved_by_peer 取消通知（携带首答
    /// 的实际决策），让未答复方收回决策 UI。
    #[tokio::test]
    async fn request_decision_fans_out_first_answer_wins_others_get_canceled() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (server, _listener) = IpcServer::bind(socket_path).expect("bind");
        let server = Arc::new(server);

        // 两个 live writer，模拟两个并发 GUI client。
        let (tx_a, mut rx_a) = mpsc::channel::<String>(8);
        let (tx_b, mut rx_b) = mpsc::channel::<String>(8);
        {
            let mut w = server.gui_writers.lock().expect("gui_writers lock");
            w.push(tx_a);
            w.push(tx_b);
        }

        let req = dummy_request();
        let request_id = req.request_id;

        let server_for_task = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            server_for_task
                .request_decision(req, Duration::from_secs(60), "inbound", None)
                .await
        });

        // fan-out：两个 writer 都必须收到决策请求帧（修复前只投第一个成功的 writer）。
        for (name, rx) in [("A", &mut rx_a), ("B", &mut rx_b)] {
            let line = tokio::time::timeout(Duration::from_secs(2), rx.recv())
                .await
                .unwrap_or_else(|_| panic!("writer {name} 应收到决策请求帧（fan-out）"))
                .expect("rx 不应关闭");
            let v: serde_json::Value =
                serde_json::from_str(line.trim_end()).expect("wire frame valid JSON");
            assert_eq!(
                v["method"], "sieve.request_decision",
                "writer {name} 应收到决策请求帧"
            );
        }

        // 首答生效（inject 模拟其中一个 GUI 应答 Allow）。
        server
            .inject_decision(DecisionResponse {
                request_id,
                decision: DecisionAction::Allow,
                decided_at: Utc::now(),
                by_user: true,
                remember: false,
                context_hint: None,
                ui_phase_when_clicked: None,
            })
            .await;

        let resp = handle
            .await
            .expect("join request_decision task")
            .expect("request_decision ok");
        assert!(resp.by_user);
        assert_eq!(resp.decision, DecisionAction::Allow, "首答应生效");

        // 解决后所有 client 收到 resolved_by_peer 取消通知（含首答方，其对未知
        // request_id 的 canceled 是 no-op）。
        for (name, rx) in [("A", &mut rx_a), ("B", &mut rx_b)] {
            let line = tokio::time::timeout(Duration::from_secs(2), rx.recv())
                .await
                .unwrap_or_else(|_| panic!("writer {name} 应收到 canceled 通知"))
                .expect("rx 不应关闭");
            let v: serde_json::Value =
                serde_json::from_str(line.trim_end()).expect("cancel frame valid JSON");
            assert_eq!(v["method"], "sieve.request_decision_canceled");
            assert_eq!(v["params"]["request_id"], request_id.to_string());
            assert_eq!(
                v["params"]["reason"], "resolved_by_peer",
                "解决路径的取消原因应为 resolved_by_peer"
            );
            assert_eq!(
                v["params"]["auto_decision"], "allow",
                "auto_decision 应为首答的实际决策"
            );
        }
    }

    /// BUG-L1c 回归锚点（真机实证场景）：GUI 重启过渡期「半死」连接——mpsc channel
    /// 仍开着（receiver 未 drop、try_send 成功）但对端 GUI 已是僵尸、不再消费任何帧。
    /// 旧实现「投给第一个 try_send 成功的 writer」会把决策帧喂给僵尸 → live GUI 永不
    /// 弹窗、daemon 等满 60s 后 fail-closed。fan-out 修复后 live writer 必须同时收到
    /// 请求帧并能解决决策。
    #[tokio::test]
    async fn request_decision_survives_half_dead_writer_via_fanout() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (server, _listener) = IpcServer::bind(socket_path).expect("bind");
        let server = Arc::new(server);

        // index 0：半死 writer——receiver 持有但永不消费（try_send 成功、帧被吞）。
        let (zombie_tx, _zombie_rx_kept_alive) = mpsc::channel::<String>(8);
        // index 1：live writer。
        let (live_tx, mut live_rx) = mpsc::channel::<String>(8);
        {
            let mut w = server.gui_writers.lock().expect("gui_writers lock");
            w.push(zombie_tx);
            w.push(live_tx);
        }

        let req = dummy_request();
        let request_id = req.request_id;

        let server_for_task = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            server_for_task
                .request_decision(req, Duration::from_secs(60), "inbound", None)
                .await
        });

        // 修复前：帧只投给 index 0 的半死 writer（try_send 成功即停）→ live_rx 永不
        // 到达 → 此处超时即红。修复后 fan-out 保证 live writer 同时收到。
        let line = tokio::time::timeout(Duration::from_secs(2), live_rx.recv())
            .await
            .expect("live writer 应收到决策请求帧（修复前被半死连接吞掉）")
            .expect("live_rx 不应关闭");
        let v: serde_json::Value =
            serde_json::from_str(line.trim_end()).expect("wire frame valid JSON");
        assert_eq!(v["method"], "sieve.request_decision");

        // live GUI 应答 → 决策解决，不等满 60s timeout。
        server
            .inject_decision(DecisionResponse {
                request_id,
                decision: DecisionAction::Deny,
                decided_at: Utc::now(),
                by_user: true,
                remember: false,
                context_hint: None,
                ui_phase_when_clicked: None,
            })
            .await;

        let resp = handle
            .await
            .expect("join request_decision task")
            .expect("request_decision ok");
        assert!(
            resp.by_user,
            "应为 live GUI 的真实决策而非 timeout fallback"
        );
        assert_eq!(resp.decision, DecisionAction::Deny);
    }

    /// BUG-L1b 回归（真机实证：curl 在决策等待期被 kill）：request_decision future 被
    /// 调用方 cancel（HTTP 连接任务 abort 导致 drop）后，timeout 分支永不执行——修复前
    /// pending map 条目滞留（`sieve decisions list` 可见 age 超 timeout 的僵尸条目）且
    /// 不广播 cancel（GUI 决策 UI 悬空）。修复后 PendingCleanupGuard 的 Drop 必须清空
    /// pending + 广播 upstream_disconnected。
    #[tokio::test]
    async fn request_decision_dropped_future_cleans_pending_and_broadcasts_cancel() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (server, _listener) = IpcServer::bind(socket_path).expect("bind");
        let server = Arc::new(server);

        let (tx, mut rx) = mpsc::channel::<String>(8);
        server
            .gui_writers
            .lock()
            .expect("gui_writers lock")
            .push(tx);

        let req = dummy_request(); // default_on_timeout = Block → auto_decision deny
        let request_id = req.request_id;

        let server_for_task = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            server_for_task
                .request_decision(req, Duration::from_secs(60), "inbound", None)
                .await
        });

        // 等请求帧已投出、future 停在 rx.await（确保 abort 命中等待期而非注册期）。
        let line = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("writer 应收到决策请求帧")
            .expect("rx 不应关闭");
        let v: serde_json::Value =
            serde_json::from_str(line.trim_end()).expect("wire frame valid JSON");
        assert_eq!(v["method"], "sieve.request_decision");
        assert_eq!(server.inflight_decisions().await, 1, "pending 应已注册");

        // 模拟 HTTP 客户端断连：连接任务 abort → request_decision future 被 drop。
        handle.abort();
        let join = handle.await;
        assert!(join.is_err(), "task 应以 cancelled 结束");

        // guard Drop 兜底：pending 条目必须被清空（修复前滞留到进程重启）。
        // Drop 内 try_lock 失败时走 spawn 的异步清理，轮询等待兜底完成。
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if server.inflight_decisions().await == 0 {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "pending 条目应在 future 被 drop 后清空（BUG-L1b：修复前泄漏滞留）"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // 且已向 GUI 广播 upstream_disconnected 取消通知（修复前 GUI 决策 UI 悬空）。
        let cancel_line = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("writer 应收到 canceled 通知")
            .expect("rx 不应关闭");
        let cv: serde_json::Value =
            serde_json::from_str(cancel_line.trim_end()).expect("cancel frame valid JSON");
        assert_eq!(cv["method"], "sieve.request_decision_canceled");
        assert_eq!(cv["params"]["request_id"], request_id.to_string());
        assert_eq!(
            cv["params"]["reason"], "upstream_disconnected",
            "future 被 cancel 的取消原因应为 upstream_disconnected"
        );
        assert_eq!(
            cv["params"]["auto_decision"], "deny",
            "default_on_timeout=Block → auto_decision deny"
        );
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

    /// SPEC-005 §3.2（D-5 回归）：daemon 处暂停态时握手 hello 必须带 `paused_until`，
    /// 不能只发 `paused: true` 而丢弃截止时间——否则 client 握手进入暂停态却拿不到
    /// until，状态降级、菜单栏假装正常。修复前握手只取 paused 布尔，本测试会失败。
    #[tokio::test]
    async fn hello_carries_paused_until_when_paused() {
        use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};
        use tokio::net::UnixStream;

        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("ipc.sock");
        let (server, listener) = IpcServer::bind(socket_path.clone()).expect("bind");
        let server = Arc::new(server);

        // daemon 进入暂停态，截止时间在未来 30 分钟。
        let until = Utc::now() + chrono::Duration::minutes(30);
        server.set_paused_until(Some(until));
        server
            .set_hello_builder(HelloBuilder {
                daemon_boot_id: Uuid::now_v7(),
                daemon_version: "0.0.0-test".to_owned(),
                audit_db_user_version: 2,
                started_at: Utc::now(),
                preset: "default".to_owned(),
            })
            .await;

        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(20)).await;

        let stream = UnixStream::connect(&socket_path).await.expect("connect");
        let mut lines = TokioBufReader::new(stream).lines();
        let first_line = tokio::time::timeout(Duration::from_secs(2), lines.next_line())
            .await
            .expect("timeout waiting for sieve.hello")
            .expect("io error")
            .expect("connection closed before hello");

        let val: serde_json::Value =
            serde_json::from_str(&first_line).expect("hello must be valid JSON");
        let params = val.get("params").expect("hello must have params");
        assert_eq!(params["paused"], true, "暂停态握手 paused 必须为 true");
        let pu = params
            .get("paused_until")
            .and_then(|v| v.as_str())
            .expect("暂停态握手必须带 paused_until 字符串（D-5）");
        let parsed = DateTime::parse_from_rfc3339(pu)
            .expect("paused_until 必须是 RFC3339")
            .with_timezone(&Utc);
        assert!(parsed > Utc::now(), "paused_until 应为未来时间");
        assert!(
            pu.ends_with('Z'),
            "paused_until 必须 Z 后缀（SPEC-005 §4A）"
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

    // ── F1-a / M1：list_pending + resolve_decision A 方案授权门禁 ──────────────

    /// 构造带单条指定 severity detection 的请求（daemon 侧 max_severity 计算依据）。
    fn req_with_severity(sev: Severity) -> DecisionRequest {
        use crate::protocol::{DetectionPayload, Disposition};
        let mut req = dummy_request();
        req.detections = vec![DetectionPayload {
            rule_id: "TEST-RULE".to_owned(),
            severity: sev,
            disposition: Disposition::GuiPopup,
            title: "测试".to_owned(),
            one_line_summary: "测试 detection".to_owned(),
            details: serde_json::json!({}),
            recommendation: None,
        }];
        req
    }

    /// 注册一个 live writer 占位（避免 request_decision 因空 writer 立即 fallback），
    /// 并 spawn request_decision，等其发出 wire 帧（确认已插入 pending map）。
    /// 返回 (request_id, join handle, writer rx)。
    async fn spawn_pending(
        server: &Arc<IpcServer>,
        req: DecisionRequest,
        provider_id: Option<&str>,
    ) -> (
        Uuid,
        tokio::task::JoinHandle<Result<DecisionResponse, IpcError>>,
        mpsc::Receiver<String>,
    ) {
        let (tx, mut rx) = mpsc::channel::<String>(4);
        server
            .gui_writers
            .lock()
            .expect("gui_writers lock")
            .push(tx);
        let request_id = req.request_id;
        let s = Arc::clone(server);
        let provider_owned = provider_id.map(str::to_owned);
        let handle = tokio::spawn(async move {
            s.request_decision(
                req,
                Duration::from_secs(60),
                "inbound",
                provider_owned.as_deref(),
            )
            .await
        });
        // 等 request_decision 发出 wire 帧（此时 pending 已插入）。
        let _ = tokio::time::timeout(Duration::from_secs(2), rx.recv())
            .await
            .expect("request_decision 应发出 wire 帧");
        (request_id, handle, rx)
    }

    /// list_pending 无 pending 时返回空集（空 ≠ 错误）。
    #[tokio::test]
    async fn list_pending_empty_returns_empty() {
        let dir = tempdir().expect("tempdir");
        let (server, _listener) = IpcServer::bind(dir.path().join("ipc.sock")).expect("bind");
        let listed = server.list_pending().await;
        assert!(listed.pending.is_empty(), "无 pending 应返回空集");
    }

    /// list_pending 返回 daemon 侧计算的 max_severity + provider_id 快照。
    #[tokio::test]
    async fn list_pending_returns_snapshot_with_max_severity_and_provider() {
        let dir = tempdir().expect("tempdir");
        let (server, _listener) = IpcServer::bind(dir.path().join("ipc.sock")).expect("bind");
        let server = Arc::new(server);
        let req = req_with_severity(Severity::Critical);
        let (request_id, handle, _rx) = spawn_pending(&server, req, Some("anthropic-main")).await;

        let listed = server.list_pending().await;
        assert_eq!(listed.pending.len(), 1, "应有 1 条 pending");
        let snap = &listed.pending[0];
        assert_eq!(snap.request_id, request_id);
        assert_eq!(
            snap.max_severity,
            Severity::Critical,
            "max_severity 由 daemon 侧从 detections 计算"
        );
        assert_eq!(snap.provider_id.as_deref(), Some("anthropic-main"));
        assert_eq!(snap.direction, "inbound");

        // 清理：resolve 让 spawn 的 request_decision 返回，join 不泄漏。
        let _ = server
            .resolve_decision(request_id, DecisionAction::Deny, None)
            .await;
        let _ = handle.await;
    }

    /// A 方案核心：Critical pending 的 allow resolve 被静默改写为 deny。
    #[tokio::test]
    async fn resolve_critical_allow_is_silently_denied() {
        let dir = tempdir().expect("tempdir");
        let (server, _listener) = IpcServer::bind(dir.path().join("ipc.sock")).expect("bind");
        let server = Arc::new(server);
        let req = req_with_severity(Severity::Critical);
        let (request_id, handle, _rx) = spawn_pending(&server, req, None).await;

        // headless resolve allow → daemon 按 max_severity=Critical 静默改 deny。
        let res = server
            .resolve_decision(
                request_id,
                DecisionAction::Allow,
                Some("尝试放行".to_owned()),
            )
            .await;
        assert_eq!(res.status, ResolveStatus::Resolved);
        assert_eq!(
            res.effective_decision,
            Some(DecisionAction::Deny),
            "Critical 类 allow 必须被静默改写为 deny（A 方案）"
        );

        // 原始 request_decision 应收到 deny（by_user=true，remember=false）。
        let resp = handle.await.expect("join").expect("request_decision");
        assert_eq!(resp.decision, DecisionAction::Deny);
        assert!(resp.by_user, "headless resolve 是主动决策");
        assert!(!resp.remember, "CLI resolve 恒不 remember");
    }

    /// A 方案：Critical pending 的 redact_and_allow 同样被静默改写为 deny。
    #[tokio::test]
    async fn resolve_critical_redact_is_silently_denied() {
        let dir = tempdir().expect("tempdir");
        let (server, _listener) = IpcServer::bind(dir.path().join("ipc.sock")).expect("bind");
        let server = Arc::new(server);
        let req = req_with_severity(Severity::Critical);
        let (request_id, handle, _rx) = spawn_pending(&server, req, None).await;

        let res = server
            .resolve_decision(request_id, DecisionAction::RedactAndAllow, None)
            .await;
        assert_eq!(
            res.effective_decision,
            Some(DecisionAction::Deny),
            "Critical 类 redact_and_allow 也必须被静默改写为 deny"
        );
        let resp = handle.await.expect("join").expect("request_decision");
        assert_eq!(resp.decision, DecisionAction::Deny);
    }

    /// A 方案：High 及以下的 allow resolve 正常放行（不改写）。
    #[tokio::test]
    async fn resolve_high_allow_passes_through() {
        let dir = tempdir().expect("tempdir");
        let (server, _listener) = IpcServer::bind(dir.path().join("ipc.sock")).expect("bind");
        let server = Arc::new(server);
        let req = req_with_severity(Severity::High);
        let (request_id, handle, _rx) = spawn_pending(&server, req, None).await;

        let res = server
            .resolve_decision(request_id, DecisionAction::Allow, None)
            .await;
        assert_eq!(res.status, ResolveStatus::Resolved);
        assert_eq!(
            res.effective_decision,
            Some(DecisionAction::Allow),
            "High 及以下 allow 正常放行，不改写"
        );
        let resp = handle.await.expect("join").expect("request_decision");
        assert_eq!(resp.decision, DecisionAction::Allow);
        assert!(resp.by_user);
    }

    /// Critical pending 的 deny resolve 正常执行（deny 不受 A 方案改写影响）。
    #[tokio::test]
    async fn resolve_critical_deny_passes_through() {
        let dir = tempdir().expect("tempdir");
        let (server, _listener) = IpcServer::bind(dir.path().join("ipc.sock")).expect("bind");
        let server = Arc::new(server);
        let req = req_with_severity(Severity::Critical);
        let (request_id, handle, _rx) = spawn_pending(&server, req, None).await;

        let res = server
            .resolve_decision(request_id, DecisionAction::Deny, None)
            .await;
        assert_eq!(res.effective_decision, Some(DecisionAction::Deny));
        let resp = handle.await.expect("join").expect("request_decision");
        assert_eq!(resp.decision, DecisionAction::Deny);
    }

    /// resolve 不存在的 request_id → NotFound（无 effective_decision）。
    #[tokio::test]
    async fn resolve_unknown_id_returns_not_found() {
        let dir = tempdir().expect("tempdir");
        let (server, _listener) = IpcServer::bind(dir.path().join("ipc.sock")).expect("bind");
        let res = server
            .resolve_decision(Uuid::now_v7(), DecisionAction::Deny, None)
            .await;
        assert_eq!(res.status, ResolveStatus::NotFound);
        assert!(res.effective_decision.is_none());
    }

    /// 同一 pending resolve 两次：第二次 NotFound（幂等，pending 已消费）。
    #[tokio::test]
    async fn resolve_twice_second_is_not_found() {
        let dir = tempdir().expect("tempdir");
        let (server, _listener) = IpcServer::bind(dir.path().join("ipc.sock")).expect("bind");
        let server = Arc::new(server);
        let req = req_with_severity(Severity::High);
        let (request_id, handle, _rx) = spawn_pending(&server, req, None).await;

        let first = server
            .resolve_decision(request_id, DecisionAction::Deny, None)
            .await;
        assert_eq!(first.status, ResolveStatus::Resolved);
        let second = server
            .resolve_decision(request_id, DecisionAction::Deny, None)
            .await;
        assert_eq!(
            second.status,
            ResolveStatus::NotFound,
            "同一 id 再次 resolve 应 NotFound（已消费）"
        );
        let _ = handle.await;
    }
}
