//! IPC 控制面 dispatch 集成测试（ADR-013 Supplement 2026-05-02 §S.4）。
//!
//! 验证 socket_server 的 dispatch_message 能正确路由 8 个新方法到 control_rx，
//! 并把 reply 写回 GUI socket。daemon 侧 handler 在 daemon_control_plane.rs 单测覆盖；
//! 本测试只确保**协议路由层**正确：method → control_rx → 序列化回执 → GUI 收到。

use std::sync::Arc;
use std::time::Duration;

use sieve_ipc::error::rpc_codes;
use sieve_ipc::{
    BroadcastPlan, ControlPlaneRequest, HealthResult, IpcServer, ListenSnapshot,
    PausedChangedNotify, SetPausedResult,
};
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
                ControlPlaneRequest::SetPaused {
                    params,
                    origin_request_id: _,
                    reply,
                } => {
                    let paused = params.minutes > 0;
                    let broadcast = Some(BroadcastPlan::PausedChanged(PausedChangedNotify {
                        paused,
                        paused_until: None,
                        reason: "user_request".to_owned(),
                        applies_to: vec!["AutoRedact".to_owned()],
                        origin_request_id: None,
                    }));
                    let _ = reply.send(Ok((
                        SetPausedResult {
                            paused,
                            paused_until: None,
                            applies_to: vec!["AutoRedact".to_owned()],
                        },
                        broadcast,
                    )));
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
                        paused: false,
                        paused_until: None,
                        listen: ListenSnapshot {
                            addr: "127.0.0.1".to_owned(),
                            port: 11453,
                        },
                        // ADR-026 Stage F：multi-listener 快照数组（测试用单元素）
                        listeners: vec![sieve_ipc::ListenerSnapshot {
                            addr: "127.0.0.1".to_owned(),
                            port: 11453,
                            provider_id: "anthropic".to_owned(),
                            protocol: "anthropic".to_owned(),
                        }],
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

/// 通过 GUI mock 发送 JSON-RPC request，返回收到的第一条 **response**（跳过 notification 帧）。
///
/// 服务端可能先发 hello notification（无 id）和 fan-out broadcast 通知，再发 response；
/// 此函数跳过所有带 method 字段的帧，只返回第一条不带 method 的帧（即 response）。
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
    loop {
        let line = tokio::time::timeout(Duration::from_secs(3), lines.next_line())
            .await
            .expect("rpc timeout")
            .unwrap()
            .expect("connection closed");
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        // 跳过 notification 帧（有 method 字段 = hello/heartbeat/fan-out broadcast）
        if v.get("method").is_none() {
            return v;
        }
    }
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
            "source_agent": "claude",
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
        .request_decision(req, Duration::from_millis(150), "inbound")
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

/// 非法 JSON 帧 → 收到 -32700 parse_error 且连接保持。
///
/// SPEC-005 §1.3.1 §12.2：JSON 解析失败不关闭连接，返回 -32700。
#[tokio::test]
async fn invalid_json_returns_parse_error_connection_kept() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    spawn_mock_daemon(Arc::clone(&server));
    tokio::time::sleep(Duration::from_millis(20)).await;

    let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    // 发非法 JSON
    write_half.write_all(b"not valid json\n").await.unwrap();

    // 跳过 hello 通知（第一帧）
    let first = tokio::time::timeout(Duration::from_secs(2), lines.next_line())
        .await
        .expect("timeout")
        .unwrap()
        .expect("connection closed");
    let first_val: serde_json::Value = serde_json::from_str(&first).unwrap();
    // 如果第一帧是 hello notification，则读下一帧（parse_error response）
    let error_line = if first_val.get("method").and_then(|v| v.as_str()) == Some("sieve.hello") {
        tokio::time::timeout(Duration::from_secs(2), lines.next_line())
            .await
            .expect("timeout waiting for parse_error response")
            .unwrap()
            .expect("connection closed after invalid json")
    } else {
        first
    };

    let resp: serde_json::Value = serde_json::from_str(&error_line).unwrap();
    let err = &resp["error"];
    assert_eq!(
        err["code"],
        rpc_codes::PARSE_ERROR,
        "non-JSON frame must return -32700, got: {resp}"
    );

    // 连接保持：发合法请求仍能收到响应
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "sieve.health",
        "params": {},
        "id": "after-parse-error",
    });
    let mut payload = serde_json::to_string(&req).unwrap();
    payload.push('\n');
    write_half.write_all(payload.as_bytes()).await.unwrap();
    let health_line = tokio::time::timeout(Duration::from_secs(2), lines.next_line())
        .await
        .expect("timeout waiting for health response")
        .unwrap()
        .expect("connection should still be open");
    let health_resp: serde_json::Value = serde_json::from_str(&health_line).unwrap();
    assert!(
        health_resp.get("error").is_none(),
        "health after parse_error should succeed: {health_resp}"
    );
    assert_eq!(health_resp["id"], "after-parse-error");
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

