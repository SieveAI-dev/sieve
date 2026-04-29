//! Sieve daemon 出站拦截集成测试。
//!
//! 启动真实 sieve 二进制 + mock 上游 + 临时配置，验证 fake key paste → 426 拦截。
//!
//! 测试需要 release 二进制存在（`cargo build --release` 先跑过）。
//! 若 release 二进制不存在，fallback 到 debug 二进制。
//!
//! .cursorrules §3.2：测试代码允许使用 .unwrap()。

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1 as server_http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::convert::Infallible;
use std::io::Write as _;
use std::net::{SocketAddr, TcpListener as StdListener};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

// ─── helpers ──────────────────────────────────────────────────────────────────

fn find_free_port() -> u16 {
    let l = StdListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // sieve-cli → crates/
    p.pop(); // crates/ → workspace root
    p
}

fn sieve_binary() -> PathBuf {
    let root = workspace_root();
    let release = root.join("target/release/sieve");
    if release.exists() {
        return release;
    }
    root.join("target/debug/sieve")
}

fn outbound_rules_path() -> PathBuf {
    workspace_root().join("crates/sieve-rules/rules/outbound.toml")
}

fn inbound_rules_path() -> PathBuf {
    workspace_root().join("crates/sieve-rules/rules/inbound.toml")
}

/// 在 :0 端口启动 plain-HTTP mock 上游，返回 (addr, shutdown sender)。
async fn spawn_mock_upstream<F, Fut>(responder: F) -> (SocketAddr, oneshot::Sender<()>)
where
    F: Fn(Request<Bytes>) -> Fut + Clone + Send + Sync + 'static,
    Fut: std::future::Future<Output = Response<Full<Bytes>>> + Send,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, mut rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => break,
                accept = listener.accept() => {
                    let Ok((stream, _)) = accept else { continue };
                    let io = TokioIo::new(stream);
                    let r = responder.clone();
                    tokio::spawn(async move {
                        let svc = service_fn(move |req: Request<Incoming>| {
                            let r = r.clone();
                            async move {
                                let (parts, body) = req.into_parts();
                                let bytes = body
                                    .collect()
                                    .await
                                    .unwrap_or_default()
                                    .to_bytes();
                                let req_collected = Request::from_parts(parts, bytes);
                                Ok::<_, Infallible>(r(req_collected).await)
                            }
                        });
                        let _ = server_http1::Builder::new()
                            .serve_connection(io, svc)
                            .await;
                    });
                }
            }
        }
    });

    (addr, tx)
}

/// daemon spawn / shutdown guard。Drop 时 SIGKILL。
struct DaemonGuard {
    proc: Child,
    // 持有 tempfile 引用，防止进程运行期间被删除
    _config_file: tempfile::NamedTempFile,
    /// 持有 sieve home 临时目录（若设置了），防止 Drop 时被清理
    _sieve_home: Option<tempfile::TempDir>,
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

/// 启动真实 sieve daemon，返回 (listen_port, guard)。
///
/// 写临时 sieve.toml，其中 rules_path 为绝对路径（避免 cwd 歧义）。
/// `tls_verify_upstream = false`：mock 上游是 plain HTTP，不需要 TLS 握手。
fn spawn_sieve_daemon(upstream_url: &str, dry_run: bool) -> (u16, DaemonGuard) {
    spawn_sieve_daemon_with_home(upstream_url, dry_run, None)
}

/// 启动真实 sieve daemon，支持传入自定义 `sieve_home`（供 IPC 集成测试使用）。
///
/// `sieve_home`：若 Some，则设置 `SIEVE_HOME` 环境变量；daemon 会把 IPC socket 放在此目录下。
fn spawn_sieve_daemon_with_home(
    upstream_url: &str,
    dry_run: bool,
    sieve_home: Option<&std::path::Path>,
) -> (u16, DaemonGuard) {
    let port = find_free_port();
    let rules = outbound_rules_path();
    assert!(
        rules.exists(),
        "outbound rules not found at {}",
        rules.display()
    );
    let inbound_rules = inbound_rules_path();
    assert!(
        inbound_rules.exists(),
        "inbound rules not found at {}",
        inbound_rules.display()
    );

    let mut config_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        config_file,
        r#"upstream_url = "{}"
port = {}
bind_addr = "127.0.0.1"
rules_path = "{}"
inbound_rules_path = "{}"
tls_verify_upstream = false
dry_run = {}
"#,
        upstream_url,
        port,
        rules.display(),
        inbound_rules.display(),
        dry_run,
    )
    .unwrap();

