//! multi-agent 路由集成测试（v1.5，ADR-018 + ADR-019）。
//!
//! 验证：
//! 1. Anthropic 路径（/v1/messages）正常路由
//! 2. OpenAI 路径（/v1/chat/completions）正常路由，规则引擎能扫到 secret
//! 3. X-Sieve-Origin claude:0 → DecisionRequest source_agent=Claude, chain_depth=0
//! 4. X-Sieve-Origin hermes-delegate-claude:1 → source_agent + origin_chain.len()=1
//! 5. chain_depth=2 → HookTerminal 类规则升级为 GUI hold
//! 6. chain_depth=5 → 直接 426 拒绝
//! 7. 缺 header → source_agent=Unknown，chain_depth=0
//! 8. 格式错误 header → source_agent=Unknown + audit 警告
//! 9. X-Sieve-Source-Channel=whatsapp → DecisionRequest.source_channel="whatsapp"
//!
//! 注：测试 3/4/5/9 需要 IPC 路径验证 DecisionRequest 字段，
//!     当前通过观察 daemon 行为（426 / 透传 / sieve_blocked 注入）来间接验证。
//!
//! 关联：PRD v1.5 §6.1 §4.5 §4.6 / ADR-018（OpenAI 协议）/ ADR-019（multi-agent header）。

