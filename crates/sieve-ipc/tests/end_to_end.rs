//! SPEC-005 端到端集成测试 harness（A1）。
//!
//! 目标：spawn 真实 `IpcServer`，用 Rust mock GUI client（`tokio::net::UnixStream`
//! + newline-delimited JSON）跑通完整 HIPS flow，从 wire 格式角度验证协议合规。
//!
//! 覆盖场景：
//! 1. **握手 + heartbeat**：mock GUI 连接 → 收 `sieve.hello`（7 字段齐全）→
//!    `tokio::time::pause/advance` 加速 26s → 收 `sieve.heartbeat`（§3 §4）
//! 2. **request_decision 单 issue**：`received_at_daemon` 存在 / `merged=false` /
//!    5 个 required response 字段 / daemon 清理 pending（§6.1.1 §6.2）
//! 3. **request_decision 多 issue（merged）**：wire `merged=true` + `issues[]`（§6.1.2）
//! 4. **重连 inflight 丢弃（§3.4）**：断开重连后 `daemon_boot_id` 与上次相同
//! 5. **set_paused 串行化（§10.0.1）**：B GUI 先收 `paused_changed` / A GUI 后收 result
//!
//! 关联：SPEC-005 §3 §4 §6.1 §6.2 §10.0.1 / PROGRESS A1

use std::sync::Arc;
use std::time::Duration;

use sieve_ipc::{
    BroadcastPlan, ControlPlaneRequest, DecisionAction, DecisionRequest, DecisionResponse,
    DefaultOnTimeout, DetectionPayload, Disposition, HelloBuilder, IpcServer, ListRulesResult,
    PausedChangedNotify, PurgeHistoryResult, SetPausedResult, Severity, SourceAgent,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use uuid::Uuid;

// ── 辅助：启动服务端 ──────────────────────────────────────────────────────────

/// 启动一个带 HelloBuilder 的 IpcServer，返回 Arc。
///
/// `boot_id` 固定传入，使 daemon_boot_id 在测试中可断言。
async fn start_server_with_boot_id(socket_path: &std::path::Path, boot_id: Uuid) -> Arc<IpcServer> {
    let (server, listener) = IpcServer::bind(socket_path.to_owned()).unwrap();
    let server = Arc::new(server);

    server
        .set_hello_builder(HelloBuilder {
            daemon_boot_id: boot_id,
            daemon_version: "test-0.1.0".to_owned(),
            audit_db_user_version: 2,
            started_at: chrono::Utc::now(),
            preset: "default".to_owned(),
        })
        .await;

    let s = Arc::clone(&server);
    tokio::spawn(async move { s.run(listener).await });
    // 等服务端 socket 就绪
    tokio::time::sleep(Duration::from_millis(20)).await;
    server
}

/// 连接 socket，返回拆分后的 (lines_reader, write_half)。
async fn connect_gui(
    socket_path: &std::path::Path,
) -> (
    tokio::io::Lines<BufReader<tokio::net::unix::OwnedReadHalf>>,
    tokio::net::unix::OwnedWriteHalf,
) {
    let stream = UnixStream::connect(socket_path).await.unwrap();
    let (read_half, write_half) = stream.into_split();
    let lines = BufReader::new(read_half).lines();
    (lines, write_half)
}

/// 从 `lines` 读取下一个非空帧，带超时（3s）。
async fn next_frame(
    lines: &mut tokio::io::Lines<BufReader<tokio::net::unix::OwnedReadHalf>>,
) -> serde_json::Value {
    let line = tokio::time::timeout(Duration::from_secs(3), lines.next_line())
        .await
        .expect("frame read timeout")
        .expect("io error")
        .expect("connection closed");
    serde_json::from_str(&line).expect("invalid JSON frame")
}

/// 跳过所有 notification 帧（有 method 字段），返回第一个 response 帧（无 method）。
async fn next_response(
    lines: &mut tokio::io::Lines<BufReader<tokio::net::unix::OwnedReadHalf>>,
) -> serde_json::Value {
    loop {
        let v = next_frame(lines).await;
        if v.get("method").is_none() {
            return v;
        }
    }
}

/// 发送一条 JSON-RPC 请求帧。
async fn send_request(
    write_half: &mut tokio::net::unix::OwnedWriteHalf,
    method: &str,
    params: serde_json::Value,
    id: &str,
) {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id,
    });
    let mut payload = serde_json::to_string(&req).unwrap();
    payload.push('\n');
    write_half.write_all(payload.as_bytes()).await.unwrap();
}

