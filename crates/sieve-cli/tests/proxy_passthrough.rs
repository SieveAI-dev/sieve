//! Sieve daemon 透传集成测试。
//!
//! 设计：启动 mock 上游（plain HTTP） + 最小 Sieve proxy（复用 sieve_core::Forwarder），
//! 验证：
//! 1. 请求 body / headers 透传（字节级一致）
//! 2. 响应 body 字节级一致（SSE chunk 边界保留）
//! 3. 错误码透传（401 / 429 / 500）
//! 4. tool_use SSE 多 event 不被切断
//!
//! 为何不直接调 daemon::run：
//! - daemon::run 内部自己 bind，无法从外部拿到 OS 分配的 :0 端口。
//! - 用 spawn_minimal_sieve_proxy 复刻同样的 Forwarder 逻辑，
//!   sieve_core::Forwarder 是共用实现，功能等价。
//! - .cursorrules §3.2：测试代码允许使用 .unwrap()。

use bytes::Bytes;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1 as server_http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

// ─── 内部类型别名 ─────────────────────────────────────────────────────────────

/// 与 daemon.rs 相同的 ResponseBody 类型（统一 BoxBody 错误为 Box<dyn Error>）。
type ResponseBody =
    http_body_util::combinators::BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>;

// ─── helpers ──────────────────────────────────────────────────────────────────

/// 在 :0 端口启动 plain-HTTP mock 上游，返回 (实际地址, shutdown sender)。
///
/// `responder` 接收完整收集的请求（body 已 collect），返回完整 Response。
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
                                // 在 service_fn 内收集 body，避免跨 async boundary 携带 Incoming
                                let (parts, body) = req.into_parts();
                                let bytes = body
                                    .collect()
                                    .await
                                    .unwrap_or_default()
                                    .to_bytes();
                                let req_collected = Request::from_parts(parts, bytes);
                                let resp = r(req_collected).await;
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