use bytes::Bytes;
use http_body_util::{BodyExt, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::server::conn::http1 as server_http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::io::Write as _;
use std::net::{SocketAddr, TcpListener as StdListener};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

// ─── helpers（从 inbound_block.rs 提取共用部分）─────────────────────────────────

fn find_free_port() -> u16 {
    let l = StdListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
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

type MockBody = StreamBody<tokio_stream::Once<Result<Frame<Bytes>, Infallible>>>;

fn bytes_to_chunked_body(data: Bytes) -> MockBody {
    let stream = tokio_stream::once(Ok::<_, Infallible>(Frame::data(data)));
    StreamBody::new(stream)
}

async fn spawn_mock_upstream<F, Fut>(responder: F) -> (SocketAddr, oneshot::Sender<()>)
where
    F: Fn(Request<Bytes>) -> Fut + Clone + Send + Sync + 'static,
    Fut: std::future::Future<Output = (hyper::StatusCode, Bytes)> + Send,
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
                                let bytes = body.collect().await.unwrap_or_default().to_bytes();
                                let req_collected = Request::from_parts(parts, bytes);
                                let (status, body_bytes) = r(req_collected).await;
                                let resp: Response<MockBody> = Response::builder()
                                    .status(status)
                                    .header(http::header::CONTENT_TYPE, "application/json")
                                    .body(bytes_to_chunked_body(body_bytes))
                                    .unwrap();
                                Ok::<_, Infallible>(resp)
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

struct DaemonGuard {
    proc: Child,
    _config_file: tempfile::NamedTempFile,
    /// 测试隔离：sieve_home tempdir，防止污染真实 ~/.sieve。
    _sieve_home: tempfile::TempDir,
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

fn spawn_sieve_daemon(upstream_url: &str) -> Option<(u16, DaemonGuard)> {
    let port = find_free_port();
    let rules = outbound_rules_path();
    if !rules.exists() {
        eprintln!(
            "SKIP: 规则文件不存在（需安装签名规则包），跳过 ({})",
            rules.display()
        );
        return None;
    }
    let inbound_rules = inbound_rules_path();
    if !inbound_rules.exists() {
        eprintln!(
            "SKIP: 规则文件不存在（需安装签名规则包），跳过 ({})",
            inbound_rules.display()
        );
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

    let binary = sieve_binary();
    assert!(
        binary.exists(),
        "sieve binary not found at {}; run `cargo build --release` first",
        binary.display()
    );

    let sieve_home = tempfile::tempdir().unwrap();

    let proc = Command::new(&binary)
        .arg("start")
        .arg("--config")
        .arg(config_file.path())
        .env("SIEVE_LOG", "warn")
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1")
        .env("SIEVE_HOME", sieve_home.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sieve daemon");

    wait_for_http_ready(port, Duration::from_secs(10));

    Some((
        port,
        DaemonGuard {
            proc,
            _config_file: config_file,
            _sieve_home: sieve_home,
        },
    ))
}

/// 等 daemon TCP listener 就绪。HTTP-level probe 在 #[tokio::test] 上会死锁
/// （详见 outbound_block.rs::wait_for_http_ready 注释）。
fn wait_for_http_ready(port: u16, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    loop {
        if std::net::TcpStream::connect_timeout(
            &format!("127.0.0.1:{port}").parse().unwrap(),
            Duration::from_millis(500),
        )
        .is_ok()
        {
            return;
        }
        if Instant::now() >= deadline {
            panic!("sieve daemon did not listen on :{port} within {timeout:?}");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// 发送原始 HTTP 请求，支持自定义 path、body 和 headers。
fn send_raw_request(
    port: u16,
    method: &str,
    path: &str,
    body_json: &str,
    extra_headers: &[(&str, &str)],
) -> (hyper::StatusCode, Bytes) {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let mut header_lines = String::new();
    for (name, value) in extra_headers {
        header_lines.push_str(&format!("{name}: {value}\r\n"));
    }

    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n{extra}\r\n{body}",
        method = method,
        path = path,
        port = port,
        len = body_json.len(),
        extra = header_lines,
        body = body_json,
    );

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).ok();

    let raw_str = String::from_utf8_lossy(&raw);
    let status_code = raw_str
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(0);
    let status = hyper::StatusCode::from_u16(status_code).unwrap_or(hyper::StatusCode::OK);

    let sep = b"\r\n\r\n";
    let raw_body = if let Some(pos) = raw.windows(sep.len()).position(|w| w == sep) {
        raw[pos + sep.len()..].to_vec()
    } else {
        raw.clone()
    };

    // 简单 chunked decode
    let decoded = decode_chunked(&raw_body);
    (status, Bytes::from(decoded))
}

fn decode_chunked(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut pos = 0;
    while pos < input.len() {
        let Some(crlf_pos) = find_crlf(input, pos) else {
            result.extend_from_slice(input);
            return result;
        };
        let size_str = std::str::from_utf8(&input[pos..crlf_pos]).unwrap_or("0");
        let chunk_size = usize::from_str_radix(size_str.trim(), 16).unwrap_or(0);
        pos = crlf_pos + 2;
        if chunk_size == 0 {
            break;
        }
        if pos + chunk_size > input.len() {
            result.extend_from_slice(&input[pos..]);
            break;
        }
        result.extend_from_slice(&input[pos..pos + chunk_size]);
        pos += chunk_size + 2;
    }
    if result.is_empty() {
        result.extend_from_slice(input);
    }
    result
}

fn find_crlf(data: &[u8], start: usize) -> Option<usize> {
    (start..data.len().saturating_sub(1)).find(|&i| data[i] == b'\r' && data[i + 1] == b'\n')
}

async fn send_raw_async(
    port: u16,
    method: &str,
    path: &str,
    body_json: &str,
    extra_headers: Vec<(String, String)>,
) -> (hyper::StatusCode, Bytes) {
    let method = method.to_string();
    let path = path.to_string();
    let body_json = body_json.to_string();
    tokio::task::spawn_blocking(move || {
        let refs: Vec<(&str, &str)> = extra_headers
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        send_raw_request(port, &method, &path, &body_json, &refs)
    })
    .await
    .unwrap()
}

// ─── 公共 mock 上游响应：benign JSON ──────────────────────────────────────────

fn benign_anthropic_sse() -> Bytes {
    Bytes::from(
        "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"m\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"claude\",\"usage\":{\"input_tokens\":1,\"output_tokens\":1}}}\n\n\
         event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n\
         event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"ok\"}}\n\n\
         event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n"
    )
}

fn benign_openai_json() -> Bytes {
    Bytes::from(
        r#"{"id":"chat-1","object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":"ok"},"finish_reason":"stop"}]}"#,
    )
}

// ─── 测试 1：Anthropic 路径（/v1/messages）────────────────────────────────────

/// POST /v1/messages → 走 Anthropic 解析路径，benign 内容透传，返回 200。
///
/// 验证：v1.4 Anthropic 路径在 v1.5 路径分发后仍正常工作（回归）。
/// 关联：ADR-018 §路径分发、PRD v1.5 §6.1。
#[tokio::test]
async fn test_1_anthropic_path_routes_correctly() {
    let sse = benign_anthropic_sse();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = sse.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, body) = send_raw_async(port, "POST", "/v1/messages", body_json, vec![]).await;

    assert_eq!(status, hyper::StatusCode::OK, "Anthropic 路径应返回 200");
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("sieve_blocked"),
        "benign Anthropic 请求不应触发 sieve_blocked:\n{body_str}"
    );
}

// ─── 测试 2：OpenAI 路径（/v1/chat/completions）──────────────────────────────

/// POST /v1/chat/completions + benign OpenAI body → 透传，返回 200。
///
/// 验证：OpenAI 路径路由正确，benign 内容不触发拦截。
/// 关联：ADR-018 §路由、PRD v1.5 §6.1。
#[tokio::test]
async fn test_2_openai_path_routes_correctly() {
    let oai_resp = benign_openai_json();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = oai_resp.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"gpt-4o","messages":[{"role":"user","content":"hello"}]}"#;
    let (status, _body) =
        send_raw_async(port, "POST", "/v1/chat/completions", body_json, vec![]).await;

    assert_eq!(status, hyper::StatusCode::OK, "OpenAI 路径应返回 200");
}

/// POST /v1/chat/completions + 含 secret 的 OpenAI body → 规则引擎应触发出站拦截（426）。
///
/// 验证：OpenAI 路径的出站扫描与 Anthropic 路径对称，规则引擎能扫到 secret。
/// 关联：ADR-018 §检测兼容性、PRD v1.5 §6.1。
#[tokio::test]
async fn test_2b_openai_path_outbound_secret_blocked() {
    let oai_resp = benign_openai_json();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = oai_resp.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    // 含 PEM 私钥头，触发 OUT-07（disposition=block，无 auto_redact）
    let body_json = r#"{"model":"gpt-4o","messages":[{"role":"user","content":"my key: -----BEGIN RSA PRIVATE KEY----- abcdef"}]}"#;
    let (status, body) =
        send_raw_async(port, "POST", "/v1/chat/completions", body_json, vec![]).await;

    assert_eq!(
        status,
        hyper::StatusCode::UPGRADE_REQUIRED,
        "OpenAI 路径含 secret 应触发 426:\n{}",
        String::from_utf8_lossy(&body)
    );
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("sieve_blocked"),
        "426 响应应含 sieve_blocked:\n{body_str}"
    );
}

// ─── 测试 3：X-Sieve-Origin claude:0 ─────────────────────────────────────────

/// X-Sieve-Origin: claude:<uuid>:0 → chain_depth=0，benign 请求正常透传。
///
/// chain_depth=0 = 用户直接调用，不触发升级。
/// 验证：source_agent=Claude + chain_depth=0 不影响正常流量。
/// 关联：ADR-019 §header 格式、PRD v1.5 §6.5。
#[tokio::test]
async fn test_3_origin_header_claude_depth_0_passthrough() {
    let sse = benign_anthropic_sse();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = sse.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, body) = send_raw_async(
        port,
        "POST",
        "/v1/messages",
        body_json,
        vec![(
            "X-Sieve-Origin".to_string(),
            "claude:01901234-5678-7abc-def0-123456789abc:0".to_string(),
        )],
    )
    .await;

    assert_eq!(
        status,
        hyper::StatusCode::OK,
        "chain_depth=0 benign 请求应透传:\n{}",
        String::from_utf8_lossy(&body)
    );
}

// ─── 测试 4：X-Sieve-Origin hermes-delegate-claude:<uuid>:1 ──────────────────

/// X-Sieve-Origin: hermes-delegate-claude:<uuid>:1 → source_agent=Hermes, chain_depth=1。
///
/// chain_depth=1 < 2，不触发强制 GuiPopup，benign 请求正常透传。
/// 验证：Hermes 来源解析正确，chain_depth=1 不升级 disposition。
/// 关联：ADR-019 §agent 识别、PRD v1.5 §4.6。
#[tokio::test]
async fn test_4_origin_header_hermes_depth_1_passthrough() {
    let sse = benign_anthropic_sse();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = sse.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, body) = send_raw_async(
        port,
        "POST",
        "/v1/messages",
        body_json,
        vec![(
            "X-Sieve-Origin".to_string(),
            "hermes-delegate-claude:01901234-5678-7abc-def0-111111111111:1".to_string(),
        )],
    )
    .await;

    assert_eq!(
        status,
        hyper::StatusCode::OK,
        "chain_depth=1 benign 请求应透传:\n{}",
        String::from_utf8_lossy(&body)
    );
}

