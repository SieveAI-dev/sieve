//! 透传 daemon（架构图节点 ①③⑤⑧）。
//!
//! Week 2：POST /v1/messages body 收集 → 出站规则扫描 → Critical 命中时返回 426；
//! 非 messages 路径 / 解析失败 / 无命中 → 流式透传（Week 1 行为保持不变）。
//!
//! Week 3：出站 dry_run+Critical fail-closed 修正 + 入站 SSE tee 截流检测。
//!
//! Week 4（v1.4）：
//! - 出站 AutoRedact：命中 Redact action 时脱敏 body bytes 后转发，**不返回 426**；
//! - 入站 Hook 类（HookMark）：写 IPC pending 文件，SSE 流原样转发，**不调用 sieve_blocked**；
//! - 入站 GUI 类（HoldForDecision）：hold SSE 流 + keep-alive，等用户决策后 Allow/Deny；
//! - IpcServer 随 daemon 启动，accept loop 在后台 spawn。
//!
//! 关联：PRD v1.4 §6.1 §6.7 / ADR-013（IPC）/ ADR-014（双层防御）/ ADR-016（处置矩阵）。

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use futures_util::StreamExt as _;
use http_body_util::{combinators::BoxBody, BodyExt, StreamBody};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use sieve_core::detection::Action;
use sieve_core::pipeline::inbound::{InboundEngine, InboundFilter};
use sieve_core::pipeline::outbound::OutboundFilter;
use sieve_core::pipeline::outbound_redact::{redact_segments, RedactHit};
use sieve_core::pipeline::streaming::StreamingPipelineNode as _;
use sieve_core::sse::parser::SseParser;
use sieve_core::tool_use_aggregator::Aggregator;
use sieve_core::Forwarder;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::config::Config;

/// 响应 body 的统一类型：错误为装箱 trait object，兼容 h1/h2 body 差异。
type ResponseBody = BoxBody<Bytes, Box<dyn std::error::Error + Send + Sync>>;

