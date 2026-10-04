//! 双协议流式响应检测、审批等待和放行。
use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) async fn forward_with_inbound_inspection(
    forwarder: Arc<Forwarder>,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
    meta: MultiAgentMeta,
    ctx: RequestCtx,
    billing_ctx: BillingCtxHandle,
) -> Result<Response<ResponseBody>> {
    // 解构 ctx 供内部使用（避免在 spawn move 时多次 clone Arc）
    let RequestCtx {
        caller,
        audit: audit_store,
        listener_protocol,
        listener_provider_id,
    } = ctx;
    use http_body_util::Full;

    // 修 A2-#2：把 source_channel 注入 InboundFilter，使 IN-GEN-06 运行时提级逻辑
    // 能感知来源 channel。必须在 SSE 检测开始前调用。
    inbound_filter.set_source_channel(meta.source_channel.clone());

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

    // 漏洞修复（lessons.md 2026-04-27 [安全]）：按 Content-Type 路由入站检测路径。
    //
    // 原实现假设入站永远是 SSE 流（text/event-stream），上游返回 application/json
    // 时响应 body 直接透传，所有入站规则失效。修复：
    //   - text/event-stream → 走现有 SSE 路径（tokio::spawn + channel tee）
    //   - application/json  → 收集完整 body → 解析 content[] → 提取 tool_use →
    //                         喂 InboundFilter → 命中 Critical 时替换为 sieve_blocked JSON
    let is_json_response = resp_parts
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(is_json_media_type)
        .unwrap_or(false);

    if is_json_response {
        return handle_json_inbound(
            &sieve_core::protocol::AnthropicCodec,
            resp_parts,
            resp_body,
            inbound_filter,
            dry_run,
            meta,
            ipc.clone(),
            RequestCtx::new(
                caller.clone(),
                Arc::clone(&audit_store),
                listener_protocol,
                listener_provider_id.clone(),
            ),
            billing_ctx,
        )
        .await;
    }

    // P0-5：bounded channel，深度 64，上游读取自然受背压限制。
    const INBOUND_CHANNEL_DEPTH: usize = 64;
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
        INBOUND_CHANNEL_DEPTH,
    );

    // meta 需要在 spawn 闭包中 capture（用于入站 DecisionRequest 注入）
    let inbound_meta = meta;

    tokio::spawn(async move {
        let meta = inbound_meta;
        let mut parser = SseParser::new();
        let mut aggregator = Aggregator::new();
        let mut wire_buffer = sieve_core::sse::parser::InspectedEventBuffer::default();
        // 仅 billing 启用（billing_ctx=Some）时累计 SSE usage + completion。
        let mut billing_acc = billing_ctx
            .as_ref()
            .map(|_| BillingSseAccumulator::default());

        use http_body_util::BodyStream;
        let mut stream = BodyStream::new(resp_body);

        loop {
            let frame_result =
                match tokio::time::timeout(crate::resource_limits::BODY_TIMEOUT, stream.next())
                    .await
                {
                    Ok(Some(frame)) => frame,
                    Ok(None) => break,
                    Err(_) => {
                        let _ = tx
                            .send(Err(std::io::Error::new(
                                std::io::ErrorKind::TimedOut,
                                "upstream stream stalled",
                            )))
                            .await;
                        return;
                    }
                };
            match frame_result {
                Ok(frame) => {
                    let Some(frame_bytes) = frame.data_ref().cloned() else {
                        if tx.send(Ok(frame)).await.is_err() {
                            return;
                        }
                        continue;
                    };

                    // P0-5：push_chunk 超限时 fail-closed（IN-CAP-01）
                    let frame_bytes = match wire_buffer.push_chunk(&frame_bytes) {
                        Ok(bytes) if bytes.is_empty() => continue,
                        Ok(bytes) => Bytes::from(bytes),
                        Err(_) => {
                            let payload = build_sieve_blocked_sse(&[build_cap_detection(
                                "IN-CAP-01",
                                "wire-event-too-large",
                            )]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(payload))).await;
                            return;
                        }
                    };
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

                    // 累计本批 SSE usage + completion（Anthropic SSE 观测）。
                    if let Some(acc) = billing_acc.as_mut() {
                        acc.observe_events(&events);
                    }

                    // 收集本批 events 的 detections，按 action 分组处理
                    // 修 R8-#2：传入 meta.chain_depth，chain_depth ≥ 2 时 HookMark 升级为 GuiPopup
                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                        &events,
                        &mut inbound_filter,
                        &mut aggregator,
                        dry_run,
                        meta.chain_depth,
                        &ipc,
                        &audit_store,
                        caller.as_ref(),
                        &listener_provider_id,
                    );

                    // 修 #4（fail-closed 被绕过修复）：Block 检查必须在 Hold 之前。
                    // 原代码 Hold allow 后 continue 会跳过 Block 检查，导致同批同时含
                    // Block + Hold 时，用户 GUI allow 可绕过 Critical fail-closed。
                    // 新顺序：1. Block（有 block 立即截流）→ 2. Hook → 3. Hold
                    // 关联：双层防御。

                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
                    if !blocking.is_empty() {
                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED");
                        for d in &blocking {
                            tracing::warn!(rule = %d.rule_id, "inbound detection");
                        }
                        spawn_inbound_blocked_audit(
                            &audit_store,
                            &listener_provider_id,
                            &caller,
                            &blocking,
                            "anthropic_sse",
                        );
                        let blocked_payload = build_sieve_blocked_sse(&blocking);
                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                        return;
                    }

                    // 2. Hook 类：写 pending 文件，失败时 fail-closed（不允许 fail-open）
                    for d in &hook_detections {
                        if let Err(e) = write_hook_pending_or_fail_closed(d, &meta) {
                            tracing::error!(
                                error = %e,
                                rule = %d.rule_id,
                                "Hook pending write failed; fail-closed: truncating SSE stream"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
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
                                    recommendation: None,
                                })
                                .collect();

                            // v2.0：计算 allow_remember
                            let inbound_sse_rule_ids: Vec<&str> =
                                hold_detections.iter().map(|d| d.rule_id.as_str()).collect();
                            let allow_remember = compute_allow_remember(&inbound_sse_rule_ids);

                            let ipc_req = sieve_ipc::DecisionRequest {
                                request_id,
                                created_at: Utc::now(),
                                timeout_seconds,
                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                                detections: ipc_detections,
                                // v1.5：注入 multi-agent 元数据
                                source_agent: meta.source_agent,
                                origin_chain: meta.origin_chain.clone(),
                                source_channel: meta.source_channel.clone(),
                                // 修 R7-#5：填入 header 真实 chain_depth
                                explicit_chain_depth: Some(meta.chain_depth),
                                allow_remember,
                            };

                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
                                Arc::clone(ipc_server),
                                ipc_req,
                                ka_tx,
                                "inbound",
                                Some(listener_provider_id.as_str()),
                            )
                            .await;

                            ka_fwd_handle.abort();

                            match outcome {
                                Ok(sieve_core::pipeline::HoldOutcome::Allow {
                                    remember,
                                    context_hint,
                                })
                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow {
                                    remember,
                                    context_hint,
                                }) => {
                                    // 修 R2-#3：用户允许后，补发缓存的触发帧（hold 前未发），
                                    // 然后继续转发后续 SSE。

                                    // remember=true 时写灰名单
                                    if remember && allow_remember {
                                        let agent_str =
                                            format!("{:?}", meta.source_agent).to_lowercase();
                                        for det in &hold_detections {
                                            try_write_graylist(
                                                &det.rule_id,
                                                &det.evidence_truncated,
                                                "",
                                                "anthropic",
                                                "inbound_sse",
                                                &agent_str,
                                                context_hint.clone(),
                                                &request_id.to_string(),
                                                &audit_store,
                                                &caller,
                                                &listener_provider_id,
                                            );
                                        }
                                    }

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
                                    spawn_inbound_blocked_audit(
                                        &audit_store,
                                        &listener_provider_id,
                                        &caller,
                                        &hold_detections,
                                        "anthropic_sse",
                                    );
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
        let incomplete_event = wire_buffer.has_pending();
        let _ = parser.feed(&wire_buffer.take_pending());
        let flushed = parser.flush();
        // 累计 flush 残留 events（流尾 usage / 末段文本可能在此）。
        if let Some(acc) = billing_acc.as_mut() {
            acc.observe_events(&flushed);
        }
        // 修 R8-#2：flush 阶段同样传入 chain_depth，HookMark 升级逻辑一致
        let (blocking, hook_detections, flush_hold_detections) = classify_inbound_detections(
            &flushed,
            &mut inbound_filter,
            &mut aggregator,
            dry_run,
            meta.chain_depth,
            &ipc,
            &audit_store,
            caller.as_ref(),
            &listener_provider_id,
        );

        // flush 阶段 Hook 类同样 fail-closed：写失败即截流
        for d in &hook_detections {
            if let Err(e) = write_hook_pending_or_fail_closed(d, &meta) {
                tracing::error!(
                    error = %e,
                    rule = %d.rule_id,
                    "Hook pending write failed (flush); fail-closed: truncating SSE stream"
                );
                let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                return;
            }
        }

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (flush)");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, "inbound detection (flush)");
            }
            spawn_inbound_blocked_audit(
                &audit_store,
                &listener_provider_id,
                &caller,
                &blocking,
                "anthropic_sse",
            );
            let blocked_payload = build_sieve_blocked_sse(&blocking);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
            return;
        }

        // 修 #5（flush 阶段 hold 丢失修复）：
        // flush 路径的 HoldForDecision 命中不能静默丢弃。
        // 此时流已断无法 hold + IPC 通知 GUI，必须 fail-closed。
        // 关联：双层防御。
        if !flush_hold_detections.is_empty() {
            tracing::warn!(
                count = flush_hold_detections.len(),
                "INBOUND BLOCKED (flush-hold): GuiPopup detection at EOF, fail-closed"
            );
            for d in &flush_hold_detections {
                tracing::warn!(rule = %d.rule_id, "flush-hold detection → fail-closed");
            }
            spawn_inbound_blocked_audit(
                &audit_store,
                &listener_provider_id,
                &caller,
                &flush_hold_detections,
                "anthropic_sse",
            );
            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
        }

        // 流处理结束 → 超额计费观测（completion + relay usage 已跨 chunk 累计）。
        // 仅在流自然走到结尾时触发；中途被拦截 return 的流不观测（无完整 usage，可接受缺口）。
        if incomplete_event {
            let payload = build_sieve_blocked_sse(&[build_cap_detection(
                "IN-CAP-01",
                "incomplete-upstream-event",
            )]);
            let _ = tx.send(Ok(hyper::body::Frame::data(payload))).await;
            return;
        }
        if let (Some(bctx), Some(acc)) = (billing_ctx, billing_acc) {
            let claimed = acc.claimed();
            spawn_billing_observation(Some(bctx), acc.completion, claimed);
        }
    });

    let body_stream = ReceiverStream::new(rx);
    let response_body: ResponseBody = StreamBody::new(body_stream)
        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();

    Ok(Response::from_parts(resp_parts, response_body))
}

