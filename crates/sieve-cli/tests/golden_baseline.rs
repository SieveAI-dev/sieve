//! A0.4 行为等价 golden 基线（网关三层重构 A1-A3 的回归网）。
//!
//! 对一组**固定**出站请求样本，把「daemon 脱敏后转发给上游的 body 字节」录成 golden
//! 文件（`tests/fixtures/golden/<name>.golden`）。后续 A1（ProviderCodec）/ A2（Transport）/
//! A3（Endpoint 路由）每阶段重构后跑本测试，golden 字节不一致即重构破坏了行为等价 → 失败。
//!
//! 录制：`SIEVE_GOLDEN_RECORD=1 cargo test -p sieve-cli --test golden_baseline`（首次或刻意改基线时）。
//! 校验：默认模式逐字节比对，不一致打印 diff 并失败。
//!
//! 样本只用 **AutoRedact 类**（OUT-01，disposition=auto_redact）→ 无需 IPC/GUI、确定性强，
//! 覆盖 Anthropic（/v1/messages，content[] schema）+ OpenAI（/v1/chat/completions，
//! messages[].content 多态 schema）两条出站编码路径，外加 benign 原样透传基线。
//!
//! .cursorrules §3.2：测试代码允许 `.unwrap()`。

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1 as server_http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::convert::Infallible;
use std::io::Write as _;
use std::net::{SocketAddr, TcpListener as StdListener};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

// ─── harness（lift 自 outbound_block.rs，证实可用）─────────────────────────────

fn find_free_port() -> u16 {
    StdListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
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

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/golden")
}

/// 在 :0 启动 plain-HTTP mock 上游，捕获收到的 body，返回 (addr, body_handle, shutdown)。
async fn spawn_capturing_upstream() -> (
    SocketAddr,
    Arc<tokio::sync::Mutex<Bytes>>,
    oneshot::Sender<()>,
) {
    let received = Arc::new(tokio::sync::Mutex::new(Bytes::new()));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, mut rx) = oneshot::channel::<()>();
    let received_for_task = received.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => break,
                accept = listener.accept() => {
                    let Ok((stream, _)) = accept else { continue };
                    let io = TokioIo::new(stream);
                    let cap = received_for_task.clone();
                    tokio::spawn(async move {
                        let svc = service_fn(move |req: Request<Incoming>| {
                            let cap = cap.clone();
                            async move {
                                let (_parts, body) = req.into_parts();
                                let bytes = body.collect().await.unwrap_or_default().to_bytes();
                                *cap.lock().await = bytes;
                                Ok::<_, Infallible>(
                                    Response::builder()
                                        .status(200)
                                        .body(Full::new(Bytes::from_static(b"ok-from-upstream")))
                                        .unwrap(),
                                )
                            }
                        });
                        let _ = server_http1::Builder::new().serve_connection(io, svc).await;
                    });
                }
            }
        }
    });

    (addr, received, tx)
}

struct DaemonGuard {
    proc: Child,
    _config_file: tempfile::NamedTempFile,
    _sieve_home: tempfile::TempDir,
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

/// 启动真实 sieve daemon（隔离 SIEVE_HOME），规则缺失返回 None（优雅 SKIP）。
fn spawn_sieve_daemon(upstream_url: &str) -> Option<(u16, DaemonGuard)> {
    let port = find_free_port();
    let rules = outbound_rules_path();
    let inbound_rules = inbound_rules_path();
    if !rules.exists() || !inbound_rules.exists() {
        eprintln!("SKIP: 规则文件不存在（需签名规则包），跳过 golden 基线");
        return None;
    }

    let mut config_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        config_file,
        r#"upstream_url = "{}"
port = {}
bind_addr = "127.0.0.1"
rules_path = "{}"
inbound_rules_path = "{}"
tls_verify_upstream = false
dry_run = false
"#,
        upstream_url,
        port,
        rules.display(),
        inbound_rules.display(),
    )
    .unwrap();
    config_file.flush().unwrap();

    let binary = sieve_binary();
    assert!(binary.exists(), "sieve binary 不存在，先 cargo build");
    let home = tempfile::tempdir().unwrap();

    let mut cmd = Command::new(&binary);
    cmd.arg("start")
        .arg("--config")
        .arg(config_file.path())
        .env("SIEVE_LOG", "warn")
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1")
        .env("SIEVE_HOME", home.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let proc = cmd.spawn().expect("spawn sieve daemon");

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
        assert!(Instant::now() < deadline, "daemon 未在 10s 内监听 :{port}");
        std::thread::sleep(Duration::from_millis(100));
    }