/// 启动 daemon，永久阻塞直到进程收到信号。
///
/// `filter` 是出站规则引擎包装；`inbound_engine` + `inbound_sieveignore` 用于每连接构造
/// [`InboundFilter`]（每连接独立实例，共享 engine Arc）。
/// `cfg.dry_run` 决定是否实际拦截。
///
/// v1.4：启动时绑定 IpcServer Unix socket，accept loop 在后台 spawn。
///
/// # Errors
/// bind 端口失败或 Forwarder 初始化失败时返回错误。
pub async fn run(
    cfg: Config,
    filter: Arc<OutboundFilter>,
    inbound_engine: Arc<dyn InboundEngine>,
    inbound_sieveignore: Arc<HashSet<String>>,
) -> Result<()> {
    let listen = cfg.listen_addr()?;
    let dry_run = cfg.dry_run;
    let forwarder =
        Arc::new(Forwarder::new(&cfg.upstream_url).map_err(|e| anyhow!("init forwarder: {e}"))?);

    // v1.4：初始化 IpcServer（Unix socket），供 GUI 类 hold 流使用。
    // socket path = ~/.sieve/ipc.sock（或 $SIEVE_HOME/ipc.sock）。
    // 若初始化失败（如 $HOME 未设置），打印警告后继续——GuiPopup detection 会以 fail-closed 处理。
    let ipc_server: Option<Arc<sieve_ipc::IpcServer>> = match sieve_ipc::paths::sieve_home() {
        Ok(home) => {
            let socket_path = sieve_ipc::paths::ipc_socket_path(&home);
            match sieve_ipc::IpcServer::bind(socket_path.clone()) {
                Ok((server, listener)) => {
                    let server = Arc::new(server);
                    let srv_clone = Arc::clone(&server);
                    tokio::spawn(async move {
                        srv_clone.run(listener).await;
                    });
                    tracing::info!(socket = %socket_path.display(), "IPC server started");
                    Some(server)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "IPC server bind failed; GUI popup decisions will use fail-closed fallback");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "SIEVE_HOME not set; IPC server disabled");
            None
        }
    };

    let listener = TcpListener::bind(listen)
        .await
        .with_context(|| format!("bind {}", listen))?;

    tracing::info!(
        listen = %listen,
        upstream = %cfg.upstream_url,
        dry_run = dry_run,
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
        let filter = filter.clone();
        let inbound_engine = inbound_engine.clone();
        let inbound_sieveignore = inbound_sieveignore.clone();
        let ipc_server = ipc_server.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let f = forwarder.clone();
                let flt = filter.clone();
                // 每连接独立 InboundFilter（&mut self trait 要求）
                let ib_filter =
                    InboundFilter::new(inbound_engine.clone(), inbound_sieveignore.clone());
                let ipc = ipc_server.clone();
                async move { proxy(f, flt, ib_filter, dry_run, ipc, req).await }
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
async fn proxy(
    forwarder: Arc<Forwarder>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>, hyper::Error> {
    match proxy_inner(forwarder, filter, inbound_filter, dry_run, ipc, req).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            tracing::error!(error = %e, "proxy failed");
            let body = format!("sieve proxy error: {e}");
            let resp = Response::builder()
                .status(http::StatusCode::BAD_GATEWAY)
                .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(string_body(body))
                .unwrap_or_else(|_| Response::new(empty_body()));
            Ok(resp)
        }
    }
}

/// 核心代理逻辑。
///
/// - POST /v1/messages → collect body → 出站扫描 → 426 / 脱敏转发 / 入站 SSE tee 检测
/// - 其他路径 → 流式透传（Week 1 行为）
async fn proxy_inner(
    forwarder: Arc<Forwarder>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>> {
    let (parts, body) = req.into_parts();
    let path = parts.uri.path().to_string();
    let method = parts.method.clone();

    let is_messages_post = method == http::Method::POST && path == "/v1/messages";

    if is_messages_post {
        // 1. collect 完整 body（出站扫描需要全文）
        let collected = body
            .collect()
            .await
            .map_err(|e| anyhow!("collect body: {e}"))?;
        let body_bytes = collected.to_bytes();

        // 2. 解析 AnthropicRequest；解析失败则直接透传（上游会返回 400）
        let anthropic_req: sieve_core::protocol::anthropic::AnthropicRequest =
            match serde_json::from_slice(&body_bytes) {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!("non-anthropic body, passing through: {e}");
                    return forward_raw(forwarder, parts, body_bytes).await;
                }
            };

        // 3. 提取文本段 → 逐段扫描
        let texts = anthropic_req.extract_text_content();
        let mut all_detections: Vec<sieve_core::Detection> = Vec::new();

        for (offset, text) in &texts {
            use sieve_core::pipeline::PipelineNode;
            use sieve_core::protocol::unified_message::{
                ContentBlock, ContentSpan, Direction, MessageMetadata, UpstreamProvider,
            };
            use std::time::SystemTime;

            let mut msg = sieve_core::UnifiedMessage {
                role: sieve_core::Role::User,
                content_blocks: vec![ContentBlock::Text {
                    text: text.clone(),
                    span: Some(ContentSpan {
                        start: *offset,
                        end: *offset + text.len(),
                    }),
                }],
                tool_uses: vec![],
                tool_results: vec![],
                metadata: MessageMetadata {
                    session_id: "outbound-scan".into(),
                    direction: Direction::Outbound,
                    upstream_provider: UpstreamProvider::Anthropic,
                    received_at: SystemTime::now(),
                },
            };

            let hits = filter
                .process(&mut msg)
                .map_err(|e| anyhow!("outbound filter: {e}"))?;
            all_detections.extend(hits);
        }

        // 4. 决策：
        //    a. AutoRedact（Action::Redact）→ 脱敏 body bytes 后转发
        //    b. fail-closed Critical Block → 426（PRD §9 #3）
        //    c. 非 fail-closed Critical Block：dry_run=true 时仅 warn，dry_run=false 时 426
        //    d. GuiPopup（Action::HoldForDecision）→ hold HTTP 长连接等 GUI 决策（R2-#1）
        //    e. 其余 → 透传

        // 4a. 收集需要脱敏的 hit（累计文本偏移，不是 raw body 字节偏移）
        //
        // 修 #1（AutoRedact 偏移修复）：Detection.span 来自 extract_text_content() 的
        // 累计文本字符偏移，不是 raw JSON body 的字节范围。
        // 正确做法：用 redact_segments() 在文本段字符串内替换，然后重新序列化 JSON。
        // 原 redact_body_bytes(&body_bytes, ...) 路径只保留给 fuzz/单测，不在这里使用。
        let redact_hits: Vec<RedactHit> = all_detections
            .iter()
            .filter(|d| matches!(d.action, Action::Redact { .. }))
            .map(|d| RedactHit {
                rule_id: d.rule_id.clone(),
                start: d.span.start,
                end: d.span.end,
            })
            .collect();

        // 4b/c. 收集需要 Block 的 detection
        let blocking: Vec<&sieve_core::Detection> = all_detections
            .iter()
            .filter(|d| {
                if d.action != Action::Block {
                    return false;
                }
                if d.severity != sieve_core::Severity::Critical {
                    return false;
                }
                sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run
            })
            .collect();

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "OUTBOUND BLOCKED");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection");
            }
            let cloned: Vec<sieve_core::Detection> =
                blocking.iter().map(|d| (*d).clone()).collect();
            return Ok(build_426_response(&cloned));
        }

        // 4d. 出站 GuiPopup（HoldForDecision）：hold HTTP 长连接等待 GUI 决策（R2-#1 修复）。
        //
        // 出站请求是非流式 HTTP：body 已 collect，无需 SSE keep-alive（入站才需要）。
        // 客户端等待期间持有普通 HTTP 长连接（reqwest / Claude Code client 的超时决定等待上限）。
        //
        // 决策映射：
        //   Allow → 原 body 转发上游
        //   RedactAndAllow → redact_hits 非空则脱敏，否则原 body 转发
        //   Deny → 426 拒绝
        //   超时 → 按 default_on_timeout（OUT-06/08 = Redact，OUT-07/09/10 = Block）
        //
        // 关联：PRD v1.4 §5.4.2 出站超时策略表、ADR-016（二维处置矩阵）。
        let hold_detections_outbound: Vec<&sieve_core::Detection> = all_detections
            .iter()
            .filter(|d| matches!(d.action, Action::HoldForDecision { .. }))
            .collect();

        if !hold_detections_outbound.is_empty() {
            if let Some(ref ipc_server) = ipc {
                use chrono::Utc;

                let request_id = uuid::Uuid::new_v4();
                let (timeout_seconds, default_on_timeout) = hold_detections_outbound
                    .iter()
                    .find_map(|d| {
                        if let Action::HoldForDecision {
                            timeout_seconds, ..
                        } = d.action
                        {
                            // 取第一个 HoldForDecision detection 的规则 timeout/default
                            // default_on_timeout 从 detection 的 rule_id 对应规则读取，
                            // 此处用 Block 作为保守默认（规则未设则 fail-closed）
                            Some((timeout_seconds, sieve_ipc::DefaultOnTimeout::Block))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((60, sieve_ipc::DefaultOnTimeout::Block));

                let ipc_detections = hold_detections_outbound
                    .iter()
                    .map(|d| sieve_ipc::protocol::DetectionPayload {
                        rule_id: d.rule_id.clone(),
                        severity: map_severity_to_ipc(d.severity),
                        disposition: sieve_ipc::Disposition::GuiPopup,
                        title: format!("出站检测命中：{}", d.rule_id),
                        one_line_summary: d.evidence_truncated.clone(),
                        details: serde_json::json!({}),
                    })
                    .collect();

                let ipc_req = sieve_ipc::DecisionRequest {
                    request_id,
                    created_at: Utc::now(),
                    timeout_seconds,
                    default_on_timeout,
                    detections: ipc_detections,
                };

                // 出站 hold：无 SSE keep-alive，直接 await 决策
                let timeout_dur = std::time::Duration::from_secs(u64::from(timeout_seconds).max(1));
                let outcome = ipc_server.request_decision(ipc_req, timeout_dur).await;

                match outcome {
                    Ok(resp) => match resp.decision {
                        sieve_ipc::DecisionAction::Allow => {
                            tracing::info!("OUTBOUND GUI: Allow → 转发原 body");
                            // 继续往下，走正常转发路径
                        }
                        sieve_ipc::DecisionAction::RedactAndAllow => {
                            tracing::info!("OUTBOUND GUI: RedactAndAllow → 脱敏后转发");
                            // 若有 redact_hits 则脱敏，否则原 body 转发（与 Allow 同逻辑）
                            // 直接 fall-through 到下方 redact_hits 处理
                        }
                        sieve_ipc::DecisionAction::Deny => {
                            tracing::warn!("OUTBOUND GUI: Deny → 426");
                            let held: Vec<sieve_core::Detection> = hold_detections_outbound
                                .iter()
                                .map(|d| (*d).clone())
                                .collect();
                            return Ok(build_426_response(&held));
                        }
                    },
                    Err(e) => {
                        // IPC 错误：按 default_on_timeout 兜底（fail-closed）
                        tracing::warn!(error = %e, "OUTBOUND GUI: IPC error, fail-closed → 426");
                        let held: Vec<sieve_core::Detection> = hold_detections_outbound
                            .iter()
                            .map(|d| (*d).clone())
                            .collect();
                        return Ok(build_426_response(&held));
                    }
                }
            } else {
                // IPC 未初始化：fail-closed → 426
                tracing::warn!("OUTBOUND GUI: IPC not initialized, fail-closed → 426");
                let held: Vec<sieve_core::Detection> = hold_detections_outbound
                    .iter()
                    .map(|d| (*d).clone())
                    .collect();
                return Ok(build_426_response(&held));
            }
        }

        // 4a. AutoRedact：在文本段层脱敏，重新序列化 JSON 后转发（不返回 426）
        //
        // 修 #1：不再用 redact_body_bytes(&body_bytes, ...)，改为：
        // 1. redact_segments() 在文本字符串层替换
        // 2. 把替换后的文本写回 AnthropicRequest messages
        // 3. serde_json 重新序列化为新 body
        // 这样保证脱敏后 raw body 里不含原始 secret，且 JSON 结构合法。
        if !redact_hits.is_empty() {
            let seg_result = redact_segments(&texts, &redact_hits);
            tracing::info!(
                count = seg_result.redacted_count,
                rules = %seg_result.redacted_summary,
                "OUTBOUND AUTO-REDACT"
            );

            // 把替换后文本写回 AnthropicRequest，然后重新序列化
            let new_body_bytes =
                apply_redacted_texts_to_request(&anthropic_req, &texts, &seg_result.texts)
                    .and_then(|req| {
                        serde_json::to_vec(&req).map_err(|e| anyhow!("re-serialize json: {e}"))
                    })?;

            // 验证脱敏后 JSON 仍然合法（关键回归断言）
            if serde_json::from_slice::<serde_json::Value>(&new_body_bytes).is_err() {
                return Err(anyhow!("redact_segments 产生了非法 JSON，fail-closed 拦截"));
            }

            let new_body = Bytes::from(new_body_bytes);
            let new_len = new_body.len();

            // 更新 Content-Length header
            let mut new_parts = parts.clone();
            new_parts.headers.insert(
                http::header::CONTENT_LENGTH,
                http::HeaderValue::from(new_len),
            );

            // 5. prompt 地址 seed（脱敏后仍需 seed，基于原始地址）
            for (_, text) in &texts {
                if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                    tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
                }
            }

            return forward_with_inbound_inspection(
                forwarder,
                inbound_filter,
                dry_run,
                ipc,
                new_parts,
                new_body,
            )
            .await;
        }

        if dry_run && !all_detections.is_empty() {
            tracing::warn!(
                count = all_detections.len(),
                "OUTBOUND DRY-RUN: would have flagged"
            );
            for d in &all_detections {
                tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "detection (dry_run)");
            }
        }

        // 5. prompt 地址 seed
        for (_, text) in &texts {
            if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                tracing::warn!(error = %e, "seed_known_addresses_from_text failed");
            }
        }

        // 6. 出站通过 → 入站 SSE tee 截流检测
        return forward_with_inbound_inspection(
            forwarder,
            inbound_filter,
            dry_run,
            ipc,
            parts,
            body_bytes,
        )
        .await;
    }

    // 非 messages 路径：Week 1 流式透传
    forward_streaming(forwarder, parts, body).await
}