// ─── 测试 5：chain_depth=2 → HookTerminal 升级为 GUI hold ────────────────────

/// X-Sieve-Origin: claude:<uuid>:2 → chain_depth=2，HookMark（hook_terminal）升级为 GuiPopup。
///
/// 正常流量（benign）在 chain_depth=2 时：无命中 → 正常透传。
/// 注：IN-CR-02 类规则在有命中时会升级为 HoldForDecision，无 GUI 时 fail-closed。
/// 本测试验证 chain_depth=2 不影响 benign 流量（无误报），
/// 且 chain_depth ≥ 2 的请求不会直接被 426 拒绝。
///
/// 关联：ADR-019 §chain_depth 升级策略、PRD v1.5 §6.5。
#[tokio::test]
async fn test_5_chain_depth_2_benign_still_passes() {
    let sse = benign_anthropic_sse();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = sse.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, body) = send_raw_async(
        port,
        "POST",
        "/v1/messages",
        body_json,
        vec![(
            "X-Sieve-Origin".to_string(),
            "claude:01901234-5678-7abc-def0-123456789abc:2".to_string(),
        )],
    )
    .await;

    // chain_depth=2 benign → 透传（无命中，不触发 GuiPopup）
    assert_ne!(
        status,
        hyper::StatusCode::UPGRADE_REQUIRED,
        "chain_depth=2 benign 请求不应触发 426，status={status}"
    );
    let body_str = String::from_utf8_lossy(&body);
    // benign 流量应透传（不含 sieve_blocked）
    // 注：如果 IPC 未初始化且有命中，fail-closed 会注入 sieve_blocked，但本测试无命中
    assert!(
        !body_str.contains("nested_call_too_deep"),
        "chain_depth=2 不应触发 nested_call_too_deep:\n{body_str}"
    );
}