/// 启动最小 Sieve proxy（与 daemon::proxy_inner 逻辑等价）。
///
/// 直接复用 `sieve_core::Forwarder`，不调用 daemon::run，
/// 从而能拿到 OS 分配的 :0 端口。
async fn spawn_minimal_sieve_proxy(upstream_url: String) -> (SocketAddr, oneshot::Sender<()>) {
    use sieve_core::forwarder::Forwarder;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let forwarder = Arc::new(Forwarder::new(&upstream_url).unwrap());
    let (tx, mut rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => break,
                accept = listener.accept() => {
                    let Ok((stream, _)) = accept else { continue };
                    let io = TokioIo::new(stream);
                    let f = forwarder.clone();
                    tokio::spawn(async move {
                        let svc = service_fn(move |req: Request<Incoming>| {
                            let f = f.clone();
                            async move {
                                let resp = proxy_inner(f, req).await.unwrap_or_else(|e| {
                                    // 与 daemon::proxy() 相同的 502 fallback
                                    let body = format!("sieve proxy error: {e}");
                                    let b: ResponseBody = Full::new(Bytes::from(body))
                                        .map_err(
                                            |err| -> Box<dyn std::error::Error + Send + Sync> {
                                                match err {}
                                            },
                                        )
                                        .boxed();
                                    Response::builder()
                                        .status(StatusCode::BAD_GATEWAY)
                                        .body(b)
                                        .unwrap()
                                });
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

/// 核心透传逻辑（镜像 daemon::proxy_inner）。
///
/// 复刻原因：daemon::proxy_inner 是 crate-private；
/// 此处用相同的 Forwarder 公开 API 实现，语义完全一致。
async fn proxy_inner(
    forwarder: Arc<sieve_core::forwarder::Forwarder>,
    req: Request<Incoming>,
) -> anyhow::Result<Response<ResponseBody>> {
    use anyhow::anyhow;

    let (mut parts, body) = req.into_parts();

    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;

    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    // Incoming::Error = hyper::Error，直接 map_err 恒等再 boxed 即可
    let upstream_body = body.map_err(|e| -> hyper::Error { e }).boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (resp_parts, resp_body) = upstream_resp.into_parts();
    let boxed_body: ResponseBody = resp_body
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();

    Ok(Response::from_parts(resp_parts, boxed_body))
}

// ─── 测试工具函数 ─────────────────────────────────────────────────────────────

/// 构建仅支持 plain HTTP 的测试客户端（向 sieve proxy 发请求）。
fn plain_http_client() -> Client<HttpConnector, Empty<Bytes>> {
    Client::builder(TokioExecutor::new()).build(HttpConnector::new())
}

fn plain_http_client_with_body() -> Client<HttpConnector, Full<Bytes>> {
    Client::builder(TokioExecutor::new()).build(HttpConnector::new())
}

// ─── 测试 ──────────────────────────────────────────────────────────────────────

/// 验证简单 GET 请求能透传并返回正确 body。
#[tokio::test]
async fn passthrough_simple_get() {
    let (upstream_addr, _up_shutdown) = spawn_mock_upstream(|req| async move {
        assert_eq!(req.method(), &http::Method::GET);
        assert_eq!(req.uri().path(), "/v1/messages");

        Response::builder()
            .status(StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from_static(b"{\"ok\":true}")))
            .unwrap()
    })
    .await;

    let upstream_url = format!("http://{}", upstream_addr);
    let (sieve_addr, _sv_shutdown) = spawn_minimal_sieve_proxy(upstream_url).await;

    let client = plain_http_client();
    let req = Request::builder()
        .method(http::Method::GET)
        .uri(format!("http://{}/v1/messages", sieve_addr))
        .header(http::header::HOST, sieve_addr.to_string())
        .body(Empty::<Bytes>::new())
        .unwrap();

    let resp = client.request(req).await.expect("client request failed");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"{\"ok\":true}");
}

/// 验证上游 4xx/5xx 状态码透传（401 / 429 / 500）。
#[tokio::test]
async fn passthrough_preserves_status_code() {
    for code in [401u16, 429, 500] {
        let (upstream_addr, _up) = spawn_mock_upstream(move |_req| async move {
            Response::builder()
                .status(code)
                .body(Full::new(Bytes::from_static(b"")))
                .unwrap()
        })
        .await;

        let (sieve_addr, _sv) =
            spawn_minimal_sieve_proxy(format!("http://{}", upstream_addr)).await;

        let client = plain_http_client();
        let req = Request::builder()
            .uri(format!("http://{}/v1/messages", sieve_addr))
            .header(http::header::HOST, sieve_addr.to_string())
            .body(Empty::<Bytes>::new())
            .unwrap();

        let resp = client.request(req).await.unwrap();
        assert_eq!(
            resp.status().as_u16(),
            code,
            "status {} not transparently forwarded",
            code
        );
    }
}

/// 验证自定义请求 headers（x-api-key / anthropic-version / anthropic-beta）
/// 透传到上游，且 Host 被重写为上游 authority。
#[tokio::test]
async fn passthrough_preserves_request_headers() {
    let captured: Arc<tokio::sync::Mutex<Option<http::HeaderMap>>> =
        Arc::new(tokio::sync::Mutex::new(None));
    let captured_clone = captured.clone();

    let (upstream_addr, _up) = spawn_mock_upstream(move |req| {
        let cap = captured_clone.clone();
        async move {
            *cap.lock().await = Some(req.headers().clone());
            Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from_static(b"OK")))
                .unwrap()
        }
    })
    .await;

    let (sieve_addr, _sv) = spawn_minimal_sieve_proxy(format!("http://{}", upstream_addr)).await;

    let client = plain_http_client();
    let req = Request::builder()
        .uri(format!("http://{}/v1/messages", sieve_addr))
        .header("x-api-key", "sk-ant-test-1234")
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "tools-2024-04-04")
        .header(http::header::HOST, sieve_addr.to_string())
        .body(Empty::<Bytes>::new())
        .unwrap();

    let _ = client.request(req).await.unwrap();

    let guard = captured.lock().await;
    let h = guard
        .as_ref()
        .expect("upstream did not capture request headers");

    assert_eq!(
        h.get("x-api-key").and_then(|v| v.to_str().ok()),
        Some("sk-ant-test-1234"),
        "x-api-key header missing or wrong"
    );
    assert_eq!(
        h.get("anthropic-version").and_then(|v| v.to_str().ok()),
        Some("2023-06-01"),
        "anthropic-version header missing or wrong"
    );
    assert_eq!(
        h.get("anthropic-beta").and_then(|v| v.to_str().ok()),
        Some("tools-2024-04-04"),
        "anthropic-beta header missing or wrong"
    );

    // Forwarder::rewrite_uri + Host 替换：上游收到的 Host 应是上游 authority
    let host = h
        .get(http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        host.starts_with("127.0.0.1:"),
        "expected upstream host authority, got {host:?}"
    );
}

/// 验证 SSE 流式 body 字节级一致：多 event 粘包 + Content-Type 头保留。
///
/// 对应 PRD §9 第 5 条 fuzz 约束的手工验证版本（完整 fuzz 见 fuzz/ 目录）。
#[tokio::test]
async fn passthrough_preserves_sse_chunk_boundaries() {
    // 模拟 Anthropic SSE 典型响应：5 个 event，多行粘在一个 HTTP chunk 里
    static SSE_BODY: &[u8] = b"\
event: message_start\n\
data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_01\",\"role\":\"assistant\"}}\n\
\n\
event: content_block_start\n\
data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\
\n\
event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hi\"}}\n\
\n\
event: content_block_stop\n\
data: {\"type\":\"content_block_stop\",\"index\":0}\n\
\n\
event: message_stop\n\
data: {\"type\":\"message_stop\"}\n\
\n";

    let expected = Bytes::from_static(SSE_BODY);
    let expected_clone = expected.clone();

    let (upstream_addr, _up) = spawn_mock_upstream(move |_req| {
        let body = expected_clone.clone();
        async move {
            Response::builder()
                .status(StatusCode::OK)
                .header(http::header::CONTENT_TYPE, "text/event-stream")
                .body(Full::new(body))
                .unwrap()
        }
    })
    .await;

    let (sieve_addr, _sv) = spawn_minimal_sieve_proxy(format!("http://{}", upstream_addr)).await;

    let client = plain_http_client();
    let req = Request::builder()
        .uri(format!("http://{}/v1/messages", sieve_addr))
        .header(http::header::HOST, sieve_addr.to_string())
        .body(Empty::<Bytes>::new())
        .unwrap();

    let resp = client.request(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Content-Type: text/event-stream 必须保留
    assert_eq!(
        resp.headers()
            .get(http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("text/event-stream"),
        "Content-Type header not forwarded"
    );

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(body, expected, "SSE body byte-level mismatch");
}

/// 验证 POST 请求 body 字节级一致透传到上游。
#[tokio::test]
async fn passthrough_preserves_request_body() {
    static BODY_BYTES: &[u8] = b"{\"model\":\"claude-sonnet-4-6\",\"max_tokens\":1024,\
\"messages\":[{\"role\":\"user\",\"content\":\"hi\"}]}";

    let captured: Arc<tokio::sync::Mutex<Option<Bytes>>> = Arc::new(tokio::sync::Mutex::new(None));
    let cap_clone = captured.clone();

    let (upstream_addr, _up) = spawn_mock_upstream(move |req| {
        let cap = cap_clone.clone();
        async move {
            // req.into_parts().1 已在 spawn_mock_upstream 内 collect 好了
            let (_, body_bytes) = req.into_parts();
            *cap.lock().await = Some(body_bytes);
            Response::builder()
                .status(StatusCode::OK)
                .body(Full::new(Bytes::from_static(b"OK")))
                .unwrap()
        }
    })
    .await;

    let (sieve_addr, _sv) = spawn_minimal_sieve_proxy(format!("http://{}", upstream_addr)).await;

    let client = plain_http_client_with_body();
    let req = Request::builder()
        .method(http::Method::POST)
        .uri(format!("http://{}/v1/messages", sieve_addr))
        .header(http::header::HOST, sieve_addr.to_string())
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::from_static(BODY_BYTES)))
        .unwrap();

    let resp = client.request(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let guard = captured.lock().await;
    let received = guard
        .as_ref()
        .expect("upstream did not capture request body");
    assert_eq!(
        received.as_ref(),
        BODY_BYTES,
        "request body byte-level mismatch"
    );
}
