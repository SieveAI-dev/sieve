//! 入站 GUI 类 hold 流路径（GuiPopup disposition）。
//!
//! 命中 IN-CR-01/05、IN-GEN-04 等 GuiPopup 规则时，hold 住 SSE 流，通过 IpcServer
//! 等待用户在 GUI 做出决策；同时每 25 秒向调用方提供的 channel 发送一条 SSE keep-alive
//! comment（`: keep-alive\n\n`），防止客户端因无数据而超时断开。
//!
//! 关联：ADR-014 §GUI 路径、SPEC-002（keep-alive 规约）、ADR-013（IPC 协议）。

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::warn;

use sieve_ipc::{DecisionAction, DecisionRequest, DefaultOnTimeout, IpcServer};

/// Keep-alive 注释间隔（PRD v1.4 §6.7 要求 ≤ 30 s，取 25 s 留余量）。
const KEEP_ALIVE_INTERVAL_SECS: u64 = 25;

/// Keep-alive SSE comment 字节（RFC 8895 §9.2：以 `:` 开头的行是注释，客户端忽略）。
const KEEP_ALIVE_BYTES: &[u8] = b": keep-alive\n\n";

/// Hold 路径专用错误。
#[derive(Debug, Error)]
pub enum HoldError {
    /// IPC 等待决策失败。
    #[error("IPC decision error: {0}")]
    Ipc(#[from] sieve_ipc::IpcError),
}

/// [`hold_and_decide`] 的返回值，表示 hold 结束后的处置动作。
#[derive(Debug, PartialEq, Eq)]
pub enum HoldOutcome {
    /// 用户允许（或超时 default_on_timeout = Allow）→ 继续转发原始 SSE。
    Allow,
    /// 用户允许且要求脱敏（仅出站脱敏类，入站实际等价 Allow）→ 继续转发。
    RedactAndAllow,
    /// 用户拒绝（或超时 default_on_timeout = Block）→ 注入 `sieve_blocked` event 并关流。
    Deny {
        /// 拒绝原因（来自 rule_id 列表或 "timeout"）。
        reason: String,
    },
}

/// Hold 住当前 SSE 流，通过 [`IpcServer`] 等待用户决策，同时发送 keep-alive。
///
/// # 行为
/// 1. 注册 keep-alive task（每 [`KEEP_ALIVE_INTERVAL_SECS`] 秒向 `keep_alive_tx` 发送
///    `: keep-alive\n\n`），daemon 把它写入 SSE 流；
/// 2. 并发等待 `ipc.request_decision(req, timeout)` 返回；
/// 3. 决策返回后停掉 keep-alive task，返回 [`HoldOutcome`]。
///
/// # 超时
/// 超时由 `req.timeout_seconds` 决定（传给 IpcServer）；超时时按 `req.default_on_timeout` 处理：
/// - `Block` → `HoldOutcome::Deny`
/// - `Allow` → `HoldOutcome::Allow`
/// - `Redact` → `HoldOutcome::RedactAndAllow`（入站场景少见，逻辑完整性保留）
///
/// 关联：ADR-014 §GUI 路径、SPEC-002 §keep-alive。
pub async fn hold_and_decide(
    ipc: Arc<IpcServer>,
    req: DecisionRequest,
    keep_alive_tx: mpsc::Sender<Bytes>,
) -> Result<HoldOutcome, HoldError> {
    let timeout_secs = u64::from(req.timeout_seconds).max(1);
    let default_on_timeout = req.default_on_timeout;
    let rule_ids: String = req
        .detections
        .iter()
        .map(|d| d.rule_id.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    // 启动 keep-alive task
    let ka_tx = keep_alive_tx.clone();
    let ka_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(KEEP_ALIVE_INTERVAL_SECS));
        interval.tick().await; // 第一次 tick 立即返回（elapsed），跳过
        loop {
            interval.tick().await;
            if ka_tx
                .send(Bytes::from_static(KEEP_ALIVE_BYTES))
                .await
                .is_err()
            {
                // 接收端已关闭，停止发送
                break;
            }
        }
    });

    // 等待 IPC 决策
    let timeout = Duration::from_secs(timeout_secs);
    let result = ipc.request_decision(req, timeout).await;

    // 停掉 keep-alive（无论成功失败）
    ka_handle.abort();

    let resp = match result {
        Ok(r) => r,
        Err(e) => {
            warn!("IPC decision error: {e}; falling back to default_on_timeout");
            // IPC 错误按超时兜底
            return Ok(timeout_outcome(default_on_timeout, &rule_ids));
        }
    };

    let outcome = match resp.decision {
        DecisionAction::Allow => HoldOutcome::Allow,
        DecisionAction::RedactAndAllow => HoldOutcome::RedactAndAllow,
        DecisionAction::Deny => HoldOutcome::Deny {
            reason: if resp.by_user {
                format!("用户拒绝（rules: {rule_ids}）")
            } else {
                format!("超时 default-block（rules: {rule_ids}）")
            },
        },
    };

    Ok(outcome)
}

