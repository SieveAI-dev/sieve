//! F1-b（SPEC-005 §6.2.4）：GUI wire 应答通道的 peer 核验 gate e2e。
//!
//! 真连 Unix socket 模拟 GUI 回 decision_response，注入闭包 verifier 断言 gate 行为：
//! - verifier 拒 + Critical + allow → 静默改写 deny（fail-closed）
//! - gate 只管 Critical 的 allow / redact_and_allow；High 的 allow、任何 deny 不受影响
//! - verifier 未注入（默认）→ 现状保持（既有 9 个模拟 GUI 测试的兼容性由此保证）
//! - verifier 每连接懒执行且至多一次（PeerGate 缓存）
//!
//! 真实核验（SecCode 代码签名）的决定性负例在 sieve-cli 侧
//! `tests/gui_peer_codesign_redteam.rs`（本文件只测 gate 接线语义）。

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use sieve_ipc::{
    socket_client::IpcClient, socket_server::IpcServer, DecisionAction, DecisionRequest,
    DecisionResponse, DefaultOnTimeout, DetectionPayload, Disposition, Severity, SourceAgent,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use uuid::Uuid;

fn req_with_severity(id: Uuid, sev: Severity) -> DecisionRequest {
    DecisionRequest {
        request_id: id,
        created_at: Utc::now(),
        timeout_seconds: 30,
        default_on_timeout: DefaultOnTimeout::Block,
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

async fn start_server(socket_path: &std::path::Path) -> Arc<IpcServer> {
    let (server, listener) = IpcServer::bind(socket_path.to_owned()).unwrap();
    let server = Arc::new(server);
    let s = Arc::clone(&server);
    tokio::spawn(async move { s.run(listener).await });
    tokio::time::sleep(Duration::from_millis(10)).await;
    server
}

/// gate 核心：verifier 拒 + Critical + GUI 回 allow → 主代理拿到的是 deny。
#[tokio::test]
async fn rejected_peer_critical_allow_rewritten_to_deny() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    server.set_peer_verifier(Arc::new(|_fd| false));

    let id = Uuid::now_v7();
    let path_clone = socket_path.clone();
    tokio::spawn(async move {
        IpcClient::auto_respond(path_clone, id, DecisionAction::Allow)
            .await
            .expect("auto_respond failed");
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    let result = server
        .request_decision(
            req_with_severity(id, Severity::Critical),
            Duration::from_secs(3),
            "inbound",
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        result.decision,
        DecisionAction::Deny,
        "peer 核验未通过的 Critical allow 必须静默改写为 deny（F1-b）"
    );
    assert!(!result.remember, "改写为 deny 时 remember 必须清零");
}

/// gate 只管 Critical：verifier 拒 + High + allow → allow 原样放行。
#[tokio::test]
async fn rejected_peer_high_allow_passes_through() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    server.set_peer_verifier(Arc::new(|_fd| false));

    let id = Uuid::now_v7();
    let path_clone = socket_path.clone();
    tokio::spawn(async move {
        IpcClient::auto_respond(path_clone, id, DecisionAction::Allow)
            .await
            .expect("auto_respond failed");
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    let result = server
        .request_decision(
            req_with_severity(id, Severity::High),
            Duration::from_secs(3),
            "inbound",
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        result.decision,
        DecisionAction::Allow,
        "gate 只作用于 Critical，High 的 allow 不受影响"
    );
}

/// verifier 通过 → Critical allow 正常放行。
#[tokio::test]
async fn verified_peer_critical_allow_passes() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    server.set_peer_verifier(Arc::new(|_fd| true));

    let id = Uuid::now_v7();
    let path_clone = socket_path.clone();
    tokio::spawn(async move {
        IpcClient::auto_respond(path_clone, id, DecisionAction::Allow)
            .await
            .expect("auto_respond failed");
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    let result = server
        .request_decision(
            req_with_severity(id, Severity::Critical),
            Duration::from_secs(3),
            "inbound",
            None,
        )
        .await
        .unwrap();

    assert_eq!(result.decision, DecisionAction::Allow);
}

/// verifier 拒 + Critical + GUI 回 deny → deny 原样（拒绝是安全方向，不经核验）。
#[tokio::test]
async fn rejected_peer_critical_deny_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;
    server.set_peer_verifier(Arc::new(|_fd| false));

    let id = Uuid::now_v7();
    let path_clone = socket_path.clone();
    tokio::spawn(async move {
        IpcClient::auto_respond(path_clone, id, DecisionAction::Deny)
            .await
            .expect("auto_respond failed");
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    let result = server
        .request_decision(
            req_with_severity(id, Severity::Critical),
            Duration::from_secs(3),
            "inbound",
            None,
        )
        .await
        .unwrap();

    assert_eq!(result.decision, DecisionAction::Deny);
    assert!(result.by_user);
}

/// verifier 未注入（默认）→ Critical allow 照旧放行（现状保持；gate 是 opt-in）。
#[tokio::test]
async fn no_verifier_keeps_status_quo() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;

    let id = Uuid::now_v7();
    let path_clone = socket_path.clone();
    tokio::spawn(async move {
        IpcClient::auto_respond(path_clone, id, DecisionAction::Allow)
            .await
            .expect("auto_respond failed");
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    let result = server
        .request_decision(
            req_with_severity(id, Severity::Critical),
            Duration::from_secs(3),
            "inbound",
            None,
        )
        .await
        .unwrap();

    assert_eq!(result.decision, DecisionAction::Allow);
}

/// verifier 每连接懒执行且至多一次（PeerGate 缓存）：
/// 同一连接对两条 Critical allow 应答，verifier 只被真调 1 次；两条都被改写 deny。
#[tokio::test]
async fn verifier_invoked_once_per_connection() {
    let tmp = tempfile::tempdir().unwrap();
    let socket_path = tmp.path().join("ipc.sock");
    let server = start_server(&socket_path).await;

    let calls = Arc::new(AtomicUsize::new(0));
    let calls_in_verifier = Arc::clone(&calls);
    server.set_peer_verifier(Arc::new(move |_fd| {
        calls_in_verifier.fetch_add(1, Ordering::SeqCst);
        false
    }));

    let id1 = Uuid::now_v7();
    let id2 = Uuid::now_v7();

    // 手工 GUI mock：长连接读 request_decision，对每条 request 回 allow。
    let path_clone = socket_path.clone();
    let mock = tokio::spawn(async move {
        let stream = UnixStream::connect(&path_clone).await.unwrap();
        let (read_half, mut write_half) = stream.into_split();
        let mut lines = BufReader::new(read_half).lines();
        let mut answered = 0;
        while let Ok(Some(line)) = lines.next_line().await {
            let val: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            // 只应答 request_decision（带 id 的请求帧）
            if val.get("method").and_then(|m| m.as_str()) != Some("sieve.request_decision") {
                continue;
            }
            let Some(request_id) = val
                .pointer("/params/request_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
            else {
                continue;
            };
            let resp = DecisionResponse {
                request_id,
                decision: DecisionAction::Allow,
                decided_at: Utc::now(),
                by_user: true,
                remember: false,
                context_hint: None,
                ui_phase_when_clicked: None,
            };
            let frame = serde_json::json!({
                "jsonrpc": "2.0",
                "result": serde_json::to_value(&resp).unwrap(),
                "id": request_id.to_string(),
            });
            let mut payload = serde_json::to_string(&frame).unwrap();
            payload.push('\n');
            write_half.write_all(payload.as_bytes()).await.unwrap();
            answered += 1;
            if answered == 2 {
                break;
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(30)).await;

    let r1 = server
        .request_decision(
            req_with_severity(id1, Severity::Critical),
            Duration::from_secs(3),
            "inbound",
            None,
        )
        .await
        .unwrap();
    let r2 = server
        .request_decision(
            req_with_severity(id2, Severity::Critical),
            Duration::from_secs(3),
            "inbound",
            None,
        )
        .await
        .unwrap();
    mock.abort();

    assert_eq!(r1.decision, DecisionAction::Deny);
    assert_eq!(r2.decision, DecisionAction::Deny);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "verifier 应按连接缓存，至多真调一次"
    );
}
