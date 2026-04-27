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

    let proc = Command::new(&binary)
        .arg("start")
        .arg("--config")
        .arg(config_file.path())
        .env("SIEVE_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sieve daemon");

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

/// POST /v1/messages 含 fake Anthropic key → 返回 426 + sieve_blocked JSON body。
///
/// 关联 PRD §5.1 OUT-01 / ADR-008。
#[tokio::test]
async fn fake_anthropic_key_blocked_with_426() {
    // 1. 启动 mock 上游（若 sieve 转发了请求，计数器就不为 0，测试 fail）
    let upstream_call_count = Arc::new(AtomicUsize::new(0));
    let counter_clone = upstream_call_count.clone();

    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(move |_req| {
        let c = counter_clone.clone();
        async move {
            c.fetch_add(1, Ordering::SeqCst);
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from_static(b"upstream-not-blocked")))
                .unwrap()
        }
    })
    .await;

    // 2. 启动 sieve daemon（指向 mock 上游，不是真实 Anthropic）
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

    // 4. 验证 status = 426
    assert_eq!(
        resp.status(),
        StatusCode::UPGRADE_REQUIRED,
        "expected 426 Upgrade Required (OUT-01 block)"
    );

    // 5. 验证 body 是 sieve_blocked JSON
    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body_bytes);
    assert!(
        body_str.contains(r#""type":"sieve_blocked""#),
        "body missing sieve_blocked type: {body_str}"
    );
    assert!(
        body_str.contains("OUT-01"),
        "body should mention OUT-01 rule: {body_str}"
    );
    assert!(
        body_str.contains("guidance"),
        "body should contain guidance field: {body_str}"
    );

    // 6. 验证上游未被调用
    assert_eq!(
        upstream_call_count.load(Ordering::SeqCst),
        0,
        "upstream should NOT be called when blocked"
    );
}

/// dry_run = true 时：OUT-01 属于 fail-closed 规则，即使 dry_run 也返回 426。
///
/// 关联 PRD §9 #3 / ADR-007：fail-closed 规则在任何模式（含 dry_run）下都强制 Block。
/// dry_run 只豁免非 fail-closed 的 Critical；OUT-01~12 全部在 FAIL_CLOSED_RULES 名单。
#[tokio::test]
async fn dry_run_fail_closed_still_blocks() {
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

    // OUT-01 是 fail-closed：dry_run 不豁免，仍返回 426
    assert_eq!(
        resp.status(),
        StatusCode::UPGRADE_REQUIRED,
        "dry_run must NOT bypass fail-closed OUT-01 (PRD §9 #3)"
    );
    // 上游不应被调用
    assert_eq!(
        upstream_call_count.load(Ordering::SeqCst),
        0,
        "upstream must NOT be called when fail-closed rule blocks"
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