/// 发送 decision_response（JSON-RPC response 形式，复用 request 的 id）。
async fn send_decision_response(
    write_half: &mut tokio::net::unix::OwnedWriteHalf,
    request_id: Uuid,
    decision: DecisionAction,
) {
    let resp = DecisionResponse {
        request_id,
        decision,
        decided_at: chrono::Utc::now(),
        by_user: true,
        remember: false,
        context_hint: None,
        ui_phase_when_clicked: None,
    };
    let rpc_resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": serde_json::to_value(&resp).unwrap(),
        "id": request_id.to_string(),
    });
    let mut payload = serde_json::to_string(&rpc_resp).unwrap();
    payload.push('\n');
    write_half.write_all(payload.as_bytes()).await.unwrap();
}

// ── 场景 1：握手 + heartbeat（tokio time pause 加速）─────────────────────────

/// SPEC-005 §3 + §4：mock GUI 连接后收到 `sieve.hello` 7 字段，
/// time::advance(26s) 后收到 `sieve.heartbeat`。
#[tokio::test(start_paused = true)]
async fn scenario_1_handshake_and_heartbeat() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let boot_id = Uuid::now_v7();
    let server = start_server_with_boot_id(&socket_path, boot_id).await;
    let _ = &server; // keep server alive

    let (mut lines, _write_half) = connect_gui(&socket_path).await;

    // ── 验证 sieve.hello ──

    // 由于 time::pause，server 的 sleep 不会前进，需要主动 yield + advance
    tokio::time::advance(Duration::from_millis(1)).await;
    // 等 server 处理 connect（yield 若干次）
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }

    let hello = tokio::time::timeout(Duration::from_millis(100), lines.next_line())
        .await
        .expect("hello timeout")
        .expect("io error")
        .expect("connection closed");
    let hello: serde_json::Value = serde_json::from_str(&hello).expect("hello JSON");

    assert_eq!(
        hello["method"].as_str(),
        Some("sieve.hello"),
        "第一帧必须是 sieve.hello (§3)"
    );
    let params = &hello["params"];
    assert_eq!(
        params["protocol_version"].as_str(),
        Some("v2"),
        "protocol_version 必须是 v2"
    );
    assert!(
        params.get("daemon_version").is_some(),
        "hello 必须含 daemon_version"
    );
    assert!(params.get("paused").is_some(), "hello 必须含 paused 字段");
    assert!(params.get("preset").is_some(), "hello 必须含 preset 字段");
    assert!(
        params.get("uptime_seconds").is_some(),
        "hello 必须含 uptime_seconds"
    );
    assert!(
        params.get("audit_db_user_version").is_some(),
        "hello 必须含 audit_db_user_version"
    );
    let got_boot_id = params["daemon_boot_id"]
        .as_str()
        .expect("daemon_boot_id 必须是字符串");
    assert_eq!(
        got_boot_id,
        boot_id.to_string().as_str(),
        "daemon_boot_id 必须与注入值一致"
    );

    // ── 验证 sieve.heartbeat（time::advance 26s 触发心跳）──

    // 心跳间隔 25s，advance 26s 确保至少一次心跳
    tokio::time::advance(Duration::from_secs(26)).await;
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }

    let heartbeat_line = tokio::time::timeout(Duration::from_millis(100), lines.next_line())
        .await
        .expect("heartbeat timeout")
        .expect("io error")
        .expect("connection closed");
    let heartbeat: serde_json::Value =
        serde_json::from_str(&heartbeat_line).expect("heartbeat JSON");

    assert_eq!(
        heartbeat["method"].as_str(),
        Some("sieve.heartbeat"),
        "26s 后必须收到 sieve.heartbeat (§4)"
    );
    assert!(
        heartbeat.get("params").is_none(),
        "heartbeat 不应有 params 字段"
    );
    assert_eq!(
        heartbeat["jsonrpc"].as_str(),
        Some("2.0"),
        "heartbeat 必须包含 jsonrpc: \"2.0\""
    );
}

