//! Mock 上游服务器（plain-HTTP，泛型 responder）。
//!
//! lift 自 `sieve-cli/tests/outbound_block.rs::spawn_mock_upstream`（`Full<Bytes>` body）和
//! `sieve-cli/tests/inbound_block.rs::spawn_mock_sse_upstream`（`StreamBody` chunked body）。
//!
//! 返回的 [`MockUpstream`] 持有 shutdown sender，Drop 时关闭 accept loop。

use bytes::Bytes;
use http_body_util::{BodyExt, Full, StreamBody};
use hyper::body::{Frame, Incoming};
use hyper::server::conn::http1 as server_http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

/// 运行中的 mock 上游句柄。Drop 时关闭服务器 accept loop。
///
/// 必须在测试期间保持存活（持有变量，别 `let _ = ...` 立即 drop）。
pub struct MockUpstream {
    /// 监听地址（`127.0.0.1:<port>`）。
    pub addr: SocketAddr,
    /// shutdown sender；Drop 时触发 accept loop 退出。字段名以 `_` 前缀标记「仅靠 Drop 起作用」。
    _shutdown: oneshot::Sender<()>,
}

impl MockUpstream {
    /// 上游 URL，形如 `http://127.0.0.1:<port>`（无尾斜杠）。
    #[must_use]
    pub fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    /// 监听端口。
    #[must_use]
    pub fn port(&self) -> u16 {
        self.addr.port()
    }
}

/// 在 `127.0.0.1:0` 启动 plain-HTTP mock 上游，responder 返回完整 [`Response<Full<Bytes>>`]。
///
/// responder 收到已 collect 完 body 的 `Request<Bytes>`，返回带 `Full<Bytes>` body 的响应
/// （hyper 自动补 `content-length`）。适合模拟非流式 JSON API。
///
/// lift 自 `outbound_block.rs::spawn_mock_upstream`。
///
/// # Panics
///
/// bind `127.0.0.1:0` 失败时 panic。
pub async fn spawn_mock_upstream<F, Fut>(responder: F) -> MockUpstream
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
                                let bytes = body.collect().await.unwrap_or_default().to_bytes();
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

    MockUpstream {
        addr,
        _shutdown: tx,
    }
}

/// chunked / SSE 流式 mock 上游 body 类型。
///
/// `StreamBody` 的 size_hint 未知 → hyper 用 chunked transfer（不加 `content-length`），
/// 模拟流式 SSE 响应；sieve 因此也不透传 `content-length`，注入 `sieve_blocked` 后不会因
/// HTTP 长度不一致而出错。
type MockStreamBody = StreamBody<tokio_stream::Once<Result<Frame<Bytes>, Infallible>>>;

/// 把 `Bytes` 包成 chunked `StreamBody`（无精确 size_hint）。
///
/// lift 自 `inbound_block.rs::bytes_to_chunked_body`。
fn bytes_to_chunked_body(data: Bytes) -> MockStreamBody {
    let stream = tokio_stream::once(Ok::<_, Infallible>(Frame::data(data)));
    StreamBody::new(stream)
}

/// 在 `127.0.0.1:0` 启动 chunked / SSE mock 上游，responder 返回 `(status, body_bytes)`。
///
/// body 用 [`StreamBody`] 包装（无 `content-length` → chunked transfer），`Content-Type`
/// 固定为传入的 `content_type`（如 `text/event-stream`）。适合模拟流式 SSE 响应。
///
/// lift 自 `inbound_block.rs::spawn_mock_sse_upstream`（content_type 由调用方指定，更通用）。
///
/// # Panics
///
/// bind `127.0.0.1:0` 失败时 panic。
pub async fn spawn_mock_streaming_upstream<F, Fut>(
    content_type: &'static str,
    responder: F,
) -> MockUpstream
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
                                let resp: Response<MockStreamBody> = Response::builder()
                                    .status(status)
                                    .header(hyper::header::CONTENT_TYPE, content_type)
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

    MockUpstream {
        addr,
        _shutdown: tx,
    }
}