    Some((
        port,
        DaemonGuard {
            proc,
            _config_file: config_file,
            _sieve_home: home,
        },
    ))
}

fn plain_http_client() -> Client<HttpConnector, Full<Bytes>> {
    Client::builder(TokioExecutor::new()).build(HttpConnector::new())
}

/// 固定的 OUT-01 Anthropic key（auto_redact）：sk-ant-api03- + 93 [a-zA-Z0-9_-] + "AA"。
fn out01_key() -> String {
    let suffix: String = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_-"
        .chars()
        .cycle()
        .take(93)
        .collect();
    format!("sk-ant-api03-{suffix}AA")
}

/// 比对（或录制）golden。`actual` 为 daemon 脱敏后转发给上游的 body 字节。
fn assert_golden(name: &str, actual: &[u8]) {
    let path = golden_dir().join(format!("{name}.golden"));
    if std::env::var("SIEVE_GOLDEN_RECORD").is_ok() {
        std::fs::create_dir_all(golden_dir()).unwrap();
        std::fs::write(&path, actual).unwrap();
        eprintln!(
            "RECORDED golden: {} ({} bytes)",
            path.display(),
            actual.len()
        );
        return;
    }
    let expected = std::fs::read(&path).unwrap_or_else(|_| {
        panic!(
            "golden 文件不存在：{}（先跑 SIEVE_GOLDEN_RECORD=1 录制）",
            path.display()
        )
    });
    assert_eq!(
        String::from_utf8_lossy(actual),
        String::from_utf8_lossy(&expected),
        "golden 行为等价破坏：{name}\n  golden={}\n  重构后 daemon 脱敏输出与基线字节不一致",
        path.display(),
    );
}

/// 跑一个出站样本：发请求经 daemon → 捕获上游收到的（脱敏后）body → 比对/录制 golden。
async fn run_golden_outbound(name: &str, path: &str, request_body: String) {
    let (upstream_addr, received, _shutdown) = spawn_capturing_upstream().await;
    let Some((sieve_port, _guard)) = spawn_sieve_daemon(&format!("http://{upstream_addr}")) else {
        return; // 规则缺失，优雅 SKIP
    };

    let resp = plain_http_client()
        .request(
            Request::builder()
                .method(http::Method::POST)
                .uri(format!("http://127.0.0.1:{sieve_port}{path}"))
                .header(http::header::CONTENT_TYPE, "application/json")
                .header(http::header::HOST, format!("127.0.0.1:{sieve_port}"))
                .body(Full::new(Bytes::from(request_body)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "{name}: AutoRedact 脱敏后应转发，上游 200"
    );

    let body = received.lock().await.clone();
    assert!(!body.is_empty(), "{name}: 上游应收到（脱敏后）body");
    assert_golden(name, &body);
}

// ─── golden 样本（A1 出站编码路径行为等价基线）──────────────────────────────────

/// M-out-1：Anthropic /v1/messages，OUT-01 key（auto_redact）→ 上游收到脱敏 body。
#[tokio::test]
async fn golden_out01_anthropic_messages() {
    let body = serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 16,
        "messages": [{ "role": "user", "content": format!("leaked: {}", out01_key()) }],
    })
    .to_string();
    run_golden_outbound("out01_anthropic_messages", "/v1/messages", body).await;
}

/// M-out-2：OpenAI /v1/chat/completions，OUT-01 key（auto_redact）→ 上游收到脱敏 body。
/// 验证 OpenAI 出站编码路径（messages[].content 多态 schema）的脱敏写回字节等价。
#[tokio::test]
async fn golden_out01_openai_chat() {
    let body = serde_json::json!({
        "model": "gpt-4o",
        "stream": false,
        "messages": [{ "role": "user", "content": format!("leaked: {}", out01_key()) }],
    })
    .to_string();
    run_golden_outbound("out01_openai_chat", "/v1/chat/completions", body).await;
}

/// M-out-3：Anthropic benign（无命中）→ 上游收到**原样** body（透传基线）。
#[tokio::test]
async fn golden_benign_anthropic_messages() {
    let body = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"hello world, tell me a joke"}]}"#.to_string();
    run_golden_outbound("benign_anthropic_messages", "/v1/messages", body).await;
}

/// M-out-4：OpenAI benign（无命中）→ 上游收到原样 body（透传基线）。
#[tokio::test]
async fn golden_benign_openai_chat() {
    let body = r#"{"model":"gpt-4o","stream":false,"messages":[{"role":"user","content":"hello world, tell me a joke"}]}"#.to_string();
    run_golden_outbound("benign_openai_chat", "/v1/chat/completions", body).await;
}