/// 透传并同步做入站 SSE 解析检测（tee 模式）。
///
/// 字节流同时被：
/// 1. 原样 forward 给客户端（via bounded channel）
/// 2. 异步喂给 SseParser → Aggregator → InboundFilter 检测
///
/// v1.4 分支逻辑：
/// - `Action::Block`（fail-closed Critical）→ 注入 `sieve_blocked` event 并截流
/// - `Action::HookMark` → 写 IPC pending 文件，SSE 流原样转发（**不注入 sieve_blocked**）
/// - `Action::HoldForDecision` → hold 流 + keep-alive，等用户决策
/// - 其余 → 透传
///
/// 关联：ADR-014 §双层防御、ADR-016 §dispatch 路由、PRD v1.4 §6.7。
async fn forward_with_inbound_inspection(
    forwarder: Arc<Forwarder>,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
) -> Result<Response<ResponseBody>> {
    use http_body_util::Full;

    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = Full::new(body_bytes)
        .map_err(|e| -> hyper::Error { match e {} })
        .boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (mut resp_parts, resp_body) = upstream_resp.into_parts();

    // 入站响应可能被 sieve 注入 sieve_blocked event 截流，实际 body 长度不一定等于上游
    // content-length。剥掉 content-length 强制 chunked transfer，防止 hyper client 截断。
    resp_parts.headers.remove(http::header::CONTENT_LENGTH);

    // P0-5：bounded channel，深度 64，上游读取自然受背压限制。
    const INBOUND_CHANNEL_DEPTH: usize = 64;
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
        INBOUND_CHANNEL_DEPTH,
    );

    tokio::spawn(async move {
        let mut parser = SseParser::new();
        let mut aggregator = Aggregator::new();

        use http_body_util::BodyStream;
        let mut stream = BodyStream::new(resp_body);

        while let Some(frame_result) = stream.next().await {
            match frame_result {
                Ok(frame) => {
                    let Some(frame_bytes) = frame.data_ref().cloned() else {
                        if tx.send(Ok(frame)).await.is_err() {
                            return;
                        }
                        continue;
                    };

                    // P0-5：push_chunk 超限时 fail-closed（IN-CAP-01）
                    let events = match parser.push_chunk(&frame_bytes) {
                        Ok(evts) => evts,
                        Err(e) => {
                            tracing::warn!(error = %e, "SSE parser 容量超限，fail-closed 注入 sieve_blocked");
                            let cap_detection =
                                build_cap_detection("IN-CAP-01", "cap-sse-event-too-large");
                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    };

                    // 收集本批 events 的 detections，按 action 分组处理
                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                        &events,
                        &mut inbound_filter,
                        &mut aggregator,
                        dry_run,
                    );

                    // 修 #4（fail-closed 被绕过修复）：Block 检查必须在 Hold 之前。
                    // 原代码 Hold allow 后 continue 会跳过 Block 检查，导致同批同时含
                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed（PRD §9 #3）。
                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
                    // 关联：ADR-014 §双层防御、PRD §9 #3。

                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
                    if !blocking.is_empty() {
                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
                        for d in &blocking {
                            tracing::warn!(rule = %d.rule_id, "inbound detection");
                        }
                        let blocked_payload = build_sieve_blocked_sse(&blocking);
                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                        return;
                    }

                    // 2. Hook 类：写 pending 文件，继续转发（不截流，不注入 sieve_blocked）
                    for d in &hook_detections {
                        write_hook_pending_silent(d);
                    }

                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
                    if !hold_detections.is_empty() {
                        if let Some(ref ipc_server) = ipc {
                            // keep-alive channel：daemon 把心跳写入 SSE 流
                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
                            let tx_ka = tx.clone();

                            // 修 R2-#3：触发帧不先发给客户端——暂存在 frame_bytes 变量里。
                            // 决策 Allow/RedactAndAllow 后再发（见下方 match 分支）；
                            // 决策 Deny 时不发，避免恶意内容已污染客户端上下文。
                            // hold 期间只向客户端发 keep-alive comment（不是模型内容）。

                            // 启动 keep-alive 转发 task
                            let ka_fwd_handle = tokio::spawn(async move {
                                while let Some(ka_bytes) = ka_rx.recv().await {
                                    if tx_ka
                                        .send(Ok(hyper::body::Frame::data(ka_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        break;
                                    }
                                }
                            });

                            // 构造 IPC 请求
                            use chrono::Utc;
                            let request_id = uuid::Uuid::new_v4();
                            let timeout_seconds = hold_detections
                                .iter()
                                .find_map(|d| {
                                    if let Action::HoldForDecision {
                                        timeout_seconds, ..
                                    } = d.action
                                    {
                                        Some(timeout_seconds)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(60);

                            let ipc_detections = hold_detections
                                .iter()
                                .map(|d| sieve_ipc::protocol::DetectionPayload {
                                    rule_id: d.rule_id.clone(),
                                    severity: map_severity_to_ipc(d.severity),
                                    disposition: sieve_ipc::Disposition::GuiPopup,
                                    title: format!("检测命中：{}", d.rule_id),
                                    one_line_summary: d.evidence_truncated.clone(),
                                    details: serde_json::json!({}),
                                })
                                .collect();

                            let ipc_req = sieve_ipc::DecisionRequest {
                                request_id,
                                created_at: Utc::now(),
                                timeout_seconds,
                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                                detections: ipc_detections,
                            };

                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
                                Arc::clone(ipc_server),
                                ipc_req,
                                ka_tx,
                            )
                            .await;

                            ka_fwd_handle.abort();

                            match outcome {
                                Ok(sieve_core::pipeline::HoldOutcome::Allow)
                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow) => {
                                    // 修 R2-#3：用户允许后，补发缓存的触发帧（hold 前未发），
                                    // 然后继续转发后续 SSE。
                                    if tx
                                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                                        .await
                                        .is_err()
                                    {
                                        return;
                                    }
                                    continue;
                                }
                                Ok(sieve_core::pipeline::HoldOutcome::Deny { reason }) => {
                                    // 修 R2-#3：用户拒绝时不发触发帧，直接注入 sieve_blocked 并关流。
                                    tracing::warn!(%reason, "INBOUND BLOCKED by GUI decision");
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "IPC hold error, fail-closed");
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                            }
                        } else {
                            // IPC 未初始化：fail-closed，阻断
                            tracing::warn!(
                                "GuiPopup detection but IPC server not initialized; fail-closed"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    }

                    // 无 blocking / hold：透传原始 frame
                    if tx
                        .send(Ok(hyper::body::Frame::data(frame_bytes)))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(Err(std::io::Error::other(format!(
                            "upstream body error: {e}"
                        ))))
                        .await;
                    return;
                }
            }
        }

        // 流结束（EOF / 提前断流），flush parser 解析残留未闭合 event
        let flushed = parser.flush();
        let (blocking, hook_detections, flush_hold_detections) =
            classify_inbound_detections(&flushed, &mut inbound_filter, &mut aggregator, dry_run);

        for d in &hook_detections {
            write_hook_pending_silent(d);
        }

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, "inbound detection (flush)");
            }
            let blocked_payload = build_sieve_blocked_sse(&blocking);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
            return;
        }

        // 修 #5（flush 阶段 hold 丢失修复）：
        // flush 路径的 HoldForDecision 命中不能静默丢弃。
        // 此时流已断无法 hold + IPC 通知 GUI，必须 fail-closed。
        // 关联：ADR-014 §双层防御、PRD §9 #3。
        if !flush_hold_detections.is_empty() {
            tracing::warn!(
                count = flush_hold_detections.len(),
                "INBOUND BLOCKED (flush-hold): GuiPopup detection at EOF, fail-closed"
            );
            for d in &flush_hold_detections {
                tracing::warn!(rule = %d.rule_id, "flush-hold detection → fail-closed");
            }
            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
        }
    });

    let body_stream = ReceiverStream::new(rx);
    let response_body: ResponseBody = StreamBody::new(body_stream)
        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();

    Ok(Response::from_parts(resp_parts, response_body))
}

/// 对一批已解析的 [`SseEvent`] 运行 inbound 检测，按 action 分类返回三个列表：
/// - `blocking`：`Action::Block` 需立即截流的 detections
/// - `hook_detections`：`Action::HookMark` 需写 pending 文件的 detections
/// - `hold_detections`：`Action::HoldForDecision` 需 hold 流的 detections
///
/// v1.4 变更：不再把所有 Critical 都返回 blocking；HookMark 和 HoldForDecision 单独处理。
///
/// 关联 ADR-016 §dispatch 路由、ADR-014 §双层防御。
fn classify_inbound_detections(
    events: &[sieve_core::sse::parser::SseEvent],
    inbound_filter: &mut sieve_core::pipeline::inbound::InboundFilter,
    aggregator: &mut sieve_core::tool_use_aggregator::Aggregator,
    dry_run: bool,
) -> (
    Vec<sieve_core::Detection>,
    Vec<sieve_core::Detection>,
    Vec<sieve_core::Detection>,
) {
    let mut all_hits: Vec<sieve_core::Detection> = Vec::new();

    for evt in events {
        match inbound_filter.observe_event(evt) {
            Ok(hits) => all_hits.extend(hits),
            Err(e) => tracing::warn!(error = %e, "inbound observe_event error"),
        }
        match aggregator.process(evt) {
            Ok(Some(tool)) => match inbound_filter.on_tool_use_complete(&tool) {
                Ok(hits) => all_hits.extend(hits),
                Err(e) => tracing::warn!(error = %e, "inbound on_tool_use_complete error"),
            },
            Ok(None) => {}
            Err(sieve_core::tool_use_aggregator::AggregatorError::MalformedToolUse {
                ref tool_id,
                ref error,
            }) => {
                tracing::warn!(tool_id = %tool_id, error = %error, "malformed tool_use partial_json，fail-closed Critical");
                all_hits.push(build_malformed_tool_use_detection(tool_id));
            }
            Err(e) => {
                tracing::warn!(error = %e, "aggregator 容量超限，fail-closed");
                all_hits.push(build_cap_detection("IN-CAP-02", "cap-aggregator-too-large"));
            }
        }
    }

    let mut blocking: Vec<sieve_core::Detection> = Vec::new();
    let mut hook_detections: Vec<sieve_core::Detection> = Vec::new();
    let mut hold_detections: Vec<sieve_core::Detection> = Vec::new();

    for d in all_hits {
        match &d.action {
            Action::Block => {
                // fail-closed Critical Block 永远阻断；非 fail-closed 遵 dry_run
                if d.severity == sieve_core::Severity::Critical
                    && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id) || !dry_run)
                {
                    blocking.push(d);
                }
                // 其余 Block（低于 Critical 或 dry_run 豁免）静默记录
            }
            Action::HookMark => {
                // Hook 类：写 pending 文件，SSE 流继续转发
                hook_detections.push(d);
            }
            Action::HoldForDecision { .. } => {
                // GUI 类：hold 流等决策
                // fail-closed 规则 GuiPopup 也走 hold，失败时 fail-closed
                hold_detections.push(d);
            }
            Action::MarkOnly | Action::SilentLog | Action::Redact { .. } => {
                // 静默 / 状态栏 / 脱敏（入站脱敏暂不实现，Week 5）
            }
        }
    }

    (blocking, hook_detections, hold_detections)
}