    let binary = sieve_binary();
    assert!(
        binary.exists(),
        "sieve binary not found at {}; run `cargo build --release` first",
        binary.display()
    );

    let mut cmd = Command::new(&binary);
    cmd.arg("start")
        .arg("--config")
        .arg(config_file.path())
        .env("SIEVE_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(home) = sieve_home {
        cmd.env("SIEVE_HOME", home);
    }

    let proc = cmd.spawn().expect("spawn sieve daemon");

    // 等 daemon 监听，最长 10 秒
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_millis(500),
        )
        .is_ok()
        {
            break;
        }
        if Instant::now() >= deadline {
            panic!("sieve daemon did not listen on :{port} within 10 s");
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    (
        port,
        DaemonGuard {
            proc,
            _config_file: config_file,
            _sieve_home: None,
        },
    )
}

/// 构造含 fake Anthropic key 的 /v1/messages 请求 body。
///
/// key 格式：sk-ant-api03- + 93 个 [a-zA-Z0-9_-] + "AA"（符合 OUT-01 pattern）。
fn fake_key_body() -> String {
    // sk-ant-api03- + 93 个 [a-zA-Z0-9_-] + "AA" = 108 chars,符合 OUT-01 pattern
    let suffix_93: String = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_-"
        .chars()
        .cycle()
        .take(93)
        .collect();
    let api_key = format!("sk-ant-api03-{}AA", suffix_93);
    serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{
            "role": "user",
            "content": format!("leaked: {}", api_key),
        }],
    })
    .to_string()
}

fn plain_http_client() -> Client<HttpConnector, Full<Bytes>> {
    Client::builder(TokioExecutor::new()).build(HttpConnector::new())
}

// ─── 测试 ──────────────────────────────────────────────────────────────────────

/// POST /v1/messages 含 fake Anthropic key → AutoRedact 脱敏后转发上游（200）。
///
/// OUT-01 有 disposition=auto_redact，修 #2（disposition 优先于 enforce_action）后：
/// fail-closed 名单里的 OUT-01 不再直接 Block，而是先脱敏再转发。
/// 验证 PRD v1.4 §6.1（AutoRedact 路径）。
///
/// 关联 PRD §5.1 OUT-01 / ADR-016（二维处置矩阵）。
#[tokio::test]
async fn fake_anthropic_key_auto_redacted_and_forwarded() {
    // 1. 启动 mock 上游（OUT-01 AutoRedact → sieve 脱敏后转发，计数器应为 1）
    let upstream_call_count = Arc::new(AtomicUsize::new(0));
    let counter_clone = upstream_call_count.clone();

    // 记录上游收到的请求 body（验证 key 已被脱敏）
    let upstream_body_received = Arc::new(tokio::sync::Mutex::new(Bytes::new()));
    let body_clone = upstream_body_received.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |req| {
        let c = counter_clone.clone();
        let b = body_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            let mut guard = b.lock().await;
            *guard = req.body().clone();
            drop(guard);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"ok-from-upstream")))
                .unwrap()
        }
    })
    .await;

    // 2. 启动 sieve daemon（指向 mock 上游）
    let (sieve_port, _guard) = spawn_sieve_daemon(
        &format!("http://{upstream_addr}"),
        false, /* dry_run=false */
    );

    // 3. 发含 fake key 的请求
    let body = fake_key_body();
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    // 4. OUT-01 AutoRedact：脱敏后转发，上游返回 200
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "OUT-01 AutoRedact 应脱敏后转发，上游返回 200"
    );

    // 5. 上游应被调用一次
    assert_eq!(
        upstream_call_count.load(Ordering::SeqCst),
        1,
        "OUT-01 AutoRedact 后上游应被调用"
    );

    // 6. 上游收到的 body 中不应含原始 key（已被 [REDACTED:OUT-01] 替换）
    let received = upstream_body_received.lock().await.clone();
    let received_str = String::from_utf8_lossy(&received);
    assert!(
        !received_str.contains("sk-ant-api03-"),
        "脱敏后上游不应收到原始 key：{received_str}"
    );
    assert!(
        received_str.contains("REDACTED"),
        "脱敏后 body 应含 REDACTED 占位符：{received_str}"
    );
}