// ─── 测试 6：chain_depth=5 → 直接 426 ────────────────────────────────────────

/// X-Sieve-Origin: claude:<uuid>:5 → chain_depth ≥ 5，直接返回 426。
///
/// ADR-019 §嵌套深度限制：超过 5 层视为攻击模式，跳过所有检测直接拒绝。
/// 关联：ADR-019 §嵌套深度限制、PRD v1.5 §6.5。
#[tokio::test]
async fn test_6_chain_depth_5_rejected_immediately() {
    // 上游不应被调用（直接 426 返回），但仍需有效地址
    let (upstream, _up) =
        spawn_mock_upstream(move |_req| async move { (hyper::StatusCode::OK, Bytes::from("{}")) })
            .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, body) = send_raw_async(
        port,
        "POST",
        "/v1/messages",
        body_json,
        vec![(
            "X-Sieve-Origin".to_string(),
            "claude:01901234-5678-7abc-def0-123456789abc:5".to_string(),
        )],
    )
    .await;

    assert_eq!(
        status,
        hyper::StatusCode::UPGRADE_REQUIRED,
        "chain_depth=5 应触发 426"
    );
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("nested_call_too_deep"),
        "426 响应应含 nested_call_too_deep:\n{body_str}"
    );
    assert!(
        body_str.contains("\"chain_depth\":5"),
        "426 响应应含 chain_depth:\n{body_str}"
    );
}