/// 上游 SSE / JSON 响应体生成器。
///
/// lift 自 `content_type_matrix.rs` + `inbound_block.rs` 的响应构造逻辑，保证 event 名 /
/// 字段与真实 Anthropic / OpenAI 协议一致（`message_start` / `content_block_delta` /
/// `message_stop` 等）。
///
/// 同时提供 `*_bytes`（裸 body bytes，喂 [`spawn_mock_streaming_upstream`]）和
/// `*_response`（完整 `Response<Full<Bytes>>`，喂 [`spawn_mock_upstream`]）两类。
pub mod responses {
    use bytes::Bytes;
    use http_body_util::Full;
    use hyper::Response;

    /// 把 `(event_name, data)` 列表序列化为 Anthropic SSE bytes。
    ///
    /// lift 自 `inbound_block.rs::sse_response` / `content_type_matrix.rs::anthropic_sse_response`。
    #[must_use]
    pub fn anthropic_sse_raw(events: &[(&str, &str)]) -> Bytes {
        let mut s = String::new();
        for (event_name, data) in events {
            s.push_str(&format!("event: {event_name}\ndata: {data}\n\n"));
        }
        Bytes::from(s)
    }

    /// 把 `data:` 块列表序列化为 OpenAI SSE bytes（无 `event:` 行，末尾 `data: [DONE]`）。
    ///
    /// lift 自 `content_type_matrix.rs::openai_sse_response`。
    #[must_use]
    pub fn openai_sse_raw(chunks: &[&str]) -> Bytes {
        let mut s = String::new();
        for chunk in chunks {
            s.push_str(&format!("data: {chunk}\n\n"));
        }
        s.push_str("data: [DONE]\n\n");
        Bytes::from(s)
    }

    /// Anthropic 非流式 JSON 响应（单个 text content block），返回裸 body bytes。
    #[must_use]
    pub fn anthropic_json_bytes(text: &str) -> Bytes {
        let json = serde_json::json!({
            "id": "msg_01",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-5",
            "content": [{ "type": "text", "text": text }],
            "stop_reason": "end_turn",
            "stop_sequence": serde_json::Value::Null,
            "usage": { "input_tokens": 10, "output_tokens": 20 }
        });
        Bytes::from(json.to_string())
    }

    /// Anthropic 非流式 JSON 响应，返回完整 `Response<Full<Bytes>>`（喂 `spawn_mock_upstream`）。
    #[must_use]
    pub fn anthropic_json_response(text: &str) -> Response<Full<Bytes>> {
        json_200(anthropic_json_bytes(text))
    }