// ── 场景 2：request_decision 单 issue（wire DTO 验证）─────────────────────────

/// SPEC-005 §6.1.1 §6.2：单 issue request_decision wire 格式验证 +
/// mock GUI 发回 decision_response → daemon 清理 pending。
#[tokio::test]
async fn scenario_2_request_decision_single_issue() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let boot_id = Uuid::now_v7();
    let server = start_server_with_boot_id(&socket_path, boot_id).await;

    let request_id = Uuid::now_v7();
    let req = DecisionRequest {
        request_id,
        created_at: chrono::Utc::now(),
        timeout_seconds: 30,
        default_on_timeout: DefaultOnTimeout::Block,
        detections: vec![DetectionPayload {
            rule_id: "IN-CR-05".to_owned(),
            severity: Severity::Critical,
            disposition: Disposition::GuiPopup,
            title: "signTransaction 签名工具调用".to_owned(),
            one_line_summary: "检测到高风险签名操作".to_owned(),
            details: serde_json::json!({
                "template": "signing_tool_use",
                "tool_name": "signTransaction"
            }),
            recommendation: None,
        }],
        source_agent: SourceAgent::Claude,
        origin_chain: vec![],
        source_channel: None,
        explicit_chain_depth: Some(0),
        allow_remember: false,
    };

    // GUI mock：连接并持续接收，等到 request_decision 后回 deny
    let path_clone = socket_path.clone();
    let gui_task = tokio::spawn(async move {
        let (mut lines, mut write_half) = connect_gui(&path_clone).await;

        // 跳过 hello notification，等到 request_decision 帧
        let mut frames_received = 0usize;
        let rd_frame = loop {
            let frame = next_frame(&mut lines).await;
            if frame["method"].as_str() == Some("sieve.request_decision") {
                break frame;
            }
            frames_received += 1;
            if frames_received > 10 {
                panic!("too many frames without request_decision");
            }
        };

        // ── wire 格式断言（§6.1.1）──
        let params = &rd_frame["params"];

        // merged=false（单 issue 形式）
        assert_eq!(params["merged"], false, "单 issue merged 必须是 false");

        // received_at_daemon 字段（P2-2）
        let ts = params["received_at_daemon"]
            .as_str()
            .expect("received_at_daemon 必须存在");
        assert!(ts.ends_with('Z'), "received_at_daemon 必须以 Z 结尾: {ts}");
        assert!(
            !serde_json::to_string(params)
                .unwrap()
                .contains("\"created_at\""),
            "wire 不得出现 created_at 字段名（已改名 received_at_daemon）"
        );

        // 顶层 rule_id（单 issue 平铺）
        assert_eq!(
            params["rule_id"].as_str(),
            Some("IN-CR-05"),
            "单 issue wire 必须有顶层 rule_id"
        );
        assert_eq!(params["severity"].as_str(), Some("critical"));
        assert_eq!(params["direction"].as_str(), Some("inbound"));
        assert_eq!(params["disposition"].as_str(), Some("gui_popup"));
        assert!(
            params.get("issues").is_none(),
            "单 issue 不应有 issues[] 字段"
        );

        // 回复 decision_response（5 个必须字段：request_id/decision/decided_at/by_user/remember）
        let rid = params["request_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        send_decision_response(&mut write_half, rid, DecisionAction::Deny).await;

        // 保持连接
        tokio::time::sleep(Duration::from_secs(2)).await;
    });

    // 等 GUI mock 连接
    tokio::time::sleep(Duration::from_millis(50)).await;

    // daemon 发起 request_decision
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        server.request_decision(req, Duration::from_secs(30), "inbound"),
    )
    .await
    .expect("request_decision timeout")
    .expect("request_decision error");

    // 验证 daemon 收到了正确的 decision
    assert_eq!(result.decision, DecisionAction::Deny);
    assert!(result.by_user);

    gui_task.abort();
}

