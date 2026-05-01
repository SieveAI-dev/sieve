use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use uuid::Uuid;

use crate::{
    error::IpcError,
    protocol::{jsonrpc, DecisionResponse},
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
    pub async fn recv_request(&self) -> Result<serde_json::Value, IpcError> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
        let line = lines.next_line().await?.ok_or_else(|| {
            IpcError::UnexpectedResponse("connection closed without data".to_owned())
        })?;
        let val: serde_json::Value = serde_json::from_str(&line)?;
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
        let mut lines = BufReader::new(reader_half).lines();

        // 读一条请求（忽略内容，只要 request_id 匹配就回）。
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let resp = DecisionResponse {
                request_id,
                decision,
                decided_at: chrono::Utc::now(),
                by_user: true,
                remember: false,
                context_hint: None,
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