/// dry_run = true 时：OUT-01（disposition=auto_redact）仍然脱敏转发，不受 dry_run 影响。
///
/// 修 #2（disposition 优先）后，OUT-01 走 AutoRedact 路径；
/// dry_run 只影响 Block 路径（是否拦截），不影响 AutoRedact（始终脱敏）。
/// 验证 PRD v1.4 §6.1（AutoRedact 路径）、ADR-016（二维处置矩阵）。
#[tokio::test]
async fn dry_run_auto_redact_still_redacts() {
    let upstream_call_count = Arc::new(AtomicUsize::new(0));
    let counter_clone = upstream_call_count.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |_req| {
        let c = counter_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"OK")))
                .unwrap()
        }
    })
    .await;

    let (sieve_port, _guard) = spawn_sieve_daemon(
        &format!("http://{upstream_addr}"),
        true, /* dry_run=true */
    );

    let body = fake_key_body();
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    // OUT-01 AutoRedact：dry_run 不影响脱敏逻辑，脱敏后转发 → 200
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "OUT-01 AutoRedact 在 dry_run 模式下仍然脱敏转发（200）"
    );
    // 上游应被调用（脱敏后转发）
    assert_eq!(
        upstream_call_count.load(Ordering::SeqCst),
        1,
        "OUT-01 AutoRedact 脱敏后上游应被调用"
    );
}

/// benign 消息（无 Critical 命中）→ 正常透传，返回上游 200。
///
/// 关联 PRD §9 #7：Critical 拦截 FP < 0.5%。
#[tokio::test]
async fn benign_message_passes_through() {
    let upstream_call_count = Arc::new(AtomicUsize::new(0));
    let counter_clone = upstream_call_count.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |_req| {
        let c = counter_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"OK")))
                .unwrap()
        }
    })
    .await;

    let (sieve_port, _guard) = spawn_sieve_daemon(
        &format!("http://{upstream_addr}"),
        false, /* dry_run=false */
    );

    let benign_body = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"hello world, tell me a joke"}]}"#;

    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from_static(benign_body.as_bytes())))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "benign message should pass through with 200"
    );
    assert_eq!(
        upstream_call_count.load(Ordering::SeqCst),
        1,
        "upstream should be called for benign message"
    );
}

// ─── 出站 GUI hold 测试（R2-#1 修复验证）────────────────────────────────────────

/// 模拟 GUI 客户端：连接 IPC socket，通知 ready channel，等待 request_decision，用真实 request_id 回复。
///
/// 时序：
/// 1. 连接 Unix socket（阻塞等待 socket 出现）
/// 2. 发送 ready 信号（via `ready_tx`），通知调用方 GUI 已就绪
/// 3. 阻塞等待服务端推来的 `request_decision` JSON-RPC 帧
/// 4. 提取真实 request_id，用传入 decision 回复
///
/// 从请求帧提取 request_id 而非使用外部传入值，确保 IPC pending map 路由正确。
async fn mock_gui_respond_with_ready(
    socket_path: &std::path::Path,
    decision: sieve_ipc::DecisionAction,
    ready_tx: tokio::sync::oneshot::Sender<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    // 等 socket 出现
    let mut stream = None;
    for _ in 0..200 {
        match UnixStream::connect(socket_path).await {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
    let stream = stream.ok_or("IPC socket not ready after 20s")?;

    // 稍等让 IPC server 完成 handle_connection spawn 和 gui_writer 注册（async 调度延迟）
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 通知主任务：GUI 已连接且 IPC server 已注册，可以发 HTTP 请求了
    let _ = ready_tx.send(());

    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    // 读服务端推来的 request_decision 帧
    while let Some(line) = lines.next_line().await? {
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }
        // 从 JSON-RPC 帧中提取真实 request_id
        let rpc: serde_json::Value = serde_json::from_str(&line)?;
        let params = rpc.get("params").ok_or("no params")?;
        let real_id: uuid::Uuid =
            serde_json::from_value(params["request_id"].clone()).map_err(|e| e.to_string())?;

        // 构造回复
        let resp = sieve_ipc::protocol::DecisionResponse {
            request_id: real_id,
            decision,
            decided_at: chrono::Utc::now(),
            by_user: true,
            remember: false,
        };
        let rpc_resp = sieve_ipc::protocol::jsonrpc::Response {
            jsonrpc: "2.0".to_owned(),
            result: Some(serde_json::to_value(&resp)?),
            error: None,
            id: serde_json::Value::String(real_id.to_string()),
        };
        let mut payload = serde_json::to_string(&rpc_resp)?;
        payload.push('\n');
        writer.write_all(payload.as_bytes()).await?;
        break;
    }
    Ok(())
}

/// 构造含 PEM private key 的请求 body（触发 OUT-07，disposition=gui_popup）。
fn pem_key_body() -> String {
    serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{
            "role": "user",
            "content": "这是我的密钥：-----BEGIN EC PRIVATE KEY-----\nMHQCAQEEINsamplekey\n-----END EC PRIVATE KEY-----",
        }],
    })
    .to_string()
}

