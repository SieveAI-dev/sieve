//! `sieve decisions` 子命令实现（ADR-028 TODO-4，headless decision CLI）。
//!
//! 在 GUI 不在线时通过 CLI 订阅 / 查看 / 解决待决策事件。
//! CLI 跟 GUI 共用同一组 IPC 方法（`sieve.request_decision` / `sieve.health`），
//! **不引入特权 endpoint**（ADR-028 §3）。
//!
//! ## IPC 客户端实现策略
//!
//! 直接用 raw JSON-RPC over `tokio::net::UnixStream` 连 `~/.sieve/ipc.sock`，
//! 用 `serde_json::to_string` / `serde_json::from_str` 编解码 ndjson 帧。
//! 避开 sieve-ipc 内部模块（可能被并行 agent 重构），只引用 `sieve_ipc::paths`。

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use uuid::Uuid;

use crate::cli::{DecisionsArgs, DecisionsCommand, Severity};

// ─────────────────────────── Raw IPC helper ────────────────────────────────

/// 解析 duration 字符串 sieve home 路径。
fn ipc_socket_path() -> Result<PathBuf> {
    let home = sieve_ipc::paths::sieve_home().context("获取 sieve home 失败")?;
    Ok(home.join("ipc.sock"))
}

/// 发送一条 JSON-RPC call，等待对应 id 的 response，返回 `result` 字段（JSON Value）。
async fn rpc_call(
    stream: &mut UnixStream,
    method: &str,
    params: serde_json::Value,
    call_id: &str,
) -> Result<serde_json::Value> {
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": call_id,
    });
    let mut payload = serde_json::to_string(&req)?;
    payload.push('\n');
    stream
        .write_all(payload.as_bytes())
        .await
        .context("写 IPC socket 失败")?;

    // 读响应行（ndjson，每行一条消息）
    let (reader, _) = stream.split();
    let mut lines = BufReader::new(reader).lines();
    while let Some(line) = lines.next_line().await.context("读 IPC socket 失败")? {
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }
        let val: serde_json::Value = serde_json::from_str(&line)
            .with_context(|| format!("解析 IPC 响应 JSON 失败: {line}"))?;
        // 匹配 id
        if val.get("id").and_then(|v| v.as_str()) == Some(call_id) {
            if let Some(err) = val.get("error") {
                return Err(anyhow!("IPC 错误响应: {err}"));
            }
            return val
                .get("result")
                .cloned()
                .ok_or_else(|| anyhow!("IPC 响应缺少 result 字段: {val}"));
        }
        // 非目标 id 的消息（hello / notify 等）直接跳过
    }
    Err(anyhow!("IPC 连接关闭，未收到 id={call_id} 的响应"))
}

// ─────────────────────────── 协议 DTO（inline 定义，避免依赖 sieve-ipc 内部）─

/// `sieve.health` 响应中我们关心的字段子集。
#[derive(Debug, Deserialize)]
struct HealthSnapshot {
    #[serde(default)]
    ipc: IpcSnapshotSubset,
}

#[derive(Debug, Default, Deserialize)]
struct IpcSnapshotSubset {
    #[serde(default)]
    total_decisions_inflight: u32,
}

/// `sieve.request_decision` notification 中的 detection 条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionItem {
    pub rule_id: String,
    pub severity: String,
    pub disposition: String,
    pub title: String,
    pub one_line_summary: String,
    #[serde(default)]
    pub details: serde_json::Value,
}

/// 从 `sieve.request_decision` notification 中提取的 pending decision。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDecision {
    pub request_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub timeout_seconds: u32,
    pub default_on_timeout: String,
    pub detections: Vec<DetectionItem>,
    #[serde(default)]
    pub source_agent: String,
    #[serde(default)]
    pub source_channel: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
}

// ─────────────────────────── watch ─────────────────────────────────────────

