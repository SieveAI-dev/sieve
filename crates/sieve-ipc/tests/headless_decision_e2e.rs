//! headless 决策 e2e（`sieve.list_pending` / `sieve.resolve_decision` wire 往返）。
//!
//! 验证 SPEC-005 §11D / §11E 的完整链路：client 连 socket → dispatch_message →
//! control_rx → daemon handler 调 IpcServer 方法 → reply → 序列化回执 → client 收到。
//! 覆盖 A 方案授权门禁（Critical 静默 deny）与 not_found 语义。

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use sieve_ipc::{
    ControlPlaneRequest, DecisionAction, DecisionRequest, DetectionPayload, Disposition, IpcServer,
    Severity, SourceAgent,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use uuid::Uuid;

/// 启动 IPC server accept 循环。
async fn start_server(socket_path: &std::path::Path) -> Arc<IpcServer> {
    let (server, listener) = IpcServer::bind(socket_path.to_owned()).unwrap();
    let server = Arc::new(server);
    let s = Arc::clone(&server);
    tokio::spawn(async move { s.run(listener).await });
    tokio::time::sleep(Duration::from_millis(20)).await;
    server
}

/// 启动 headless daemon control loop：把 ListPending / ResolveDecision 路由到真实
/// IpcServer 方法（与 daemon_control_plane 生产路径同款）。
fn spawn_headless_daemon(server: Arc<IpcServer>) {
    tokio::spawn(async move {
        let mut rx = server.control_rx().await.expect("control_rx");
        while let Some(req) = rx.recv().await {
            match req {
                ControlPlaneRequest::ListPending { params: _, reply } => {
                    let _ = reply.send(Ok(server.list_pending().await));
                }
                ControlPlaneRequest::ResolveDecision { params, reply } => {
                    let r = server
                        .resolve_decision(params.request_id, params.decision, params.context_hint)
                        .await;
                    let _ = reply.send(Ok(r));
                }
                _ => {}
            }
        }
    });
}

/// 连 socket，作 GUI mock：读 request_decision 帧但**不回复**（让决策保持 pending）。
/// 返回 join handle（测试结束时可 abort）。
fn spawn_silent_gui(socket_path: std::path::PathBuf) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let stream = UnixStream::connect(&socket_path).await.unwrap();
        let (read_half, _write_half) = stream.into_split();
        let mut lines = BufReader::new(read_half).lines();
        // 持续读（含 hello / request_decision），不回复任何决策。
        while let Ok(Some(_line)) = lines.next_line().await {
            // 只保持连接活着（注册进 gui_writers），不应答。
        }
    })
}

/// 构造带单条指定 severity detection 的入站请求。
fn req_with_severity(sev: Severity) -> DecisionRequest {
    DecisionRequest {
        request_id: Uuid::now_v7(),
        created_at: Utc::now(),
        timeout_seconds: 60,
        default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
        detections: vec![DetectionPayload {
            rule_id: "IN-CR-05".to_owned(),
            severity: sev,
            disposition: Disposition::GuiPopup,
            title: "签名工具调用".to_owned(),
            one_line_summary: "检测到签名工具调用".to_owned(),
            details: serde_json::json!({}),
            recommendation: None,
        }],
        source_agent: SourceAgent::Claude,
        origin_chain: vec![],
        source_channel: None,
        explicit_chain_depth: None,
        allow_remember: false,
    }
}

/// client 发一条 JSON-RPC request，跳过 notification 帧，返回第一条 response。
async fn rpc_call(
    socket_path: &std::path::Path,
    method: &str,
    params: serde_json::Value,
    id: &str,
) -> serde_json::Value {
    let stream = UnixStream::connect(socket_path).await.unwrap();
    let (read_half, mut write_half) = stream.into_split();
    let req = serde_json::json!({
        "jsonrpc": "2.0", "method": method, "params": params, "id": id,
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
        if v.get("method").is_none() {
            return v;
        }
    }
}

/// 造一个 pending：GUI mock 连接后 spawn request_decision，等 pending 插入。
/// 返回 request_id 与 request_decision 的 join handle。
async fn make_pending(
    server: &Arc<IpcServer>,
    socket_path: &std::path::Path,
    sev: Severity,
    provider_id: Option<&str>,
) -> (
    Uuid,
    tokio::task::JoinHandle<Result<sieve_ipc::DecisionResponse, sieve_ipc::IpcError>>,
) {
    spawn_silent_gui(socket_path.to_owned());
    tokio::time::sleep(Duration::from_millis(40)).await; // 等 GUI 连上并注册

    let req = req_with_severity(sev);
    let request_id = req.request_id;
    let s = Arc::clone(server);
    let provider_owned = provider_id.map(str::to_owned);
    let handle = tokio::spawn(async move {
        s.request_decision(
            req,
            Duration::from_secs(60),
            "inbound",
            provider_owned.as_deref(),
        )
        .await
    });
    tokio::time::sleep(Duration::from_millis(60)).await; // 等 pending 插入 + 帧发出
    (request_id, handle)
}

// ── 验收 10：list 无 pending → 空集 ───────────────────────────────────────────

#[tokio::test]
async fn list_pending_empty_wire() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("ipc.sock");
    let server = start_server(&sock).await;
    spawn_headless_daemon(Arc::clone(&server));

    let resp = rpc_call(&sock, "sieve.list_pending", serde_json::json!({}), "c1").await;
    let pending = resp["result"]["pending"].as_array().expect("pending array");
    assert!(pending.is_empty(), "无 pending 应返回空集，got: {resp}");
}

// ── 验收 4 + 6 + 8：list 含 id → Critical resolve 静默 deny → 再 resolve not_found ──

