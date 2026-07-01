//! `sieve decisions` 子命令实现（headless decision CLI）。
//!
//! 在 GUI 不在线时通过 CLI 查看 / 订阅 / 解决待决策事件。
//! CLI 跟 GUI 共用同一组 IPC 方法（`sieve.list_pending` / `sieve.resolve_decision` /
//! `sieve.request_decision` push），**不引入特权 endpoint**。
//!
//! ## A 方案授权（决策授权模型）
//!
//! `resolve` 对 daemon 侧判定为 `Critical` 的 pending 一律静默 deny（headless 不批准
//! 不可逆动作，摩擦要求人在场）；`High` 及以下允许 headless 批准。判定在 daemon 端按
//! daemon 侧计算的 `max_severity` 做，**不信 CLI 自报**。`remember` 恒为 false。
//!
//! IPC 客户端封装见 [`crate::commands::ipc_client`]。

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixStream;
use uuid::Uuid;

use crate::cli::{DecisionsArgs, DecisionsCommand, Severity};
use crate::commands::ipc_client;

// ─────────────────────────── 协议 DTO（inline 定义，避免依赖 sieve-ipc 内部）─

/// `sieve.request_decision` push notification / `list_pending` 中的 detection 摘要。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionItem {
    pub rule_id: String,
    pub severity: String,
    #[serde(default)]
    pub disposition: String,
    pub title: String,
    #[serde(default)]
    pub one_line_summary: String,
    #[serde(default)]
    pub details: serde_json::Value,
}

/// 从 `sieve.request_decision` notification 中提取的 pending decision（watch 用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDecision {
    pub request_id: Uuid,
    pub created_at: DateTimeString,
    pub timeout_seconds: u32,
    pub default_on_timeout: String,
    #[serde(default)]
    pub detections: Vec<DetectionItem>,
    #[serde(default)]
    pub source_agent: String,
    #[serde(default)]
    pub source_channel: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
    /// listener 上游 provider_id（多 listener 路由；`watch --provider-id` 据此过滤）。
    #[serde(default)]
    pub provider_id: Option<String>,
}

/// wire 时间戳（字符串透传，避免 chrono 解析耦合）。
type DateTimeString = String;

// ─────────────────────────── list ──────────────────────────────────────────