/// OUT-07 GuiPopup hold：GUI Deny → 客户端收到 426，上游未被调用。
///
/// 验证 R2-#1 修复：daemon 出站路径正确处理 HoldForDecision action。
/// 关联：PRD v1.4 §5.4.2（出站超时策略表）、ADR-016（二维处置矩阵）。
#[tokio::test]
async fn outbound_gui_popup_deny_returns_426() {
    let upstream_call_count = Arc::new(AtomicUsize::new(0));
    let counter_clone = upstream_call_count.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |_req| {
        let c = counter_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"should-not-reach")))
                .unwrap()
        }
    })
    .await;

    // 为 IPC 准备临时目录
    let sieve_home_dir = tempfile::tempdir().unwrap();
    let sieve_home = sieve_home_dir.path().to_owned();
    let socket_path = sieve_home.join("ipc.sock");

    let (sieve_port, _guard) =
        spawn_sieve_daemon_with_home(&format!("http://{upstream_addr}"), false, Some(&sieve_home));

    // 启动 GUI 模拟任务：先连接 IPC socket（通知 ready），再等 request_decision，回复 Deny
    let socket_path_clone = socket_path.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let ipc_task = tokio::spawn(async move {
        let _ = mock_gui_respond_with_ready(
            &socket_path_clone,
            sieve_ipc::DecisionAction::Deny,
            ready_tx,
        )
        .await;
    });

    // 等 GUI 已连接后再发 HTTP 请求，确保 IPC gui_writer 不为 None（最多等 15 秒）
    let _ = tokio::time::timeout(Duration::from_secs(15), ready_rx).await;

    let body = pem_key_body();
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = ipc_task.await;

    // GUI Deny → 426
    assert_eq!(
        resp.status(),
        StatusCode::UPGRADE_REQUIRED,
        "OUT-07 GuiPopup GUI Deny 应返回 426"
    );

    // 上游不应被调用
    assert_eq!(
        upstream_call_count.load(Ordering::SeqCst),
        0,
        "GUI Deny 后上游不应被调用"
    );
}

// ─── R3-#3 修复验证：RedactAndAllow 脱敏路径 ─────────────────────────────────

/// R3-#3（Anthropic 路径）：只命中 OUT-07 PEM 私钥（gui_popup disposition），
/// mock GUI 返回 `RedactAndAllow`，验证上游收到的 body 不含原 PEM 内容。
///
/// 修复前：redact_hits 为空（仅含 AutoRedact 类），fall-through 原样转发上游 → 泄漏。
/// 修复后：RedactAndAllow 分支把 hold_detections_outbound span 加入 redact_hits，
///        上游收到 [REDACTED:OUT-07] 占位符而非原始 PEM 内容。
#[tokio::test]
async fn r3_fix_gui_redact_and_allow_anthropic_redacts_pem() {
    let upstream_body = Arc::new(tokio::sync::Mutex::new(Bytes::new()));
    let body_clone = upstream_body.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |req| {
        let b = body_clone.clone();
        async move {
            let mut guard = b.lock().await;
            *guard = req.body().clone();
            drop(guard);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"upstream-ok")))
                .unwrap()
        }
    })
    .await;

    let sieve_home_dir = tempfile::tempdir().unwrap();
    let sieve_home = sieve_home_dir.path().to_owned();
    let socket_path = sieve_home.join("ipc.sock");

    let (sieve_port, _guard) =
        spawn_sieve_daemon_with_home(&format!("http://{upstream_addr}"), false, Some(&sieve_home));

    // mock GUI 回复 RedactAndAllow
    let socket_path_clone = socket_path.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let ipc_task = tokio::spawn(async move {
        let _ = mock_gui_respond_with_ready(
            &socket_path_clone,
            sieve_ipc::DecisionAction::RedactAndAllow,
            ready_tx,
        )
        .await;
    });

    let _ = tokio::time::timeout(Duration::from_secs(15), ready_rx).await;

    let body = pem_key_body();
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = ipc_task.await;

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "RedactAndAllow 后应返回 200（脱敏转发）"
    );

    let received = upstream_body.lock().await.clone();
    let received_str = String::from_utf8_lossy(&received);

    assert!(
        !received_str.contains("BEGIN EC PRIVATE KEY"),
        "R3-#3 修复：上游不应收到原始 PEM key header：\n{received_str}"
    );
    assert!(
        received_str.contains("REDACTED"),
        "R3-#3 修复：上游 body 应含 REDACTED 占位符：\n{received_str}"
    );
}