// ── 场景 3：request_decision 多 issue（merged wire 格式）─────────────────────

/// SPEC-005 §6.1.2：多 issue 时 wire 必须是 `merged=true` + `issues[]` 形式。
#[tokio::test]
async fn scenario_3_request_decision_merged_issues() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let boot_id = Uuid::now_v7();
    let server = start_server_with_boot_id(&socket_path, boot_id).await;

    let request_id = Uuid::now_v7();
    let req = DecisionRequest {
        request_id,
        created_at: chrono::Utc::now(),
        timeout_seconds: 30,
        default_on_timeout: DefaultOnTimeout::Block,
        detections: vec![
            DetectionPayload {
                rule_id: "IN-CR-05".to_owned(),
                severity: Severity::Critical,
                disposition: Disposition::GuiPopup,
                title: "签名操作".to_owned(),
                one_line_summary: "高风险签名".to_owned(),
                details: serde_json::json!({ "template": "signing_tool_use" }),
                recommendation: None,
            },
            DetectionPayload {
                rule_id: "IN-GEN-04".to_owned(),
                severity: Severity::High,
                disposition: Disposition::GuiPopup,
                title: "Markdown 外链".to_owned(),
                one_line_summary: "检测到外链".to_owned(),
                details: serde_json::json!({ "template": "markdown_exfil" }),
                recommendation: None,
            },
        ],
        source_agent: SourceAgent::Claude,
        origin_chain: vec![],
        source_channel: None,
        explicit_chain_depth: Some(0),
        allow_remember: false,
    };

    let path_clone = socket_path.clone();
    let gui_task = tokio::spawn(async move {
        let (mut lines, mut write_half) = connect_gui(&path_clone).await;

        // 等 request_decision
        let rd_frame = loop {
            let frame = next_frame(&mut lines).await;
            if frame["method"].as_str() == Some("sieve.request_decision") {
                break frame;
            }
        };

        let params = &rd_frame["params"];

        // ── 多 issue wire 格式验证（§6.1.2）──
        assert_eq!(params["merged"], true, "多 issue wire 必须 merged=true");
        assert!(
            params.get("rule_id").is_none(),
            "merged 形式顶层不应有 rule_id（§6.1.2 规定不发）"
        );

        let issues = params["issues"]
            .as_array()
            .expect("merged wire 必须有 issues[] 数组");
        assert_eq!(issues.len(), 2, "must have 2 issues");
        assert_eq!(issues[0]["issue_id"].as_str(), Some("i-1"));
        assert_eq!(issues[0]["rule_id"].as_str(), Some("IN-CR-05"));
        assert_eq!(issues[1]["issue_id"].as_str(), Some("i-2"));
        assert_eq!(issues[1]["rule_id"].as_str(), Some("IN-GEN-04"));

        // 顶层 severity = max(critical, high) = critical
        assert_eq!(
            params["severity"].as_str(),
            Some("critical"),
            "多 issue 顶层 severity 取最高"
        );

        // 回复 decision_response
        let rid = params["request_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        send_decision_response(&mut write_half, rid, DecisionAction::Allow).await;

        tokio::time::sleep(Duration::from_secs(2)).await;
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        server.request_decision(req, Duration::from_secs(30), "inbound"),
    )
    .await
    .expect("request_decision timeout")
    .expect("request_decision error");

    assert_eq!(result.decision, DecisionAction::Allow);
    assert!(result.by_user);

    gui_task.abort();
}

// ── 场景 4：重连后 daemon_boot_id 不变（§3.4 inflight 丢弃）────────────────

/// SPEC-005 §3.4：同一 daemon 进程下，GUI 断开重连后 hello.daemon_boot_id 必须相同。
/// 重连时不应重发之前 inflight 的 request_decision。
#[tokio::test]
async fn scenario_4_reconnect_boot_id_consistent() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let boot_id = Uuid::now_v7();
    let server = start_server_with_boot_id(&socket_path, boot_id).await;
    let _ = &server;

    // 第一次连接：接收 hello，记录 boot_id，然后断开
    let first_boot_id = {
        let (mut lines, _write_half) = connect_gui(&socket_path).await;
        let frame = next_frame(&mut lines).await;
        assert_eq!(frame["method"].as_str(), Some("sieve.hello"));
        frame["params"]["daemon_boot_id"]
            .as_str()
            .unwrap()
            .to_owned()
        // _write_half drop → 连接断开
    };

    // 短暂等待断开被 server 感知
    tokio::time::sleep(Duration::from_millis(30)).await;

    // 第二次连接：接收 hello，boot_id 必须相同（同一 daemon 进程）
    let second_boot_id = {
        let (mut lines, _write_half) = connect_gui(&socket_path).await;
        let frame = next_frame(&mut lines).await;
        assert_eq!(frame["method"].as_str(), Some("sieve.hello"));
        let id = frame["params"]["daemon_boot_id"]
            .as_str()
            .unwrap()
            .to_owned();
        id
    };

    assert_eq!(
        first_boot_id, second_boot_id,
        "同一 daemon 进程多次连接的 daemon_boot_id 必须不变（§3.4）"
    );
    assert_eq!(
        first_boot_id,
        boot_id.to_string(),
        "daemon_boot_id 必须等于注入的 boot_id"
    );
}