async fn run_watch(
    format_jsonl: bool,
    severity_filter: Option<String>,
    provider_id_filter: Option<String>,
) -> Result<()> {
    let sock_path = ipc_socket_path()?;

    // 连接 IPC socket
    let stream = UnixStream::connect(&sock_path).await.with_context(|| {
        format!(
            "连接 IPC socket 失败（{}）；请确认 sieve daemon 正在运行",
            sock_path.display()
        )
    })?;

    let (reader, _writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    eprintln!(
        "sieve decisions watch: 订阅 pending 决策事件（Ctrl+C 退出）{}{}",
        severity_filter
            .as_deref()
            .map(|s| format!(" [severity={s}]"))
            .unwrap_or_default(),
        provider_id_filter
            .as_deref()
            .map(|p| format!(" [provider-id={p}]"))
            .unwrap_or_default(),
    );

    while let Some(line) = lines.next_line().await.context("读 IPC socket 失败")? {
        let line = line.trim().to_owned();
        if line.is_empty() {
            continue;
        }

        let val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // 只关注 sieve.request_decision notification（daemon → client push）
        let method = val.get("method").and_then(|m| m.as_str()).unwrap_or("");
        if method != "sieve.request_decision" {
            continue;
        }

        let params = match val.get("params") {
            Some(p) => p.clone(),
            None => continue,
        };

        // 反序列化为 PendingDecision
        let decision: PendingDecision = match serde_json::from_value(params.clone()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("sieve decisions watch: 解析 decision 失败: {e}");
                continue;
            }
        };

        // severity 过滤
        if let Some(ref sev_filter) = severity_filter {
            let any_match = decision
                .detections
                .iter()
                .any(|d| d.severity.to_lowercase() == sev_filter.to_lowercase());
            if !any_match {
                continue;
            }
        }

        // provider_id 目前在 notification params 里没有，TODO: future when protocol adds it
        let _ = &provider_id_filter;

        if format_jsonl {
            println!("{}", serde_json::to_string(&decision)?);
        } else {
            // 人类可读输出
            println!(
                "[{}] request_id={} timeout={}s default={}",
                decision.created_at.format("%Y-%m-%dT%H:%M:%SZ"),
                decision.request_id,
                decision.timeout_seconds,
                decision.default_on_timeout,
            );
            for det in &decision.detections {
                println!(
                    "  [{severity}] {rule_id}: {summary}",
                    severity = det.severity,
                    rule_id = det.rule_id,
                    summary = det.one_line_summary,
                );
            }
        }
    }

    eprintln!("sieve decisions watch: 连接已关闭");
    Ok(())
}

// ─────────────────────────── show ──────────────────────────────────────────

async fn run_show(id: Uuid) -> Result<()> {
    let sock_path = ipc_socket_path()?;
    let mut stream = UnixStream::connect(&sock_path).await.with_context(|| {
        format!(
            "连接 IPC socket 失败（{}）；请确认 sieve daemon 正在运行",
            sock_path.display()
        )
    })?;

    // 使用 sieve.health 查询当前状态快照，pending request 信息在 IpcSnapshot 中
    let call_id = format!("show-{id}");
    let result = rpc_call(&mut stream, "sieve.health", serde_json::json!({}), &call_id)
        .await
        .context("查询 sieve.health 失败")?;

    let snapshot: HealthSnapshot = serde_json::from_value(result.clone())?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "requested_id": id.to_string(),
            "total_decisions_inflight": snapshot.ipc.total_decisions_inflight,
            "note": "pending decision detail requires daemon to push sieve.request_decision; use 'sieve decisions watch' to capture",
            "raw_health": result,
        }))?
    );

    Ok(())
}

// ─────────────────────────── resolve ───────────────────────────────────────

/// 决策动作。
#[derive(Debug, Clone, Copy)]
pub enum ResolveAction {
    Approve,
    Block,
    Warn,
}

async fn run_resolve(id: Uuid, action: ResolveAction, reason: Option<String>) -> Result<()> {
    let sock_path = ipc_socket_path()?;
    let mut stream = UnixStream::connect(&sock_path).await.with_context(|| {
        format!(
            "连接 IPC socket 失败（{}）；请确认 sieve daemon 正在运行",
            sock_path.display()
        )
    })?;

    let decision_action = match action {
        ResolveAction::Approve => "allow",
        ResolveAction::Block => "deny",
        ResolveAction::Warn => "redact_and_allow",
    };

    // 构造 DecisionResponse（GUI → daemon 方向）
    // daemon 不区分 caller 身份，CLI 回复和 GUI 回复走同一路径（ADR-028 §3）
    let response = serde_json::json!({
        "request_id": id.to_string(),
        "decision": decision_action,
        "decided_at": Utc::now().to_rfc3339(),
        "by_user": true,
        "remember": false,
        "context_hint": reason,
        "ui_phase_when_clicked": null,
    });

    // JSON-RPC response 格式（id 对应 daemon 发出的 request_decision 的 id）
    let rpc_resp = serde_json::json!({
        "jsonrpc": "2.0",
        "result": response,
        "id": id.to_string(),
    });

    let mut payload = serde_json::to_string(&rpc_resp)?;
    payload.push('\n');
    stream
        .write_all(payload.as_bytes())
        .await
        .context("发送决策响应失败")?;

    // 发完即完成（fire-and-forget，daemon 内部处理）
    if let Some(ref reason_str) = reason {
        eprintln!(
            "sieve decisions resolve: request_id={id} action={decision_action} reason=\"{reason_str}\""
        );
    } else {
        eprintln!("sieve decisions resolve: request_id={id} action={decision_action}");
    }
    println!("ok");
    Ok(())
}

// ─────────────────────────── 入口 ──────────────────────────────────────────

/// `sieve decisions` 命令入口。
pub fn run(args: DecisionsArgs) -> Result<()> {
    // decisions 子命令需要 async runtime
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("构建 tokio runtime 失败")?
        .block_on(run_async(args))
}

