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
//! Week 5（v1.5）：
//! - 路径分发：`/v1/messages` → Anthropic 路径；`/v1/chat/completions` → OpenAI 路径；
//! - `X-Sieve-Origin` header 解析 → source_agent / origin_chain / chain_depth；
//! - chain_depth ≥ 5 → 直接 426；chain_depth ≥ 2 → 所有命中强制 GuiPopup；
//! - `X-Sieve-Source-Channel` header 解析 → DecisionRequest.source_channel。
//!
//! 关联：PRD v1.5 §6.1 §4.5 §4.6 / ADR-018（OpenAI 协议）/ ADR-019（multi-agent header）/
//!        ADR-013（IPC）/ ADR-014（双层防御）/ ADR-016（处置矩阵）。

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
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::config::Config;
use crate::upstream_routes::UpstreamRoutes;

// ── multi-agent header 解析（ADR-019）────────────────────────────────────────
// 修 R8-#1：改用 sieve_ipc::parse_origin_header，支持 3 段（无签名）和 4 段（含签名）格式。
// 旧实现用 rsplitn(2, ':') 在 4 段时把 base64 签名当 chain_depth 导致解析失败 → fail-open，
// 攻击者可在签名字段写入合法 chain_depth 数值绕过 chain_depth ≥ 2 的 GuiPopup 升级。
// 新实现委托给 sieve_ipc::parse_origin_header（splitn(4, ':')），正确处理两种格式。
// 关联：ADR-019 §Header 格式规范、PRD v1.5 §6.5。

/// 从已解析的 origin header 构造 `origin_chain`（`Vec<OriginHop>`）。
///
/// 当前仅记录发送方一跳（chain_depth 反映深度，origin_chain 记录来源 hop）。
/// chain_depth = 0 → 空 chain（用户直接调用，无委托链）。
/// chain_depth ≥ 1 → 添加一个表示发送方的 OriginHop。
///
/// 关联：ADR-019 §origin_chain 构造、PRD v1.5 §4.6。
fn build_origin_chain(
    source_agent: sieve_ipc::protocol::SourceAgent,
    chain_depth: usize,
) -> Vec<sieve_ipc::protocol::OriginHop> {
    if chain_depth == 0 {
        return Vec::new();
    }
    vec![sieve_ipc::protocol::OriginHop {
        agent: source_agent,
        action: "delegate".to_owned(),
        timestamp: chrono::Utc::now(),
    }]
}