    /// Anthropic JSON 响应，**自定义 `usage`**（ADR-038 超额计费检测测试：模拟 relay 虚报
    /// token 用量——声明值远高于内容实际）。
    #[must_use]
    pub fn anthropic_json_response_with_usage(
        text: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Response<Full<Bytes>> {
        let json = serde_json::json!({
            "id": "msg_01",
            "type": "message",
            "role": "assistant",
            "model": "claude-sonnet-4-5",
            "content": [{ "type": "text", "text": text }],
            "stop_reason": "end_turn",
            "stop_sequence": serde_json::Value::Null,
            "usage": { "input_tokens": input_tokens, "output_tokens": output_tokens }
        });
        json_200(Bytes::from(json.to_string()))
    }

    /// Anthropic 流式 SSE 响应：每个 delta 一个 `content_block_delta` text_delta，返回裸 bytes。
    ///
    /// 含完整的 `message_start` / `content_block_start` / `content_block_stop` /
    /// `message_delta` / `message_stop` 骨架。
    #[must_use]
    pub fn anthropic_sse_bytes(deltas: &[&str]) -> Bytes {
        let mut events: Vec<(String, String)> = Vec::new();
        events.push((
            "message_start".to_owned(),
            r#"{"type":"message_start","message":{"id":"m","type":"message","role":"assistant","content":[],"model":"x","usage":{"input_tokens":1,"output_tokens":1}}}"#
                .to_owned(),
        ));
        events.push((
            "content_block_start".to_owned(),
            r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#
                .to_owned(),
        ));
        for delta in deltas {
            let escaped = json_escape(delta);
            events.push((
                "content_block_delta".to_owned(),
                format!(
                    r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{escaped}"}}}}"#
                ),
            ));
        }
        events.push((
            "content_block_stop".to_owned(),
            r#"{"type":"content_block_stop","index":0}"#.to_owned(),
        ));
        events.push((
            "message_delta".to_owned(),
            r#"{"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":5}}"#
                .to_owned(),
        ));
        events.push((
            "message_stop".to_owned(),
            r#"{"type":"message_stop"}"#.to_owned(),
        ));

        let pairs: Vec<(&str, &str)> = events
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        anthropic_sse_raw(&pairs)
    }

    /// Anthropic 流式 SSE 响应：一个 `tool_use` content block（含 `input_json_delta`），返回裸 bytes。
    ///
    /// `input_json` 为完整 JSON 字符串（如 `{"to":"0xabc"}`），会被拆成一个 `input_json_delta`。
    #[must_use]
    pub fn anthropic_tool_use_sse_bytes(tool_name: &str, input_json: &str) -> Bytes {
        let name_escaped = json_escape(tool_name);
        let input_escaped = json_escape(input_json);
        let events: Vec<(String, String)> = vec![
            (
                "message_start".to_owned(),
                r#"{"type":"message_start","message":{"id":"m","type":"message","role":"assistant","content":[],"model":"x","usage":{"input_tokens":1,"output_tokens":1}}}"#
                    .to_owned(),
            ),
            (
                "content_block_start".to_owned(),
                format!(
                    r#"{{"type":"content_block_start","index":0,"content_block":{{"type":"tool_use","id":"tu1","name":"{name_escaped}","input":{{}}}}}}"#
                ),
            ),
            (
                "content_block_delta".to_owned(),
                format!(
                    r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"input_json_delta","partial_json":"{input_escaped}"}}}}"#
                ),
            ),
            (
                "content_block_stop".to_owned(),
                r#"{"type":"content_block_stop","index":0}"#.to_owned(),
            ),
            (
                "message_delta".to_owned(),
                r#"{"type":"message_delta","delta":{"stop_reason":"tool_use","stop_sequence":null},"usage":{"output_tokens":5}}"#
                    .to_owned(),
            ),
            (
                "message_stop".to_owned(),
                r#"{"type":"message_stop"}"#.to_owned(),
            ),
        ];
        let pairs: Vec<(&str, &str)> = events
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        anthropic_sse_raw(&pairs)
    }

    /// OpenAI 非流式 JSON 响应（单个 message content），返回裸 body bytes。
    #[must_use]
    pub fn openai_json_bytes(text: &str) -> Bytes {
        let json = serde_json::json!({
            "id": "chatcmpl-01",
            "object": "chat.completion",
            "created": 1_700_000_000u64,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": { "role": "assistant", "content": text },
                "finish_reason": "stop"
            }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 20, "total_tokens": 30 }
        });
        Bytes::from(json.to_string())
    }

    /// OpenAI 非流式 JSON 响应，返回完整 `Response<Full<Bytes>>`。
    #[must_use]
    pub fn openai_json_response(text: &str) -> Response<Full<Bytes>> {
        json_200(openai_json_bytes(text))
    }

    /// OpenAI 流式 SSE 响应：每个 delta 一个 `chat.completion.chunk` content delta，返回裸 bytes。
    #[must_use]
    pub fn openai_sse_bytes(deltas: &[&str]) -> Bytes {
        let mut chunks: Vec<String> = Vec::new();
        for delta in deltas {
            let escaped = json_escape(delta);
            chunks.push(format!(
                r#"{{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4o","choices":[{{"index":0,"delta":{{"content":"{escaped}"}},"finish_reason":null}}]}}"#
            ));
        }
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        openai_sse_raw(&refs)
    }

    /// Anthropic 流式 SSE，**自定义 usage**（ADR-038 超额计费检测测试）：`message_start` 带
    /// `input_tokens`、`message_delta` 带 `output_tokens`，模拟 relay 虚报。
    #[must_use]
    pub fn anthropic_sse_bytes_with_usage(
        text: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Bytes {
        let escaped = json_escape(text);
        let msg_start = format!(
            r#"{{"type":"message_start","message":{{"id":"m","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4-5","usage":{{"input_tokens":{input_tokens},"output_tokens":1}}}}}}"#
        );
        let cb_delta = format!(
            r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{escaped}"}}}}"#
        );
        let msg_delta = format!(
            r#"{{"type":"message_delta","delta":{{"stop_reason":"end_turn","stop_sequence":null}},"usage":{{"output_tokens":{output_tokens}}}}}"#
        );
        let events: Vec<(&str, &str)> = vec![
            ("message_start", &msg_start),
            (
                "content_block_start",
                r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
            ),
            ("content_block_delta", &cb_delta),
            (
                "content_block_stop",
                r#"{"type":"content_block_stop","index":0}"#,
            ),
            ("message_delta", &msg_delta),
            ("message_stop", r#"{"type":"message_stop"}"#),
        ];
        anthropic_sse_raw(&events)
    }

    /// OpenAI 流式 SSE，**自定义 usage**（ADR-038）：content chunk + finish chunk + 末尾
    /// usage chunk（`choices:[]` + `usage:{prompt_tokens, completion_tokens}`，即 include_usage）。
    #[must_use]
    pub fn openai_sse_bytes_with_usage(
        text: &str,
        prompt_tokens: u64,
        completion_tokens: u64,
    ) -> Bytes {
        let escaped = json_escape(text);
        let content = format!(
            r#"{{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4o","choices":[{{"index":0,"delta":{{"content":"{escaped}"}},"finish_reason":null}}]}}"#
        );
        let finish = r#"{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4o","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#.to_owned();
        let usage = format!(
            r#"{{"id":"x","object":"chat.completion.chunk","created":0,"model":"gpt-4o","choices":[],"usage":{{"prompt_tokens":{prompt_tokens},"completion_tokens":{completion_tokens},"total_tokens":{}}}}}"#,
            prompt_tokens + completion_tokens
        );
        openai_sse_raw(&[&content, &finish, &usage])
    }

    /// OpenAI 非流式 JSON 响应，**自定义 usage**（ADR-038），返回完整 `Response<Full<Bytes>>`。
    #[must_use]
    pub fn openai_json_response_with_usage(
        text: &str,
        prompt_tokens: u64,
        completion_tokens: u64,
    ) -> Response<Full<Bytes>> {
        let json = serde_json::json!({
            "id": "chatcmpl-01",
            "object": "chat.completion",
            "created": 1_700_000_000u64,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": { "role": "assistant", "content": text },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": prompt_tokens,
                "completion_tokens": completion_tokens,
                "total_tokens": prompt_tokens + completion_tokens
            }
        });
        json_200(Bytes::from(json.to_string()))
    }

    /// 401 认证错误 JSON 响应（Anthropic 风格 error envelope），返回完整 `Response<Full<Bytes>>`。
    #[must_use]
    pub fn auth_error_401() -> Response<Full<Bytes>> {
        let json = serde_json::json!({
            "type": "error",
            "error": { "type": "authentication_error", "message": "invalid x-api-key" }
        });
        Response::builder()
            .status(401)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from(json.to_string())))
            .unwrap()
    }

    /// 把裸 body bytes 包成 `200 application/json` 的 `Response<Full<Bytes>>`。
    fn json_200(body: Bytes) -> Response<Full<Bytes>> {
        Response::builder()
            .status(200)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .body(Full::new(body))
            .unwrap()
    }

    /// 转义字符串使其能安全嵌入 JSON 字符串字面量（处理 `"` / `\` / 换行）。
    ///
    /// 仅覆盖测试 payload 常见字符；不是完整 JSON 字符串编码器。
    fn json_escape(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }
}