/// severity 过滤：pending 的任一 detection 命中目标 severity 即保留。
///
/// list_pending 快照顶层有 `max_severity`，也逐条带 `detections[].severity`；
/// 与 watch 一致按"任一 detection 命中"判定。
fn severity_matches(snapshot: &serde_json::Value, filter: &str) -> bool {
    let f = filter.to_lowercase();
    // 顶层 max_severity 命中即算。
    if snapshot
        .get("max_severity")
        .and_then(|v| v.as_str())
        .map(|s| s.eq_ignore_ascii_case(&f))
        .unwrap_or(false)
    {
        return true;
    }
    snapshot
        .get("detections")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter().any(|d| {
                d.get("severity")
                    .and_then(|v| v.as_str())
                    .map(|s| s.eq_ignore_ascii_case(&f))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// provider_id 过滤：pending 的 `provider_id` 等于目标值。
fn provider_matches(snapshot: &serde_json::Value, filter: &str) -> bool {
    snapshot
        .get("provider_id")
        .and_then(|v| v.as_str())
        .map(|p| p == filter)
        .unwrap_or(false)
}

async fn run_list(
    format_jsonl: bool,
    severity_filter: Option<String>,
    provider_id_filter: Option<String>,
) -> Result<()> {
    let result = ipc_client::rpc_call_oneshot("sieve.list_pending", serde_json::json!({}))
        .await
        .context("查询 sieve.list_pending 失败")?;

    let pending = result
        .get("pending")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let filtered: Vec<&serde_json::Value> = pending
        .iter()
        .filter(|snap| {
            severity_filter
                .as_deref()
                .map(|f| severity_matches(snap, f))
                .unwrap_or(true)
                && provider_id_filter
                    .as_deref()
                    .map(|f| provider_matches(snap, f))
                    .unwrap_or(true)
        })
        .collect();

    if format_jsonl {
        for snap in &filtered {
            println!("{}", serde_json::to_string(snap)?);
        }
    } else if filtered.is_empty() {
        println!("(无 pending 决策)");
    } else {
        for snap in &filtered {
            print_snapshot_pretty(snap);
        }
    }
    // 空集 exit 0（空 ≠ 错误）。
    Ok(())
}

/// pretty 打印单条 pending 快照（人类可读）。
fn print_snapshot_pretty(snap: &serde_json::Value) {
    let request_id = snap
        .get("request_id")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let max_sev = snap
        .get("max_severity")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let age = snap
        .get("age_seconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let timeout = snap
        .get("timeout_seconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let default_on_timeout = snap
        .get("default_on_timeout")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let direction = snap
        .get("direction")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let provider = snap
        .get("provider_id")
        .and_then(|v| v.as_str())
        .unwrap_or("-");
    println!(
        "request_id={request_id} severity={max_sev} direction={direction} \
         age={age}s timeout={timeout}s default_on_timeout={default_on_timeout} provider={provider}"
    );
    if let Some(dets) = snap.get("detections").and_then(|v| v.as_array()) {
        for d in dets {
            let rule_id = d.get("rule_id").and_then(|v| v.as_str()).unwrap_or("?");
            let severity = d.get("severity").and_then(|v| v.as_str()).unwrap_or("?");
            let summary = d
                .get("one_line_summary")
                .and_then(|v| v.as_str())
                .or_else(|| d.get("title").and_then(|v| v.as_str()))
                .unwrap_or("");
            println!("  [{severity}] {rule_id}: {summary}");
        }
    }
}

// ─────────────────────────── show ──────────────────────────────────────────

async fn run_show(id: Uuid) -> Result<()> {
    let result = ipc_client::rpc_call_oneshot("sieve.list_pending", serde_json::json!({}))
        .await
        .context("查询 sieve.list_pending 失败")?;

    let pending = result
        .get("pending")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let target = pending.iter().find(|snap| {
        snap.get("request_id")
            .and_then(|v| v.as_str())
            .map(|s| s == id.to_string())
            .unwrap_or(false)
    });

    match target {
        Some(snap) => {
            println!("{}", serde_json::to_string_pretty(snap)?);
            Ok(())
        }
        None => {
            eprintln!(
                "sieve decisions show: 未找到 request_id={id} 的 pending 决策\
                 （可能已超时 / 已被解决 / id 不存在）"
            );
            std::process::exit(1);
        }
    }
}

// ─────────────────────────── resolve ───────────────────────────────────────

/// 决策动作（CLI flag → wire decision_action）。
#[derive(Debug, Clone, Copy)]
pub enum ResolveAction {
    Approve,
    Block,
    Warn,
}

impl ResolveAction {
    fn wire_value(self) -> &'static str {
        match self {
            ResolveAction::Approve => "allow",
            ResolveAction::Block => "deny",
            ResolveAction::Warn => "redact_and_allow",
        }
    }
}

async fn run_resolve(id: Uuid, action: ResolveAction, reason: Option<String>) -> Result<()> {
    let mut params = serde_json::json!({
        "request_id": id.to_string(),
        "decision": action.wire_value(),
    });
    if let Some(ref r) = reason {
        params["context_hint"] = serde_json::Value::String(r.clone());
    }

    let result = ipc_client::rpc_call_oneshot("sieve.resolve_decision", params)
        .await
        .context("调用 sieve.resolve_decision 失败")?;

    let status = result.get("status").and_then(|v| v.as_str()).unwrap_or("");
    match status {
        "resolved" => {
            let effective = result
                .get("effective_decision")
                .and_then(|v| v.as_str())
                .unwrap_or(action.wire_value());
            // stdout：机器可读结果（含实际生效决策，A 方案下 Critical 会显示 deny）。
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "status": "resolved",
                    "request_id": id.to_string(),
                    "effective_decision": effective,
                }))?
            );
            // stderr：客观事实一行（headless 日志可追溯），不含 GUI 引导文案。
            eprintln!("decision {effective} (request_id={id})");
            Ok(())
        }
        "not_found" => {
            eprintln!(
                "sieve decisions resolve: request_id={id} not_found\
                 （可能已超时 / 已被 GUI 解决 / id 不存在）"
            );
            std::process::exit(1);
        }
        other => {
            eprintln!("sieve decisions resolve: 未知 status={other}");
            std::process::exit(1);
        }
    }
}

// ─────────────────────────── watch ─────────────────────────────────────────