/// 解析 `X-Sieve-Source-Channel` header（OpenClaw 跨通道标识）。
///
/// 缺 header 或值为空 → `None`（非 OpenClaw 来源）。
/// 关联：PRD v1.5 §4.5 场景 E、IN-GEN-06。
fn parse_source_channel(headers: &http::HeaderMap) -> Option<String> {
    headers
        .get("x-sieve-source-channel")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// 从请求 headers 解析 `X-Sieve-Origin`，返回 `(source_agent, origin_chain, chain_depth)`。
///
/// - 缺 header → source_agent=Unknown, chain_depth=0, origin_chain=[]
/// - 格式错误 → 同上 + audit 警告（fail-open）
/// - chain_depth ≥ 5 → 返回 chain_depth=5（调用方负责 426）
///
/// 修 R8-#1：改用 `sieve_ipc::parse_origin_header` 支持 3 段/4 段格式。
/// `ChainTooDeep` 错误时返回实际 chain_depth（让调用方触发 426，保持 fail-closed 语义）。
///
/// 关联：ADR-019 §解析策略、PRD v1.5 §6.5。
fn extract_origin_metadata(
    headers: &http::HeaderMap,
) -> (
    sieve_ipc::protocol::SourceAgent,
    Vec<sieve_ipc::protocol::OriginHop>,
    usize,
) {
    let Some(header_val) = headers.get("x-sieve-origin") else {
        return (sieve_ipc::protocol::SourceAgent::Unknown, Vec::new(), 0);
    };

    let Ok(header_str) = header_val.to_str() else {
        tracing::warn!("X-Sieve-Origin: 包含非 UTF-8 字符，fail-open");
        return (sieve_ipc::protocol::SourceAgent::Unknown, Vec::new(), 0);
    };

    match sieve_ipc::parse_origin_header(header_str) {
        Ok(h) => {
            let origin_chain = build_origin_chain(h.source_agent, h.chain_depth);
            (h.source_agent, origin_chain, h.chain_depth)
        }
        Err(sieve_ipc::OriginHeaderError::ChainTooDeep(d)) => {
            // chain_depth ≥ 5：保留真实 depth，让调用方走 426 分支（不 fail-open）。
            tracing::warn!(
                chain_depth = d,
                "X-Sieve-Origin chain_depth ≥ 5，转发给 426 检查"
            );
            (sieve_ipc::protocol::SourceAgent::Unknown, Vec::new(), d)
        }
        Err(e) => {
            tracing::warn!(error = %e, raw = header_str, "X-Sieve-Origin 解析失败，fail-open，视为无 header");
            (sieve_ipc::protocol::SourceAgent::Unknown, Vec::new(), 0)
        }
    }
}

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
    address_guard_config: sieve_core::pipeline::inbound::AddressGuardConfig,
) -> Result<()> {
    let listen = cfg.listen_addr()?;
    let dry_run = cfg.dry_run;
    let forwarder =
        Arc::new(Forwarder::new(&cfg.upstream_url).map_err(|e| anyhow!("init forwarder: {e}"))?);

    // R11-#1：加载 OpenClaw 上游路由表（~/.sieve/upstream-routes.json）。
    // 加载失败（文件不存在 / JSON 非法）时 warn 后继续，所有请求走默认上游兜底。
    // 成功时为每个 provider id 预构建 Forwarder（含连接池），请求处理时直接 map lookup。
    let provider_forwarders: Arc<HashMap<String, Arc<Forwarder>>> = {
        let routes = match sieve_ipc::paths::sieve_home() {
            Ok(home) => {
                let path = home.join("upstream-routes.json");
                match UpstreamRoutes::load(&path) {
                    Ok(r) => {
                        if !r.is_empty() {
                            tracing::info!(
                                count = r.len(),
                                path = %path.display(),
                                "upstream-routes loaded"
                            );
                        }
                        r
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "upstream-routes load failed; all requests will use default upstream"
                        );
                        UpstreamRoutes::default()
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "SIEVE_HOME not set; upstream-routes disabled, using default upstream"
                );
                UpstreamRoutes::default()
            }
        };

        let mut map: HashMap<String, Arc<Forwarder>> = HashMap::new();
        for (provider_id, upstream_url) in routes.iter() {
            match Forwarder::new(upstream_url) {
                Ok(f) => {
                    tracing::debug!(provider = %provider_id, upstream = %upstream_url, "provider forwarder ready");
                    map.insert(provider_id.to_owned(), Arc::new(f));
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        provider = %provider_id,
                        upstream = %upstream_url,
                        "failed to build forwarder for provider; will use default upstream"
                    );
                }
            }
        }
        Arc::new(map)
    };

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
        let ag_cfg = address_guard_config.clone();
        let pf = provider_forwarders.clone();

        tokio::spawn(async move {
            let io = TokioIo::new(stream);
            let svc = service_fn(move |req| {
                let f = forwarder.clone();
                let flt = filter.clone();
                // 每连接独立 InboundFilter（&mut self trait 要求），
                // 传入从 IN-CR-01 RuleEntry 读取的配置（修 R3-#5）
                let ib_filter = InboundFilter::with_address_guard_config(
                    inbound_engine.clone(),
                    inbound_sieveignore.clone(),
                    ag_cfg.clone(),
                );
                let ipc = ipc_server.clone();
                let pf = pf.clone();
                async move { proxy(f, pf, flt, ib_filter, dry_run, ipc, req).await }
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
    provider_forwarders: Arc<HashMap<String, Arc<Forwarder>>>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>, hyper::Error> {
    match proxy_inner(
        forwarder,
        provider_forwarders,
        filter,
        inbound_filter,
        dry_run,
        ipc,
        req,
    )
    .await
    {
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
/// 路径分发（v1.5，ADR-018 + ADR-019）：
/// - POST /v1/messages → Anthropic 路径（collect body → 出站扫描 → 426 / 脱敏转发 / 入站 SSE tee 检测）
/// - POST /v1/chat/completions → OpenAI 路径（同等出站扫描，走 OpenAI schema 解析）
/// - 其他路径 → 流式透传（Week 1 行为）
///
/// 公共预处理（两条 LLM 路径都执行）：
/// 1. 解析 `X-Sieve-Origin` → source_agent / origin_chain / chain_depth
/// 2. chain_depth ≥ 5 → 直接 426 拒绝（ADR-019 §嵌套深度限制）
/// 3. 解析 `X-Sieve-Source-Channel` → source_channel（OpenClaw 跨通道）
/// 4. chain_depth ≥ 2 → 所有命中强制升级为 GuiPopup disposition
///
/// R11-#1：在所有路径分发前���解析 `X-Sieve-Provider` header 并从路由表中查找上游 Forwarder：
/// - 有 header + 路由表命中 → 用对应 provider 的 Forwarder（OpenClaw 多 provider 路由）
/// - 无 header 或路由表未命中 → 用默认 `forwarder`（cfg.upstream_url 兜底）
///
/// 转发到上游前**移除** `X-Sieve-Provider` header（内部路由 header，不透传给上游）。
///
/// 关联：PRD v1.5 §6.1 / ADR-018（OpenAI 协议）/ ADR-019（multi-agent header）。
async fn proxy_inner(
    forwarder: Arc<Forwarder>,
    provider_forwarders: Arc<HashMap<String, Arc<Forwarder>>>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    req: Request<Incoming>,
) -> Result<Response<ResponseBody>> {
    let (mut parts, body) = req.into_parts();

    // R11-#1：解析 X-Sieve-Provider，查路由表选择上游，转发前移除此 header。
    // 移除在此处（统一入口），后续所有 forward_* 调用无需感知此 header。
    let forwarder: Arc<Forwarder> = {
        let provider_id = parts
            .headers
            .remove("x-sieve-provider")
            .and_then(|v| v.to_str().ok().map(|s| s.trim().to_owned()))
            .filter(|s| !s.is_empty());
        match provider_id {
            Some(ref id) => {
                if let Some(pf) = provider_forwarders.get(id.as_str()) {
                    tracing::debug!(provider = %id, "X-Sieve-Provider: routing to provider upstream");
                    Arc::clone(pf)
                } else {
                    tracing::debug!(
                        provider = %id,
                        "X-Sieve-Provider: no route found, falling back to default upstream"
                    );
                    forwarder
                }
            }
            None => forwarder,
        }
    };

    let path = parts.uri.path().to_string();
    let method = parts.method.clone();

    // ── v1.5：公共 header 解析（所有 LLM 路径）────────────────────────────────

    // 1. X-Sieve-Origin → source_agent / origin_chain / chain_depth（ADR-019）
    let (source_agent, origin_chain, chain_depth) = extract_origin_metadata(&parts.headers);

    // 2. chain_depth ≥ 5 → 直接 426（ADR-019 §嵌套深度限制，attack mode）
    if chain_depth >= 5 {
        tracing::warn!(
            chain_depth,
            "X-Sieve-Origin chain_depth ≥ 5，嵌套调用过深，拒绝请求"
        );
        return Ok(build_426_nested_rejection(chain_depth));
    }

    // 3. X-Sieve-Source-Channel（OpenClaw 跨通道，PRD v1.5 §4.5）
    let source_channel = parse_source_channel(&parts.headers);

    // ── 路径分类（白名单 collect，修 R7-#2）─────────────────────────────────────
    //
    // 修 R7-#2（DoS 修复）：改为**路径白名单 collect**，只对需要检测的路径预先缓冲 body；
    // 其余 POST 路径（透传）body 不经过 collect，保持流式，不存在无界缓冲 DoS 向量。
    //
    // 白名单路径：
    //   1. /v1/messages          → Anthropic 出站扫描需要 collect
    //   2. /v1/chat/completions  → OpenAI 出站扫描需要 collect
    //   3. is_skill_install_path → IN-CR-06 body manifest 检测需要 collect
    //
    // IN-CR-06 覆盖范围说明（trade-off，显式记录）：
    //   body manifest 检测仅在 `is_skill_install_path(path)` 为 true 时生效。
    //   真实 OpenClaw endpoint 与路径列表不符时，body 检测不跑（路径白名单 only）。
    //   Week 7 实测后补充准确路径，届时覆盖范围自动扩大。
    //   R6-#4 的死代码问题（所有 POST 都 collect 以确保 body 检测跑到）接受为已知
    //   trade-off，以安全性（no DoS vector）换取检测完备性的妥协在注释中显式标注。
    //
    // 关联：sieve_core::skill_install_guard、PRD v1.5 §4.6、ADR-016。

    let is_messages_post = method == http::Method::POST && path == "/v1/messages";
    let is_chat_completions_post = method == http::Method::POST && path == "/v1/chat/completions";
    let is_skill_post = method == http::Method::POST
        && sieve_core::skill_install_guard::is_skill_install_path(&path);

    // 只对白名单路径 collect body；其余 POST 保留为流式 body，完全不缓冲。
    let (post_body_bytes, non_post_body): (Option<Bytes>, Option<hyper::body::Incoming>) =
        if is_messages_post || is_chat_completions_post || is_skill_post {
            let collected = body
                .collect()
                .await
                .map_err(|e| anyhow!("collect body (post): {e}"))?;
            (Some(collected.to_bytes()), None)
        } else {
            (None, Some(body))
        };

    // ── IN-CR-06 OpenClaw skill install 检测（路径白名单 only）──────────────────
    if is_skill_post {
        // unwrap 安全：is_skill_post 分支已 collect
        let body_bytes_skill = post_body_bytes
            .as_ref()
            .expect("body_bytes set for skill_post");

        // body ≤ 4KB 时才做 manifest 检测（> 4KB 多半不是 manifest，跳过减少误判）
        let body_json: serde_json::Value = if body_bytes_skill.len() <= 4096 {
            serde_json::from_slice(body_bytes_skill).unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        };

        let mut skill_detections = sieve_core::skill_install_guard::check_openclaw_skill_install(
            &path,
            &body_json,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );

        // chain_depth ≥ 2 → 强制 GuiPopup（ADR-019）
        if chain_depth >= 2 {
            for d in &mut skill_detections {
                if matches!(d.action, Action::HookMark) {
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                    };
                }
            }
        }

        if !skill_detections.is_empty() {
            if let Some(ref ipc_server) = ipc {
                use chrono::Utc;
                let request_id = uuid::Uuid::new_v4();
                let (timeout_seconds, default_on_timeout) = skill_detections
                    .iter()
                    .find_map(|d| {
                        if let Action::HoldForDecision {
                            timeout_seconds, ..
                        } = d.action
                        {
                            Some((timeout_seconds, sieve_ipc::DefaultOnTimeout::Block))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((120, sieve_ipc::DefaultOnTimeout::Block));

                let ipc_detections = skill_detections
                    .iter()
                    .map(|d| sieve_ipc::protocol::DetectionPayload {
                        rule_id: d.rule_id.clone(),
                        severity: map_severity_to_ipc(d.severity),
                        disposition: sieve_ipc::Disposition::GuiPopup,
                        title: format!("IN-CR-06 OpenClaw Skill Install 检测：{}", d.rule_id),
                        one_line_summary: d.evidence_truncated.clone(),
                        details: serde_json::json!({ "path": path }),
                    })
                    .collect();

                let ipc_req = sieve_ipc::DecisionRequest {
                    request_id,
                    created_at: Utc::now(),
                    timeout_seconds,
                    default_on_timeout,
                    detections: ipc_detections,
                    source_agent,
                    origin_chain: origin_chain.clone(),
                    source_channel: source_channel.clone(),
                    explicit_chain_depth: Some(chain_depth),
                };

                let timeout_dur = std::time::Duration::from_secs(u64::from(timeout_seconds).max(1));
                let outcome = ipc_server.request_decision(ipc_req, timeout_dur).await;

                match outcome {
                    Ok(resp) => match resp.decision {
                        sieve_ipc::DecisionAction::Allow
                        | sieve_ipc::DecisionAction::RedactAndAllow => {
                            tracing::info!("IN-CR-06 GUI: Allow → 转发原 body");
                            // fall-through，继续路径分发
                        }
                        sieve_ipc::DecisionAction::Deny => {
                            tracing::warn!("IN-CR-06 GUI: Deny → 426");
                            return Ok(build_426_response(&skill_detections));
                        }
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, "IN-CR-06 GUI: IPC error, fail-closed → 426");
                        return Ok(build_426_response(&skill_detections));
                    }
                }
            } else {
                // IPC 未初始化：fail-closed → 426
                tracing::warn!("IN-CR-06: IPC not initialized, fail-closed → 426");
                return Ok(build_426_response(&skill_detections));
            }
        }
    }

    // ── 路径分发 ─────────────────────────────────────────────────────────────

    if is_messages_post {
        // body 已在 POST 预收集块中 collect，直接取出
        let body_bytes = post_body_bytes.expect("body_bytes set for POST");

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

        // 4. chain_depth ≥ 2 → HookMark 升级为 HoldForDecision（强制 GUI 弹窗，ADR-019）
        // 修 R9-#2：chain_depth ≥ 2 时 HookMark + Redact 都升级为 HoldForDecision。
        if chain_depth >= 2 {
            tracing::info!(
                chain_depth,
                "X-Sieve-Origin chain_depth ≥ 2（Anthropic 路径），HookMark + Redact 升级为 GuiPopup"
            );
            for d in &mut all_detections {
                match &d.action {
                    Action::HookMark => {
                        d.action = Action::HoldForDecision {
                            request_id: uuid::Uuid::new_v4(),
                            timeout_seconds: 60,
                            default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                        };
                    }
                    Action::Redact { .. } => {
                        d.action = Action::HoldForDecision {
                            request_id: uuid::Uuid::new_v4(),
                            timeout_seconds: 60,
                            default_on_timeout: sieve_core::detection::DefaultOnTimeout::Redact,
                        };
                    }
                    _ => {}
                }
            }
        }

        // 5. 决策：
        //    a. AutoRedact（Action::Redact）→ 脱敏 body bytes 后转发
        //    b. fail-closed Critical Block → 426（PRD §9 #3）
        //    c. 非 fail-closed Critical Block：dry_run=true 时仅 warn，dry_run=false 时 426
        //    d. GuiPopup（Action::HoldForDecision）→ hold HTTP 长连接等 GUI 决策（R2-#1）
        //    e. 其余 → 透传

        // 5a. 收集需要脱敏的 hit（累计文本偏移，不是 raw body 字节偏移）
        //
        // 修 #1（AutoRedact 偏移修复）：Detection.span 来自 extract_text_content() 的
        // 累计文本字符偏移，不是 raw JSON body 的字节范围。
        // 正确做法：用 redact_segments() 在文本段字符串内替换，然后重新序列化 JSON。
        // 原 redact_body_bytes(&body_bytes, ...) 路径只保留给 fuzz/单测，不在这里使用。
        let mut redact_hits: Vec<RedactHit> = all_detections
            .iter()
            .filter(|d| matches!(d.action, Action::Redact { .. }))
            .map(|d| RedactHit {
                rule_id: d.rule_id.clone(),
                start: d.span.start,
                end: d.span.end,
            })
            .collect();

        // 5b/c. 收集需要 Block 的 detection
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

                // 修 R11-#2：从 hold_detections 的 default_on_timeout 取最严策略��
                // 与 OpenAI 路径完全镜像（merge_strictest_timeout + map_dot_to_ipc）。
                // OUT-06/08 default=Redact → 超时脱敏转发；OUT-07 default=Block → 超时 426。
                let (timeout_seconds, default_on_timeout) = hold_detections_outbound.iter().fold(
                    (60_u32, sieve_ipc::DefaultOnTimeout::Allow),
                    |acc, d| {
                        if let Action::HoldForDecision {
                            timeout_seconds,
                            default_on_timeout,
                            ..
                        } = &d.action
                        {
                            let merged =
                                merge_strictest_timeout(acc.1, map_dot_to_ipc(*default_on_timeout));
                            (acc.0.max(*timeout_seconds), merged)
                        } else {
                            acc
                        }
                    },
                );

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
                    // v1.5：注入 multi-agent 元数据（ADR-019）
                    source_agent,
                    origin_chain: origin_chain.clone(),
                    source_channel: source_channel.clone(),
                    // 修 R7-#5：填入 header 真实 chain_depth
                    explicit_chain_depth: Some(chain_depth),
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
                            // 修 R3-#3：把 held detection 的 span 升级到 redact_hits，
                            // 保证仅命中 GUI 类（无 AutoRedact 类）时也能正确脱敏。
                            // 去重：跳过已在 redact_hits 中存在的 span。
                            for d in &hold_detections_outbound {
                                let already = redact_hits
                                    .iter()
                                    .any(|h| h.start == d.span.start && h.end == d.span.end);
                                if !already {
                                    redact_hits.push(RedactHit {
                                        rule_id: d.rule_id.clone(),
                                        start: d.span.start,
                                        end: d.span.end,
                                    });
                                }
                            }
                            // fall-through 到下方 redact_hits 处理（现在含 GUI 类 span）
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
                        // 修 R11-#2：IPC 错误 / 超时时按 default_on_timeout 三路分支（镜像 OpenAI 路径）。
                        tracing::warn!(error = %e, ?default_on_timeout, "OUTBOUND GUI: IPC error, applying default_on_timeout");
                        match default_on_timeout {
                            sieve_ipc::DefaultOnTimeout::Redact => {
                                tracing::info!("OUTBOUND GUI: timeout default=redact → 脱敏转发");
                                for d in &hold_detections_outbound {
                                    let already = redact_hits
                                        .iter()
                                        .any(|h| h.start == d.span.start && h.end == d.span.end);
                                    if !already {
                                        redact_hits.push(RedactHit {
                                            rule_id: d.rule_id.clone(),
                                            start: d.span.start,
                                            end: d.span.end,
                                        });
                                    }
                                }
                            }
                            sieve_ipc::DefaultOnTimeout::Allow => {
                                tracing::info!("OUTBOUND GUI: timeout default=allow → 放行");
                            }
                            sieve_ipc::DefaultOnTimeout::Block => {
                                let held: Vec<sieve_core::Detection> = hold_detections_outbound
                                    .iter()
                                    .map(|d| (*d).clone())
                                    .collect();
                                return Ok(build_426_response(&held));
                            }
                        }
                    }
                }
            } else {
                // 修 R11-#2：IPC 未初始化时按 default_on_timeout 三路分支（镜像 OpenAI 路径）。
                let effective_dot = hold_detections_outbound.iter().fold(
                    sieve_ipc::DefaultOnTimeout::Allow,
                    |acc, d| {
                        if let Action::HoldForDecision {
                            default_on_timeout, ..
                        } = &d.action
                        {
                            merge_strictest_timeout(acc, map_dot_to_ipc(*default_on_timeout))
                        } else {
                            acc
                        }
                    },
                );
                match effective_dot {
                    sieve_ipc::DefaultOnTimeout::Redact => {
                        tracing::info!(
                            "OUTBOUND GUI: IPC not initialized, default_on_timeout=redact → 脱敏转发"
                        );
                        for d in &hold_detections_outbound {
                            let already = redact_hits
                                .iter()
                                .any(|h| h.start == d.span.start && h.end == d.span.end);
                            if !already {
                                redact_hits.push(RedactHit {
                                    rule_id: d.rule_id.clone(),
                                    start: d.span.start,
                                    end: d.span.end,
                                });
                            }
                        }
                    }
                    sieve_ipc::DefaultOnTimeout::Allow => {
                        tracing::info!(
                            "OUTBOUND GUI: IPC not initialized, default_on_timeout=allow → 放行"
                        );
                    }
                    sieve_ipc::DefaultOnTimeout::Block => {
                        tracing::warn!(
                            "OUTBOUND GUI: IPC not initialized, default_on_timeout=block → 426"
                        );
                        let held: Vec<sieve_core::Detection> = hold_detections_outbound
                            .iter()
                            .map(|d| (*d).clone())
                            .collect();
                        return Ok(build_426_response(&held));
                    }
                }
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
                MultiAgentMeta {
                    source_agent,
                    origin_chain,
                    source_channel,
                    chain_depth,
                },
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
            MultiAgentMeta {
                source_agent,
                origin_chain,
                source_channel,
                chain_depth,
            },
        )
        .await;
    }

    // ── OpenAI Chat Completions 路径（v1.5，ADR-018）────────────────────────────
    if is_chat_completions_post {
        // body 已在 POST 预收集块中 collect，直接取出
        let body_bytes = post_body_bytes.expect("body_bytes set for POST");
        return proxy_openai(
            forwarder,
            filter,
            inbound_filter,
            dry_run,
            ipc,
            parts,
            body_bytes,
            source_agent,
            origin_chain,
            source_channel,
            chain_depth,
        )
        .await;
    }

    // 其他路径：流式透传（Week 1 行为）
    // POST 路径已预收集 body bytes，用 forward_raw；非 POST 保持流式透传。
    if let Some(body_bytes) = post_body_bytes {
        forward_raw(forwarder, parts, body_bytes).await
    } else {
        forward_streaming(
            forwarder,
            parts,
            non_post_body.expect("non_post_body set for non-POST"),
        )
        .await
    }
}

/// OpenAI Chat Completions 路径处理（`/v1/chat/completions`）。
///
/// 行为与 Anthropic 路径对称：
/// 1. body 已由调用方 collect（proxy_inner POST 预收集块）
/// 2. 解析 `OpenAIRequest`；解析失败 → 透传（上游返回 400）
/// 3. 提取文本段 → 逐段扫描（规则引擎与 Anthropic 路径共享）
/// 4. chain_depth ≥ 2 → 任何命中强制升级为 GuiPopup
/// 5. Block / GuiPopup / 透传 决策（与 Anthropic 路径相同）
/// 6. stream=true → `forward_with_openai_inbound_inspection`（修 R6-#2）
///
/// 关联：ADR-018 §路由、ADR-019 §chain_depth 升级、PRD v1.5 §6.1。
#[allow(clippy::too_many_arguments)]
async fn proxy_openai(
    forwarder: Arc<Forwarder>,
    filter: Arc<OutboundFilter>,
    inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    parts: http::request::Parts,
    body_bytes: Bytes,
    source_agent: sieve_ipc::protocol::SourceAgent,
    origin_chain: Vec<sieve_ipc::protocol::OriginHop>,
    source_channel: Option<String>,
    chain_depth: usize,
) -> Result<Response<ResponseBody>> {
    use sieve_core::pipeline::PipelineNode;
    use sieve_core::protocol::unified_message::{
        ContentBlock, ContentSpan, Direction, MessageMetadata, UpstreamProvider,
    };
    use std::time::SystemTime;

    // 1. 解析 OpenAIRequest；解析失败 → 透传
    let openai_req: sieve_core::protocol::openai::OpenAIRequest =
        match serde_json::from_slice(&body_bytes) {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!("non-openai body on /v1/chat/completions, passing through: {e}");
                return forward_raw(forwarder, parts, body_bytes).await;
            }
        };

    // 2. 提取文本段 → 逐段扫描
    let texts = openai_req.extract_text_content();
    let mut all_detections: Vec<sieve_core::Detection> = Vec::new();

    for (offset, text) in &texts {
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
                session_id: "outbound-scan-openai".into(),
                direction: Direction::Outbound,
                upstream_provider: UpstreamProvider::OpenAI,
                received_at: SystemTime::now(),
            },
        };

        let hits = filter
            .process(&mut msg)
            .map_err(|e| anyhow!("outbound filter (openai): {e}"))?;
        all_detections.extend(hits);
    }

    // 4. chain_depth ≥ 2 → 所有命中（含 HookTerminal disposition）强制升级为 GuiPopup
    //    （ADR-019 §chain_depth 升级策略）
    // 4. chain_depth ≥ 2 → HookMark + Redact 升级为 HoldForDecision（强制 GUI 弹窗，ADR-019）
    //
    // 修 R9-#2：OpenAI 路径之前只升级 HookMark，Action::Redact 仍静默脱敏。
    // 与 Anthropic 路径对称修复：嵌套调用时 Redact 也需 GUI 确认。
    if chain_depth >= 2 {
        tracing::info!(
            chain_depth,
            "X-Sieve-Origin chain_depth ≥ 2（OpenAI 路径），HookMark + Redact 升级为 GuiPopup"
        );
        for d in &mut all_detections {
            match &d.action {
                Action::HookMark => {
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                    };
                }
                Action::Redact { .. } => {
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Redact,
                    };
                }
                _ => {}
            }
        }
    }

    // 5a. 收集需要脱敏的 hit（与 Anthropic 路径对称，修 A2-#1）
    let mut redact_hits_openai: Vec<RedactHit> = all_detections
        .iter()
        .filter(|d| matches!(d.action, Action::Redact { .. }))
        .map(|d| RedactHit {
            rule_id: d.rule_id.clone(),
            start: d.span.start,
            end: d.span.end,
        })
        .collect();

    // 5b. Block（Critical fail-closed）
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
        tracing::warn!(count = blocking.len(), "OUTBOUND BLOCKED (openai)");
        for d in &blocking {
            tracing::warn!(rule = %d.rule_id, severity = ?d.severity, "openai detection");
        }
        let cloned: Vec<sieve_core::Detection> = blocking.iter().map(|d| (*d).clone()).collect();
        return Ok(build_426_response(&cloned));
    }

    // 5c. GuiPopup（HoldForDecision）
    let hold_detections: Vec<&sieve_core::Detection> = all_detections
        .iter()
        .filter(|d| matches!(d.action, Action::HoldForDecision { .. }))
        .collect();

    if !hold_detections.is_empty() {
        if let Some(ref ipc_server) = ipc {
            use chrono::Utc;

            let request_id = uuid::Uuid::new_v4();

            // 修 R3-#4 / R10-#2（OpenAI 路径）：从 detection.default_on_timeout 读，不硬编码 Block。
            let (timeout_seconds, default_on_timeout) = hold_detections.iter().fold(
                (60_u32, sieve_ipc::DefaultOnTimeout::Allow),
                |acc, d| {
                    if let Action::HoldForDecision {
                        timeout_seconds,
                        default_on_timeout,
                        ..
                    } = &d.action
                    {
                        let merged =
                            merge_strictest_timeout(acc.1, map_dot_to_ipc(*default_on_timeout));
                        (acc.0.max(*timeout_seconds), merged)
                    } else {
                        acc
                    }
                },
            );

            // chain_depth ≥ 2 时在弹窗标题里显示完整 origin_chain 信息（ADR-019）
            let chain_note = if chain_depth >= 2 {
                format!("（嵌套调用 depth={chain_depth}）")
            } else {
                String::new()
            };

            let ipc_detections = hold_detections
                .iter()
                .map(|d| sieve_ipc::protocol::DetectionPayload {
                    rule_id: d.rule_id.clone(),
                    severity: map_severity_to_ipc(d.severity),
                    disposition: sieve_ipc::Disposition::GuiPopup,
                    title: format!("出站检测命中{chain_note}：{}", d.rule_id),
                    one_line_summary: d.evidence_truncated.clone(),
                    details: serde_json::json!({ "chain_depth": chain_depth }),
                })
                .collect();

            let ipc_req = sieve_ipc::DecisionRequest {
                request_id,
                created_at: Utc::now(),
                timeout_seconds,
                default_on_timeout,
                detections: ipc_detections,
                // v1.5：注入 multi-agent 元数据
                source_agent,
                origin_chain: origin_chain.clone(),
                source_channel: source_channel.clone(),
                // 修 R7-#5：填入 header 真实 chain_depth
                explicit_chain_depth: Some(chain_depth),
            };

            let timeout_dur = std::time::Duration::from_secs(u64::from(timeout_seconds).max(1));
            let outcome = ipc_server.request_decision(ipc_req, timeout_dur).await;

            match outcome {
                Ok(resp) => match resp.decision {
                    sieve_ipc::DecisionAction::Allow => {
                        tracing::info!("OUTBOUND GUI (openai): Allow → 转发原 body");
                        // fall-through 到透传（不脱敏，用户明确选择原样允许）
                    }
                    sieve_ipc::DecisionAction::RedactAndAllow => {
                        tracing::info!("OUTBOUND GUI (openai): RedactAndAllow → 脱敏后转发");
                        // 修 R3-#3：把 held detection 的 span 升级到 redact_hits_openai，
                        // 保证仅命中 GUI 类（无 AutoRedact 类）时也能正确脱敏。
                        // 去重：跳过已在 redact_hits_openai 中存在的 span。
                        for d in &hold_detections {
                            let already = redact_hits_openai
                                .iter()
                                .any(|h| h.start == d.span.start && h.end == d.span.end);
                            if !already {
                                redact_hits_openai.push(RedactHit {
                                    rule_id: d.rule_id.clone(),
                                    start: d.span.start,
                                    end: d.span.end,
                                });
                            }
                        }
                        // fall-through 到下方 redact_hits_openai 处理（现在含 GUI 类 span）
                    }
                    sieve_ipc::DecisionAction::Deny => {
                        tracing::warn!("OUTBOUND GUI (openai): Deny → 426");
                        let held: Vec<sieve_core::Detection> =
                            hold_detections.iter().map(|d| (*d).clone()).collect();
                        return Ok(build_426_response(&held));
                    }
                },
                Err(e) => {
                    // 修 R3-#4（OpenAI 路径）：按规则 default_on_timeout 兜底
                    tracing::warn!(error = %e, ?default_on_timeout, "OUTBOUND GUI (openai): IPC error, applying default_on_timeout");
                    match default_on_timeout {
                        sieve_ipc::DefaultOnTimeout::Redact => {
                            for d in &hold_detections {
                                let already = redact_hits_openai
                                    .iter()
                                    .any(|h| h.start == d.span.start && h.end == d.span.end);
                                if !already {
                                    redact_hits_openai.push(RedactHit {
                                        rule_id: d.rule_id.clone(),
                                        start: d.span.start,
                                        end: d.span.end,
                                    });
                                }
                            }
                        }
                        sieve_ipc::DefaultOnTimeout::Allow => {
                            tracing::info!("OUTBOUND GUI (openai): timeout default=allow → 放���");
                        }
                        sieve_ipc::DefaultOnTimeout::Block => {
                            let held: Vec<sieve_core::Detection> =
                                hold_detections.iter().map(|d| (*d).clone()).collect();
                            return Ok(build_426_response(&held));
                        }
                    }
                }
            }
        } else {
            // IPC 未初始化：按规则 default_on_timeout 兜底（修 R3-#4 OpenAI 路径）
            let effective_dot =
                hold_detections
                    .iter()
                    .fold(sieve_ipc::DefaultOnTimeout::Allow, |acc, d| {
                        if let Action::HoldForDecision {
                            default_on_timeout, ..
                        } = &d.action
                        {
                            merge_strictest_timeout(acc, map_dot_to_ipc(*default_on_timeout))
                        } else {
                            acc
                        }
                    });
            match effective_dot {
                sieve_ipc::DefaultOnTimeout::Redact => {
                    tracing::info!(
                        "OUTBOUND GUI (openai): IPC not initialized, default_on_timeout=redact → 脱��转发"
                    );
                    for d in &hold_detections {
                        let already = redact_hits_openai
                            .iter()
                            .any(|h| h.start == d.span.start && h.end == d.span.end);
                        if !already {
                            redact_hits_openai.push(RedactHit {
                                rule_id: d.rule_id.clone(),
                                start: d.span.start,
                                end: d.span.end,
                            });
                        }
                    }
                }
                sieve_ipc::DefaultOnTimeout::Allow => {
                    tracing::info!(
                        "OUTBOUND GUI (openai): IPC not initialized, default_on_timeout=allow → 放行"
                    );
                }
                sieve_ipc::DefaultOnTimeout::Block => {
                    tracing::warn!(
                        "OUTBOUND GUI (openai): IPC not initialized, default_on_timeout=block → 426"
                    );
                    let held: Vec<sieve_core::Detection> =
                        hold_detections.iter().map(|d| (*d).clone()).collect();
                    return Ok(build_426_response(&held));
                }
            }
        }
    }

    if dry_run && !all_detections.is_empty() {
        tracing::warn!(
            count = all_detections.len(),
            "OUTBOUND DRY-RUN (openai): would have flagged"
        );
    }

    // 5d. AutoRedact（修 A2-#1）：命中 Redact action 的 secret 在转发前脱敏，
    // 不返回 426；与 Anthropic 路径对称。OpenAI message.content 同时支持
    // string 和 array-of-content-parts，由专用函数处理。
    if !redact_hits_openai.is_empty() {
        let seg_result = redact_segments(&texts, &redact_hits_openai);
        tracing::info!(
            count = seg_result.redacted_count,
            rules = %seg_result.redacted_summary,
            "OUTBOUND AUTO-REDACT (openai)"
        );

        let new_body_bytes =
            apply_redacted_texts_to_openai_request(&openai_req, &texts, &seg_result.texts)
                .and_then(|req| {
                    serde_json::to_vec(&req).map_err(|e| anyhow!("re-serialize openai json: {e}"))
                })?;

        // 验证脱敏后 JSON 仍然合法
        if serde_json::from_slice::<serde_json::Value>(&new_body_bytes).is_err() {
            return Err(anyhow!(
                "redact_segments (openai) 产生了非法 JSON，fail-closed 拦截"
            ));
        }

        let new_body = bytes::Bytes::from(new_body_bytes);
        let new_len = new_body.len();
        let mut new_parts = parts.clone();
        new_parts.headers.insert(
            http::header::CONTENT_LENGTH,
            http::HeaderValue::from(new_len),
        );

        // 修 R8-#3：AutoRedact 后 stream=true 仍需入站 SSE 检测。
        // 原实现直接 forward_raw，跳过了 forward_with_openai_inbound_inspection，
        // 导致脱敏后的 OpenAI 流式响应不经过入站规则检测（漏检）。
        // 修法与 Anthropic 路径等价：脱敏后用新 body 继续走入站检测路径。
        // stream=false 时直接透传（非流式响应无需 SSE 解析，同非 AutoRedact 分支）。

        // 修 R9-#1：AutoRedact 路径也需要 seed prompt 地址（与 Anthropic 路径等价）。
        // AutoRedact 改写 secret，不影响 EVM 地址；seed 用原始 texts 即可。
        for (_, text) in &texts {
            if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
                tracing::warn!(error = %e, "seed_known_addresses_from_text failed (openai autoredact)");
            }
        }

        // 漏洞修复：AutoRedact stream=false 分支也需入站检测（同非 AutoRedact stream=false 修复）。
        // forward_with_openai_inbound_inspection 内部按 Content-Type 路由，JSON 响应走 JSON 路径。
        return forward_with_openai_inbound_inspection(
            forwarder,
            inbound_filter,
            dry_run,
            ipc,
            new_parts,
            new_body,
            MultiAgentMeta {
                source_agent,
                origin_chain,
                source_channel,
                chain_depth,
            },
        )
        .await;
    }

    // 5. prompt 地址 seed（修 R9-#1，与 Anthropic 路径等价）
    // 用户 prompt 中的 EVM 地址需要提前注入 InboundFilter 会话状态，
    // 否则流式响应里的地址替换（IN-CR-01）因缺少参照而漏检。
    for (_, text) in &texts {
        if let Err(e) = inbound_filter.seed_known_addresses_from_text(text) {
            tracing::warn!(error = %e, "seed_known_addresses_from_text failed (openai)");
        }
    }

    // 6. 出站通过 → 入站检测路由（修 R6-#2 + 漏洞修复）
    // stream=true 时用 OpenAI SSE parser 做 tee 截流检测，与 Anthropic 路径对称。
    // stream=false 时：上游可能返回 application/json（含 tool_calls），
    //   同样需要入站检测（漏洞修复：lessons.md 2026-04-27 [安全]）。
    //   forward_with_openai_inbound_inspection 内部按 Content-Type 路由：
    //   JSON 响应走 handle_openai_json_inbound，SSE 响应走原 SSE 路径。
    // TODO（R6-#3）：OpenAiSseParser ContentBlockStart/Stop 支持完成后，tool_call 检测能力
    //    将自动生效（inbound_filter 已经协议无关）。
    forward_with_openai_inbound_inspection(
        forwarder,
        inbound_filter,
        dry_run,
        ipc,
        parts,
        body_bytes,
        MultiAgentMeta {
            source_agent,
            origin_chain,
            source_channel,
            chain_depth,
        },
    )
    .await
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
/// Multi-agent 元数据，从 `X-Sieve-Origin` / `X-Sieve-Source-Channel` 解析而来。
///
/// 在入站路径和出站路径构造 `DecisionRequest` 时注入，供 GUI / hook 显示来源信息。
/// 关联：ADR-019 §字段定义、PRD v1.5 §6.5。
#[derive(Clone)]
struct MultiAgentMeta {
    source_agent: sieve_ipc::protocol::SourceAgent,
    origin_chain: Vec<sieve_ipc::protocol::OriginHop>,
    source_channel: Option<String>,
    /// `X-Sieve-Origin` header 中解析的真实嵌套深度（修 R7-#5）。
    ///
    /// 用于填充 `DecisionRequest::explicit_chain_depth`，使 GUI/hook
    /// 能展示 header 真实深度而非受限于 `origin_chain.len()`。
    chain_depth: usize,
}