/// R3-#3（Anthropic 路径）：GUI Allow（不脱敏）路径回归——上游收到原始 body。
///
/// Allow 路径不应受 R3-#3 修复影响（用户明确选择原样允许）。
#[tokio::test]
async fn r3_fix_gui_allow_forwards_original_body_regression() {
    let upstream_body = Arc::new(tokio::sync::Mutex::new(Bytes::new()));
    let body_clone = upstream_body.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |req| {
        let b = body_clone.clone();
        async move {
            let mut guard = b.lock().await;
            *guard = req.body().clone();
            drop(guard);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"upstream-ok")))
                .unwrap()
        }
    })
    .await;

    let sieve_home_dir = tempfile::tempdir().unwrap();
    let sieve_home = sieve_home_dir.path().to_owned();
    let socket_path = sieve_home.join("ipc.sock");

    let (sieve_port, _guard) =
        spawn_sieve_daemon_with_home(&format!("http://{upstream_addr}"), false, Some(&sieve_home));

    // mock GUI 回复 Allow（不脱敏）
    let socket_path_clone = socket_path.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let ipc_task = tokio::spawn(async move {
        let _ = mock_gui_respond_with_ready(
            &socket_path_clone,
            sieve_ipc::DecisionAction::Allow,
            ready_tx,
        )
        .await;
    });

    let _ = tokio::time::timeout(Duration::from_secs(15), ready_rx).await;

    let body = pem_key_body();
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = ipc_task.await;

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Allow 后应返回 200（原样转发）"
    );

    let received = upstream_body.lock().await.clone();
    let received_str = String::from_utf8_lossy(&received);

    // Allow 路径：上游应收到原始 PEM key（用户明确允许）
    assert!(
        received_str.contains("BEGIN EC PRIVATE KEY"),
        "Allow 路径（不脱敏）：上游应收到原始 PEM key header：\n{received_str}"
    );
}

/// R3-#3（OpenAI 路径）：只命中 OUT-08 Stripe live key（gui_popup disposition），
/// mock GUI 返回 `RedactAndAllow`，验证上游收到的 body 不含原 Stripe key。
///
/// OpenAI 路径（/v1/chat/completions）与 Anthropic 路径对称修复。
#[tokio::test]
async fn r3_fix_gui_redact_and_allow_openai_redacts_stripe_key() {
    let upstream_body = Arc::new(tokio::sync::Mutex::new(Bytes::new()));
    let body_clone = upstream_body.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |req| {
        let b = body_clone.clone();
        async move {
            let mut guard = b.lock().await;
            *guard = req.body().clone();
            drop(guard);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"upstream-ok")))
                .unwrap()
        }
    })
    .await;

    let sieve_home_dir = tempfile::tempdir().unwrap();
    let sieve_home = sieve_home_dir.path().to_owned();
    let socket_path = sieve_home.join("ipc.sock");

    let (sieve_port, _guard) =
        spawn_sieve_daemon_with_home(&format!("http://{upstream_addr}"), false, Some(&sieve_home));

    let socket_path_clone = socket_path.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let ipc_task = tokio::spawn(async move {
        let _ = mock_gui_respond_with_ready(
            &socket_path_clone,
            sieve_ipc::DecisionAction::RedactAndAllow,
            ready_tx,
        )
        .await;
    });

    let _ = tokio::time::timeout(Duration::from_secs(15), ready_rx).await;

    // Stripe live key 触发 OUT-08（disposition=gui_popup）
    let stripe_key = "sk_live_abcdefghij1234567890";
    let body = serde_json::json!({
        "model": "gpt-4o",
        "stream": false,
        "messages": [{
            "role": "user",
            "content": format!("my stripe key: {stripe_key}"),
        }],
    })
    .to_string();

    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/chat/completions"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = ipc_task.await;

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "OpenAI RedactAndAllow 后应返回 200（脱敏转发）"
    );

    let received = upstream_body.lock().await.clone();
    let received_str = String::from_utf8_lossy(&received);

    assert!(
        !received_str.contains("sk_live_"),
        "R3-#3 OpenAI 修复：上游不应收到原始 Stripe live key：\n{received_str}"
    );
    assert!(
        received_str.contains("REDACTED"),
        "R3-#3 OpenAI 修复：上游 body 应含 REDACTED 占位符：\n{received_str}"
    );
}