#[tokio::test]
async fn list_then_resolve_critical_silently_denied_wire() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("ipc.sock");
    let server = start_server(&sock).await;
    spawn_headless_daemon(Arc::clone(&server));

    let (request_id, handle) =
        make_pending(&server, &sock, Severity::Critical, Some("anthropic-main")).await;

    // 验收 4：list_pending 非空且含该 id + max_severity=critical + provider_id。
    let listed = rpc_call(&sock, "sieve.list_pending", serde_json::json!({}), "c1").await;
    let pending = listed["result"]["pending"]
        .as_array()
        .expect("pending array");
    assert_eq!(pending.len(), 1, "应有 1 条 pending，got: {listed}");
    assert_eq!(
        pending[0]["request_id"].as_str(),
        Some(request_id.to_string().as_str())
    );
    assert_eq!(pending[0]["max_severity"].as_str(), Some("critical"));
    assert_eq!(pending[0]["provider_id"].as_str(), Some("anthropic-main"));
    assert_eq!(pending[0]["direction"].as_str(), Some("inbound"));

    // 验收 6：Critical resolve allow → 静默改 deny。
    let resolved = rpc_call(
        &sock,
        "sieve.resolve_decision",
        serde_json::json!({ "request_id": request_id.to_string(), "decision": "allow" }),
        "c2",
    )
    .await;
    assert_eq!(resolved["result"]["status"].as_str(), Some("resolved"));
    assert_eq!(
        resolved["result"]["effective_decision"].as_str(),
        Some("deny"),
        "Critical 类 allow 必须静默改 deny（A 方案），got: {resolved}"
    );

    // 原始 request_decision 应收到 deny。
    let outcome = handle.await.expect("join").expect("request_decision");
    assert_eq!(outcome.decision, DecisionAction::Deny);
    assert!(outcome.by_user);
    assert!(!outcome.remember);

    // 验收 8：再次 resolve 同 id → not_found。
    let again = rpc_call(
        &sock,
        "sieve.resolve_decision",
        serde_json::json!({ "request_id": request_id.to_string(), "decision": "deny" }),
        "c3",
    )
    .await;
    assert_eq!(
        again["result"]["status"].as_str(),
        Some("not_found"),
        "已解决的 id 再次 resolve 应 not_found，got: {again}"
    );
}

// ── 验收 7：High resolve allow → 正常放行 ─────────────────────────────────────

#[tokio::test]
async fn resolve_high_allow_passes_wire() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("ipc.sock");
    let server = start_server(&sock).await;
    spawn_headless_daemon(Arc::clone(&server));

    let (request_id, handle) = make_pending(&server, &sock, Severity::High, None).await;

    let resolved = rpc_call(
        &sock,
        "sieve.resolve_decision",
        serde_json::json!({ "request_id": request_id.to_string(), "decision": "allow", "context_hint": "headless 放行" }),
        "c1",
    )
    .await;
    assert_eq!(resolved["result"]["status"].as_str(), Some("resolved"));
    assert_eq!(
        resolved["result"]["effective_decision"].as_str(),
        Some("allow"),
        "High 及以下 allow 正常放行，got: {resolved}"
    );
    let outcome = handle.await.expect("join").expect("request_decision");
    assert_eq!(outcome.decision, DecisionAction::Allow);
    assert!(outcome.by_user);
}

// ── 验收 9：resolve 随机 UUID → not_found ─────────────────────────────────────

#[tokio::test]
async fn resolve_unknown_uuid_not_found_wire() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("ipc.sock");
    let server = start_server(&sock).await;
    spawn_headless_daemon(Arc::clone(&server));

    let resolved = rpc_call(
        &sock,
        "sieve.resolve_decision",
        serde_json::json!({ "request_id": Uuid::now_v7().to_string(), "decision": "deny" }),
        "c1",
    )
    .await;
    assert_eq!(
        resolved["result"]["status"].as_str(),
        Some("not_found"),
        "随机 UUID resolve 应 not_found，got: {resolved}"
    );
    assert!(
        resolved["result"].get("effective_decision").is_none()
            || resolved["result"]["effective_decision"].is_null(),
        "not_found 不应有 effective_decision"
    );
}

// ── 回归守护：CLI 短连接查询不得清空 GUI 的未决 pending（多 client 平等缺陷）──────

#[tokio::test]
async fn cli_short_connections_do_not_clear_gui_pending() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("ipc.sock");
    let server = start_server(&sock).await;
    spawn_headless_daemon(Arc::clone(&server));

    let (request_id, handle) = make_pending(&server, &sock, Severity::Critical, None).await;

    // 连续 3 个 headless 短连接查询（每个 rpc_call 连完即断）。
    // 旧逻辑下每个短连接断开会无差别 map.clear() 清空 GUI pending → 第 2 次查询即空。
    for i in 0..3 {
        let listed = rpc_call(
            &sock,
            "sieve.list_pending",
            serde_json::json!({}),
            &format!("q{i}"),
        )
        .await;
        let pending = listed["result"]["pending"]
            .as_array()
            .expect("pending array");
        assert_eq!(
            pending.len(),
            1,
            "第 {i} 次 CLI 短连接查询后 GUI pending 应仍在（不被短连接断开清空）"
        );
    }

    // GUI pending 仍可被 resolve。
    let resolved = rpc_call(
        &sock,
        "sieve.resolve_decision",
        serde_json::json!({ "request_id": request_id.to_string(), "decision": "deny" }),
        "r",
    )
    .await;
    assert_eq!(resolved["result"]["status"].as_str(), Some("resolved"));
    let _ = handle.await;
}