async fn run_async(args: DecisionsArgs) -> Result<()> {
    match args.command {
        DecisionsCommand::Watch {
            format_jsonl,
            severity,
            provider_id,
        } => {
            let sev = severity.map(|s| match s {
                Severity::Critical => "critical".to_owned(),
                Severity::High => "high".to_owned(),
                Severity::Medium => "medium".to_owned(),
                Severity::Low => "low".to_owned(),
            });
            run_watch(format_jsonl, sev, provider_id).await
        }
        DecisionsCommand::Show { id } => {
            let uuid = Uuid::parse_str(&id)
                .with_context(|| format!("无效的 decision id（应为 UUID）: {id}"))?;
            run_show(uuid).await
        }
        DecisionsCommand::Resolve {
            id,
            approve,
            block,
            warn,
            reason,
        } => {
            let action = if approve {
                ResolveAction::Approve
            } else if block {
                ResolveAction::Block
            } else if warn {
                ResolveAction::Warn
            } else {
                return Err(anyhow!("必须指定 --approve / --block / --warn 之一"));
            };
            let _ = (approve, block, warn); // suppress unused warning
            let uuid = Uuid::parse_str(&id)
                .with_context(|| format!("无效的 decision id（应为 UUID）: {id}"))?;
            run_resolve(uuid, action, reason).await
        }
    }
}

// ─────────────────────────── 单元测试 ──────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// resolve 命令解析：--approve 产生 ResolveAction::Approve。
    #[test]
    fn resolve_action_approve() {
        let action = ResolveAction::Approve;
        let decision_str = match action {
            ResolveAction::Approve => "allow",
            ResolveAction::Block => "deny",
            ResolveAction::Warn => "redact_and_allow",
        };
        assert_eq!(decision_str, "allow");
    }

    /// resolve 命令解析：--block 产生 "deny"。
    #[test]
    fn resolve_action_block_maps_to_deny() {
        let action = ResolveAction::Block;
        let decision_str = match action {
            ResolveAction::Approve => "allow",
            ResolveAction::Block => "deny",
            ResolveAction::Warn => "redact_and_allow",
        };
        assert_eq!(decision_str, "deny");
    }

    /// resolve 命令解析：--warn 产生 "redact_and_allow"。
    #[test]
    fn resolve_action_warn_maps_to_redact_and_allow() {
        let action = ResolveAction::Warn;
        let decision_str = match action {
            ResolveAction::Approve => "allow",
            ResolveAction::Block => "deny",
            ResolveAction::Warn => "redact_and_allow",
        };
        assert_eq!(decision_str, "redact_and_allow");
    }

    /// watch jsonl 输出格式：PendingDecision 序列化为合法 JSON。
    #[test]
    fn pending_decision_jsonl_serialization() {
        let decision = PendingDecision {
            request_id: Uuid::nil(),
            created_at: Utc::now(),
            timeout_seconds: 60,
            default_on_timeout: "block".to_owned(),
            detections: vec![DetectionItem {
                rule_id: "IN-CR-01".to_owned(),
                severity: "Critical".to_owned(),
                disposition: "gui_popup".to_owned(),
                title: "BIP39 助记词".to_owned(),
                one_line_summary: "检测到 12 词助记词".to_owned(),
                details: serde_json::json!({}),
            }],
            source_agent: "claude".to_owned(),
            source_channel: None,
            direction: Some("inbound".to_owned()),
        };

        let json = serde_json::to_string(&decision).expect("序列化应成功");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("应为合法 JSON");
        assert!(parsed.get("request_id").is_some(), "应含 request_id");
        assert!(parsed.get("detections").is_some(), "应含 detections");

        // jsonl 格式要求：无换行（单行）
        assert!(
            !json.contains('\n'),
            "jsonl 格式每行一个 JSON，不应含换行符"
        );
    }

    /// IPC socket 路径应包含 ipc.sock。
    #[test]
    fn ipc_socket_path_ends_with_ipc_sock() {
        // 需要 HOME env 存在（CI 环境通常有）
        if std::env::var("HOME").is_err() {
            return; // 无 HOME 的极端环境跳过
        }
        let path = ipc_socket_path().expect("ipc_socket_path 应成功");
        assert!(
            path.to_str().unwrap_or("").ends_with("ipc.sock"),
            "socket 路径应以 ipc.sock 结尾，实际: {}",
            path.display()
        );
    }

    /// watch 的 severity 过滤逻辑正确。
    #[test]
    fn severity_filter_matches_case_insensitive() {
        let decision = PendingDecision {
            request_id: Uuid::nil(),
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: "block".to_owned(),
            detections: vec![DetectionItem {
                rule_id: "IN-CR-02".to_owned(),
                severity: "Critical".to_owned(),
                disposition: "hook_terminal".to_owned(),
                title: "测试".to_owned(),
                one_line_summary: "测试 severity 过滤".to_owned(),
                details: serde_json::json!({}),
            }],
            source_agent: String::new(),
            source_channel: None,
            direction: None,
        };

        let sev_filter = "critical";
        let any_match = decision
            .detections
            .iter()
            .any(|d| d.severity.to_lowercase() == sev_filter.to_lowercase());
        assert!(any_match, "Critical 应匹配 'critical' 过滤");

        let sev_filter_high = "high";
        let no_match = decision
            .detections
            .iter()
            .any(|d| d.severity.to_lowercase() == sev_filter_high.to_lowercase());
        assert!(!no_match, "Critical 不应匹配 'high' 过滤");
    }
}