/// R3-#3（混合命中）：同时命中 OUT-01（AutoRedact）+ OUT-07（GUI），
/// mock GUI 返回 `RedactAndAllow`，验证两个 span 都被脱敏（去重验证）。
///
/// 混合场景：AutoRedact span 已在 redact_hits 中，GUI span 通过 R3-#3 修复追加。
/// 最终 redact_segments 应同时处理两个 span，上游 body 不含任何原始 secret。
#[tokio::test]
async fn r3_fix_gui_redact_and_allow_mixed_both_spans_redacted() {
    let upstream_body = Arc::new(tokio::sync::Mutex::new(Bytes::new()));
    let body_clone = upstream_body.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |req| {
        let b = body_clone.clone();
        async move {
            let mut guard = b.lock().await;
            *guard = req.body().clone();
            drop(guard);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"upstream-ok")))
                .unwrap()
        }
    })
    .await;

    let sieve_home_dir = tempfile::tempdir().unwrap();
    let sieve_home = sieve_home_dir.path().to_owned();
    let socket_path = sieve_home.join("ipc.sock");

    let (sieve_port, _guard) =
        spawn_sieve_daemon_with_home(&format!("http://{upstream_addr}"), false, Some(&sieve_home));

    let socket_path_clone = socket_path.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let ipc_task = tokio::spawn(async move {
        let _ = mock_gui_respond_with_ready(
            &socket_path_clone,
            sieve_ipc::DecisionAction::RedactAndAllow,
            ready_tx,
        )
        .await;
    });

    let _ = tokio::time::timeout(Duration::from_secs(15), ready_rx).await;

    // 同一 body 同时含 OUT-01（Anthropic key，AutoRedact）+ OUT-07（PEM key，GUI）
    let suffix_93: String = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_-"
        .chars()
        .cycle()
        .take(93)
        .collect();
    let anthropic_key = format!("sk-ant-api03-{}AA", suffix_93);
    let body = serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{
            "role": "user",
            "content": format!(
                "leaked key: {anthropic_key} and pem: -----BEGIN EC PRIVATE KEY-----\nMHQCAQEEINsample\n-----END EC PRIVATE KEY-----"
            ),
        }],
    })
    .to_string();

    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = ipc_task.await;

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "混合命中 RedactAndAllow 后应返回 200"
    );

    let received = upstream_body.lock().await.clone();
    let received_str = String::from_utf8_lossy(&received);

    assert!(
        !received_str.contains("sk-ant-api03-"),
        "混合命中：上游不应含 OUT-01 Anthropic key：\n{received_str}"
    );
    assert!(
        !received_str.contains("BEGIN EC PRIVATE KEY"),
        "混合命中：上游不应含 OUT-07 PEM key header：\n{received_str}"
    );
    assert!(
        received_str.contains("REDACTED"),
        "混合命中：上游 body 应含至少一个 REDACTED 占位符：\n{received_str}"
    );
}

// ─── R11-#1 OpenClaw upstream_routes 路由测试 ─────────────────────────────────

/// 构造含 JWT token 的请求 body（触发 OUT-06，disposition=gui_popup，default_on_timeout=redact）。
fn jwt_body() -> String {
    // 标准 JWT 三段式格式，满足 OUT-06 pattern
    let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0LXN1YmplY3QifQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{
            "role": "user",
            "content": format!("my token: {jwt}"),
        }],
    })
    .to_string()
}

/// R11-#1：X-Sieve-Provider header 存在且路由表命中 → 请求发往 provider 对应上游（port A），
/// 不发往默认上游（port B）。
///
/// 验证 OpenClaw 多 provider 路由核心路径：正确 provider 被调用，默认上游不被调用。
#[tokio::test]
async fn r11_provider_header_routes_to_correct_upstream() {
    // 起两个 fake 上游：A（provider 路由目标）和 B（默认上游）
    let provider_count = Arc::new(AtomicUsize::new(0));
    let default_count = Arc::new(AtomicUsize::new(0));

    let pc = provider_count.clone();
    let (provider_addr, _up_provider) = spawn_mock_upstream(move |_req| {
        let c = pc.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"from-provider-upstream")))
                .unwrap()
        }
    })
    .await;

    let dc = default_count.clone();
    let (default_addr, _up_default) = spawn_mock_upstream(move |_req| {
        let c = dc.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"from-default-upstream")))
                .unwrap()
        }
    })
    .await;

    // 写 upstream-routes.json：openai → provider_addr
    let sieve_home_dir = tempfile::tempdir().unwrap();
    let sieve_home = sieve_home_dir.path().to_owned();
    let routes_json = serde_json::json!({
        "openai": format!("http://{provider_addr}")
    });
    std::fs::write(
        sieve_home.join("upstream-routes.json"),
        routes_json.to_string(),
    )
    .unwrap();

    let (sieve_port, _guard) =
        spawn_sieve_daemon_with_home(&format!("http://{default_addr}"), false, Some(&sieve_home));

    // 发 benign 请求（无 Critical 命中），带 X-Sieve-Provider: openai header
    let benign = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"hello"}]}"#;
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .header("x-sieve-provider", "openai")
                .body(Full::new(Bytes::from_static(benign.as_bytes())))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "provider-routed request should succeed"
    );
    assert_eq!(
        provider_count.load(Ordering::SeqCst),
        1,
        "R11-#1: provider upstream 应被调用一次"
    );
    assert_eq!(
        default_count.load(Ordering::SeqCst),
        0,
        "R11-#1: default upstream 不应被调用"
    );
}