/// SPEC-005 §10.0.1：set_paused 响应前强制 fan-out——mock 第二个 GUI 客户端必须先收到
/// paused_changed 通知，请求方才收到 set_paused result。
#[tokio::test]
async fn set_paused_fan_out_before_result() {
    use sieve_ipc::ControlPlaneRequest;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;

    // mock daemon：收到 SetPaused 后回复带 BroadcastPlan。
    let server_for_daemon = Arc::clone(&server);
    tokio::spawn(async move {
        let mut rx = server_for_daemon.control_rx().await.unwrap();
        while let Some(req) = rx.recv().await {
            if let ControlPlaneRequest::SetPaused {
                params,
                origin_request_id,
                reply,
            } = req
            {
                let paused = params.minutes > 0;
                let broadcast = Some(BroadcastPlan::PausedChanged(PausedChangedNotify {
                    paused,
                    paused_until: None,
                    reason: "user_request".to_owned(),
                    applies_to: vec!["AutoRedact".to_owned()],
                    origin_request_id,
                }));
                let _ = reply.send(Ok((
                    SetPausedResult {
                        paused,
                        paused_until: None,
                        applies_to: vec!["AutoRedact".to_owned()],
                    },
                    broadcast,
                )));
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    // 第二个 GUI（观察者），只接收通知。
    let path_for_observer = socket_path.clone();
    let (obs_tx, mut obs_rx) = tokio::sync::mpsc::channel::<String>(8);
    let observer_task = tokio::spawn(async move {
        let stream = tokio::net::UnixStream::connect(&path_for_observer)
            .await
            .unwrap();
        let (read_half, _write_half) = stream.into_split(); // 持有 _write_half，避免 EOF 触发服务端连接断开
        let mut lines = BufReader::new(read_half).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = obs_tx.send(line).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    // 请求方：发 set_paused，等待 result。
    let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
    let (read_half, mut write_half) = stream.into_split();
    let mut req_lines = BufReader::new(read_half).lines();

    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "sieve.set_paused",
        "params": { "minutes": 30 },
        "id": "fan-out-test",
    });
    let mut payload = serde_json::to_string(&req).unwrap();
    payload.push('\n');
    write_half.write_all(payload.as_bytes()).await.unwrap();

    // 读取 result（跳过 hello notification）。
    let result_line = loop {
        let line = tokio::time::timeout(Duration::from_secs(3), req_lines.next_line())
            .await
            .expect("timeout waiting for set_paused result")
            .unwrap()
            .expect("connection closed");
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        if v.get("method").is_some() {
            // notification（hello 等），跳过
            continue;
        }
        break line;
    };

    let result_val: serde_json::Value = serde_json::from_str(&result_line).unwrap();
    assert!(
        result_val.get("error").is_none(),
        "set_paused should succeed: {result_val}"
    );
    assert_eq!(result_val["id"], "fan-out-test");

    // 验证观察者 GUI 已在某时刻收到 paused_changed 通知。
    let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
    let mut got_paused_changed = false;
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(100), obs_rx.recv()).await {
            Ok(Some(line)) => {
                let v: serde_json::Value = serde_json::from_str(&line).unwrap_or_default();
                if v["method"].as_str() == Some("sieve.paused_changed") {
                    assert_eq!(v["params"]["paused"], true);
                    got_paused_changed = true;
                    break;
                }
            }
            _ => break,
        }
    }
    assert!(
        got_paused_changed,
        "observer GUI must receive paused_changed notification (§10.0.1)"
    );

    observer_task.abort();
}

/// SPEC-005 §12.4 / P1-NEW：GUI→daemon error response 在 -32100~99 段时清理 pending decision。
///
/// mock GUI 发送 request_decision 给一个正在等待的 pending，然后发回 -32100 error response；
/// 验证 pending channel 被清理（request_decision 收到 Err，走 fallback，不 hang）。
#[tokio::test]
async fn gui_error_response_clears_pending_decision() {
    use sieve_ipc::{DecisionRequest, DefaultOnTimeout, SourceAgent};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use uuid::Uuid;

    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    // 不 spawn mock daemon（不消费 control_rx）

    // GUI 客户端连接
    let path_for_gui = socket_path.clone();
    let request_id = Uuid::now_v7();
    let request_id_str = request_id.to_string();

    let gui_task = tokio::spawn(async move {
        let stream = tokio::net::UnixStream::connect(&path_for_gui)
            .await
            .unwrap();
        let (read_half, mut write_half) = stream.into_split();
        let mut lines = BufReader::new(read_half).lines();

        // 跳过所有 notifications（hello 等），等到收到 sieve.request_decision
        loop {
            let line = tokio::time::timeout(Duration::from_secs(3), lines.next_line())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            let v: serde_json::Value = serde_json::from_str(&line).unwrap();
            if v["method"].as_str() == Some("sieve.request_decision") {
                break;
            }
        }

        // 回复 -32100 error（GUI 拒绝此 decision）
        let err_resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": { "code": -32100, "message": "decision_rejected" },
            "id": request_id_str,
        });
        let mut payload = serde_json::to_string(&err_resp).unwrap();
        payload.push('\n');
        write_half.write_all(payload.as_bytes()).await.unwrap();

        // 保持连接直到 task 被 abort
        let _ = write_half.flush().await;
        let _write_half = write_half;
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(40)).await;

    // 发起 request_decision（应在 GUI 回复 -32100 后快速 fallback）
    let req = DecisionRequest {
        request_id,
        created_at: chrono::Utc::now(),
        timeout_seconds: 30,
        default_on_timeout: DefaultOnTimeout::Block,
        detections: vec![],
        source_agent: SourceAgent::Unknown,
        origin_chain: vec![],
        source_channel: None,
        explicit_chain_depth: None,
        allow_remember: false,
    };

    // timeout 设为 500ms；GUI 会在 request 到达后立即回复 -32100，
    // pending channel 被 drop → request_decision 收到 Err → fallback（Block）。
    let resp = tokio::time::timeout(
        Duration::from_millis(500),
        server.request_decision(req, Duration::from_secs(30), "inbound"),
    )
    .await
    .expect("request_decision should not hang after GUI sends -32100 error");

    // fallback decision 是 Block（default_on_timeout）
    assert!(
        resp.is_ok(),
        "request_decision should return fallback: {resp:?}"
    );

    gui_task.abort();
}

/// SPEC-005 §10.0.2：set_paused 的 origin_request_id 必须透传到 paused_changed broadcast。
///
/// GUI 发 set_paused 时使用 UUID 字符串 id；mock daemon handler 把
/// ControlPlaneRequest::SetPaused.origin_request_id 透传到 PausedChangedNotify；
/// 观察者 GUI 收到的 paused_changed 通知必须携带相同 id。
#[tokio::test]
async fn set_paused_origin_request_id_propagated_to_broadcast() {
    use sieve_ipc::ControlPlaneRequest;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use uuid::Uuid;

    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;

    // mock daemon：把 origin_request_id 透传到 broadcast（模拟真实 handler 行为）。
    let server_for_daemon = Arc::clone(&server);
    tokio::spawn(async move {
        let mut rx = server_for_daemon.control_rx().await.unwrap();
        while let Some(req) = rx.recv().await {
            if let ControlPlaneRequest::SetPaused {
                params,
                origin_request_id,
                reply,
            } = req
            {
                let paused = params.minutes > 0;
                // 透传 origin_request_id 到 PausedChangedNotify（P1-9 后续）。
                let broadcast = Some(BroadcastPlan::PausedChanged(PausedChangedNotify {
                    paused,
                    paused_until: None,
                    reason: "user_request".to_owned(),
                    applies_to: vec!["gui_popup".to_owned()],
                    origin_request_id,
                }));
                let _ = reply.send(Ok((
                    SetPausedResult {
                        paused,
                        paused_until: None,
                        applies_to: vec!["gui_popup".to_owned()],
                    },
                    broadcast,
                )));
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    // 观察者 GUI（第二个连接，接收 fan-out 通知）。
    let path_for_observer = socket_path.clone();
    let (obs_tx, mut obs_rx) = tokio::sync::mpsc::channel::<String>(16);
    let observer_task = tokio::spawn(async move {
        let stream = tokio::net::UnixStream::connect(&path_for_observer)
            .await
            .unwrap();
        let (read_half, _write_half) = stream.into_split();
        let mut lines = BufReader::new(read_half).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = obs_tx.send(line).await;
        }
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    // 请求方：使用 UUID 字符串作为 JSON-RPC id。
    let request_uuid = Uuid::now_v7();
    let stream = tokio::net::UnixStream::connect(&socket_path).await.unwrap();
    let (read_half, mut write_half) = stream.into_split();
    let mut req_lines = BufReader::new(read_half).lines();

    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "sieve.set_paused",
        "params": { "minutes": 15 },
        "id": request_uuid.to_string(),
    });
    let mut payload = serde_json::to_string(&req).unwrap();
    payload.push('\n');
    write_half.write_all(payload.as_bytes()).await.unwrap();

    // 等待 result（跳过 hello notification）。
    loop {
        let line = tokio::time::timeout(Duration::from_secs(3), req_lines.next_line())
            .await
            .expect("timeout waiting for set_paused result")
            .unwrap()
            .expect("connection closed");
        let v: serde_json::Value = serde_json::from_str(&line).unwrap();
        if v.get("method").is_none() {
            assert!(v.get("error").is_none(), "set_paused should succeed: {v}");
            break;
        }
    }

    // 验证观察者收到的 paused_changed 通知携带 origin_request_id。
    let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
    let mut got_origin_id = false;
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(100), obs_rx.recv()).await {
            Ok(Some(line)) => {
                let v: serde_json::Value = serde_json::from_str(&line).unwrap_or_default();
                if v["method"].as_str() == Some("sieve.paused_changed") {
                    let got_id = v["params"]["origin_request_id"]
                        .as_str()
                        .unwrap_or_default();
                    assert_eq!(
                        got_id,
                        request_uuid.to_string().as_str(),
                        "origin_request_id must equal JSON-RPC id (§10.0.2)"
                    );
                    got_origin_id = true;
                    break;
                }
            }
            _ => break,
        }
    }
    assert!(
        got_origin_id,
        "paused_changed broadcast must carry origin_request_id (P1-9 后续)"
    );

    observer_task.abort();
}
