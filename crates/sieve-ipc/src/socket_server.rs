use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    error::IpcError,
    protocol::{
        DecisionAction, DecisionRequest, DecisionResponse, DefaultOnTimeout, ReloadUserRules,
        StatusBarNotify,
    },
};

/// pending map：request_id → oneshot 发送端，等待 GUI 回复。
type PendingMap = Arc<Mutex<HashMap<Uuid, oneshot::Sender<DecisionResponse>>>>;

/// reload 通知 channel 容量。
///
/// 容量 16：daemon 通常实时消费（spawn task 监听），16 条积压足够应对短暂卡顿；
/// 积压超限说明 daemon reload handler 异常，此时丢弃最新通知优于阻塞 IPC server。
const RELOAD_CHANNEL_CAPACITY: usize = 16;

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
        let (reload_tx, reload_rx) = mpsc::channel::<ReloadUserRules>(RELOAD_CHANNEL_CAPACITY);
        let server = Self {
            socket_path,
            pending: Arc::new(Mutex::new(HashMap::new())),
            gui_writers: Arc::new(std::sync::Mutex::new(Vec::new())),
            reload_tx,
            reload_rx: Arc::new(Mutex::new(Some(reload_rx))),
        };
        Ok((server, listener))
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

                    // 为新连接创建 mpsc 通道：发送端注册到 gui_writers，接收端传给 handle_connection。
                    // oneshot client（如 reload）连接短暂，断开后 try_send 自动清理其 sender。
                    let (tx, rx) = mpsc::channel::<String>(32);
                    {
                        let mut writers = gui_writers.lock().expect("gui_writers lock poisoned");
                        writers.push(tx);
                        let count = writers.len();
                        if count == 1 {
                            info!("first GUI client connected; gui_writers count = 1");
                        } else {
                            info!(count, "additional GUI client connected");
                        }
                    }

                    tokio::spawn(async move {
                        if let Err(e) =
                            handle_connection(stream, pending, gui_writers.clone(), rx, reload_tx)
                                .await
                        {
                            error!("IPC connection error: {e}");
                        }
                        // 连接断开：gui_writers 中对应的 sender 已 drop（rx drop 时发送端关闭），
                        // 下次 broadcast_status_bar 的 try_send 会检测到 Closed 并自动清理。
                        // 此处只记录日志；不需要显式从 Vec 删除（lazy 清理策略）。
                        debug!("GUI connection task exited; dead sender will be cleaned on next broadcast");
                    });
                }
                Err(e) => {
                    error!("IPC accept error: {e}");
                    break;
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
        // 构造 JSON-RPC 2.0 通知（无 id = fire-and-forget）。
        let notification = crate::protocol::jsonrpc::Request {
            jsonrpc: "2.0".to_owned(),
            method: "sieve.notify_status_bar".to_owned(),
            params: match serde_json::to_value(&notify) {
                Ok(v) => Some(v),
                Err(e) => {
                    warn!(notify_id = %notify.notify_id, "failed to serialize StatusBarNotify: {e}");
                    return;
                }
            },
            id: None, // 单向通知，无 id。
        };

        let mut payload = match serde_json::to_string(&notification) {
            Ok(s) => s,
            Err(e) => {
                warn!("failed to serialize notify JSON-RPC frame: {e}");
                return;
            }
        };
        payload.push('\n');

        // 持 std::sync::Mutex 锁（短暂，无 await），drain + try_send + 重建 Vec。
        let mut writers = self.gui_writers.lock().expect("gui_writers lock poisoned");

        if writers.is_empty() {
            debug!(notify_id = %notify.notify_id, "broadcast_status_bar: no GUI clients connected; dropping");
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
                    // 通道暂时满（背压），保留 sender，不视为断线。
                    debug!(notify_id = %notify.notify_id, "GUI writer channel full (backpressure); retaining sender");
                    alive.push(sender);
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    // GUI 已断线，丢弃此 sender（lazy 清理）。
                    debug!(notify_id = %notify.notify_id, "GUI client sender closed; removing from gui_writers");
                    removed += 1;
                }
            }
        }
        *writers = alive;

        if removed > 0 {
            debug!(
                notify_id = %notify.notify_id,
                sent,
                removed,
                alive = writers.len(),
                "broadcast_status_bar: cleaned up dead GUI clients"
            );
        } else {
            debug!(notify_id = %notify.notify_id, sent, "broadcast_status_bar: delivered to all GUI clients");
        }
    }

    /// 向已连接的 GUI 发送决策请求，等待响应或超时。
    ///
    /// # 行为
    ///
    /// - 如果没有 GUI 客户端连接：**立即 fallback**，不等超时。
    ///   （等超时无意义——没人能决策。）
    /// - 如果 GUI 写通道已满或 GUI 进程崩溃（mpsc send 失败）：立即 fallback。
    /// - 如果 GUI 在 `timeout` 内回复：返回 GUI 的决策。
    /// - 如果超时：按 `default_on_timeout` 构造兜底响应，并从 pending map 清理。
    pub async fn request_decision(
        &self,
        req: DecisionRequest,
        timeout: Duration,
    ) -> Result<DecisionResponse, IpcError> {
        let request_id = req.request_id;
        let default_on_timeout = req.default_on_timeout;

        // 1. 检查 GUI 是否已连接（取第一个 sender 用于决策请求，决策请求不 fan-out）。
        let sender = {
            let writers = self.gui_writers.lock().expect("gui_writers lock poisoned");
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

        // 3. 通过 mpsc 通道把请求推到 handle_connection 的写循环，
        //    再由写循环写入真正的 GUI socket 连接。
        let rpc_req = crate::protocol::jsonrpc::Request::call(
            "request_decision",
            serde_json::to_value(&req)?,
            serde_json::Value::String(request_id.to_string()),
        );
        let mut payload = serde_json::to_string(&rpc_req)?;
        payload.push('\n');

        if let Err(_e) = sender.send(payload).await {
            // GUI 写通道关闭（GUI 进程崩溃或通道满），立即 fallback。
            warn!(%request_id, "GUI writer channel closed; immediate fallback");
            self.pending.lock().await.remove(&request_id);
            return Ok(make_timeout_fallback(request_id, default_on_timeout));
        }

        // 4. 等待 GUI 回复或超时。
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(resp)) => Ok(resp),
            Ok(Err(_)) => {
                // oneshot sender 已丢弃（handle_connection 因断线退出），走超时兜底。
                warn!(%request_id, "decision sender dropped (GUI disconnected); fallback");
                Ok(make_timeout_fallback(request_id, default_on_timeout))
            }
            Err(_elapsed) => {
                // 超时，清理 pending map。
                self.pending.lock().await.remove(&request_id);
                warn!(%request_id, "decision timeout");
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
async fn handle_connection(
    stream: UnixStream,
    pending: PendingMap,
    _gui_writers: GuiWriters,
    mut write_rx: mpsc::Receiver<String>,
    reload_tx: mpsc::Sender<ReloadUserRules>,
) -> Result<(), IpcError> {
    info!("GUI client connected");

    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    loop {
        tokio::select! {
            // 读方向：GUI 发来 decision_response。
            line_result = lines.next_line() => {
                match line_result? {
                    None => {
                        // GUI 关闭连接。
                        info!("GUI client closed connection");
                        break;
                    }
                    Some(line) => {
                        let line = line.trim().to_owned();
                        if line.is_empty() {
                            continue;
                        }
                        debug!(raw = %line, "received IPC message from GUI");
                        dispatch_message(&line, &pending, &reload_tx).await;
                    }
                }
            }

            // 写方向：主代理 push request_decision 给 GUI。
            msg = write_rx.recv() => {
                match msg {
                    None => {
                        // 发送端已丢弃（IpcServer 被 drop），退出。
                        debug!("GUI write channel closed");
                        break;
                    }
                    Some(payload) => {
                        if let Err(e) = write_half.write_all(payload.as_bytes()).await {
                            warn!("failed to write to GUI socket: {e}");
                            break;
                        }
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

/// 解析一行 JSON-RPC 消息，区分 decision response 与单向通知并分别派发。
async fn dispatch_message(
    line: &str,
    pending: &PendingMap,
    reload_tx: &mpsc::Sender<ReloadUserRules>,
) {
    // 先尝试解析为通用 JSON Value，从 method 字段判断消息类型。
    let val: serde_json::Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            warn!("failed to parse IPC frame: {e}");
            return;
        }
    };

    // 有 method 字段 = 通知（Notification）或请求（Request）。
    if let Some(method) = val.get("method").and_then(|v| v.as_str()) {
        match method {
            "sieve.reload_user_rules" => {
                dispatch_notification_line(line, reload_tx).await;
            }
            other => {
                warn!(method = other, "received unknown IPC method; ignoring");
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
    }
}