/// R11-#1：X-Sieve-Provider header 存在但路由表未命中 → 使用默认上游（cfg.upstream_url）。
///
/// 验证兜底行为：provider id 不在路由表中时降级到默认上游。
#[tokio::test]
async fn r11_unknown_provider_falls_back_to_default_upstream() {
    let default_count = Arc::new(AtomicUsize::new(0));
    let dc = default_count.clone();
    let (default_addr, _up_default) = spawn_mock_upstream(move |_req| {
        let c = dc.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"from-default")))
                .unwrap()
        }
    })
    .await;

    let sieve_home_dir = tempfile::tempdir().unwrap();
    let sieve_home = sieve_home_dir.path().to_owned();
    // 路由表中没有 "gemini"
    let routes_json = serde_json::json!({ "openai": "http://127.0.0.1:9999" });
    std::fs::write(
        sieve_home.join("upstream-routes.json"),
        routes_json.to_string(),
    )
    .unwrap();

    let (sieve_port, _guard) =
        spawn_sieve_daemon_with_home(&format!("http://{default_addr}"), false, Some(&sieve_home));

    let benign = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"hello"}]}"#;
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .header("x-sieve-provider", "gemini") // 路由表中不存在
                .body(Full::new(Bytes::from_static(benign.as_bytes())))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "unknown provider 应兜底到默认上游"
    );
    assert_eq!(
        default_count.load(Ordering::SeqCst),
        1,
        "R11-#1: 未命中 provider 时默认上游应被调用"
    );
}

/// R11-#1：upstream-routes.json 不存在 → daemon 启动正常，请求走默认上游。
///
/// 验证鲁棒性：路由文件缺失时 daemon 正常启动并转发请求。
#[tokio::test]
async fn r11_missing_routes_json_daemon_starts_normally() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let cc = call_count.clone();
    let (upstream_addr, _up) = spawn_mock_upstream(move |_req| {
        let c = cc.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"ok")))
                .unwrap()
        }
    })
    .await;

    // sieve_home 存在但没有 upstream-routes.json
    let sieve_home_dir = tempfile::tempdir().unwrap();
    let sieve_home = sieve_home_dir.path().to_owned();
    // 不写 upstream-routes.json

    let (sieve_port, _guard) =
        spawn_sieve_daemon_with_home(&format!("http://{upstream_addr}"), false, Some(&sieve_home));

    let benign = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"hello"}]}"#;
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from_static(benign.as_bytes())))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "routes.json 不存在时 daemon 应正常启动并转发请求"
    );
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        1,
        "R11-#1: routes.json 缺失时默认上游应被调用"
    );
}

// ─── R11-#2 Anthropic out 站 default_on_timeout 修复验证 ──────────────────────