/// 静默写 IPC pending 文件（错误只 warn，不中断 SSE 流）。
///
/// Hook 类：SSE 流继续转发，**不注入 sieve_blocked**。
/// 关联 ADR-014 §Hook 路径、SPEC-001 §3.1。
fn write_hook_pending_silent(d: &sieve_core::Detection) {
    use chrono::Utc;

    let sieve_home = match sieve_ipc::paths::sieve_home() {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!(error = %e, rule = %d.rule_id, "cannot get SIEVE_HOME for hook pending write");
            return;
        }
    };

    let request_id = uuid::Uuid::new_v4();
    let ipc_req = sieve_ipc::DecisionRequest {
        request_id,
        created_at: Utc::now(),
        timeout_seconds: 60,
        default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
        detections: vec![sieve_ipc::protocol::DetectionPayload {
            rule_id: d.rule_id.clone(),
            severity: map_severity_to_ipc(d.severity),
            disposition: sieve_ipc::Disposition::HookTerminal,
            title: format!("检测命中：{}", d.rule_id),
            one_line_summary: d.evidence_truncated.clone(),
            details: serde_json::json!({}),
        }],
    };

    if let Err(e) = sieve_ipc::pending_file::write_pending(&ipc_req, &sieve_home) {
        tracing::warn!(error = %e, rule = %d.rule_id, "failed to write hook pending file");
    } else {
        tracing::info!(
            rule = %d.rule_id,
            request_id = %request_id,
            "HookMark: pending file written, SSE stream continues"
        );
    }
}

