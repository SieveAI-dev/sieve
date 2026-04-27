//! 透传 daemon（架构图节点 ①③⑤⑧）。
//!
//! Week 1：不做规则匹配，纯字节透传。SSE 流式 body 通过 hyper `Incoming` 自动
//! chunk-by-chunk 转发，不缓冲，不解析。
//!
//! hyper-util `auto::Builder` 选型理由：
//! - 自动协商客户端连接 h1/h2（ALPN），对 Claude Code 客户端透明；
//! - 上游侧 sieve-core Forwarder 同样 ALPN h1+h2，端到端协议不损失；
//! - 单一入口点，Week 2+ 在 `proxy_inner` 中插入 Pipeline 节点即可，
//!   不需要改 server 结构（关联 pipeline::Pipeline trait）。

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use sieve_core::Forwarder;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::config::Config;

/// 响应 body 的统一类型：错误为装箱 trait object，兼容 h1/h2 body 差异。
type ResponseBody = BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>;

/// 启动透传 daemon，永久阻塞直到进程收到信号。
///
/// # Errors
/// bind 端口失败或 Forwarder 初始化失败时返回错误。
pub async fn run(cfg: Config) -> Result<()> {
    let listen = cfg.listen_addr()?;
    let forwarder = Arc::new(
        Forwarder::new(&cfg.upstream_url)
            .map_err(|e| anyhow!("init forwarder: {e}"))?,
    );

    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("bind {}", listen))?;

    tracing::info!(
        listen = %listen,
        upstream = %cfg.upstream_url,
        "sieve daemon started"
    );

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "accept failed");
                continue;
            }
        };

        let forwarder = forwarder.clone();
        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let f = forwarder.clone();
                async move { proxy(f, req).await }
            });

            if let Err(e) = auto::Builder::new(TokioExecutor::new())
                .serve_connection(io, svc)
                .await
            {
                tracing::debug!(peer = %peer, error = %e, "connection closed with error");
            }
        });
    }
}

/// 请求入口：捕获 `proxy_inner` 的所有错误，转换为 502 Bad Gateway 响应。
///
/// hyper service_fn 要求返回 `Result<_, hyper::Error>`；业务错误不会 panic，
/// 只会被转为 502 响应，确保请求路径不 unwrap。
async fn proxy(
    forwarder: Arc<Forwarder>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>, hyper::Error> {
    match proxy_inner(forwarder, req).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            tracing::error!(error = %e, "proxy failed");
            let body = format!("sieve proxy error: {e}");
            // builder().body() 只在 header 值非法时失败（此处不会），
            // unwrap_or_else fallback 保证不 panic。
            let resp = Response::builder()
                .status(http::StatusCode::BAD_GATEWAY)
                .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(string_body(body))
                .unwrap_or_else(|_| Response::new(empty_body()));
            Ok(resp)
        }
    }
}

/// 核心透传逻辑：重写 URI → 替换 Host header → 转发到上游 → 流式返回响应。
///
/// Week 2+ 在此函数中插入 Pipeline 节点（InspectRequest / InspectResponse）。
async fn proxy_inner(
    forwarder: Arc<Forwarder>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>> {
    let (mut parts, body) = req.into_parts();

    // URI 重写：保留 path + query，替换 scheme + authority 为上游。
    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;

    // Host header 必须与上游 authority 一致，否则上游可能拒绝请求。
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    // 将客户端 body（Incoming）的错误类型对齐为 hyper::Error，
    // 再 boxed() 成 sieve-core Forwarder::forward 期望的 BoxBody 类型。
    let upstream_body = body
        .map_err(|e| -> hyper::Error { e })
        .boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    // 透传到上游；body 是流式 Incoming，不缓冲。
    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    // 将上游响应 body 的错误类型提升为 Box<dyn Error>，统一 ResponseBody 类型。
    let (resp_parts, resp_body) = upstream_resp.into_parts();
    let body: ResponseBody = resp_body
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();

    Ok(Response::from_parts(resp_parts, body))
}

/// 把字符串包成 `ResponseBody`（用于错误响应）。
fn string_body(s: String) -> ResponseBody {
    use http_body_util::Full;
    Full::new(Bytes::from(s))
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
        .boxed()
}

/// 空 body（fallback 错误响应）。
fn empty_body() -> ResponseBody {
    use http_body_util::Empty;
    Empty::<Bytes>::new()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
        .boxed()
}
