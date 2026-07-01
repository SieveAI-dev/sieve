//! 共享 IPC client helper（raw JSON-RPC over Unix socket）。
//!
//! 供 headless CLI 各子命令（`decisions` / `pause` / `preset` / `graylist` /
//! `audit purge` / `reload` / `status`）连 `~/.sieve/ipc.sock` 调控制面方法。
//!
//! ## 隔离策略
//!
//! 直接用 raw JSON-RPC over `tokio::net::UnixStream`，ndjson 帧编解码，
//! **只引用 `sieve_ipc::paths`**、避开 sieve-ipc 内部 server/client 模块——
//! CLI 与 daemon 走同一组 IPC 方法（不引入特权 endpoint），但 wire 编解码保持
//! 独立以隔离内部重构。

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use uuid::Uuid;

/// `~/.sieve/ipc.sock` 路径。
pub fn ipc_socket_path() -> Result<PathBuf> {
    let home = sieve_ipc::paths::sieve_home().context("获取 sieve home 失败")?;
    Ok(home.join("ipc.sock"))
}

/// 连接 IPC socket；daemon 不在线时返回含 socket 路径的清晰错误。
pub async fn connect() -> Result<UnixStream> {
    let sock_path = ipc_socket_path()?;
    UnixStream::connect(&sock_path).await.with_context(|| {
        format!(
            "连接 IPC socket 失败（{}）；请确认 sieve daemon 正在运行（sieve status）",
            sock_path.display()
        )
    })
}

/// 在已连接的 stream 上发一条 JSON-RPC call，等待对应 `id` 的 response，返回 `result`。
///
/// 跳过所有非目标 id 的消息（`sieve.hello` / heartbeat / notify 等 daemon 主动推送帧）。
pub async fn rpc_call(
    stream: &mut UnixStream,
    method: &str,
    params: serde_json::Value,
    call_id: &str,
) -> Result<serde_json::Value> {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": call_id,
    });
    let mut payload = serde_json::to_string(&req)?;
    payload.push('\n');
    stream
        .write_all(payload.as_bytes())
        .await
        .context("写 IPC socket 失败")?;

    let (reader, _) = stream.split();
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await.context("读 IPC socket 失败")? {
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }
        let val: serde_json::Value = serde_json::from_str(&line)
            .with_context(|| format!("解析 IPC 响应 JSON 失败: {line}"))?;
        if val.get("id").and_then(|v| v.as_str()) == Some(call_id) {
            if let Some(err) = val.get("error") {
                return Err(anyhow!("IPC 错误响应: {err}"));
            }
            return val
                .get("result")
                .cloned()
                .ok_or_else(|| anyhow!("IPC 响应缺少 result 字段: {val}"));
        }
        // 非目标 id 的消息（hello / heartbeat / notify 等）直接跳过。
    }
    Err(anyhow!("IPC 连接关闭，未收到 id={call_id} 的响应"))
}

/// 连接 + 单次 call + 返回 `result`（供无需长连接的一次性控制面命令用）。
///
/// 自动生成随机 `call_id`（UUID），避免与 daemon 主动推送帧的 id 撞车。
pub async fn rpc_call_oneshot(
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    let mut stream = connect().await?;
    let call_id = Uuid::now_v7().to_string();
    rpc_call(&mut stream, method, params, &call_id).await
}