/// 把 `sieve_core::Severity` 映射为 `sieve_ipc::Severity`。
fn map_severity_to_ipc(s: sieve_core::Severity) -> sieve_ipc::Severity {
    match s {
        sieve_core::Severity::Critical => sieve_ipc::Severity::Critical,
        sieve_core::Severity::High => sieve_ipc::Severity::High,
        sieve_core::Severity::Medium => sieve_ipc::Severity::Medium,
        sieve_core::Severity::Low => sieve_ipc::Severity::Low,
    }
}

/// 构造注入给客户端的 `sieve_blocked` SSE event 字节块。
fn build_sieve_blocked_sse(detections: &[sieve_core::Detection]) -> Bytes {
    let payload = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": epoch_secs_string(),
        "detections": detections.iter().map(|d| serde_json::json!({
            "rule_id": d.rule_id,
            "severity": d.severity,
            "fingerprint": d.fingerprint,
        })).collect::<Vec<_>>(),
        "guidance": {
            "zh": format!(
                "Sieve 检测到 {} 条入站 Critical 命中。流已截断，响应不完整。\
                 Critical 级别命中不可通过白名单绕过，请人工审查当前上下文后重试。",
                detections.len()
            ),
            "en": format!(
                "Sieve blocked {} inbound critical detection(s). Stream truncated. \
                 Critical detections cannot be bypassed via allowlist. Please review the context and retry.",
                detections.len()
            ),
        }
    });
    Bytes::from(format!("\nevent: sieve_blocked\ndata: {}\n\n", payload))
}