/// OpenAI 路径入站 SSE 解析检测（tee 模式，修 R6-#2）。
///
/// 与 [`forward_with_inbound_inspection`] 逻辑完全对称，唯一区别是使用
/// [`sieve_core::sse::openai_parser::OpenAiSseParser`] 而非 Anthropic [`SseParser`]。
///
/// OpenAI SSE 格式：`data: {...}\n\n`，无 `event:` 头。
/// 产出的 [`SseEvent`] 类型与 Anthropic 相同，inbound_filter 无需感知协议差异。
///
/// R6-#3 RESOLVED：OpenAiSseParser 已支持 ContentBlockStart/Stop（含 tool_call 首帧），
/// Aggregator 的 tool_use 完整检测能力已经生效。
///
/// 关联：流式解析 / R6-#2。
#[allow(clippy::too_many_arguments)]
pub(super) async fn forward_with_openai_inbound_inspection(
    forwarder: Arc<Forwarder>,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
    meta: MultiAgentMeta,
    ctx: RequestCtx,
    billing_ctx: BillingCtxHandle,
) -> Result<Response<ResponseBody>> {
    // 解构 ctx 供内部使用
    let RequestCtx {
        caller,
        audit: audit_store,
        listener_protocol,
        listener_provider_id,
    } = ctx;
    use http_body_util::Full;
    use sieve_core::sse::openai_parser::OpenAiSseParser;
    use sieve_core::sse::parser::SseParse as _;

    inbound_filter.set_source_channel(meta.source_channel.clone());

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

    // 剥掉 content-length，防止 hyper client 截断注入的 sieve_blocked event。
    resp_parts.headers.remove(http::header::CONTENT_LENGTH);

    // 漏洞修复（lessons.md 2026-04-27 [安全]）：OpenAI 路径同样按 Content-Type 路由。
    // application/json 非流式响应里的 tool_calls 数组否则会完全绕过入站检测。
    let is_json_response = resp_parts
        .headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(is_json_media_type)
        .unwrap_or(false);

    if is_json_response {
        return handle_json_inbound(
            &sieve_core::protocol::OpenAiCodec,
            resp_parts,
            resp_body,
            inbound_filter,
            dry_run,
            meta,
            ipc.clone(),
            RequestCtx::new(
                caller.clone(),
                Arc::clone(&audit_store),
                listener_protocol,
                listener_provider_id.clone(),
            ),
            billing_ctx,
        )
        .await;
    }

    const INBOUND_CHANNEL_DEPTH: usize = 64;
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<hyper::body::Frame<Bytes>, std::io::Error>>(
        INBOUND_CHANNEL_DEPTH,
    );

    let inbound_meta = meta;

    tokio::spawn(async move {
        let meta = inbound_meta;
        let mut parser = OpenAiSseParser::new();
        let mut aggregator = Aggregator::new();
        let mut wire_buffer = sieve_core::sse::parser::InspectedEventBuffer::default();
        // 仅 billing 启用时累计 OpenAI SSE usage + completion。
        let mut billing_acc = billing_ctx
            .as_ref()
            .map(|_| BillingSseAccumulator::default());

        use http_body_util::BodyStream;
        let mut stream = BodyStream::new(resp_body);

        loop {
            let frame_result =
                match tokio::time::timeout(crate::resource_limits::BODY_TIMEOUT, stream.next())
                    .await
                {
                    Ok(Some(frame)) => frame,
                    Ok(None) => break,
                    Err(_) => {
                        let _ = tx
                            .send(Err(std::io::Error::new(
                                std::io::ErrorKind::TimedOut,
                                "upstream stream stalled",
                            )))
                            .await;
                        return;
                    }
                };
            match frame_result {
                Ok(frame) => {
                    let Some(frame_bytes) = frame.data_ref().cloned() else {
                        if tx.send(Ok(frame)).await.is_err() {
                            return;
                        }
                        continue;
                    };

                    // P0-5：feed 超限时 fail-closed（IN-CAP-01）
                    let frame_bytes = match wire_buffer.push_chunk(&frame_bytes) {
                        Ok(bytes) if bytes.is_empty() => continue,
                        Ok(bytes) => Bytes::from(bytes),
                        Err(_) => {
                            let payload = build_sieve_blocked_sse(&[build_cap_detection(
                                "IN-CAP-01",
                                "wire-event-too-large",
                            )]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(payload))).await;
                            return;
                        }
                    };
                    let events = match parser.feed(&frame_bytes) {
                        Ok(evts) => evts,
                        Err(e) => {
                            tracing::warn!(error = %e, "OpenAI SSE parser 容量超限，fail-closed 注入 sieve_blocked");
                            let cap_detection =
                                build_cap_detection("IN-CAP-01", "cap-sse-event-too-large");
                            let blocked_payload = build_sieve_blocked_sse(&[cap_detection]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    };

                    // 累计本批 SSE usage + completion（OpenAI SSE 观测）。
                    if let Some(acc) = billing_acc.as_mut() {
                        acc.observe_events(&events);
                    }

                    // 修 R8-#2：传入 meta.chain_depth，chain_depth ≥ 2 时 HookMark 升级为 GuiPopup
                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                        &events,
                        &mut inbound_filter,
                        &mut aggregator,
                        dry_run,
                        meta.chain_depth,
                        &ipc,
                        &audit_store,
                        caller.as_ref(),
                        &listener_provider_id,
                    );

                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
                    if !blocking.is_empty() {
                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (openai)");
                        for d in &blocking {
                            tracing::warn!(rule = %d.rule_id, "openai inbound detection");
                        }
                        spawn_inbound_blocked_audit(
                            &audit_store,
                            &listener_provider_id,
                            &caller,
                            &blocking,
                            "openai_sse",
                        );
                        let blocked_payload = build_sieve_blocked_sse(&blocking);
                        let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                        return;
                    }

                    // 2. Hook 类：写 pending 文件，失败时 fail-closed
                    for d in &hook_detections {
                        if let Err(e) = write_hook_pending_or_fail_closed(d, &meta) {
                            tracing::error!(
                                error = %e,
                                rule = %d.rule_id,
                                "Hook pending write failed (openai); fail-closed"
                            );
                            let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                            return;
                        }
                    }

                    // 3. GUI 类：hold 流 + keep-alive + 等用户决策
                    if !hold_detections.is_empty() {
                        if let Some(ref ipc_server) = ipc {
                            let (ka_tx, mut ka_rx) = mpsc::channel::<Bytes>(8);
                            let tx_ka = tx.clone();

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
                                    title: format!("检测命中（openai）：{}", d.rule_id),
                                    one_line_summary: d.evidence_truncated.clone(),
                                    details: serde_json::json!({}),
                                    recommendation: None,
                                })
                                .collect();

                            // v2.0：计算 allow_remember
                            let openai_sse_rule_ids: Vec<&str> =
                                hold_detections.iter().map(|d| d.rule_id.as_str()).collect();
                            let allow_remember = compute_allow_remember(&openai_sse_rule_ids);

                            let ipc_req = sieve_ipc::DecisionRequest {
                                request_id,
                                created_at: Utc::now(),
                                timeout_seconds,
                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                                detections: ipc_detections,
                                source_agent: meta.source_agent,
                                origin_chain: meta.origin_chain.clone(),
                                source_channel: meta.source_channel.clone(),
                                // 修 R7-#5：填入 header 真实 chain_depth
                                explicit_chain_depth: Some(meta.chain_depth),
                                allow_remember,
                            };

                            let outcome = sieve_core::pipeline::inbound_hold::hold_and_decide(
                                Arc::clone(ipc_server),
                                ipc_req,
                                ka_tx,
                                "inbound",
                                Some(listener_provider_id.as_str()),
                            )
                            .await;

                            ka_fwd_handle.abort();

                            match outcome {
                                Ok(sieve_core::pipeline::HoldOutcome::Allow {
                                    remember,
                                    context_hint,
                                })
                                | Ok(sieve_core::pipeline::HoldOutcome::RedactAndAllow {
                                    remember,
                                    context_hint,
                                }) => {
                                    // v2.0 §5.4.2：remember=true 时写灰名单（OpenAI 入站 SSE 路径）
                                    if remember && allow_remember {
                                        let agent_str =
                                            format!("{:?}", meta.source_agent).to_lowercase();
                                        for det in &hold_detections {
                                            try_write_graylist(
                                                &det.rule_id,
                                                &det.evidence_truncated,
                                                "",
                                                "openai",
                                                "inbound_sse",
                                                &agent_str,
                                                context_hint.clone(),
                                                &request_id.to_string(),
                                                &audit_store,
                                                &caller,
                                                &listener_provider_id,
                                            );
                                        }
                                    }

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
                                    tracing::warn!(%reason, "INBOUND BLOCKED (openai) by GUI decision");
                                    spawn_inbound_blocked_audit(
                                        &audit_store,
                                        &listener_provider_id,
                                        &caller,
                                        &hold_detections,
                                        "openai_sse",
                                    );
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                                Err(e) => {
                                    tracing::warn!(error = %e, "IPC hold error (openai), fail-closed");
                                    let blocked_payload = build_sieve_blocked_sse(&hold_detections);
                                    let _ = tx
                                        .send(Ok(hyper::body::Frame::data(blocked_payload)))
                                        .await;
                                    return;
                                }
                            }
                        } else {
                            tracing::warn!(
                                "GuiPopup detection (openai) but IPC server not initialized; fail-closed"
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
                            "upstream body error (openai): {e}"
                        ))))
                        .await;
                    return;
                }
            }
        }

        // 流结束（EOF / 提前断流），flush parser 解析残留
        let incomplete_event = wire_buffer.has_pending();
        let _ = parser.feed(&wire_buffer.take_pending());
        let flushed = parser.flush();
        // 累计 flush 残留 events（流尾 usage / 末段文本可能在此）。
        if let Some(acc) = billing_acc.as_mut() {
            acc.observe_events(&flushed);
        }
        // 修 R8-#2：flush 阶段同样传入 chain_depth，HookMark 升级逻辑一致
        let (blocking, hook_detections, flush_hold_detections) = classify_inbound_detections(
            &flushed,
            &mut inbound_filter,
            &mut aggregator,
            dry_run,
            meta.chain_depth,
            &ipc,
            &audit_store,
            caller.as_ref(),
            &listener_provider_id,
        );

        for d in &hook_detections {
            if let Err(e) = write_hook_pending_or_fail_closed(d, &meta) {
                tracing::error!(
                    error = %e,
                    rule = %d.rule_id,
                    "Hook pending write failed (openai flush); fail-closed"
                );
                let blocked_payload = build_sieve_blocked_sse(&[d.clone()]);
                let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
                return;
            }
        }

        if !blocking.is_empty() {
            tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (openai flush)");
            for d in &blocking {
                tracing::warn!(rule = %d.rule_id, "openai inbound detection (flush)");
            }
            spawn_inbound_blocked_audit(
                &audit_store,
                &listener_provider_id,
                &caller,
                &blocking,
                "openai_sse",
            );
            let blocked_payload = build_sieve_blocked_sse(&blocking);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
            return;
        }

        if !flush_hold_detections.is_empty() {
            tracing::warn!(
                count = flush_hold_detections.len(),
                "INBOUND BLOCKED (openai flush-hold): GuiPopup at EOF, fail-closed"
            );
            for d in &flush_hold_detections {
                tracing::warn!(rule = %d.rule_id, "openai flush-hold detection → fail-closed");
            }
            spawn_inbound_blocked_audit(
                &audit_store,
                &listener_provider_id,
                &caller,
                &flush_hold_detections,
                "openai_sse",
            );
            let blocked_payload = build_sieve_blocked_sse(&flush_hold_detections);
            let _ = tx.send(Ok(hyper::body::Frame::data(blocked_payload))).await;
        }

        // 流处理结束 → 超额计费观测（OpenAI SSE）。
        if incomplete_event {
            let payload = build_sieve_blocked_sse(&[build_cap_detection(
                "IN-CAP-01",
                "incomplete-upstream-event",
            )]);
            let _ = tx.send(Ok(hyper::body::Frame::data(payload))).await;
            return;
        }
        if let (Some(bctx), Some(acc)) = (billing_ctx, billing_acc) {
            let claimed = acc.claimed();
            spawn_billing_observation(Some(bctx), acc.completion, claimed);
        }
    });

    let body_stream = ReceiverStream::new(rx);
    let response_body: ResponseBody = StreamBody::new(body_stream)
        .map_err(|e: std::io::Error| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })
        .boxed();

    Ok(Response::from_parts(resp_parts, response_body))
}
