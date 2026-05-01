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

/// GUI 客户端的写通道：向其发送换行分隔的 JSON 字符串即可推送到对端。
///
/// 使用 mpsc 而非直接持有 WriteHalf，这样写检测（`send` 失败）就能代替
/// TCP keepalive 检测 GUI 进程崩溃。通道容量设为 32，满了则视为 GUI 卡死。
type GuiWriter = Arc<Mutex<Option<mpsc::Sender<String>>>>;

/// IPC 服务端，监听 Unix socket，维护与 GUI 的长连接并推送决策请求。
///
/// # 连接语义
///
/// - GUI 启动后主动连接此 socket，保持长连接。
/// - 同一时刻只允许一个 GUI 客户端（多连接时拒绝第二个，记录警告）。
/// - GUI 断线后 `gui_writer` 自动清空；下一次 `request_decision` 立即 fallback。
///
/// # 双向通信模型
///
/// ```text
/// [主代理]  ─request_decision JSON-RPC request─▶  [GUI]
/// [主代理]  ◀─decision_response JSON-RPC response─  [GUI]
/// [主代理]  ─sieve.notify_status_bar notification─▶  [GUI]  （单向）
/// [rules edit]  ─sieve.reload_user_rules notification─▶  [主代理 IpcServer]  （单向）
/// ```
///
/// 每个方向在同一条 TCP/Unix 连接上用换行分隔的 JSON-RPC 帧传输。
/// `handle_connection` 负责从 GUI 读取响应帧并派发到 `pending` map；
/// `request_decision` 通过 `gui_writer` mpsc 通道写入请求帧。
///
/// # 单向通知（v2.0）
///
/// - `broadcast_status_bar`：daemon 向所有已连接 GUI 广播状态栏通知（IN-SEQ-* 命中 / 出站脱敏等）。
///   失败（无客户端 / socket 写错）静默丢弃 + warn 日志，**daemon 主流程不阻塞**。
///   关联：PRD v2.0 §5.7、ADR-013。
/// - `reload_rx`：daemon 通过此 channel 接收来自 `sieve rules edit` 的 reload 通知。
///   关联：PRD v2.0 §5.5.5、ADR-013。
///
/// 关联：ADR-013 §3（JSON-RPC over Unix socket）、ADR-014 §5（GUI 路径）。
pub struct IpcServer {
    socket_path: PathBuf,
    pending: PendingMap,
    /// 当前已连接的 GUI 客户端写通道；无 GUI 时为 None。
    gui_writer: GuiWriter,
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
            gui_writer: Arc::new(Mutex::new(None)),
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
    /// 每个连接独立 spawn；同一时刻只接受一个 GUI 客户端，多余的直接关闭。
    /// 来自 `sieve rules edit` 的 reload 通知（短连接）通过 `reload_tx` 分发到
    /// `reload_rx` channel，daemon 通过 `reload_rx()` 取出接收端监听。
    pub async fn run(&self, listener: UnixListener) {
        info!(socket = %self.socket_path.display(), "IPC server listening");
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let pending = Arc::clone(&self.pending);
                    let gui_writer = Arc::clone(&self.gui_writer);
                    let reload_tx = self.reload_tx.clone();

                    // 检查是否已有 GUI 客户端。
                    // 用 try_lock 避免阻塞 accept 循环；如果锁被占用就放通并让
                    // handle_connection 内部处理（竞态概率极低）。
                    {
                        let mut guard = gui_writer.lock().await;
                        if guard.is_some() {
                            // 已有 GUI 长连接——此新连接可能是短连接通知（如 reload）。
                            // 放通给 handle_notification_or_reject 处理。
                            warn!("second client connected; treating as short-lived notification");
                            drop(guard);
                            tokio::spawn(async move {
                                handle_notification(stream, reload_tx).await;
                            });
                            continue;
                        }
                        // 还没有 GUI 客户端——创建 mpsc 通道，把发送端存入 gui_writer，
                        // 接收端传给 handle_connection 用于写回 GUI。
                        let (tx, rx) = mpsc::channel::<String>(32);
                        *guard = Some(tx);
                        drop(guard);

                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(
                                stream,
                                pending,
                                gui_writer.clone(),
                                rx,
                                reload_tx,
                            )
                            .await
                            {
                                error!("IPC connection error: {e}");
                            }
                            // 连接断开后清理 gui_writer，下一个 GUI 可以重连。
                            let mut w = gui_writer.lock().await;
                            *w = None;
                            info!("GUI client disconnected; gui_writer cleared");
                        });
                    }
                }
                Err(e) => {
                    error!("IPC accept error: {e}");
                    break;
                }
            }
        }
    }

    /// 向已连接的 GUI 广播 StatusBarNotify（单向，不等回复）。
    ///
    /// 失败（无 GUI 客户端 / socket 写错）静默丢弃 + warn 日志，**daemon 主流程不阻塞**。
    ///
    /// 关联：PRD v2.0 §5.7（行为序列 StatusBar 通知）+ §5.4.3（GUI 接口预留）+ ADR-013。
    pub async fn broadcast_status_bar(&self, notify: StatusBarNotify) {
        let sender = {
            let guard = self.gui_writer.lock().await;
            guard.clone()
        };

        let Some(sender) = sender else {
            debug!(
                notify_id = %notify.notify_id,
                "broadcast_status_bar: no GUI client connected; dropping"
            );
            return;
        };

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

        if let Err(_e) = sender.send(payload).await {
            warn!(
                notify_id = %notify.notify_id,
                "broadcast_status_bar: GUI writer channel closed; dropping notify"
            );
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

        // 1. 检查 GUI 是否已连接。
        let sender = {
            let guard = self.gui_writer.lock().await;
            guard.clone()
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

/// 处理单个 GUI 长连接。
///
/// 同时管理两个方向：
/// - **读方向**：从 GUI 读换行分隔的 JSON-RPC response，派发到 `pending` map；
///   同时识别 `sieve.reload_user_rules` 通知并转发到 `reload_tx`。
/// - **写方向**：从 `write_rx` mpsc 通道读取待发送的帧，写入 GUI socket。
///
/// 任一方向出错（GUI 断线 / 写失败）都会退出，调用方负责清理 `gui_writer`。
async fn handle_connection(
    stream: UnixStream,
    pending: PendingMap,
    gui_writer: GuiWriter,
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
    // gui_writer 由 run() 的 spawn closure 在此函数返回后清理。
    drop(gui_writer); // 显式 drop 避免编译器警告。

    Ok(())
}

/// 处理短连接传来的单向通知（如 `sieve.reload_user_rules`）。
///
/// 适用于无 GUI 长连接时 `sieve rules edit` 直接连 daemon 发 reload 的场景；
/// 也用于 GUI 长连接期间新连接携带 reload 的情形。
async fn handle_notification(stream: UnixStream, reload_tx: mpsc::Sender<ReloadUserRules>) {
    let (read_half, _write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }
        dispatch_notification_line(&line, &reload_tx).await;
        break; // 短连接只处理第一条消息。
    }
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