/// 用已收集的 body bytes 重新构造请求并转发。
async fn forward_raw(
    forwarder: Arc<Forwarder>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
) -> Result<Response<ResponseBody>> {
    use http_body_util::Full;

    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = Full::new(body_bytes)
        .map_err(|e| -> hyper::Error { match e {} })
        .boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (resp_parts, resp_body) = upstream_resp.into_parts();
    let body: ResponseBody = resp_body
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();
    Ok(Response::from_parts(resp_parts, body))
}

/// 流式透传（Week 1 路径），不缓冲 body。
async fn forward_streaming(
    forwarder: Arc<Forwarder>,
    mut parts: http::request::Parts,
    body: Incoming,
) -> Result<Response<ResponseBody>> {
    let new_uri = forwarder
        .rewrite_uri(&parts.uri)
        .map_err(|e| anyhow!("rewrite uri: {e}"))?;
    parts.uri = new_uri;
    parts.headers.remove(http::header::HOST);
    let host_val = http::HeaderValue::from_str(forwarder.upstream_host())
        .map_err(|e| anyhow!("invalid host header: {e}"))?;
    parts.headers.insert(http::header::HOST, host_val);

    let upstream_body = body.map_err(|e| -> hyper::Error { e }).boxed();
    let upstream_req = Request::from_parts(parts, upstream_body);

    let upstream_resp = forwarder
        .forward(upstream_req)
        .await
        .map_err(|e| anyhow!("forward: {e}"))?;

    let (resp_parts, resp_body) = upstream_resp.into_parts();
    let body: ResponseBody = resp_body
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();
    Ok(Response::from_parts(resp_parts, body))
}

