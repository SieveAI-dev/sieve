use std::path::Path;

use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use uuid::Uuid;

use crate::{
    error::IpcError,
    frame_reader::FrameReader,
    protocol::{jsonrpc, DecisionResponse, ReloadUserRules},
};

/// 测试 / mock GUI 用的 IPC 客户端。
///
/// 连接服务端 socket，发送 JSON-RPC response（模拟 GUI 完成决策后的回调）。
/// 不在生产主路径使用——主路径的 GUI 是独立进程（sieve-gui-macos）。
///
/// 关联：ADR-013 §3（协议传输）。
pub struct IpcClient {
    socket_path: std::path::PathBuf,
}

impl IpcClient {
    /// 创建指向 `socket_path` 的客户端（不立即连接）。
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_owned(),
        }
    }

    /// 向服务端发送一个 [`DecisionResponse`]（换行分隔 JSON-RPC response 格式）。
    pub async fn send_decision(&self, resp: &DecisionResponse) -> Result<(), IpcError> {
        let mut stream = UnixStream::connect(&self.socket_path).await?;
        let rpc_resp = jsonrpc::Response {
            jsonrpc: "2.0".to_owned(),
            result: Some(serde_json::to_value(resp)?),
            error: None,
            id: serde_json::Value::String(resp.request_id.to_string()),
        };
        let mut payload = serde_json::to_string(&rpc_resp)?;
        payload.push('\n');
        stream.write_all(payload.as_bytes()).await?;
        Ok(())
    }

    /// 从 socket 读取一条换行分隔的 JSON-RPC request（服务端推来的决策请求）。
    ///
    /// 主要用于 mock GUI 侧读取请求并回复。
    /// 实现使用 FrameReader（SPEC-005 §1.3.1）代替无界 BufReader::lines()。
    pub async fn recv_request(&self) -> Result<serde_json::Value, IpcError> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let mut reader = FrameReader::new(stream);
        let raw = reader.read_frame().await?.ok_or_else(|| {
            IpcError::UnexpectedResponse("connection closed without data".to_owned())
        })?;
        let line = std::str::from_utf8(&raw)
            .map_err(|_| IpcError::UnexpectedResponse("non-UTF8 frame".to_owned()))?;
        let val: serde_json::Value = serde_json::from_str(line)?;
        Ok(val)
    }

    /// 等待来自服务端的决策请求，自动回复指定的决策动作（测试辅助）。
    pub async fn auto_respond(
        socket_path: impl AsRef<Path>,
        request_id: Uuid,
        decision: crate::protocol::DecisionAction,
    ) -> Result<(), IpcError> {
        let path = socket_path.as_ref().to_owned();
        // 短暂重试以等待服务端就绪。
        let stream = retry_connect(&path, 5, std::time::Duration::from_millis(20)).await?;
        let (reader_half, mut writer_half) = stream.into_split();
        let mut frame_reader = FrameReader::new(reader_half);

        // 读帧直到收到第一条非空帧（忽略内容，只要 request_id 匹配就回）。
        while let Some(raw) = frame_reader.read_frame().await? {
            // 跳过空帧和非 UTF-8 帧；收到任意有效帧即回复决策。
            match std::str::from_utf8(&raw) {
                Ok(s) if s.trim().is_empty() => continue,
                Ok(_) => {}
                Err(_) => continue,
            }
            let resp = DecisionResponse {
                request_id,
                decision,
                decided_at: chrono::Utc::now(),
                by_user: true,
                remember: false,
                context_hint: None,
                ui_phase_when_clicked: None,
            };
            let rpc_resp = jsonrpc::Response {
                jsonrpc: "2.0".to_owned(),
                result: Some(serde_json::to_value(&resp)?),
                error: None,
                id: serde_json::Value::String(request_id.to_string()),
            };
            let mut payload = serde_json::to_string(&rpc_resp)?;
            payload.push('\n');
            writer_half.write_all(payload.as_bytes()).await?;
            break;
        }
        Ok(())
    }
}

/// 一次性连接 IPC socket → 发 `sieve.reload_user_rules` 通知 → 关闭。
///
/// 供 `sieve rules edit` 命令（commands/rules.rs）调用，在编辑器关闭后通知
/// daemon 重新加载 `~/.sieve/rules/user.toml`。
///
/// 失败静默丢弃（daemon 可能未运行；用户可手动重启 daemon 生效）。
///
/// 关联：PRD v2.0 §5.5.5（编辑器关闭后 lint + atomic backup + IPC reload）+ ADR-013。
pub async fn send_reload_user_rules_oneshot(
    socket_path: &Path,
    trigger_id: Option<Uuid>,
) -> Result<(), IpcError> {
    let mut stream = UnixStream::connect(socket_path).await?;

    let reload = ReloadUserRules { trigger_id };
    let notification = jsonrpc::Request {
        jsonrpc: "2.0".to_owned(),
        method: "sieve.reload_user_rules".to_owned(),
        params: Some(serde_json::to_value(&reload)?),
        id: None, // 单向通知，无 id。
    };

    let mut payload = serde_json::to_string(&notification)?;
    payload.push('\n');
    stream.write_all(payload.as_bytes()).await?;
    // 不等回复，直接关闭连接。
    Ok(())
}

/// 连接重试辅助——服务端 spawn 后稍有延迟才就绪。
async fn retry_connect(
    path: &std::path::Path,
    attempts: u32,
    delay: std::time::Duration,
) -> Result<UnixStream, IpcError> {
    let mut last_err = None;
    for _ in 0..attempts {
        match UnixStream::connect(path).await {
            Ok(s) => return Ok(s),
            Err(e) => {
                last_err = Some(e);
                tokio::time::sleep(delay).await;
            }
        }
    }
    Err(IpcError::Socket(last_err.unwrap()))
}