/// R11-#2（Anthropic 路径，default=Redact）：触发 OUT-06（JWT，default_on_timeout=redact），
/// IPC 未初始化（SIEVE_HOME 不存在）→ daemon 应按 default_on_timeout=Redact 脱敏后转发（200）。
///
/// 修复前：硬编码 Block → 返回 426；修复后：走规则 default_on_timeout → Redact → 脱敏转发。
#[tokio::test]
async fn r11_anthropic_out06_default_redact_no_ipc_redacts_and_forwards() {
    let upstream_body = Arc::new(tokio::sync::Mutex::new(Bytes::new()));
    let body_clone = upstream_body.clone();

    let (upstream_addr, _up) = spawn_mock_upstream(move |req| {
        let b = body_clone.clone();
        async move {
            let mut guard = b.lock().await;
            *guard = req.body().clone();
            drop(guard);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"ok")))
                .unwrap()
        }
    })
    .await;

    // 不设置 SIEVE_HOME（让 ipc_server = None via fallback to $HOME/.sieve bind collision
    // 或者：把 SIEVE_HOME 设为一个不存在的路径使 bind 失败）
    // 用法：spawn_sieve_daemon（不传 sieve_home）时 SIEVE_HOME 没设，
    // 但 HOME 存在，daemon 会用 $HOME/.sieve/ipc.sock；
    // 那个 socket 可能已存在（另一个测试留下的），或者正常 bind。
    // 为确保 IPC server None，用一个不存在的嵌套路径作为 SIEVE_HOME。
    let nonexistent_home = std::path::PathBuf::from(format!(
        "/tmp/sieve-r11-nonexistent-{}/sub",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    // 不创建这个目录，让 IPC bind 失败 → ipc_server = None

    let (sieve_port, _guard) = spawn_sieve_daemon_with_home(
        &format!("http://{upstream_addr}"),
        false,
        Some(&nonexistent_home),
    );

    let jwt_req = jwt_body();
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(jwt_req)))
                .unwrap(),
        )
        .await
        .unwrap();

    // R11-#2 修复：OUT-06 default_on_timeout=redact → 脱敏转发（200），而非 426
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "R11-#2: OUT-06 default_on_timeout=redact + no IPC → 应脱敏后转发（200），不应 426"
    );

    let received = upstream_body.lock().await.clone();
    let received_str = String::from_utf8_lossy(&received);
    // 上游不应收到原始 JWT（已被脱敏替换）
    assert!(
        !received_str.contains("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0LXN1YmplY3QifQ"),
        "R11-#2: 上游不应收到原始 JWT：{received_str}"
    );
    assert!(
        received_str.contains("REDACTED"),
        "R11-#2: 上游 body 应含 REDACTED 占位符：{received_str}"
    );
}

/// R11-#2（Anthropic 路径，default=Block）：触发 OUT-07（PEM key，default_on_timeout=block），
/// IPC 未初始化 → daemon 应按 default_on_timeout=Block 返回 426。
///
/// 修复前行为一致（偶然正确），修复后行为仍为 426；此测试为回归断言。
#[tokio::test]
async fn r11_anthropic_out07_default_block_no_ipc_returns_426() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let cc = call_count.clone();
    let (upstream_addr, _up) = spawn_mock_upstream(move |_req| {
        let c = cc.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"should-not-reach")))
                .unwrap()
        }
    })
    .await;

    let nonexistent_home = std::path::PathBuf::from(format!(
        "/tmp/sieve-r11-nonexistent-{}/sub",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            + 1 // 避免与上一个测试冲突
    ));

    let (sieve_port, _guard) = spawn_sieve_daemon_with_home(
        &format!("http://{upstream_addr}"),
        false,
        Some(&nonexistent_home),
    );

    let body = pem_key_body();
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    // OUT-07 default_on_timeout=block：无 IPC → fail-closed → 426
    assert_eq!(
        resp.status(),
        StatusCode::UPGRADE_REQUIRED,
        "R11-#2: OUT-07 default_on_timeout=block + no IPC → 应返回 426"
    );
    assert_eq!(
        call_count.load(Ordering::SeqCst),
        0,
        "R11-#2: Block 后上游不应被调用"
    );
}

/// OUT-07 GuiPopup hold：GUI Allow → 请求转发上游，上游返回 200。
///
/// 验证 R2-#1 修复：Allow 决策后原 body 转发给上游。
#[tokio::test]
async fn outbound_gui_popup_allow_forwards_to_upstream() {
    let upstream_call_count = Arc::new(AtomicUsize::new(0));
    let counter_clone = upstream_call_count.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |_req| {
        let c = counter_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"upstream-ok")))
                .unwrap()
        }
    })
    .await;

    let sieve_home_dir = tempfile::tempdir().unwrap();
    let sieve_home = sieve_home_dir.path().to_owned();
    let socket_path = sieve_home.join("ipc.sock");

    let (sieve_port, _guard) =
        spawn_sieve_daemon_with_home(&format!("http://{upstream_addr}"), false, Some(&sieve_home));

    // 启动 GUI 模拟任务：先连接 IPC socket（通知 ready），再等 request_decision，回复 Allow
    let socket_path_clone = socket_path.clone();
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let ipc_task = tokio::spawn(async move {
        let _ = mock_gui_respond_with_ready(
            &socket_path_clone,
            sieve_ipc::DecisionAction::Allow,
            ready_tx,
        )
        .await;
    });

    // 等 GUI 已连接后再发 HTTP 请求
    let _ = tokio::time::timeout(Duration::from_secs(5), ready_rx).await;

    let body = pem_key_body();
    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}/v1/messages"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = ipc_task.await;

    // GUI Allow → 请求到达上游（200）
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "OUT-07 GuiPopup GUI Allow 后应转发到上游并返回 200"
    );
    assert_eq!(
        upstream_call_count.load(Ordering::SeqCst),
        1,
        "GUI Allow 后上游应被调用一次"
    );
}