async fn run_watch(
    format_jsonl: bool,
    severity_filter: Option<String>,
    provider_id_filter: Option<String>,
) -> Result<()> {
    let sock_path = ipc_client::ipc_socket_path()?;
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

        // 只关注 sieve.request_decision notification（daemon → client push）。
        let method = val.get("method").and_then(|m| m.as_str()).unwrap_or("");
        if method != "sieve.request_decision" {
            continue;
        }

        let params = match val.get("params") {
            Some(p) => p.clone(),
            None => continue,
        };

        let decision: PendingDecision = match serde_json::from_value(params.clone()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("sieve decisions watch: 解析 decision 失败: {e}");
                continue;
            }
        };

        // severity 过滤：任一 detection 命中。
        if let Some(ref sev_filter) = severity_filter {
            let any_match = decision
                .detections
                .iter()
                .any(|d| d.severity.eq_ignore_ascii_case(sev_filter));
            if !any_match {
                continue;
            }
        }

        // provider_id 过滤（wire 现已带 provider_id 字段，真实比较）。
        if let Some(ref pid_filter) = provider_id_filter {
            let matches = decision.provider_id.as_deref() == Some(pid_filter.as_str());
            if !matches {
                continue;
            }
        }

        if format_jsonl {
            println!("{}", serde_json::to_string(&decision)?);
        } else {
            println!(
                "[{}] request_id={} timeout={}s default={}",
                decision.created_at,
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

// ─────────────────────────── 入口 ──────────────────────────────────────────

/// `sieve decisions` 命令入口。
///
/// 必须是 `async`：`main` 是 `#[tokio::main]`，本命令在该 runtime 内被 `.await`。
pub async fn run(args: DecisionsArgs) -> Result<()> {
    run_async(args).await
}

fn severity_to_wire(s: Severity) -> String {
    match s {
        Severity::Critical => "critical".to_owned(),
        Severity::High => "high".to_owned(),
        Severity::Medium => "medium".to_owned(),
        Severity::Low => "low".to_owned(),
    }
}

async fn run_async(args: DecisionsArgs) -> Result<()> {
    match args.command {
        DecisionsCommand::List {
            format_jsonl,
            severity,
            provider_id,
        } => run_list(format_jsonl, severity.map(severity_to_wire), provider_id).await,
        DecisionsCommand::Watch {
            format_jsonl,
            severity,
            provider_id,
        } => run_watch(format_jsonl, severity.map(severity_to_wire), provider_id).await,
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

    #[test]
    fn resolve_action_wire_values() {
        assert_eq!(ResolveAction::Approve.wire_value(), "allow");
        assert_eq!(ResolveAction::Block.wire_value(), "deny");
        assert_eq!(ResolveAction::Warn.wire_value(), "redact_and_allow");
    }

    #[test]
    fn severity_matches_top_level_and_detections() {
        let snap = serde_json::json!({
            "max_severity": "critical",
            "detections": [{ "severity": "high" }, { "severity": "critical" }],
        });
        assert!(severity_matches(&snap, "critical"));
        assert!(severity_matches(&snap, "high"));
        assert!(!severity_matches(&snap, "low"));
        // 大小写不敏感。
        assert!(severity_matches(&snap, "Critical"));
    }

    #[test]
    fn provider_matches_exact() {
        let snap = serde_json::json!({ "provider_id": "anthropic-main" });
        assert!(provider_matches(&snap, "anthropic-main"));
        assert!(!provider_matches(&snap, "openai"));
        // 缺失 provider_id 不匹配任何过滤。
        let no_provider = serde_json::json!({});
        assert!(!provider_matches(&no_provider, "anthropic-main"));
    }

    #[test]
    fn pending_decision_jsonl_serialization() {
        let decision = PendingDecision {
            request_id: Uuid::nil(),
            created_at: "2026-07-02T00:00:00.000Z".to_owned(),
            timeout_seconds: 60,
            default_on_timeout: "block".to_owned(),
            detections: vec![DetectionItem {
                rule_id: "IN-CR-01".to_owned(),
                severity: "critical".to_owned(),
                disposition: "gui_popup".to_owned(),
                title: "BIP39 助记词".to_owned(),
                one_line_summary: "检测到 12 词助记词".to_owned(),
                details: serde_json::json!({}),
            }],
            source_agent: "claude".to_owned(),
            source_channel: None,
            direction: Some("inbound".to_owned()),
            provider_id: Some("anthropic-main".to_owned()),
        };

        let json = serde_json::to_string(&decision).expect("序列化应成功");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("应为合法 JSON");
        assert!(parsed.get("request_id").is_some());
        assert!(parsed.get("detections").is_some());
        assert_eq!(
            parsed.get("provider_id").and_then(|v| v.as_str()),
            Some("anthropic-main")
        );
        assert!(
            !json.contains('\n'),
            "jsonl 格式每行一个 JSON，不应含换行符"
        );
    }

    #[test]
    fn provider_id_filter_matches_watch_decision() {
        let decision = PendingDecision {
            request_id: Uuid::nil(),
            created_at: "2026-07-02T00:00:00.000Z".to_owned(),
            timeout_seconds: 30,
            default_on_timeout: "block".to_owned(),
            detections: vec![],
            source_agent: String::new(),
            source_channel: None,
            direction: None,
            provider_id: Some("openai-relay".to_owned()),
        };
        assert_eq!(decision.provider_id.as_deref(), Some("openai-relay"));
        // 过滤逻辑：等值匹配。
        assert!(decision.provider_id.as_deref() == Some("openai-relay"));
        assert!(decision.provider_id.as_deref() != Some("anthropic-main"));
    }
}