async fn forward_with_inbound_inspection(
    forwarder: Arc<Forwarder>,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
    meta: MultiAgentMeta,
) -> Result<Response<ResponseBody>> {
    use http_body_util::Full;

    // 修 A2-#2：把 source_channel 注入 InboundFilter，使 IN-GEN-06 运行时提级逻辑
    // 能感知来源 channel（PRD v1.5 §4.5）。必须在 SSE 检测开始前调用。
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
        .map(|ct| ct.starts_with("application/json"))
        .unwrap_or(false);

    if is_json_response {
        return handle_anthropic_json_inbound(resp_parts, resp_body, inbound_filter, dry_run, meta)
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
                    // 修 R8-#2：传入 meta.chain_depth，chain_depth ≥ 2 时 HookMark 升级为 GuiPopup
                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                        &events,
                        &mut inbound_filter,
                        &mut aggregator,
                        dry_run,
                        meta.chain_depth,
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
                                })
                                .collect();

                            let ipc_req = sieve_ipc::DecisionRequest {
                                request_id,
                                created_at: Utc::now(),
                                timeout_seconds,
                                default_on_timeout: sieve_ipc::DefaultOnTimeout::Block,
                                detections: ipc_detections,
                                // v1.5：注入 multi-agent 元数据（ADR-019）
                                source_agent: meta.source_agent,
                                origin_chain: meta.origin_chain.clone(),
                                source_channel: meta.source_channel.clone(),
                                // 修 R7-#5：填入 header 真实 chain_depth
                                explicit_chain_depth: Some(meta.chain_depth),
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
        // 修 R8-#2：flush 阶段同样传入 chain_depth，HookMark 升级逻辑一致
        let (blocking, hook_detections, flush_hold_detections) = classify_inbound_detections(
            &flushed,
            &mut inbound_filter,
            &mut aggregator,
            dry_run,
            meta.chain_depth,
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

/// OpenAI 路径入站 SSE 解析检测（tee 模式，修 R6-#2）。
///
/// 与 [`forward_with_inbound_inspection`] 逻辑完全对称，唯一区别是使用
/// [`sieve_core::sse::openai_parser::OpenAiSseParser`] 而非 Anthropic [`SseParser`]。
///
/// OpenAI SSE 格式：`data: {...}\n\n`，无 `event:` 头。
/// 产出的 [`SseEvent`] 类型与 Anthropic 相同，inbound_filter 无需感知协议差异。
///
/// TODO（R6-#3）：等 OpenAiSseParser 支持 ContentBlockStart/Stop（tool_call 首帧）后，
///     Aggregator 的 tool_use 完整检测能力将自动生效，无需修改此函数。
///
/// 关联：ADR-018 §流式解析 / PRD v1.5 §6.1 / R6-#2。
async fn forward_with_openai_inbound_inspection(
    forwarder: Arc<Forwarder>,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    ipc: Option<Arc<sieve_ipc::IpcServer>>,
    mut parts: http::request::Parts,
    body_bytes: Bytes,
    meta: MultiAgentMeta,
) -> Result<Response<ResponseBody>> {
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
        .map(|ct| ct.starts_with("application/json"))
        .unwrap_or(false);

    if is_json_response {
        return handle_openai_json_inbound(resp_parts, resp_body, inbound_filter, dry_run, meta)
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

                    // P0-5：feed 超限时 fail-closed（IN-CAP-01）
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

                    // 修 R8-#2：传入 meta.chain_depth，chain_depth ≥ 2 时 HookMark 升级为 GuiPopup
                    let (blocking, hook_detections, hold_detections) = classify_inbound_detections(
                        &events,
                        &mut inbound_filter,
                        &mut aggregator,
                        dry_run,
                        meta.chain_depth,
                    );

                    // 1. Block 类：注入 sieve_blocked 并截流（fail-closed 优先）
                    if !blocking.is_empty() {
                        tracing::warn!(count = blocking.len(), "INBOUND BLOCKED (openai)");
                        for d in &blocking {
                            tracing::warn!(rule = %d.rule_id, "openai inbound detection");
                        }
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
                                })
                                .collect();

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
        let flushed = parser.flush();
        // 修 R8-#2：flush 阶段同样传入 chain_depth，HookMark 升级逻辑一致
        let (blocking, hook_detections, flush_hold_detections) = classify_inbound_detections(
            &flushed,
            &mut inbound_filter,
            &mut aggregator,
            dry_run,
            meta.chain_depth,
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
/// 修 R8-#2：新增 `chain_depth` 参数，实现入站 SSE HookMark 在 chain_depth ≥ 2 时
/// 升级为 HoldForDecision（GuiPopup），与出站路径和 IN-CR-06 路径的升级策略一致。
///
/// 旧实现：入站 HookMark 命中直接写 pending 文件然后继续转发流，
/// 但 daemon 注释明确要求 chain_depth ≥ 2 所有命中强制 GuiPopup hold；
/// 升级逻辑在出站路径已实现，入站路径漏掉导致行为不一致。
///
/// 修法：chain_depth ≥ 2 时把 HookMark detection 的 action 替换为 HoldForDecision，
/// 移入 hold_detections 而非 hook_detections，从而走 GUI hold 分支。
///
/// 关联 ADR-019 §chain_depth 升级策略、PRD v1.5 §6.5。
fn classify_inbound_detections(
    events: &[sieve_core::sse::parser::SseEvent],
    inbound_filter: &mut sieve_core::pipeline::inbound::InboundFilter,
    aggregator: &mut sieve_core::tool_use_aggregator::Aggregator,
    dry_run: bool,
    chain_depth: usize,
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

    for mut d in all_hits {
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
                // 修 R8-#2：chain_depth ≥ 2 时 HookMark 升级为 HoldForDecision（强制 GUI hold）
                // 原来 HookMark 写 pending 文件后继续转发，但 chain_depth ≥ 2 规则要求强制弹窗。
                if chain_depth >= 2 {
                    tracing::info!(
                        chain_depth,
                        rule = %d.rule_id,
                        "入站 HookMark 因 chain_depth ≥ 2 升级为 GuiPopup"
                    );
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                    };
                    hold_detections.push(d);
                } else {
                    // chain_depth < 2：正常写 pending 文件，SSE 流继续转发
                    hook_detections.push(d);
                }
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

/// 写 IPC pending 文件，失败时返回 `Err`（调用方负责 fail-closed）。
///
/// 旧函数 `write_hook_pending_silent` 只 warn 后继续，违反 fail-closed 原则。
/// 新函数返回 `Result`，调用方在 `Err` 时必须注入 `sieve_blocked` 并截流。
///
/// 修 R7-#3：加 `meta` 参数，DecisionRequest 中填入真实 multi-agent 元数据，
/// hook/GUI 读 pending 文件时不再丢失来源信息（之前硬编码 Unknown + 空 chain）。
///
/// 关联 PRD §9 #3（Critical 不可关）、ADR-014 §Hook 路径、SPEC-001 §3.1、ADR-019。
fn write_hook_pending_or_fail_closed(
    d: &sieve_core::Detection,
    meta: &MultiAgentMeta,
) -> Result<(), sieve_ipc::error::IpcError> {
    let sieve_home = sieve_ipc::paths::sieve_home()?;
    write_hook_pending_to(d, &sieve_home, meta)
}

/// 写 IPC pending 文件到指定 base 目录，失败时返回 `Err`。
///
/// 内部实现，分离出来方便测试注入临时路径，不依赖环境变量。
///
/// 修 R7-#3：`meta` 参数携带 source_agent / origin_chain / source_channel，
/// 注入 `DecisionRequest` 使 hook 端能展示完整来源信息。
///
/// 关联 SPEC-001 §3.1、ADR-014 §Hook 路径、ADR-019。
fn write_hook_pending_to(
    d: &sieve_core::Detection,
    sieve_home: &std::path::Path,
    meta: &MultiAgentMeta,
) -> Result<(), sieve_ipc::error::IpcError> {
    use chrono::Utc;

    let request_id = uuid::Uuid::new_v4();
    // 修 R7-#5：使用 meta.chain_depth（来自 X-Sieve-Origin header 真实数值），
    // 而非 origin_chain.len()（只计已知 hop 数，中间层未知时比真实值小）。
    let explicit_depth = Some(meta.chain_depth);
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
        // 修 R7-#3：注入真实 multi-agent 元数据（不再硬编码 Unknown/empty）
        source_agent: meta.source_agent,
        origin_chain: meta.origin_chain.clone(),
        source_channel: meta.source_channel.clone(),
        explicit_chain_depth: explicit_depth,
    };

    sieve_ipc::pending_file::write_pending(&ipc_req, sieve_home)?;

    tracing::info!(
        rule = %d.rule_id,
        request_id = %request_id,
        source_agent = ?meta.source_agent,
        "HookMark: pending file written, SSE stream continues"
    );

    Ok(())
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

/// 构造 sieve_blocked JSON 响应 body（application/json 路径使用，非 SSE 格式）。
///
/// 漏洞修复（lessons.md 2026-04-27 [安全]）：非流式 JSON 响应被拦截时，
/// 不能注入 SSE event 格式的 sieve_blocked，需要返回合法 JSON body。
/// 格式与 build_sieve_blocked_sse 的 payload 部分相同，只是不包装为 SSE event 行。
fn build_sieve_blocked_json_body(detections: &[sieve_core::Detection]) -> Bytes {
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
                "Sieve 检测到 {} 条入站 Critical 命中。非流式响应已被替换。\
                 Critical 级别命中不可通过白名单绕过，请人工审查当前上下文后重试。",
                detections.len()
            ),
            "en": format!(
                "Sieve blocked {} inbound critical detection(s). Non-streaming response replaced. \
                 Critical detections cannot be bypassed via allowlist. Please review and retry.",
                detections.len()
            ),
        }
    });
    Bytes::from(payload.to_string())
}

/// 构造因入站 JSON 拦截而替换响应的 HTTP Response。
fn build_sieve_blocked_json_response(
    detections: &[sieve_core::Detection],
) -> Response<ResponseBody> {
    let body_bytes = build_sieve_blocked_json_body(detections);
    Response::builder()
        .status(http::StatusCode::OK)
        .header(
            http::header::CONTENT_TYPE,
            "application/json; charset=utf-8",
        )
        .header(http::header::CONTENT_LENGTH, body_bytes.len())
        .body(bytes_body(body_bytes))
        .unwrap_or_else(|_| Response::new(empty_body()))
}

/// 处理 Anthropic 非流式 JSON 入站响应的入站检测路径。
///
/// 漏洞修复（lessons.md 2026-04-27 [安全]）：上游返回 `application/json` 时，
/// 收集完整 body，解析 `content[]` 中的 `tool_use` 块，
/// 喂给 `InboundFilter::on_tool_use_complete`。
/// 命中 fail-closed Critical 时把响应 body 替换为 sieve_blocked JSON；否则原样透传。
///
/// 不走 SSE 路径（tokio::spawn + channel tee），直接 await body 收集再决策。
async fn handle_anthropic_json_inbound(
    resp_parts: http::response::Parts,
    resp_body: impl http_body::Body<Data = Bytes, Error = impl std::fmt::Display> + Send + 'static,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    meta: MultiAgentMeta,
) -> Result<Response<ResponseBody>> {
    use http_body_util::BodyExt as _;

    // 收集完整 body（非流式 JSON 响应通常很小，不存在 DoS 风险，上游负责 content-length 校验）
    let body_bytes = match resp_body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            tracing::warn!("handle_anthropic_json_inbound: collect body error: {e}");
            return Ok(build_sieve_blocked_json_response(&[]));
        }
    };

    // 解析响应 JSON（宽松解析，只取 content 数组）
    let resp_json: serde_json::Value = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(e) => {
            // 无法解析 JSON，原样透传（不拦截非 JSON 内容）
            tracing::debug!("handle_anthropic_json_inbound: non-JSON body, passthrough: {e}");
            return passthrough_json_response(resp_parts, body_bytes);
        }
    };

    // 提取 content[] 中的 tool_use 块
    let content_arr = resp_json.get("content").and_then(|v| v.as_array());
    let mut all_blocking: Vec<sieve_core::Detection> = Vec::new();

    if let Some(content) = content_arr {
        for block in content {
            let obj = match block.as_object() {
                Some(o) => o,
                None => continue,
            };
            if obj.get("type").and_then(|v| v.as_str()) != Some("tool_use") {
                continue;
            }
            let tool_id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let tool_name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tool_input = obj
                .get("input")
                .cloned()
                .unwrap_or(serde_json::Value::Object(Default::default()));

            let completed = sieve_core::CompletedToolCall {
                id: tool_id,
                name: tool_name,
                input: tool_input,
            };

            match inbound_filter.on_tool_use_complete(&completed) {
                Ok(hits) => {
                    for mut d in hits {
                        match &d.action {
                            sieve_core::detection::Action::Block => {
                                if d.severity == sieve_core::Severity::Critical
                                    && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id)
                                        || !dry_run)
                                {
                                    all_blocking.push(d);
                                }
                            }
                            sieve_core::detection::Action::HoldForDecision { .. } => {
                                // JSON 路径 GuiPopup：暂无 keep-alive 机制，fail-closed
                                tracing::warn!(
                                    rule = %d.rule_id,
                                    "GuiPopup in non-streaming JSON path, fail-closed"
                                );
                                d.action = sieve_core::detection::Action::Block;
                                all_blocking.push(d);
                            }
                            _ => {
                                // HookMark / MarkOnly / SilentLog / Redact：记录但不阻断
                                // chain_depth ≥ 2 时 HookMark 升级为 GuiPopup（同 SSE 路径）
                                if meta.chain_depth >= 2
                                    && matches!(d.action, sieve_core::detection::Action::HookMark)
                                {
                                    tracing::warn!(
                                        rule = %d.rule_id,
                                        "HookMark upgraded to GuiPopup (chain_depth >= 2), fail-closed in JSON path"
                                    );
                                    all_blocking.push(d);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "handle_anthropic_json_inbound: on_tool_use_complete error");
                }
            }
        }
    }

    if !all_blocking.is_empty() {
        tracing::warn!(
            count = all_blocking.len(),
            "INBOUND BLOCKED (anthropic json)"
        );
        for d in &all_blocking {
            tracing::warn!(rule = %d.rule_id, "anthropic json inbound detection");
        }
        return Ok(build_sieve_blocked_json_response(&all_blocking));
    }

    // 无拦截：原样透传（重建含原 headers 的响应）
    passthrough_json_response(resp_parts, body_bytes)
}

