//! IPC 控制面 dispatch 集成测试（ADR-013 Supplement 2026-05-02 §S.4）。
//!
//! 验证 socket_server 的 dispatch_message 能正确路由 8 个新方法到 control_rx，
//! 并把 reply 写回 GUI socket。daemon 侧 handler 在 daemon_control_plane.rs 单测覆盖；
//! 本测试只确保**协议路由层**正确：method → control_rx → 序列化回执 → GUI 收到。

use std::sync::Arc;
use std::time::Duration;

use sieve_ipc::error::rpc_codes;
use sieve_ipc::{ControlPlaneRequest, HealthResult, IpcServer, ListenSnapshot, SetPausedResult};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// 启动 IPC server 并返回 Arc<IpcServer>。
async fn start_server(socket_path: &std::path::Path) -> Arc<IpcServer> {
    let (server, listener) = IpcServer::bind(socket_path.to_owned()).unwrap();
    let server = Arc::new(server);
    let s = Arc::clone(&server);
    tokio::spawn(async move { s.run(listener).await });
    tokio::time::sleep(Duration::from_millis(20)).await;
    server
}

/// 启动一个 mock daemon control-plane handler，对所有请求返回固定结果。
fn spawn_mock_daemon(server: Arc<IpcServer>) {
    tokio::spawn(async move {
        let mut rx = server.control_rx().await.expect("control_rx");
        while let Some(req) = rx.recv().await {
            match req {
                ControlPlaneRequest::SetPaused { params, reply } => {
                    let _ = reply.send(Ok(SetPausedResult {
                        paused: params.minutes > 0,
                        until: None,
                        applies_to: vec!["AutoRedact".to_owned()],
                    }));
                }
                ControlPlaneRequest::Health { params: _, reply } => {
                    let _ = reply.send(Ok(HealthResult {
                        daemon_version: "test".to_owned(),
                        protocol_version: "v2".to_owned(),
                        started_at: chrono::Utc::now(),
                        uptime_seconds: 1,
                        preset: sieve_ipc::PresetSnapshot {
                            mode: "default".to_owned(),
                            overrides: Default::default(),
                        },
                        paused: None,
                        listen: ListenSnapshot {
                            addr: "127.0.0.1".to_owned(),
                            port: 11453,
                        },
                        audit_db: sieve_ipc::AuditDbSnapshot {
                            path: "/tmp/audit".to_owned(),
                            size_bytes: 0,
                            schema_version: 2,
                            events_total: 0,
                            events_today: 0,
                        },
                        rules: sieve_ipc::RulesSnapshot {
                            system_count: 0,
                            user_count: 0,
                            last_reload: None,
                        },
                        graylist: sieve_ipc::GraylistSnapshot { active_count: 0 },
                        ipc: sieve_ipc::IpcSnapshot {
                            connected_clients: 1,
                            total_decisions_inflight: 0,
                        },
                    }));
                }
                _ => {}
            }
        }
    });
}

/// 通过 GUI mock 发送 JSON-RPC request，返回收到的第一条 response。
async fn rpc_call(
    socket_path: &std::path::Path,
    method: &str,
    params: serde_json::Value,
    id: &str,
) -> serde_json::Value {
    let stream = UnixStream::connect(socket_path).await.unwrap();
    let (read_half, mut write_half) = stream.into_split();

    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id,
    });
    let mut payload = serde_json::to_string(&req).unwrap();
    payload.push('\n');
    write_half.write_all(payload.as_bytes()).await.unwrap();

    let mut lines = BufReader::new(read_half).lines();
    let line = tokio::time::timeout(Duration::from_secs(2), lines.next_line())
        .await
        .expect("rpc timeout")
        .unwrap()
        .expect("connection closed");
    serde_json::from_str(&line).unwrap()
}

/// 控制面 method 全链路：dispatch_message 路由 sieve.set_paused → control_rx → reply → GUI 收到。
#[tokio::test]
async fn set_paused_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    spawn_mock_daemon(Arc::clone(&server));
    tokio::time::sleep(Duration::from_millis(20)).await;

    let resp = rpc_call(
        &socket_path,
        "sieve.set_paused",
        serde_json::json!({ "minutes": 30 }),
        "req-1",
    )
    .await;

    assert_eq!(resp["id"], "req-1");
    assert!(resp.get("error").is_none(), "should succeed: {resp}");
    let result = &resp["result"];
    assert_eq!(result["paused"], true);
}

/// health round-trip：daemon 返回 snapshot → GUI 解析无误。
#[tokio::test]
async fn health_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    spawn_mock_daemon(Arc::clone(&server));
    tokio::time::sleep(Duration::from_millis(20)).await;

    let resp = rpc_call(&socket_path, "sieve.health", serde_json::json!({}), "req-2").await;

    assert!(resp.get("error").is_none(), "should succeed: {resp}");
    assert_eq!(resp["result"]["protocol_version"], "v2");
    assert_eq!(resp["result"]["listen"]["port"], 11453);
}