/// 构造 426 Upgrade Required 拦截响应（ADR-008 候选）。
fn build_426_response(detections: &[sieve_core::Detection]) -> Response<ResponseBody> {
    let blocked_at = epoch_secs_string();
    let detections_json: Vec<serde_json::Value> = detections
        .iter()
        .map(|d| {
            serde_json::json!({
                "rule_id": d.rule_id,
                "severity": d.severity,
                "fingerprint": d.fingerprint,
            })
        })
        .collect();
    let body_json = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": blocked_at,
        "detections": detections_json,
        "guidance": {
            "zh": format!(
                "Sieve 检测到 {} 条出站 Critical 命中。请检查后用 .sieveignore 加入 fingerprint 白名单，或重新发送脱敏消息。",
                detections.len()
            ),
            "en": format!(
                "Sieve blocked {} outbound critical detection(s). Review your message, then either redact or add fingerprint(s) to .sieveignore.",
                detections.len()
            ),
        }
    });
    let body_bytes = Bytes::from(body_json.to_string());
    Response::builder()
        .status(http::StatusCode::UPGRADE_REQUIRED) // 426
        .header(
            http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .body(bytes_body(body_bytes))
        .unwrap_or_else(|_| Response::new(empty_body()))
}

/// 返回 UNIX epoch 秒字符串（Phase 1 简化，Week 4 改 RFC3339）。
fn epoch_secs_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.to_string()
}

/// 把字节包成 `ResponseBody`。
fn bytes_body(b: Bytes) -> ResponseBody {
    use http_body_util::Full;
    Full::new(b)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
        .boxed()
}

/// 把字符串包成 `ResponseBody`（用于错误响应）。
fn string_body(s: String) -> ResponseBody {
    bytes_body(Bytes::from(s))
}

/// 空 body（fallback 错误响应）。
fn empty_body() -> ResponseBody {
    use http_body_util::Empty;
    Empty::<Bytes>::new()
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { match e {} })
        .boxed()
}

/// 构造 malformed tool_use Detection（P0-6，IN-CR-05-MALFORMED）。
fn build_malformed_tool_use_detection(tool_id: &str) -> sieve_core::Detection {
    use sieve_core::detection::{Action, ContentSource};
    use sieve_core::protocol::unified_message::ContentSpan;
    use uuid::Uuid;
    sieve_core::Detection {
        id: Uuid::new_v4(),
        rule_id: "IN-CR-05-MALFORMED".into(),
        severity: sieve_core::Severity::Critical,
        action: Action::Block,
        source: ContentSource::InboundAssistantText,
        span: ContentSpan { start: 0, end: 0 },
        evidence_truncated: format!("tool_id={tool_id}"),
        fingerprint: "malformed-tool-use-partial-json".into(),
    }
}