/// chain_depth=6 也应直接 426（≥ 5 均拒绝）。
#[tokio::test]
async fn test_6b_chain_depth_6_also_rejected() {
    let (upstream, _up) =
        spawn_mock_upstream(move |_req| async move { (hyper::StatusCode::OK, Bytes::from("{}")) })
            .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, _body) = send_raw_async(
        port,
        "POST",
        "/v1/messages",
        body_json,
        vec![(
            "X-Sieve-Origin".to_string(),
            "hermes:01901234-5678-7abc-def0-123456789abc:6".to_string(),
        )],
    )
    .await;

    assert_eq!(
        status,
        hyper::StatusCode::UPGRADE_REQUIRED,
        "chain_depth=6 也应触发 426"
    );
}

// ─── 测试 7：缺 X-Sieve-Origin header ────────────────────────────────────────

/// 缺 X-Sieve-Origin header → source_agent=Unknown, chain_depth=0，正常透传。
///
/// 关联：ADR-019 §缺 header 处理、PRD v1.5 §6.5。
#[tokio::test]
async fn test_7_missing_origin_header_passes_as_unknown() {
    let sse = benign_anthropic_sse();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = sse.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    // 不带 X-Sieve-Origin
    let (status, body) = send_raw_async(port, "POST", "/v1/messages", body_json, vec![]).await;

    assert_eq!(
        status,
        hyper::StatusCode::OK,
        "缺 header 应正常透传:\n{}",
        String::from_utf8_lossy(&body)
    );
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("nested_call_too_deep"),
        "缺 header 不应触发 nested_call_too_deep:\n{body_str}"
    );
}

// ─── 测试 8：格式错误 X-Sieve-Origin header ──────────────────────────────────

/// X-Sieve-Origin 格式错误 → fail-open：视为无 header（source_agent=Unknown），正常透传。
///
/// 格式错误不应阻断请求，但 daemon 应记录 audit 警告。
/// 关联：ADR-019 §解析失败处理、PRD v1.5 §6.5。
#[tokio::test]
async fn test_8_malformed_origin_header_fail_open() {
    let sse = benign_anthropic_sse();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = sse.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    // 格式错误：只有 2 段（缺 chain_depth）
    let (status, body) = send_raw_async(
        port,
        "POST",
        "/v1/messages",
        body_json,
        vec![(
            "X-Sieve-Origin".to_string(),
            "invalid-format-no-colon".to_string(),
        )],
    )
    .await;

    assert_eq!(
        status,
        hyper::StatusCode::OK,
        "格式错误 header 应 fail-open（透传）:\n{}",
        String::from_utf8_lossy(&body)
    );
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("nested_call_too_deep"),
        "格式错误 header 不应触发 nested_call_too_deep:\n{body_str}"
    );
}

/// 另一种格式错误：chain_depth 不是数字。
#[tokio::test]
async fn test_8b_invalid_chain_depth_fail_open() {
    let sse = benign_anthropic_sse();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = sse.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, _body) = send_raw_async(
        port,
        "POST",
        "/v1/messages",
        body_json,
        vec![(
            "X-Sieve-Origin".to_string(),
            "claude:01901234-5678-7abc-def0-123456789abc:notanumber".to_string(),
        )],
    )
    .await;

    assert_eq!(
        status,
        hyper::StatusCode::OK,
        "chain_depth 非数字应 fail-open"
    );
}