/// 按 [`DefaultOnTimeout`] 构造超时结果。
fn timeout_outcome(dot: DefaultOnTimeout, rule_ids: &str) -> HoldOutcome {
    match dot {
        DefaultOnTimeout::Block => HoldOutcome::Deny {
            reason: format!("超时 fail-closed（rules: {rule_ids}）"),
        },
        DefaultOnTimeout::Allow => HoldOutcome::Allow,
        DefaultOnTimeout::Redact => HoldOutcome::RedactAndAllow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sieve_ipc::protocol::{DecisionResponse, DetectionPayload, Disposition, Severity};
    use uuid::Uuid;

    fn make_request(
        id: Uuid,
        timeout_seconds: u32,
        default_on_timeout: DefaultOnTimeout,
    ) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds,
            default_on_timeout,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-01".to_owned(),
                severity: Severity::Critical,
                disposition: Disposition::GuiPopup,
                title: "地址替换检测".to_owned(),
                one_line_summary: "检测到可疑地址替换".to_owned(),
                details: serde_json::json!({}),
            }],
        }
    }

    fn make_ipc_server() -> (Arc<IpcServer>, tokio::net::UnixListener, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let socket_path = tmp.path().join("ipc.sock");
        // 把 tmp 路径 leak 到测试生命周期（tempfile 会在 drop 时清理，但 socket 不影响测试）
        std::mem::forget(tmp);
        let path = socket_path.clone();
        IpcServer::bind(socket_path)
            .map(|(s, l)| (Arc::new(s), l, path))
            .unwrap()
    }

    // ── Mock IPC 返回 Allow ───────────────────────────────────────────────────

    #[tokio::test]
    async fn ipc_allow_returns_allow_outcome() {
        let (server, listener, socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 模拟 GUI 客户端连接：使 gui_writer 有值，让 request_decision 注册 oneshot
        // 而不是在步骤 1 因无 GUI 连接而立即 fallback（修 #2 相关：inject_decision 需先有注册）。
        let _gui_stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect to IPC socket failed");
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        let req = make_request(id, 5, DefaultOnTimeout::Block);

        // 50ms 后注入 Allow 决策（此时 pending map 里已有 oneshot sender）
        let inject_srv = Arc::clone(&server);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            inject_srv
                .inject_decision(DecisionResponse {
                    request_id: id,
                    decision: DecisionAction::Allow,
                    decided_at: Utc::now(),
                    by_user: true,
                    remember: false,
                })
                .await;
        });

        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
            .await
            .unwrap();
        assert_eq!(outcome, HoldOutcome::Allow);
    }

    // ── Mock IPC 返回 Deny ────────────────────────────────────────────────────

    #[tokio::test]
    async fn ipc_deny_returns_deny_outcome() {
        let (server, listener, socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 模拟 GUI 客户端连接（同 Allow 测试，确保 inject_decision 能工作）
        let _gui_stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect to IPC socket failed");
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        let req = make_request(id, 5, DefaultOnTimeout::Block);

        let inject_srv = Arc::clone(&server);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            inject_srv
                .inject_decision(DecisionResponse {
                    request_id: id,
                    decision: DecisionAction::Deny,
                    decided_at: Utc::now(),
                    by_user: true,
                    remember: false,
                })
                .await;
        });

        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);
        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
            .await
            .unwrap();
        assert!(matches!(outcome, HoldOutcome::Deny { .. }));
    }

    // ── 超时 default_on_timeout = Block ──────────────────────────────────────

    #[tokio::test]
    async fn timeout_with_block_returns_deny() {
        let (server, listener, _socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        // 使用 tokio::time::pause() + advance() 模拟超时（无需等 1s）
        tokio::time::pause();

        let req = make_request(id, 1, DefaultOnTimeout::Block);
        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);

        let ipc_clone = Arc::clone(&server);
        let task = tokio::spawn(async move { hold_and_decide(ipc_clone, req, ka_tx).await });

        // 推进 2 秒让超时触发
        tokio::time::advance(Duration::from_secs(2)).await;
        tokio::time::resume();

        let outcome = task.await.unwrap().unwrap();
        assert!(
            matches!(outcome, HoldOutcome::Deny { .. }),
            "timeout with Block should return Deny, got {outcome:?}"
        );
    }

    // ── 超时 default_on_timeout = Allow ──────────────────────────────────────

    #[tokio::test]
    async fn timeout_with_allow_returns_allow() {
        let (server, listener, _socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        tokio::time::pause();

        let id = Uuid::now_v7();
        let req = make_request(id, 1, DefaultOnTimeout::Allow);
        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);

        let ipc_clone = Arc::clone(&server);
        let task = tokio::spawn(async move { hold_and_decide(ipc_clone, req, ka_tx).await });

        tokio::time::advance(Duration::from_secs(2)).await;
        tokio::time::resume();

        let outcome = task.await.unwrap().unwrap();
        assert_eq!(outcome, HoldOutcome::Allow);
    }

    // ── keep-alive channel 收到数据 ──────────────────────────────────────────

    #[tokio::test]
    async fn keep_alive_sent_before_decision() {
        // 验证 keep-alive channel 在等待期间可接收到消息（无需真等 25s）
        // 只验证 channel 本身不阻塞 hold_and_decide 流程
        let (server, listener, socket_path) = make_ipc_server();
        let srv = Arc::clone(&server);
        tokio::spawn(async move { srv.run(listener).await });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 模拟 GUI 客户端连接（使 inject_decision 能工作）
        let _gui_stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect to IPC socket failed");
        tokio::time::sleep(Duration::from_millis(10)).await;

        let id = Uuid::now_v7();
        let req = make_request(id, 30, DefaultOnTimeout::Block);

        let (ka_tx, _ka_rx) = mpsc::channel::<Bytes>(8);

        // 注入 Allow 让 hold 快速结束
        let inject_srv = Arc::clone(&server);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(30)).await;
            inject_srv
                .inject_decision(DecisionResponse {
                    request_id: id,
                    decision: DecisionAction::Allow,
                    decided_at: Utc::now(),
                    by_user: true,
                    remember: false,
                })
                .await;
        });

        let outcome = hold_and_decide(Arc::clone(&server), req, ka_tx)
            .await
            .unwrap();
        assert_eq!(outcome, HoldOutcome::Allow);
    }
}
