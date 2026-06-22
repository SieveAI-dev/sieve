//! sieve-hook 的极简 IPC 客户端（仅供 `codex` 子命令用）。
//!
//! **零依赖铁律**：只用 std（`std::os::unix::net::UnixStream`）+ `serde_json` + `uuid`，
//! **不引 tokio / sieve-ipc**。手写两个小请求/响应映射，宁可复制也不松绑 crate 边界。
//!
//! 协议（与 daemon socket server 一致）：换行分隔（ndjson）JSON-RPC 2.0。
//! 流程：
//! 1. 连 `<sieve_home>/ipc.sock`；
//! 2. 发 `sieve.judge_tool_call` 请求（带唯一 `id`）；
//! 3. 循环读帧，**按 `id` 关联响应**，跳过 daemon 主动推来的 `sieve.hello` /
//!    `request_decision` / 各类通知（这些帧带 `method` 字段，response 不带）；
//! 4. 解出 `verdict` → [`CodexVerdict`]。
//!
//! 全程受 `deadline` 约束（设 socket 读/写超时）；连不上 / 超时 / 协议错误一律返回 `Err`，
//! 由调用方（main.rs codex 分支）走 fail-closed（`exit 2`），**绝不放行**。

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Instant;

use serde_json::Value;

use crate::codex::CodexVerdict;

/// 经 daemon 判定一个工具调用。
///
/// 成功返回 daemon 裁决（allow / deny）；任何失败（连不上、超时、协议错误、daemon error）
/// 返回 `Err(原因)`，调用方据此 fail-closed。
///
/// `deadline` 是本次判定的硬截止（含等 GUI 弹窗）；到点未拿到响应即返回 `Err`。
pub fn judge_tool_call(
    sieve_home: &Path,
    tool_name: &str,
    tool_input: &Value,
    tool_use_id: &str,
    cwd: &str,
    deadline: Instant,
) -> Result<CodexVerdict, String> {
    let socket_path = sieve_home.join("ipc.sock");
    let stream = UnixStream::connect(&socket_path)
        .map_err(|e| format!("connect {} failed: {e}", socket_path.display()))?;

    // 唯一请求 id，用于在 daemon 推送流中关联本请求的响应。
    let id = uuid::Uuid::now_v7().to_string();

    // 告知 daemon 本 client 的剩余预算（ms），daemon 据此 cap GUI 弹窗 timeout，
    // 力争在 client 放弃前回裁决。client 端到 deadline 仍自行 fail-closed 兜底。
    //
    // 减 `DAEMON_MARGIN_MS` 余量：让 daemon 弹窗 timeout 严格短于本 client 的读 deadline，
    // 保证 daemon 在 client 放弃读之前必能回（哪怕是 default_on_timeout=Block 的兜底裁决），
    // 避免「client 先超时 → 走自身 fail-closed」与「daemon 仍在等」竞态。
    const DAEMON_MARGIN_MS: u128 = 3000;
    let remaining_ms = deadline
        .saturating_duration_since(Instant::now())
        .as_millis();
    let timeout_ms = remaining_ms
        .saturating_sub(DAEMON_MARGIN_MS)
        .max(1)
        .min(u128::from(u32::MAX)) as u32;

    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "sieve.judge_tool_call",
        "params": {
            "tool_name": tool_name,
            "tool_input": tool_input,
            "tool_use_id": tool_use_id,
            "cwd": cwd,
            "timeout_ms": timeout_ms,
        }
    });
    let mut payload = serde_json::to_string(&req).map_err(|e| format!("serialize request: {e}"))?;
    payload.push('\n');

    // 写：设写超时后发请求。
    apply_deadline(&stream, deadline, Io::Write)?;
    (&stream)
        .write_all(payload.as_bytes())
        .map_err(|e| format!("write request: {e}"))?;
    (&stream)
        .flush()
        .map_err(|e| format!("flush request: {e}"))?;

    // 读：循环跳过非本 id 的帧（hello / request_decision 推送 / 通知），直到命中响应或超时。
    let mut reader = BufReader::new(&stream);
    loop {
        apply_deadline(&stream, deadline, Io::Read)?;
        let mut line = String::new();
        let n = reader
            .read_line(&mut line)
            .map_err(|e| format!("read response: {e}"))?;
        if n == 0 {
            return Err("connection closed before response".to_owned());
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let val: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => continue, // 非 JSON 帧，跳过
        };
        // 带 method = daemon 主动推送（hello / request_decision / notify），不是本请求的响应。
        if val.get("method").is_some() {
            continue;
        }
        // 按 id 关联：只认本请求的响应。
        if val.get("id").and_then(Value::as_str) != Some(id.as_str()) {
            continue;
        }
        if let Some(err) = val.get("error") {
            let msg = err
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("rpc error");
            return Err(format!("daemon error: {msg}"));
        }
        let result = val
            .get("result")
            .ok_or_else(|| "response missing result".to_owned())?;
        return verdict_from_result(result);
    }
}

/// 把 `judge_tool_call` 响应的 `result` 映射成 [`CodexVerdict`]。
fn verdict_from_result(result: &Value) -> Result<CodexVerdict, String> {
    match result.get("verdict").and_then(Value::as_str) {
        Some("allow") => Ok(CodexVerdict::Allow),
        Some("deny") => {
            let reason = result
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("daemon denied tool call")
                .to_owned();
            Ok(CodexVerdict::Deny { reason })
        }
        // "rewrite" 为后续扩展；本期 daemon 不产，遇到 = 协议外 → fail-closed。
        other => Err(format!("unexpected verdict: {other:?}")),
    }
}

enum Io {
    Read,
    Write,
}

/// 按 `deadline` 剩余时长设 socket 读/写超时；已过 deadline 立即 `Err`。
fn apply_deadline(stream: &UnixStream, deadline: Instant, io: Io) -> Result<(), String> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err("deadline exceeded".to_owned());
    }
    let r = match io {
        Io::Read => stream.set_read_timeout(Some(remaining)),
        Io::Write => stream.set_write_timeout(Some(remaining)),
    };
    r.map_err(|e| format!("set socket timeout: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_allow_parses() {
        let v = verdict_from_result(&serde_json::json!({"verdict": "allow"})).unwrap();
        assert_eq!(v, CodexVerdict::Allow);
    }

    #[test]
    fn verdict_deny_parses_reason() {
        let v =
            verdict_from_result(&serde_json::json!({"verdict": "deny", "reason": "x"})).unwrap();
        assert_eq!(
            v,
            CodexVerdict::Deny {
                reason: "x".to_owned()
            }
        );
    }

    #[test]
    fn verdict_deny_without_reason_has_default() {
        let v = verdict_from_result(&serde_json::json!({"verdict": "deny"})).unwrap();
        assert!(matches!(v, CodexVerdict::Deny { .. }));
    }

    /// 未知裁决 → Err（调用方 fail-closed），绝不静默放行。
    #[test]
    fn unknown_verdict_is_error() {
        assert!(verdict_from_result(&serde_json::json!({"verdict": "rewrite"})).is_err());
        assert!(verdict_from_result(&serde_json::json!({})).is_err());
    }

    /// 连一个不存在的 socket → Err（调用方 fail-closed）。
    #[test]
    fn connect_failure_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let deadline = Instant::now() + std::time::Duration::from_secs(1);
        let r = judge_tool_call(
            dir.path(),
            "exec_command",
            &serde_json::json!({"cmd": "echo hi"}),
            "tu-1",
            "/tmp",
            deadline,
        );
        assert!(r.is_err(), "连不上 daemon 必须返回 Err，调用方 fail-closed");
    }
}