// ── 场景 5：set_paused 串行化（§10.0.1）──────────────────────────────────────

/// SPEC-005 §10.0.1：set_paused 响应前强制 fan-out——
/// 观察者 GUI（B）先收到 `sieve.paused_changed`，请求方（A）后收到 result。
#[tokio::test]
async fn scenario_5_set_paused_serialized_fan_out() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let boot_id = Uuid::now_v7();
    let server = start_server_with_boot_id(&socket_path, boot_id).await;

    // mock daemon control-plane handler
    let server_for_daemon = Arc::clone(&server);
    tokio::spawn(async move {
        let mut rx = server_for_daemon.control_rx().await.expect("control_rx");
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

    // 观察者 GUI（B）：只接收通知
    let path_for_b = socket_path.clone();
    let (b_tx, mut b_rx) = tokio::sync::mpsc::channel::<serde_json::Value>(16);
    let observer_b = tokio::spawn(async move {
        let (mut lines, _write_half) = connect_gui(&path_for_b).await;
        while let Ok(Ok(Some(line))) =
            tokio::time::timeout(Duration::from_secs(3), lines.next_line()).await
        {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                let _ = b_tx.send(v).await;
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    // 请求方 GUI（A）：发 set_paused，等 result
    let (mut a_lines, mut a_write) = connect_gui(&socket_path).await;
    // 跳过 hello
    let _hello = next_frame(&mut a_lines).await;

    let request_uuid = Uuid::now_v7();
    send_request(
        &mut a_write,
        "sieve.set_paused",
        serde_json::json!({ "minutes": 30 }),
        &request_uuid.to_string(),
    )
    .await;

    // 等待 A 的 result（跳过任何 notification）
    let result = next_response(&mut a_lines).await;
    assert!(
        result.get("error").is_none(),
        "set_paused should succeed: {result}"
    );
    assert_eq!(
        result["id"].as_str(),
        Some(request_uuid.to_string().as_str())
    );

    // 验证 B 已收到 paused_changed 通知（fan-out 在 result 之前完成）
    let mut got_paused_changed = false;
    let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(100), b_rx.recv()).await {
            Ok(Some(v)) => {
                if v["method"].as_str() == Some("sieve.paused_changed") {
                    assert_eq!(v["params"]["paused"], true);
                    // origin_request_id 透传（§10.0.2）
                    let origin_id = v["params"]["origin_request_id"].as_str().unwrap_or("");
                    assert_eq!(
                        origin_id,
                        request_uuid.to_string().as_str(),
                        "origin_request_id 必须透传 request UUID（§10.0.2）"
                    );
                    got_paused_changed = true;
                    break;
                }
            }
            _ => break,
        }
    }
    assert!(
        got_paused_changed,
        "观察者 GUI 必须收到 paused_changed notification（§10.0.1 fan-out 强制先于 result）"
    );

    observer_b.abort();
}

// ── 场景 6：request_decision wire 字段完整性（received_at_daemon + merged + direction）

/// 补充验证：`direction`、`default_on_timeout`、`allow_remember`、`source_agent`
/// 在 wire 中全部使用 snake_case 字符串（§5 enum 规范）。
#[tokio::test]
async fn scenario_6_wire_field_snake_case() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let boot_id = Uuid::now_v7();
    let server = start_server_with_boot_id(&socket_path, boot_id).await;

    let request_id = Uuid::now_v7();
    let req = DecisionRequest {
        request_id,
        created_at: chrono::Utc::now(),
        timeout_seconds: 60,
        default_on_timeout: DefaultOnTimeout::Block,
        detections: vec![DetectionPayload {
            rule_id: "IN-CR-01".to_owned(),
            severity: Severity::Critical,
            disposition: Disposition::HookTerminal,
            title: "BIP39 助记词".to_owned(),
            one_line_summary: "检测到助记词".to_owned(),
            details: serde_json::Value::Null,
            recommendation: None,
        }],
        source_agent: SourceAgent::Claude,
        origin_chain: vec![],
        source_channel: None,
        explicit_chain_depth: Some(0),
        allow_remember: false,
    };

    let path_clone = socket_path.clone();
    let gui_task = tokio::spawn(async move {
        let (mut lines, mut write_half) = connect_gui(&path_clone).await;

        let rd_frame = loop {
            let frame = next_frame(&mut lines).await;
            if frame["method"].as_str() == Some("sieve.request_decision") {
                break frame;
            }
        };

        let params = &rd_frame["params"];

        // snake_case 验证
        assert_eq!(params["direction"].as_str(), Some("inbound"));
        assert_eq!(params["default_on_timeout"].as_str(), Some("block"));
        assert_eq!(params["source_agent"].as_str(), Some("claude"));
        assert_eq!(params["disposition"].as_str(), Some("hook_terminal"));
        assert_eq!(params["severity"].as_str(), Some("critical"));
        assert_eq!(params["allow_remember"], false);

        // context 在 details=null 时缺失或为 null（§6.1.1 optional）
        // （null details → context skip_serializing_if）

        let rid = params["request_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .unwrap();
        send_decision_response(&mut write_half, rid, DecisionAction::Allow).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        server.request_decision(req, Duration::from_secs(30), "inbound"),
    )
    .await
    .expect("timeout")
    .expect("error");

    assert_eq!(result.decision, DecisionAction::Allow);
    gui_task.abort();
}

// ── 场景 7：sieve.list_rules（v2.0+ 兼容扩展）──────────────────────────────

/// SPEC-005 §11A：GUI 发 `sieve.list_rules` → daemon 回包含 `rules` 数组的 result。
///
/// 此测试验证 wire 层路由正确（`sieve.list_rules` 被转发到 control plane 并正确响应），
/// 以及响应字段满足 SPEC 的最低要求（`rules` 数组存在，可能为空）。
#[tokio::test]
async fn scenario_7_list_rules_wire_route() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc_list_rules.sock");
    let boot_id = Uuid::now_v7();
    let server = start_server_with_boot_id(&socket_path, boot_id).await;

    // 模拟 control-plane handler：收到 ListRules 请求直接返回空规则列表。
    let ipc_clone = Arc::clone(&server);
    tokio::spawn(async move {
        let mut rx = ipc_clone.control_rx().await.expect("control_rx");
        while let Some(req) = rx.recv().await {
            if let ControlPlaneRequest::ListRules { reply } = req {
                let _ = reply.send(Ok(ListRulesResult { rules: vec![] }));
            }
            // 其他请求忽略
        }
    });

    let (mut lines, mut write_half) = connect_gui(&socket_path).await;

    // 消费 sieve.hello（跳过所有通知直到看到请求或直接等）
    let _hello = next_frame(&mut lines).await;

    // 发 sieve.list_rules 请求
    let req_id = "test-list-rules-001";
    send_request(
        &mut write_half,
        "sieve.list_rules",
        serde_json::json!({}),
        req_id,
    )
    .await;

    // 读 response（跳过通知帧）
    let resp = next_response(&mut lines).await;

    // 验证 JSON-RPC 格式
    assert_eq!(resp["jsonrpc"].as_str(), Some("2.0"));
    assert_eq!(resp["id"].as_str(), Some(req_id));
    assert!(
        resp.get("error").is_none(),
        "list_rules should not return error: {resp}"
    );

    // 验证 result.rules 字段存在且为数组
    let result: ListRulesResult = serde_json::from_value(resp["result"].clone())
        .expect("result should deserialize to ListRulesResult");
    assert!(
        result.rules.is_empty(),
        "mock handler returns empty rules list"
    );
}

// ── 场景 8：sieve.purge_history（v2.0+ 兼容扩展）──────────────────────────

/// SPEC-005 §11B：GUI 发 `sieve.purge_history` → daemon 回 `{purged_at, rows_deleted}` result。
///
/// 此测试验证 wire 层路由正确，以及响应字段满足 SPEC 最低要求。
#[tokio::test]
async fn scenario_8_purge_history_wire_route() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc_purge_history.sock");
    let boot_id = Uuid::now_v7();
    let server = start_server_with_boot_id(&socket_path, boot_id).await;

    let now_ms = chrono::Utc::now().timestamp_millis();

    // 模拟 control-plane handler
    let ipc_clone = Arc::clone(&server);
    tokio::spawn(async move {
        let mut rx = ipc_clone.control_rx().await.expect("control_rx");
        while let Some(req) = rx.recv().await {
            if let ControlPlaneRequest::PurgeHistory { params: _, reply } = req {
                let _ = reply.send(Ok(PurgeHistoryResult {
                    purged_at: chrono::Utc::now().timestamp_millis(),
                    rows_deleted: 42,
                }));
            }
        }
    });

    let (mut lines, mut write_half) = connect_gui(&socket_path).await;
    let _hello = next_frame(&mut lines).await;

    let req_id = "test-purge-history-001";
    send_request(
        &mut write_half,
        "sieve.purge_history",
        serde_json::json!({ "confirmed_at": now_ms }),
        req_id,
    )
    .await;

    let resp = next_response(&mut lines).await;

    assert_eq!(resp["jsonrpc"].as_str(), Some("2.0"));
    assert_eq!(resp["id"].as_str(), Some(req_id));
    assert!(
        resp.get("error").is_none(),
        "purge_history should not return error: {resp}"
    );

    let result: PurgeHistoryResult = serde_json::from_value(resp["result"].clone())
        .expect("result should deserialize to PurgeHistoryResult");
    assert_eq!(result.rows_deleted, 42);
    assert!(
        result.purged_at >= now_ms,
        "purged_at should be >= confirmed_at"
    );
}