/// 构造容量上限 Detection（P0-5，IN-CAP-01 / IN-CAP-02）。
fn build_cap_detection(rule_id: &str, fingerprint_key: &str) -> sieve_core::Detection {
    use sieve_core::detection::{Action, ContentSource};
    use sieve_core::protocol::unified_message::ContentSpan;
    use uuid::Uuid;
    sieve_core::Detection {
        id: Uuid::new_v4(),
        rule_id: rule_id.into(),
        severity: sieve_core::Severity::Critical,
        action: Action::Block,
        source: ContentSource::InboundAssistantText,
        span: ContentSpan { start: 0, end: 0 },
        evidence_truncated: String::new(),
        fingerprint: fingerprint_key.into(),
    }
}

/// 把脱敏后的文本段列表写回 [`AnthropicRequest`] 并返回新 request。
///
/// `original_texts` 是 `extract_text_content()` 返回的原始段列表；
/// `redacted_texts` 是 `redact_segments()` 返回的替换后文本列表（顺序对应）。
///
/// 实现逻辑：遍历 messages，对每个文本 content 按 segment 索引匹配并替换。
///
/// # Errors
/// 如果 `redacted_texts` 长度与 `original_texts` 不一致，返回错误。
///
/// 关联：PRD v1.4 §6.1（AutoRedact 路径），修 #1（AutoRedact 偏移修复）。
fn apply_redacted_texts_to_request(
    req: &sieve_core::protocol::anthropic::AnthropicRequest,
    original_texts: &[(usize, String)],
    redacted_texts: &[String],
) -> Result<sieve_core::protocol::anthropic::AnthropicRequest> {
    if original_texts.len() != redacted_texts.len() {
        return Err(anyhow!(
            "redacted_texts 长度 {} 与 original_texts 长度 {} 不一致",
            redacted_texts.len(),
            original_texts.len()
        ));
    }

    // 用计数器追踪当前处理到第几个 segment（与 extract_text_content 遍历顺序一致）
    let mut seg_idx = 0usize;

    let mut new_messages: Vec<sieve_core::protocol::anthropic::AnthropicMessage> = Vec::new();
    for msg in &req.messages {
        let new_content = match &msg.content {
            serde_json::Value::String(_) => {
                // String 类型：一个 segment
                let replacement = redacted_texts
                    .get(seg_idx)
                    .cloned()
                    .unwrap_or_else(|| msg.content.as_str().unwrap_or("").to_string());
                seg_idx += 1;
                serde_json::Value::String(replacement)
            }
            serde_json::Value::Array(blocks) => {
                let mut new_blocks = Vec::with_capacity(blocks.len());
                for block in blocks {
                    if let Some(block_obj) = block.as_object() {
                        if block_obj.get("type").and_then(|v| v.as_str()) == Some("text")
                            && block_obj.get("text").and_then(|v| v.as_str()).is_some()
                        {
                            let replacement =
                                redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                                    block_obj
                                        .get("text")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string()
                                });
                            seg_idx += 1;
                            let mut new_obj = block_obj.clone();
                            new_obj
                                .insert("text".to_string(), serde_json::Value::String(replacement));
                            new_blocks.push(serde_json::Value::Object(new_obj));
                            continue;
                        }
                    }
                    new_blocks.push(block.clone());
                }
                serde_json::Value::Array(new_blocks)
            }
            other => other.clone(),
        };
        new_messages.push(sieve_core::protocol::anthropic::AnthropicMessage {
            role: msg.role.clone(),
            content: new_content,
        });
    }

    // 处理 system prompt（与 extract_text_content 遍历顺序一致）
    let new_system = if let Some(system) = &req.system {
        if system.as_str().is_some() {
            let replacement = redacted_texts
                .get(seg_idx)
                .cloned()
                .unwrap_or_else(|| system.as_str().unwrap_or("").to_string());
            seg_idx += 1;
            Some(serde_json::Value::String(replacement))
        } else if let Some(blocks) = system.as_array() {
            let mut new_blocks = Vec::with_capacity(blocks.len());
            for block in blocks {
                if block.get("text").and_then(|v| v.as_str()).is_some() {
                    let replacement = redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                        block
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    });
                    seg_idx += 1;
                    let mut new_obj = block.as_object().cloned().unwrap_or_default();
                    new_obj.insert("text".to_string(), serde_json::Value::String(replacement));
                    new_blocks.push(serde_json::Value::Object(new_obj));
                } else {
                    new_blocks.push(block.clone());
                }
            }
            Some(serde_json::Value::Array(new_blocks))
        } else {
            Some(system.clone())
        }
    } else {
        None
    };

    let _ = seg_idx; // 消除 unused variable 警告

    Ok(sieve_core::protocol::anthropic::AnthropicRequest {
        model: req.model.clone(),
        max_tokens: req.max_tokens,
        messages: new_messages,
        stream: req.stream,
        system: new_system,
        tools: req.tools.clone(),
        tool_choice: req.tool_choice.clone(),
        extra: req.extra.clone(),
    })
}