// ─── 测试 9：X-Sieve-Source-Channel=whatsapp ─────────────────────────────────

/// X-Sieve-Source-Channel: whatsapp → DecisionRequest.source_channel="whatsapp"。
///
/// 当前通过观察 benign 流量正常透传来验证 header 解析不会崩溃；
/// 详细字段验证需要 IPC 侧 hook（当前无 GUI 连接）。
/// 关联：PRD v1.5 §4.5 场景 E、IN-GEN-06。
#[tokio::test]
async fn test_9_source_channel_header_parsed_without_error() {
    let sse = benign_anthropic_sse();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = sse.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, body) = send_raw_async(
        port,
        "POST",
        "/v1/messages",
        body_json,
        vec![
            (
                "X-Sieve-Origin".to_string(),
                "open_claw:01901234-5678-7abc-def0-123456789abc:0".to_string(),
            ),
            ("X-Sieve-Source-Channel".to_string(), "whatsapp".to_string()),
        ],
    )
    .await;

    assert_eq!(
        status,
        hyper::StatusCode::OK,
        "X-Sieve-Source-Channel=whatsapp 应正常透传（不影响 benign 流量）:\n{}",
        String::from_utf8_lossy(&body)
    );
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("nested_call_too_deep"),
        "Source-Channel header 不应触发 nested_call_too_deep:\n{body_str}"
    );
}

// ─── 单元测试：parse_sieve_origin_header ─────────────────────────────────────
// 注：parse_sieve_origin_header 是 daemon 模块私有函数，通过集成测试间接验证。
// 下面添加一个简单的解析逻辑验证测试（不依赖 daemon 内部实现）。

/// chain_depth=4 时（< 5），请求应正常透传（不触发 426）。
///
/// 验证 chain_depth 边界：4 不拒绝，5 拒绝。
/// 关联：ADR-019 §嵌套深度限制边界。
#[tokio::test]
async fn test_chain_depth_4_not_rejected() {
    let sse = benign_anthropic_sse();
    let (upstream, _up) = spawn_mock_upstream(move |_req| {
        let body = sse.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    let (status, body) = send_raw_async(
        port,
        "POST",
        "/v1/messages",
        body_json,
        vec![(
            "X-Sieve-Origin".to_string(),
            "claude:01901234-5678-7abc-def0-123456789abc:4".to_string(),
        )],
    )
    .await;

    assert_ne!(
        status,
        hyper::StatusCode::UPGRADE_REQUIRED,
        "chain_depth=4 应不触发 426:\n{}",
        String::from_utf8_lossy(&body)
    );
}

/// OpenAI 路径 + chain_depth=5 → 直接 426。
///
/// 验证 chain_depth ≥ 5 拒绝逻辑在 OpenAI 路径上也工作。
/// 关联：ADR-019 §嵌套深度限制、ADR-018 §路径分发。
#[tokio::test]
async fn test_openai_path_chain_depth_5_rejected() {
    let (upstream, _up) =
        spawn_mock_upstream(move |_req| async move { (hyper::StatusCode::OK, Bytes::from("{}")) })
            .await;

    let Some((port, _g)) = spawn_sieve_daemon(&format!("http://{upstream}")) else {
        return;
    };

    let body_json = r#"{"model":"gpt-4o","messages":[{"role":"user","content":"hi"}]}"#;
    let (status, body) = send_raw_async(
        port,
        "POST",
        "/v1/chat/completions",
        body_json,
        vec![(
            "X-Sieve-Origin".to_string(),
            "claude:01901234-5678-7abc-def0-123456789abc:5".to_string(),
        )],
    )
    .await;

    assert_eq!(
        status,
        hyper::StatusCode::UPGRADE_REQUIRED,
        "OpenAI 路径 chain_depth=5 应触发 426:\n{}",
        String::from_utf8_lossy(&body)
    );
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("nested_call_too_deep"),
        "426 应含 nested_call_too_deep:\n{body_str}"
    );
}