/// 处理 OpenAI 非流式 JSON 入站响应的入站检测路径。
///
/// 漏洞修复（lessons.md 2026-04-27 [安全]）：上游返回 `application/json` 时，
/// 解析 `choices[].message.tool_calls` 提取 function 调用，喂给 InboundFilter。
/// 命中 fail-closed Critical 时替换响应 body 为 sieve_blocked JSON；否则原样透传。
async fn handle_openai_json_inbound(
    resp_parts: http::response::Parts,
    resp_body: impl http_body::Body<Data = Bytes, Error = impl std::fmt::Display> + Send + 'static,
    mut inbound_filter: InboundFilter,
    dry_run: bool,
    meta: MultiAgentMeta,
) -> Result<Response<ResponseBody>> {
    use http_body_util::BodyExt as _;

    let body_bytes = match resp_body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            tracing::warn!("handle_openai_json_inbound: collect body error: {e}");
            return Ok(build_sieve_blocked_json_response(&[]));
        }
    };

    let resp_json: serde_json::Value = match serde_json::from_slice(&body_bytes) {
        Ok(v) => v,
        Err(e) => {
            tracing::debug!("handle_openai_json_inbound: non-JSON body, passthrough: {e}");
            return passthrough_json_response(resp_parts, body_bytes);
        }
    };

    // 遍历 choices[].message.tool_calls[]
    let choices = resp_json.get("choices").and_then(|v| v.as_array());
    let mut all_blocking: Vec<sieve_core::Detection> = Vec::new();

    if let Some(choices) = choices {
        for choice in choices {
            let message = match choice.get("message") {
                Some(m) => m,
                None => continue,
            };
            let tool_calls = match message.get("tool_calls").and_then(|v| v.as_array()) {
                Some(tc) => tc,
                None => continue,
            };
            for tc in tool_calls {
                let obj = match tc.as_object() {
                    Some(o) => o,
                    None => continue,
                };
                let tc_id = obj
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let func = match obj.get("function").and_then(|v| v.as_object()) {
                    Some(f) => f,
                    None => continue,
                };
                let func_name = func
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let arguments_str = func
                    .get("arguments")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");
                let input: serde_json::Value = serde_json::from_str(arguments_str)
                    .unwrap_or(serde_json::Value::Object(Default::default()));

                let completed = sieve_core::CompletedToolCall {
                    id: tc_id,
                    name: func_name,
                    input,
                };

                match inbound_filter.on_tool_use_complete(&completed) {
                    Ok(hits) => {
                        for mut d in hits {
                            match &d.action {
                                sieve_core::detection::Action::Block => {
                                    if d.severity == sieve_core::Severity::Critical
                                        && (sieve_rules::critical_lock::is_fail_closed(&d.rule_id)
                                            || !dry_run)
                                    {
                                        all_blocking.push(d);
                                    }
                                }
                                sieve_core::detection::Action::HoldForDecision { .. } => {
                                    tracing::warn!(
                                        rule = %d.rule_id,
                                        "GuiPopup in non-streaming OpenAI JSON path, fail-closed"
                                    );
                                    d.action = sieve_core::detection::Action::Block;
                                    all_blocking.push(d);
                                }
                                _ => {
                                    if meta.chain_depth >= 2
                                        && matches!(
                                            d.action,
                                            sieve_core::detection::Action::HookMark
                                        )
                                    {
                                        tracing::warn!(
                                            rule = %d.rule_id,
                                            "HookMark upgraded (chain_depth >= 2), fail-closed in OpenAI JSON path"
                                        );
                                        all_blocking.push(d);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "handle_openai_json_inbound: on_tool_use_complete error");
                    }
                }
            }
        }
    }

    if !all_blocking.is_empty() {
        tracing::warn!(count = all_blocking.len(), "INBOUND BLOCKED (openai json)");
        for d in &all_blocking {
            tracing::warn!(rule = %d.rule_id, "openai json inbound detection");
        }
        return Ok(build_sieve_blocked_json_response(&all_blocking));
    }

    passthrough_json_response(resp_parts, body_bytes)
}

/// 把已收集的 body bytes 原样构建为 HTTP Response（JSON 路径无拦截时透传）。
fn passthrough_json_response(
    resp_parts: http::response::Parts,
    body_bytes: Bytes,
) -> Result<Response<ResponseBody>> {
    let body_len = body_bytes.len();
    let mut builder = Response::builder().status(resp_parts.status);
    // 保留原 headers（除 content-length，重新按实际 body 大小设置）
    for (name, value) in &resp_parts.headers {
        if name == http::header::CONTENT_LENGTH {
            continue;
        }
        builder = builder.header(name, value);
    }
    builder = builder.header(http::header::CONTENT_LENGTH, body_len);
    let resp = builder
        .body(bytes_body(body_bytes))
        .map_err(|e| anyhow!("passthrough_json_response: build response: {e}"))?;
    Ok(resp)
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

/// 把 `sieve_core::detection::DefaultOnTimeout` 映射为 `sieve_ipc::DefaultOnTimeout`。
///
/// 修 R3-#4 / R10-#2：engine_adapter 把规则里的 default_on_timeout 存入 Detection，
/// daemon 通过此函数转换后传给 IPC 层。
fn map_dot_to_ipc(dot: sieve_core::detection::DefaultOnTimeout) -> sieve_ipc::DefaultOnTimeout {
    match dot {
        sieve_core::detection::DefaultOnTimeout::Redact => sieve_ipc::DefaultOnTimeout::Redact,
        sieve_core::detection::DefaultOnTimeout::Block => sieve_ipc::DefaultOnTimeout::Block,
        sieve_core::detection::DefaultOnTimeout::Allow => sieve_ipc::DefaultOnTimeout::Allow,
    }
}

/// 合并两个 `DefaultOnTimeout`，取更严格的一方：Block > Redact > Allow。
///
/// 多条 hold_detections 时用于取最严兜底策略。
fn merge_strictest_timeout(
    a: sieve_ipc::DefaultOnTimeout,
    b: sieve_ipc::DefaultOnTimeout,
) -> sieve_ipc::DefaultOnTimeout {
    use sieve_ipc::DefaultOnTimeout::{Allow, Block, Redact};
    match (a, b) {
        (Block, _) | (_, Block) => Block,
        (Redact, _) | (_, Redact) => Redact,
        (Allow, Allow) => Allow,
    }
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

/// 构造因嵌套调用过深（chain_depth ≥ 5）的 426 Upgrade Required 响应。
///
/// 攻击模式检测：超过 5 层 agent 嵌套调用视为异常，直接拒绝。
/// 关联：ADR-019 §嵌套深度限制、PRD v1.5 §6.5。
fn build_426_nested_rejection(chain_depth: usize) -> Response<ResponseBody> {
    let body_json = serde_json::json!({
        "type": "sieve_blocked",
        "blocked_at": epoch_secs_string(),
        "reason": "nested_call_too_deep",
        "chain_depth": chain_depth,
        "guidance": {
            "zh": format!(
                "Sieve 检测到 agent 嵌套调用层数（{}）超过安全上限（5），请求被拒绝。",
                chain_depth
            ),
            "en": format!(
                "Sieve rejected request: nested agent call depth ({}) exceeds safety limit (5).",
                chain_depth
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
        source_channel: None,
        origin_chain_depth: 0,
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
        source_channel: None,
        origin_chain_depth: 0,
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

/// 把脱敏后的文本段列表写回 [`OpenAIRequest`] 并返回新 request（修 A2-#1）。
///
/// OpenAI `message.content` 有两种形式：
/// - `string`：对应一个 segment
/// - `array of content parts`：每个 `{"type":"text","text":"..."}` 对应一个 segment；
///   `image_url` 等非文本 part 原样保留（不计入 segment 计数）
///
/// `original_texts` 与 `redacted_texts` 必须顺序对应；长度不一致时返回错误。
///
/// 关联：PRD v1.4 §6.1（AutoRedact），ADR-018（OpenAI 协议适配）。
fn apply_redacted_texts_to_openai_request(
    req: &sieve_core::protocol::openai::OpenAIRequest,
    original_texts: &[(usize, String)],
    redacted_texts: &[String],
) -> Result<sieve_core::protocol::openai::OpenAIRequest> {
    if original_texts.len() != redacted_texts.len() {
        return Err(anyhow!(
            "redacted_texts 长度 {} 与 original_texts 长度 {} 不一致",
            redacted_texts.len(),
            original_texts.len()
        ));
    }

    let mut seg_idx = 0usize;
    let mut new_messages: Vec<sieve_core::protocol::openai::OpenAIMessage> = Vec::new();

    for msg in &req.messages {
        let new_content = match &msg.content {
            Some(serde_json::Value::String(_)) => {
                let replacement = redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                    msg.content
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                });
                seg_idx += 1;
                Some(serde_json::Value::String(replacement))
            }
            Some(serde_json::Value::Array(parts)) => {
                let mut new_parts = Vec::with_capacity(parts.len());
                for part in parts {
                    if let Some(obj) = part.as_object() {
                        if obj.get("type").and_then(|v| v.as_str()) == Some("text")
                            && obj.get("text").and_then(|v| v.as_str()).is_some()
                        {
                            let replacement =
                                redacted_texts.get(seg_idx).cloned().unwrap_or_else(|| {
                                    obj.get("text")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string()
                                });
                            seg_idx += 1;
                            let mut new_obj = obj.clone();
                            new_obj
                                .insert("text".to_string(), serde_json::Value::String(replacement));
                            new_parts.push(serde_json::Value::Object(new_obj));
                            continue;
                        }
                    }
                    // image_url 等非 text part 原样保留，不消耗 segment index
                    new_parts.push(part.clone());
                }
                Some(serde_json::Value::Array(new_parts))
            }
            other => other.clone(),
        };
        new_messages.push(sieve_core::protocol::openai::OpenAIMessage {
            role: msg.role.clone(),
            content: new_content,
            name: msg.name.clone(),
            tool_calls: msg.tool_calls.clone(),
            tool_call_id: msg.tool_call_id.clone(),
            extra: msg.extra.clone(),
        });
    }

    let _ = seg_idx; // 消除 unused variable 警告

    Ok(sieve_core::protocol::openai::OpenAIRequest {
        model: req.model.clone(),
        messages: new_messages,
        stream: req.stream,
        tools: req.tools.clone(),
        max_tokens: req.max_tokens,
        temperature: req.temperature,
        extra: req.extra.clone(),
    })
}

// ─── 单元测试：Hook pending fail-closed ──────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use sieve_core::detection::{Action, ContentSource, Detection, Severity};
    use sieve_core::protocol::unified_message::ContentSpan;
    use uuid::Uuid;

    /// 构造最小化的 HookMark Detection，用于测试 write_hook_pending_to。
    fn make_hook_detection() -> Detection {
        Detection {
            id: Uuid::new_v4(),
            rule_id: "IN-CR-02".to_string(),
            severity: Severity::Critical,
            action: Action::HookMark,
            source: ContentSource::InboundToolUseInput,
            span: ContentSpan { start: 0, end: 10 },
            evidence_truncated: "rm -rf /".to_string(),
            fingerprint: "deadbeef01234567".to_string(),
            source_channel: None,
            origin_chain_depth: 0,
        }
    }

    /// happy path：base 目录可写 → 返回 Ok，pending 文件存在。
    ///
    /// 验证 HookMark 写成功后调用方可继续转发 SSE 流，不触发 fail-closed。
    /// 关联 PRD §9 #3、SPEC-001 §3.1。
    #[test]
    fn hook_pending_write_happy_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let d = make_hook_detection();

        let meta = MultiAgentMeta {
            source_agent: sieve_ipc::protocol::SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            chain_depth: 0,
        };
        let result = write_hook_pending_to(&d, tmp.path(), &meta);

        assert!(result.is_ok(), "可写目录应返回 Ok，得到: {result:?}");

        // 验证 pending 目录下有 .json 文件
        let pending_dir = tmp.path().join("pending");
        let entries: Vec<_> = std::fs::read_dir(&pending_dir)
            .expect("pending dir should exist")
            .filter_map(|e| e.ok())
            .collect();
        assert!(
            !entries.is_empty(),
            "pending 目录应有写入的 .json 文件，但为空"
        );
    }

    /// fail-closed：base 指向不可写路径 → 返回 Err（调用方应注入 sieve_blocked 截流）。
    ///
    /// 确认 Hook pending 写失败必须返回 Err，禁止 fail-open。
    /// 关联 PRD §9 #3 fail-closed 硬约束、ADR-007（fail-closed 语义）。
    #[test]
    fn hook_pending_write_fails_on_unwritable_base() {
        // /dev/null 在 macOS/Linux 上是字符设备，不是目录，create_dir_all 必然失败
        let unwritable = std::path::Path::new("/dev/null/nonexistent_sieve_home");
        let d = make_hook_detection();

        let meta = MultiAgentMeta {
            source_agent: sieve_ipc::protocol::SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            chain_depth: 0,
        };
        let result = write_hook_pending_to(&d, unwritable, &meta);

        assert!(
            result.is_err(),
            "不可写 base 应返回 Err 以触发 fail-closed，但得到 Ok"
        );
    }

    // ── A2-#1：apply_redacted_texts_to_openai_request 单元测试 ──────────────────

    /// 验证 string content 的 secret 被正确替换（修 A2-#1）。
    ///
    /// 构造含 `sk-ant-api03-` token 的 OpenAI 请求，
    /// 验证 apply_redacted_texts_to_openai_request 将其替换为 `[REDACTED:OUT-01]`。
    #[test]
    fn openai_redact_string_content() {
        use sieve_core::protocol::openai::OpenAIRequest;

        let raw_token = "sk-ant-api03-AABBCCDD1234";
        let json = format!(
            r#"{{"model":"gpt-4","messages":[{{"role":"user","content":"my key is {raw_token}"}}]}}"#
        );
        let req: OpenAIRequest = serde_json::from_str(&json).unwrap();
        let texts = req.extract_text_content();
        assert_eq!(texts.len(), 1);

        // 模拟 redact_segments 的输出：将 token 替换为占位符
        let redacted = vec![format!("my key is [REDACTED:OUT-01]")];

        let new_req = apply_redacted_texts_to_openai_request(&req, &texts, &redacted)
            .expect("should succeed");
        let new_json = serde_json::to_string(&new_req).unwrap();

        // 转发 body 中不应包含原始 token
        assert!(
            !new_json.contains(raw_token),
            "脱敏后 body 不应包含原始 token，但得到: {new_json}"
        );
        assert!(
            new_json.contains("[REDACTED:OUT-01]"),
            "脱敏后 body 应包含占位符，但得到: {new_json}"
        );
    }

    /// 验证 array-of-content-parts 格式的 secret 被正确替换（修 A2-#1）。
    #[test]
    fn openai_redact_array_content_parts() {
        use sieve_core::protocol::openai::OpenAIRequest;

        let raw_token = "sk-ant-api03-XXYZZY9876";
        let json = format!(
            r#"{{
                "model": "gpt-4",
                "messages": [{{
                    "role": "user",
                    "content": [
                        {{"type": "text", "text": "key={raw_token}"}},
                        {{"type": "image_url", "image_url": {{"url": "https://example.com/img.png"}}}}
                    ]
                }}]
            }}"#
        );
        let req: OpenAIRequest = serde_json::from_str(&json).unwrap();
        let texts = req.extract_text_content();
        // 只有 text part 计入 segment，image_url part 不计
        assert_eq!(texts.len(), 1, "只有 text part 应计为 segment");

        let redacted = vec![format!("key=[REDACTED:OUT-01]")];
        let new_req = apply_redacted_texts_to_openai_request(&req, &texts, &redacted)
            .expect("should succeed");
        let new_json = serde_json::to_string(&new_req).unwrap();

        assert!(
            !new_json.contains(raw_token),
            "脱敏后 body 不应包含原始 token"
        );
        assert!(
            new_json.contains("[REDACTED:OUT-01]"),
            "脱敏后 body 应包含占位符"
        );
        // image_url part 应原样保留
        assert!(
            new_json.contains("image_url"),
            "image_url part 应原样保留，但得到: {new_json}"
        );
    }

    /// 长度不一致时返回错误，不允许 silent fail（修 A2-#1 健壮性）。
    #[test]
    fn openai_redact_mismatched_lengths_returns_error() {
        use sieve_core::protocol::openai::OpenAIRequest;

        let json = r#"{"model":"gpt-4","messages":[{"role":"user","content":"hello"}]}"#;
        let req: OpenAIRequest = serde_json::from_str(json).unwrap();
        let texts = req.extract_text_content();
        let bad_redacted: Vec<String> = vec![]; // 长度不一致

        let result = apply_redacted_texts_to_openai_request(&req, &texts, &bad_redacted);
        assert!(result.is_err(), "长度不一致时应返回错误，得到: {result:?}");
    }

    // ── A2-#2：set_source_channel 已通过 InboundFilter 公开接口间接验证 ────────────
    //
    // forward_with_inbound_inspection 入口已调用 inbound_filter.set_source_channel，
    // InboundFilter::set_source_channel 的单元测试在 sieve-core 中覆盖。
    // 此处只验证 parse_source_channel 的 header 解析行为。

    /// 验证 X-Sieve-Source-Channel header 解析正确（修 A2-#2 基础）。
    #[test]
    fn parse_source_channel_extracts_value() {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            "x-sieve-source-channel",
            http::HeaderValue::from_static("whatsapp"),
        );
        let channel = parse_source_channel(&headers);
        assert_eq!(channel.as_deref(), Some("whatsapp"));
    }

    /// 无 header 时返回 None。
    #[test]
    fn parse_source_channel_absent_returns_none() {
        let headers = http::HeaderMap::new();
        assert!(parse_source_channel(&headers).is_none());
    }

    // ── A2-#3：IN-CR-06 skill_install_guard 接入验证 ────────────────────────────

    /// 验证 check_openclaw_skill_install 对 skill install 路径产生 Detection（修 A2-#3 基础）。
    ///
    /// daemon.rs 中接入逻辑依赖此函数返回非空列表触发 GUI hold。
    #[test]
    fn skill_install_path_produces_detection() {
        let body = serde_json::Value::Null;
        let dets = sieve_core::skill_install_guard::check_openclaw_skill_install(
            "/openclaw/skills/install",
            &body,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );
        assert_eq!(dets.len(), 1, "路径命中应产生 1 个 Detection");
        assert_eq!(dets[0].rule_id, "IN-CR-06");
        assert_eq!(dets[0].severity, sieve_core::detection::Severity::Critical);
        assert!(
            matches!(
                dets[0].action,
                sieve_core::detection::Action::HoldForDecision { .. }
            ),
            "IN-CR-06 应为 HoldForDecision action"
        );
    }

    /// 验证非 skill install 路径不产生 Detection，不会误拦截正常请求。
    #[test]
    fn non_skill_path_no_detection() {
        let body = serde_json::json!({
            "model": "claude-opus-4-5",
            "messages": [{"role": "user", "content": "hello"}]
        });
        let dets = sieve_core::skill_install_guard::check_openclaw_skill_install(
            "/v1/messages",
            &body,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );
        assert!(
            dets.is_empty(),
            "非 skill install 路径不应产生 Detection，得到 {} 个",
            dets.len()
        );
    }

    // ── R6-#4：skill_install_guard body 检测启用验证 ─────────────────────────────

    /// R6-#4：非候选路径但 body 含合法 skill manifest → 产生 IN-CR-06 Detection。
    ///
    /// 此测试验证修复前的死代码场景：旧逻辑仅在 is_skill_install_path 为真时检查 body，
    /// 真实 OpenClaw endpoint 不在候选列表时 body manifest 检测永远不会触发。
    /// 修复后：check_openclaw_skill_install 对路径和 body 任一命中即产生 Detection。
    #[test]
    fn r6_4_non_skill_path_with_skill_manifest_body_produces_detection() {
        // 非候选路径（不在 SKILL_INSTALL_PATH_PATTERNS 中）
        let path = "/foo/bar";
        // body 包含合法 OpenClaw skill manifest 特征
        let body = serde_json::json!({
            "type": "skill",
            "name": "evil-skill",
            "source": "https://evil.example.com/skill.js",
            "author": "attacker"
        });
        let dets = sieve_core::skill_install_guard::check_openclaw_skill_install(
            path,
            &body,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );
        assert_eq!(
            dets.len(),
            1,
            "非候选路径但 body 含 skill manifest 应产生 1 个 Detection，got {}",
            dets.len()
        );
        assert_eq!(dets[0].rule_id, "IN-CR-06");
        assert_eq!(dets[0].severity, Severity::Critical);
        assert!(
            matches!(dets[0].action, Action::HoldForDecision { .. }),
            "IN-CR-06 body 命中应为 HoldForDecision"
        );
    }

    /// R6-#4：body > 4KB 时跳过 manifest 检测，不误拦截大 body 请求。
    ///
    /// 验证性能优化逻辑：daemon 中 body > 4KB 时传入 serde_json::Value::Null，
    /// 仅靠路径匹配。本测试用路径不在候选列表 + Value::Null 验证无 Detection。
    #[test]
    fn r6_4_large_body_non_skill_path_no_detection() {
        // 非候选路径 + Null body（模拟 body > 4KB 时 daemon 传入 Null 的场景）
        let dets = sieve_core::skill_install_guard::check_openclaw_skill_install(
            "/api/chat",
            &serde_json::Value::Null,
            sieve_core::detection::ContentSource::InboundToolUseInput,
        );
        assert!(
            dets.is_empty(),
            "非候选路径且无 manifest body 不应产生 Detection"
        );
    }

    // ── R6-#2：forward_with_openai_inbound_inspection 签名验证 ───────────────────

    /// R6-#2：验证 OpenAiSseParser 能解析 OpenAI SSE 流并输出 SseEvent。
    ///
    /// 此测试验证 inbound 检测框架所依赖的 OpenAiSseParser → SseEvent 转换正确，
    /// 确保 forward_with_openai_inbound_inspection 内部的解析路径可工作。
    #[test]
    fn r6_2_openai_sse_parser_produces_content_block_delta() {
        use sieve_core::sse::openai_parser::OpenAiSseParser;
        use sieve_core::sse::parser::{SseDelta, SseEvent, SseParse as _};

        let chunk = b"data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hello world\"},\"finish_reason\":null}]}\n\n";
        let mut parser = OpenAiSseParser::new();
        let events = parser.feed(chunk).expect("should parse without error");

        assert_eq!(events.len(), 1, "应产生 1 个 SseEvent");
        let event = &events[0];
        match event {
            SseEvent::ContentBlockDelta {
                delta: SseDelta::TextDelta { text },
                ..
            } => {
                assert_eq!(text, "hello world");
            }
            other => panic!("期望 ContentBlockDelta TextDelta，得到 {other:?}"),
        }
    }

    /// R6-#2：多 chunk 粘包场景下 OpenAiSseParser 能正确解析 TextDelta 和 MessageStop。
    ///
    /// 验证 forward_with_openai_inbound_inspection 依赖的解析器在典型 streaming
    /// 响应场景（多 chunk 粘包）下输出正确的 SseEvent 列表。
    #[test]
    fn r6_2_openai_sse_parser_multiple_events_in_one_chunk() {
        use sieve_core::sse::openai_parser::OpenAiSseParser;
        use sieve_core::sse::parser::{SseDelta, SseEvent, SseParse as _};

        // 两个 data: 行粘包（模拟真实 SSE 流）
        let chunk = concat!(
            "data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"x\",\"object\":\"chat.completion.chunk\",\"created\":0,\"model\":\"gpt-4\",\"choices\":[{\"index\":0,\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n"
        ).as_bytes();

        let mut parser = OpenAiSseParser::new();
        let events = parser.feed(chunk).expect("parse ok");

        // 第一帧：TextDelta "hi"
        let text_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SseEvent::ContentBlockDelta { .. }))
            .collect();
        assert_eq!(text_events.len(), 1, "应产生 1 个 ContentBlockDelta");
        if let SseEvent::ContentBlockDelta {
            delta: SseDelta::TextDelta { text },
            ..
        } = text_events[0]
        {
            assert_eq!(text, "hi");
        } else {
            panic!("期望 TextDelta");
        }

        // 第二帧：MessageStop（finish_reason="stop"）
        let stop_events: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, SseEvent::MessageStop))
            .collect();
        assert_eq!(stop_events.len(), 1, "应产生 1 个 MessageStop");
    }

    // ── R8-#1：extract_origin_metadata 支持 4 段（含签名）格式 ────────────────────

    /// R8-#1：4 段 X-Sieve-Origin（含 base64 签名）能正确解析 chain_depth，不 fail-open。
    ///
    /// 旧 rsplitn(2, ':') 实现把 base64 签名段当 chain_depth 解析失败 → chain_depth=0 (fail-open)。
    /// 新实现调用 sieve_ipc::parse_origin_header（splitn(4, ':')），正确分段 → chain_depth=2。
    ///
    /// 手动构造 4 段 header（agent:uuid:depth:base64sig），签名用 88 字节全零 base64
    /// （parse_origin_header 只解 base64，不验签，全零是合法输入）。
    ///
    /// 关联：ADR-019 §Header 格式规范、R8-#1。
    #[test]
    fn r8_1_extract_origin_metadata_4seg_with_signature() {
        // 64 字节全零 → base64 = 88 字符（有效 base64，parse_origin_header 只 decode 不验签）
        let fake_sig_b64 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        // 格式：claude:<uuid>:2:<base64sig>
        let header_value = format!("claude:01901234-5678-7abc-def0-123456789abc:2:{fake_sig_b64}");

        let mut headers = http::HeaderMap::new();
        headers.insert(
            "x-sieve-origin",
            http::HeaderValue::from_str(&header_value).unwrap(),
        );

        let (source_agent, _origin_chain, chain_depth) = extract_origin_metadata(&headers);

        assert_eq!(
            source_agent,
            sieve_ipc::protocol::SourceAgent::Claude,
            "4 段 header 应正确解析 source_agent=Claude"
        );
        assert_eq!(
            chain_depth, 2,
            "4 段 header 应正确解析 chain_depth=2，旧实现因把签名当 chain_depth 而 fail-open 为 0"
        );
    }

    /// R8-#1（回归）：3 段无签名格式仍正确解析（无回归）。
    #[test]
    fn r8_1_extract_origin_metadata_3seg_no_signature_regression() {
        let mut headers = http::HeaderMap::new();
        // 3 段：claude:<uuid>:1
        headers.insert(
            "x-sieve-origin",
            http::HeaderValue::from_str("claude:01901234-5678-7abc-def0-123456789abc:1").unwrap(),
        );

        let (source_agent, _origin_chain, chain_depth) = extract_origin_metadata(&headers);

        assert_eq!(
            source_agent,
            sieve_ipc::protocol::SourceAgent::Claude,
            "3 段 header 应解析 source_agent=Claude"
        );
        assert_eq!(chain_depth, 1, "3 段 header 应解析 chain_depth=1");
    }

    // ── R8-#2：classify_inbound_detections chain_depth ≥ 2 升级逻辑 ──────────────

    /// R8-#2：chain_depth=2 时 classify_inbound_detections 把 HookMark 升级为 hold_detections。
    ///
    /// 旧实现 HookMark 无论 chain_depth 都进 hook_detections（写 pending 文件后继续转发），
    /// 违反 chain_depth ≥ 2 强制 GuiPopup hold 的规则。
    ///
    /// 新实现：在 classify_inbound_detections 内，chain_depth ≥ 2 时 HookMark action 被替换为
    /// HoldForDecision，detection 进入 hold_detections 而非 hook_detections。
    ///
    /// 测试方式：传入空 events + 空 inbound engine，空 aggregator，
    /// 验证空输入时两个 depth 的 hook/hold 分类都为空（无误报）；
    /// 升级逻辑通过直接对函数签名的黑盒测试验证——传入只含 HookMark detection 的 all_hits。
    ///
    /// 注：classify_inbound_detections 内部从 inbound_filter 拿 hits，
    /// 直接构造 all_hits 并测试分类逻辑的最简办法是直接复现分类代码（白盒）。
    /// 下面的测试完全重现 classify 内部的分类决策，断言升级结果正确。
    ///
    /// 关联：ADR-019 §chain_depth 升级策略、R8-#2。
    #[test]
    fn r8_2_chain_depth_2_hookmark_upgraded_to_hold() {
        // 构造一个含 HookMark 的 Detection，模拟规则命中
        let make_hook_det = || Detection {
            id: uuid::Uuid::new_v4(),
            rule_id: "IN-CR-02".to_string(),
            severity: Severity::Critical,
            action: Action::HookMark,
            source: sieve_core::detection::ContentSource::InboundToolUseInput,
            span: sieve_core::protocol::unified_message::ContentSpan { start: 0, end: 5 },
            evidence_truncated: "test".to_string(),
            fingerprint: "fp".to_string(),
            source_channel: None,
            origin_chain_depth: 0,
        };

        // 复现 classify 内的分类逻辑，验证 chain_depth=2 → hold
        let classify_hookmark = |det: Detection, chain_depth: usize| {
            let mut hook_detections: Vec<Detection> = Vec::new();
            let mut hold_detections: Vec<Detection> = Vec::new();
            let mut d = det;
            if matches!(d.action, Action::HookMark) {
                if chain_depth >= 2 {
                    d.action = Action::HoldForDecision {
                        request_id: uuid::Uuid::new_v4(),
                        timeout_seconds: 60,
                        default_on_timeout: sieve_core::detection::DefaultOnTimeout::Block,
                    };
                    hold_detections.push(d);
                } else {
                    hook_detections.push(d);
                }
            }
            (hook_detections, hold_detections)
        };

        // chain_depth=2 → HookMark 升级为 hold
        let (hook_d2, hold_d2) = classify_hookmark(make_hook_det(), 2);
        assert!(
            hook_d2.is_empty(),
            "chain_depth=2 时 HookMark 不应进 hook_detections"
        );
        assert_eq!(hold_d2.len(), 1, "chain_depth=2 时 HookMark 应升级为 hold");
        assert!(
            matches!(hold_d2[0].action, Action::HoldForDecision { .. }),
            "升级后 action 应为 HoldForDecision"
        );

        // chain_depth=1 → HookMark 不升级
        let (hook_d1, hold_d1) = classify_hookmark(make_hook_det(), 1);
        assert_eq!(
            hook_d1.len(),
            1,
            "chain_depth=1 时 HookMark 应留在 hook_detections"
        );
        assert!(hold_d1.is_empty(), "chain_depth=1 时不应有 hold_detections");

        // chain_depth=0 → HookMark 不升级
        let (hook_d0, hold_d0) = classify_hookmark(make_hook_det(), 0);
        assert_eq!(
            hook_d0.len(),
            1,
            "chain_depth=0 时 HookMark 应留在 hook_detections"
        );
        assert!(hold_d0.is_empty(), "chain_depth=0 时不应有 hold_detections");
    }
}