/// 未知方法：必须返回 -32601 method_not_found。
#[tokio::test]
async fn unknown_method_returns_method_not_found() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    spawn_mock_daemon(Arc::clone(&server));
    tokio::time::sleep(Duration::from_millis(20)).await;

    let resp = rpc_call(
        &socket_path,
        "sieve.no_such_method",
        serde_json::json!({}),
        "req-3",
    )
    .await;

    assert!(resp.get("result").is_none());
    let err = &resp["error"];
    assert_eq!(err["code"], rpc_codes::METHOD_NOT_FOUND);
}

/// evaluate payload 超过 64KB 必须返回 -32003 payload_too_large。
#[tokio::test]
async fn evaluate_oversized_payload_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    spawn_mock_daemon(Arc::clone(&server));
    tokio::time::sleep(Duration::from_millis(20)).await;

    let big_payload = "A".repeat(65 * 1024);
    let resp = rpc_call(
        &socket_path,
        "sieve.evaluate",
        serde_json::json!({
            "direction": "outbound",
            "content_kind": "raw_text",
            "source_agent": "claude-code",
            "payload": big_payload,
        }),
        "req-4",
    )
    .await;

    let err = &resp["error"];
    assert_eq!(err["code"], rpc_codes::PAYLOAD_TOO_LARGE);
}

/// IpcServer::is_paused 状态机：set_paused_until → is_paused 反映；过期自动恢复。
#[tokio::test]
async fn paused_state_machine() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;

    // 初始未暂停
    assert!(!server.is_paused());
    assert!(server.paused_until_snapshot().is_none());

    // 设置 1 小时后到期
    let until = chrono::Utc::now() + chrono::Duration::hours(1);
    server.set_paused_until(Some(until));
    assert!(server.is_paused());
    assert!(server.paused_until_snapshot().is_some());

    // 设置过去时间 → is_paused 应返回 false 并自动清空
    let past = chrono::Utc::now() - chrono::Duration::seconds(1);
    server.set_paused_until(Some(past));
    assert!(!server.is_paused());
    assert!(
        server.paused_until_snapshot().is_none(),
        "过期 paused 应在 is_paused() 后被自动清空"
    );

    // 显式清空
    server.set_paused_until(Some(until));
    assert!(server.is_paused());
    server.set_paused_until(None);
    assert!(!server.is_paused());
}

/// request_decision 超时时必须广播 sieve.request_decision_canceled 给所有 GUI。
#[tokio::test]
async fn timeout_broadcasts_request_decision_canceled() {
    use sieve_ipc::{DecisionRequest, DefaultOnTimeout, SourceAgent};
    use uuid::Uuid;

    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;

    // 启动 GUI mock，只接收消息但不回复
    let path_for_gui = socket_path.clone();
    let (notify_tx, mut notify_rx) = tokio::sync::mpsc::channel::<String>(8);
    tokio::spawn(async move {
        let stream = UnixStream::connect(&path_for_gui).await.unwrap();
        let (read_half, _write_half) = stream.into_split();
        let mut lines = BufReader::new(read_half).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if !line.trim().is_empty() {
                let _ = notify_tx.send(line).await;
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(40)).await;

    // 发起一个会超时的 decision 请求
    let request_id = Uuid::now_v7();
    let req = DecisionRequest {
        request_id,
        created_at: chrono::Utc::now(),
        timeout_seconds: 1,
        default_on_timeout: DefaultOnTimeout::Block,
        detections: vec![],
        source_agent: SourceAgent::Unknown,
        origin_chain: vec![],
        source_channel: None,
        explicit_chain_depth: None,
        allow_remember: false,
    };

    // 短超时让请求快速 timeout
    let _resp = server
        .request_decision(req, Duration::from_millis(150))
        .await
        .unwrap();

    // 应收到两条消息：先收 request_decision，然后收 request_decision_canceled
    let mut got_canceled = false;
    let deadline = tokio::time::Instant::now() + Duration::from_millis(800);
    while tokio::time::Instant::now() < deadline {
        let recv = tokio::time::timeout(Duration::from_millis(200), notify_rx.recv()).await;
        let Ok(Some(line)) = recv else {
            break;
        };
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        if v["method"].as_str() == Some("sieve.request_decision_canceled") {
            assert_eq!(
                v["params"]["request_id"].as_str(),
                Some(request_id.to_string().as_str())
            );
            assert_eq!(v["params"]["reason"].as_str(), Some("timeout"));
            assert_eq!(v["params"]["auto_decision"].as_str(), Some("deny"));
            got_canceled = true;
            break;
        }
    }
    assert!(got_canceled, "timeout 必须广播 request_decision_canceled");
}

/// daemon control_rx 关闭时 → 客户端收到 internal_error。
#[tokio::test]
async fn control_channel_closed_returns_internal_error() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    // 不 spawn mock daemon。control_rx 拿在哪？由 dispatch_control_plane 内部走 control_tx；
    // 取走 rx 但立刻 drop，模拟 daemon 退出后通道闭合。
    let rx = server.control_rx().await.expect("control_rx");
    drop(rx); // 通道闭合

    let resp = rpc_call(
        &socket_path,
        "sieve.set_paused",
        serde_json::json!({ "minutes": 5 }),
        "req-5",
    )
    .await;

    let err = &resp["error"];
    assert_eq!(err["code"], rpc_codes::INTERNAL_ERROR);
}