/// A3 e2e 变体：用 `[[upstream]]` + `[[upstream.routes]]` 配置化路由表启动 daemon，
/// 让非标准 `route_path` 零代码路由到 `route_provider`。规则缺失返回 None（优雅 SKIP）。
fn spawn_sieve_daemon_routed(
    upstream_url: &str,
    route_path: &str,
    route_provider: &str,
) -> Option<(u16, DaemonGuard)> {
    let port = find_free_port();
    let rules = outbound_rules_path();
    let inbound_rules = inbound_rules_path();
    if !rules.exists() || !inbound_rules.exists() {
        eprintln!("SKIP: 规则文件不存在（需签名规则包），跳过 A3 路由表 e2e");
        return None;
    }

    let mut config_file = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        config_file,
        r#"bind_addr = "127.0.0.1"
rules_path = "{rules}"
inbound_rules_path = "{inbound}"
tls_verify_upstream = false
dry_run = false

[[upstream]]
port = {port}
url = "{url}"
protocol = "auto"
provider_id = "custom-relay"

[[upstream.routes]]
path = "{route_path}"
provider = "{route_provider}"
"#,
        rules = rules.display(),
        inbound = inbound_rules.display(),
        port = port,
        url = upstream_url,
        route_path = route_path,
        route_provider = route_provider,
    )
    .unwrap();
    config_file.flush().unwrap();

    let binary = sieve_binary();
    assert!(binary.exists(), "sieve binary 不存在，先 cargo build");
    let home = tempfile::tempdir().unwrap();
    let mut cmd = Command::new(&binary);
    cmd.arg("start")
        .arg("--config")
        .arg(config_file.path())
        .env("SIEVE_LOG", "warn")
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1")
        .env("SIEVE_HOME", home.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let proc = cmd.spawn().expect("spawn sieve daemon");

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
        assert!(Instant::now() < deadline, "daemon 未在 10s 内监听 :{port}");
        std::thread::sleep(Duration::from_millis(100));
    }

    Some((
        port,
        DaemonGuard {
            proc,
            _config_file: config_file,
            _sieve_home: home,
        },
    ))
}

/// A3 配置化路由表 e2e（DoD②「加一行路由」最小可跑示例）：接「OpenAI 兼容但路径不同
/// 的中转站」只需在 `[[upstream.routes]]` 加一行 `{ path, provider = "openai" }`，
/// **零代码改动**——非标准路径 `/custom/llm/v1/chat` 经路由表落到 OpenAiCodec，
/// 出站 OUT-01 脱敏端到端生效（上游只见 `[REDACTED]`，不见原始 key）。
#[tokio::test]
async fn custom_route_relay_openai_via_config() {
    if !outbound_rules_path().exists() {
        eprintln!("SKIP: 规则文件不存在");
        return;
    }
    let (upstream_addr, received, _up_tx) = spawn_capturing_upstream().await;
    let upstream_url = format!("http://{upstream_addr}");
    let route_path = "/custom/llm/v1/chat";

    let Some((port, _guard)) = spawn_sieve_daemon_routed(&upstream_url, route_path, "openai")
    else {
        return; // 规则缺失 → 优雅 SKIP
    };

    // OpenAI chat 请求体，content 含 OUT-01 key（应被脱敏）。
    let body = serde_json::json!({
        "model": "gpt-4",
        "messages": [{ "role": "user", "content": format!("leaked: {}", out01_key()) }],
    })
    .to_string();

    let client = plain_http_client();
    let resp = client
        .request(
            Request::builder()
                .method("POST")
                .uri(format!("http://127.0.0.1:{port}{route_path}"))
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(body)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "异路径中转站请求应被路由 + 转发，上游 200"
    );

    let received_body = received.lock().await.clone();
    let received_str = String::from_utf8_lossy(&received_body);
    assert!(!received_str.is_empty(), "上游应收到（脱敏后）body");
    // 核心断言：routes 表把非标准路径路由到 OpenAiCodec → 出站脱敏生效 → 上游不见原始 key。
    assert!(
        !received_str.contains("sk-ant-api03-"),
        "异路径中转站出站脱敏应生效，上游不应见原始 key：{received_str}"
    );
    assert!(
        received_str.contains("REDACTED"),
        "上游 body 应含 [REDACTED] 占位符（证明走了 OpenAiCodec 出站扫描）：{received_str}"
    );
}
