// sieve-hook: Claude Code PreToolUse hook 二进制。
//
// 夹在 Claude Code tool_use 调用与实际执行之间，对命中 Critical 规则的工具调用
// 在 TTY 显示危险摘要并等待用户确认。
//
// 启动时延目标 < 50ms（依赖仅 serde_json + fd-lock + clap，无 tokio / vectorscan）。
// 关联：SPEC-001（hook 文件协议）、SPEC-002（弹窗行为规范）、ADR-014（双层防御）。

use std::io::{self, BufRead, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;
use uuid::Uuid;

// 从 lib target 引入共享模块，避免重复定义。
use sieve_hook_lib::codex::{
    emit_codex_decision, emit_codex_fail_closed, CodexOutput, CodexPreToolUseInput,
};
use sieve_hook_lib::codex_ipc;
use sieve_hook_lib::decision::{write_decision, DecisionOutcome};
use sieve_hook_lib::error::PendingError;
use sieve_hook_lib::hermes::{
    emit_hermes_decision, emit_hermes_fail_closed, HermesOutput, HermesPreToolCallInput,
};
use sieve_hook_lib::pending::{read_pending_checked, scan_pending_dir};
use sieve_hook_lib::protocol;

const STALE_THRESHOLD_SECS: i64 = 600;

/// codex 子命令的内部硬 deadline（秒）。
///
/// 超时预算链（严格递减）：codex hook `timeout_sec`（CodexAdapter 显式写大，如 60s）
/// 大于本 deadline（50s）大于 daemon GUI 弹窗 timeout（本 deadline 再减 ~3s 余量）。
/// **必须严格小于 codex 的 `timeout_sec`**：codex 到 `timeout_sec` 会强杀 hook，
/// 被杀 = 无 exit code = fail-OPEN；本 deadline 到点主动 `exit 2`（fail-closed）抢在被杀前。
const CODEX_INTERNAL_DEADLINE_SECS: u64 = 50;

/// `hermes` 子命令的内部硬 deadline（秒）。
///
/// Hermes hook `timeout` 默认 60s / 上限 300s（`~/.hermes/config.yaml`）。本 deadline 必须
/// **严格小于** Hermes 配置的 `timeout`：到点前主动输出 block JSON。注意 Hermes **fail-open**——
/// 超时不会被 Hermes 当成 block（仅 warn 后放行），故本 deadline 只为"尽力赶在放行前发 block"，
/// 真正 fail-closed 由 daemon 网关 `inbound_hold` 兜底（ADR-014 §6）。
const HERMES_INTERNAL_DEADLINE_SECS: u64 = 50;

/// sieve-hook: PreToolUse 安全确认 hook（Phase 1 macOS）。
#[derive(Parser, Debug)]
#[command(name = "sieve-hook", about = "Sieve PreToolUse safety hook")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// 检查 pending 决策请求并请求用户确认（Claude Code PreToolUse hook）。
    Check {
        /// 决策请求 ID（UUID）；未传则读 $SIEVE_REQUEST_ID。
        #[arg(long)]
        request_id: Option<String>,

        /// sieve home 目录；未传则读 $SIEVE_HOME，默认 $HOME/.sieve。
        #[arg(long)]
        sieve_home: Option<PathBuf>,
    },
    /// Codex PreToolUse hook：读 stdin 工具调用 JSON，经 daemon 判危，按 codex 契约输出。
    ///
    /// ⚠️ **exit code 语义与 `check` 相反**：codex 下 `exit 1 = fail-OPEN`。本子命令
    /// 只产 `exit 0`（决策在 stdout）或 `exit 2`（fail-closed，stderr 给原因），**绝不 exit 1**。
    Codex {
        /// sieve home 目录；未传则读 $SIEVE_HOME，默认 $HOME/.sieve。
        #[arg(long)]
        sieve_home: Option<PathBuf>,
    },
    /// Hermes pre_tool_call hook：读 stdin 工具调用 JSON，经 daemon 判危，按 Hermes 契约输出。
    ///
    /// ⚠️ Hermes **fail-OPEN**：退出码不 block，决策经 stdout `{"decision":"block"}` 传达；
    /// 超时即放行。本子命令固定 `exit 0`，真正 fail-closed 由网关 `inbound_hold` 兜底（ADR-014 §6）。
    Hermes {
        /// sieve home 目录；未传则读 $SIEVE_HOME，默认 $HOME/.sieve。
        #[arg(long)]
        sieve_home: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Command::Check {
            request_id,
            sieve_home,
        } => run_check_command(request_id, sieve_home),
        Command::Codex { sieve_home } => run_codex_command(sieve_home),
        Command::Hermes { sieve_home } => run_hermes_command(sieve_home),
    };

    std::process::exit(exit_code);
}

/// 解析 sieve_home：flag > $SIEVE_HOME > $HOME/.sieve。
fn resolve_sieve_home(sieve_home: Option<PathBuf>) -> Option<PathBuf> {
    sieve_home
        .or_else(|| std::env::var("SIEVE_HOME").ok().map(PathBuf::from))
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".sieve"))
        })
}

/// `check` 子命令：Claude Code PreToolUse hook（exit 0=允许，1=拒绝）。
fn run_check_command(request_id: Option<String>, sieve_home: Option<PathBuf>) -> i32 {
    let base = resolve_sieve_home(sieve_home).unwrap_or_else(|| {
        eprintln!("sieve-hook: cannot determine sieve home directory ($HOME not set)");
        std::process::exit(1);
    });

    // 解析 request_id：优先级 1（flag）> 优先级 2（env）> 优先级 3（启发式扫目录）。
    // 优先级 3 是关键修复：Claude Code settings.json 注册静态命令时无法传 request_id，
    // 必须走启发式路径；零 pending 时 fail-open（exit 0），不阻断正常工具调用。
    // 关联：SPEC-001 §4.3（启发式查 pending 目录）。
    let explicit_id = request_id.or_else(|| std::env::var("SIEVE_REQUEST_ID").ok());

    match explicit_id {
        Some(id_str) => {
            let request_id = match Uuid::parse_str(&id_str) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("sieve-hook: invalid request ID `{id_str}`: {e}");
                    std::process::exit(1);
                }
            };
            run(request_id, &base)
        }
        None => {
            // 优先级 3：启发式扫目录。
            run_heuristic(&base)
        }
    }
}

/// `codex` 子命令：Codex PreToolUse hook。
///
/// 读 stdin 的工具调用 JSON → 经极简 IPC client 让 daemon 判危 → 按 codex 契约输出。
/// **任何错误路径（读失败 / 解析失败 / daemon 不可达 / 超时）一律 `emit_codex_fail_closed`
/// （exit 2），绝不 exit 1（codex 下 = 放行）。**
fn run_codex_command(sieve_home: Option<PathBuf>) -> i32 {
    let base = match resolve_sieve_home(sieve_home) {
        Some(b) => b,
        None => {
            return emit_codex(emit_codex_fail_closed(
                "cannot determine sieve home ($HOME not set), fail-closed",
            ));
        }
    };

    // 内部硬 deadline：到点主动 fail-closed，抢在 codex `timeout_sec` 强杀（=fail-open）之前。
    let deadline = Instant::now() + Duration::from_secs(CODEX_INTERNAL_DEADLINE_SECS);

    // 读 stdin 全部（codex 的 PreToolUseCommandInput）。
    let mut buf = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut buf) {
        return emit_codex(emit_codex_fail_closed(&format!(
            "read stdin failed: {e}, fail-closed"
        )));
    }

    // 解析 codex PreToolUse JSON（容忍未知/缺失字段；解析失败也 fail-closed，绝不 exit 1）。
    let input: CodexPreToolUseInput = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(e) => {
            return emit_codex(emit_codex_fail_closed(&format!(
                "parse stdin failed: {e}, fail-closed"
            )));
        }
    };

    // 经 daemon 判危。
    let verdict = match codex_ipc::judge_tool_call(
        &base,
        &input.tool_name,
        &input.tool_input,
        &input.tool_use_id,
        &input.cwd,
        deadline,
    ) {
        Ok(v) => v,
        Err(e) => {
            return emit_codex(emit_codex_fail_closed(&format!(
                "daemon judge failed: {e}, fail-closed"
            )));
        }
    };

    emit_codex(emit_codex_decision(&verdict))
}

/// 写 codex 输出三元组到 stdout/stderr，返回退出码。
fn emit_codex(out: CodexOutput) -> i32 {
    if !out.stdout.is_empty() {
        let mut so = io::stdout();
        let _ = so.write_all(out.stdout.as_bytes());
        let _ = so.flush();
    }
    if !out.stderr.is_empty() {
        let mut se = io::stderr();
        let _ = se.write_all(out.stderr.as_bytes());
        let _ = se.flush();
    }
    out.exit_code
}

/// `hermes` 子命令：Hermes Agent `pre_tool_call` hook。
///
/// 读 stdin 的 pre_tool_call JSON → 经 daemon 判危（复用 agent-neutral 的 [`codex_ipc::judge_tool_call`]）
/// → 按 Hermes 契约输出 stdout JSON。**任何错误路径一律 `emit_hermes_fail_closed`（尽力 block JSON）。**
///
/// ⚠️ Hermes **fail-OPEN**：退出码无效、超时即放行；本进程固定 `exit 0`，真正 fail-closed
/// 由 daemon 网关 `inbound_hold` 兜底（ADR-014 §6）。
fn run_hermes_command(sieve_home: Option<PathBuf>) -> i32 {
    let base = match resolve_sieve_home(sieve_home) {
        Some(b) => b,
        None => {
            return emit_hermes(emit_hermes_fail_closed(
                "cannot determine sieve home ($HOME not set), fail-closed",
            ));
        }
    };

    // 内部硬 deadline：到点前输出 block JSON，抢在 Hermes hook timeout（默认 60s）放行之前。
    let deadline = Instant::now() + Duration::from_secs(HERMES_INTERNAL_DEADLINE_SECS);

    // 读 stdin 全部（Hermes pre_tool_call 输入）。
    let mut buf = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut buf) {
        return emit_hermes(emit_hermes_fail_closed(&format!(
            "read stdin failed: {e}, fail-closed"
        )));
    }

    // 解析 Hermes pre_tool_call JSON（容忍未知/缺失字段；解析失败也尽力 block）。
    let input: HermesPreToolCallInput = match serde_json::from_str(&buf) {
        Ok(v) => v,
        Err(e) => {
            return emit_hermes(emit_hermes_fail_closed(&format!(
                "parse stdin failed: {e}, fail-closed"
            )));
        }
    };

    // 复用 codex 的 agent-neutral 判危 IPC。Hermes 无 `cwd`，传空串（daemon 对 tool_input
    // 全文扫描不依赖 cwd）；用 `session_id` 作 daemon 请求关联键（代替 codex 的 tool_use_id）。
    let verdict = match codex_ipc::judge_tool_call(
        &base,
        &input.tool_name,
        &input.tool_input,
        &input.session_id,
        "",
        deadline,
    ) {
        Ok(v) => v,
        Err(e) => {
            return emit_hermes(emit_hermes_fail_closed(&format!(
                "daemon judge failed: {e}, fail-closed"
            )));
        }
    };

    emit_hermes(emit_hermes_decision(&verdict))
}

/// 写 Hermes 输出到 stdout/stderr，**固定返回 `exit 0`**（Hermes 退出码不影响 block，
/// 决策完全经 stdout JSON 传达；非零退出在 Hermes = fail-open，更糟）。
fn emit_hermes(out: HermesOutput) -> i32 {
    if !out.stdout.is_empty() {
        let mut so = io::stdout();
        let _ = so.write_all(out.stdout.as_bytes());
        let _ = so.flush();
    }
    if !out.stderr.is_empty() {
        let mut se = io::stderr();
        let _ = se.write_all(out.stderr.as_bytes());
        let _ = se.flush();
    }
    0
}

/// 核心逻辑，返回进程退出码（0 = 允许，1 = 拒绝）。
///
/// 关联：SPEC-001 §4（hook 决策流程）。
fn run(request_id: Uuid, base: &std::path::Path) -> i32 {
    let req = match read_pending_checked(request_id, base, STALE_THRESHOLD_SECS) {
        Ok(r) => r,
        Err(PendingError::NotFound) => {
            // fail-open：Sieve 代理未标记此请求，放行。
            return 0;
        }
        Err(PendingError::Stale) => {
            eprintln!("sieve-hook: pending request is stale (> 10 min), blocking.");
            return 1;
        }
        Err(PendingError::ParseError(e)) => {
            eprintln!("sieve-hook: failed to parse pending file: {e}");
            return 1;
        }
        Err(PendingError::IoError(e)) => {
            eprintln!("sieve-hook: IO error reading pending file: {e}");
            return 1;
        }
    };

    // 打印危险摘要（SPEC-002 §2：多 issue 合并风格）。
    print_summary(&req);

    // 倒计时交互。
    let outcome = prompt_user(&req);

    // 写决策文件。
    if let Err(e) = write_decision(request_id, &outcome, base) {
        eprintln!("sieve-hook: failed to write decision: {e}");
    }

    match outcome {
        DecisionOutcome::Allow => 0,
        DecisionOutcome::Deny => 1,
    }
}

/// 打印危险摘要到 stderr（TTY 终端显示）。
///
/// 关联：SPEC-002 §2.1（多 issue 合并显示）。
fn print_summary(req: &protocol::DecisionRequest) {
    let n = req.detections.len();
    eprintln!();
    eprintln!("┌─ Sieve 安全警告 ({n} 条检测) ────────────────────────────────");
    for (i, det) in req.detections.iter().enumerate() {
        let severity_tag = match det.severity.as_str() {
            "critical" => "CRITICAL",
            "high" => "HIGH    ",
            "medium" => "MEDIUM  ",
            _ => "LOW     ",
        };
        eprintln!(
            "│ [{:2}] [{severity_tag}] {} — {}",
            i + 1,
            det.rule_id,
            det.title
        );
        eprintln!("│       {}", det.one_line_summary);
    }
    eprintln!("└────────────────────────────────────────────────────────────");
    eprintln!();
}

/// TTY 倒计时交互，返回用户决策。
///
/// - 输入 `y`/`Y` → Allow（exit 0）
/// - 输入 `n`/`N`/回车（默认拒绝）→ Deny（exit 1）
/// - 倒计时到 → 按 default_on_timeout 决定
///
/// 用 `spawn thread + mpsc channel` 实现非阻塞输入，避免引入 tokio。
fn prompt_user(req: &protocol::DecisionRequest) -> DecisionOutcome {
    let timeout = Duration::from_secs(req.timeout_seconds as u64);
    let deadline = std::time::Instant::now() + timeout;

    let stdin = io::stdin();
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = stdin.lock().read_line(&mut line);
        let _ = tx.send(line);
    });

    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        eprint!(
            "\r允许此操作？[y/N]（{} 秒后默认{}） > ",
            remaining.as_secs(),
            default_label(req.default_on_timeout)
        );
        let _ = io::stderr().flush();

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => {
                eprintln!();
                return match line.trim().to_lowercase().as_str() {
                    "y" => DecisionOutcome::Allow,
                    _ => DecisionOutcome::Deny,
                };
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if std::time::Instant::now() >= deadline {
                    eprintln!();
                    return match req.default_on_timeout {
                        protocol::DefaultOnTimeout::Allow => DecisionOutcome::Allow,
                        _ => DecisionOutcome::Deny,
                    };
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                eprintln!();
                return DecisionOutcome::Deny;
            }
        }
    }
}

fn default_label(dot: protocol::DefaultOnTimeout) -> &'static str {
    match dot {
        protocol::DefaultOnTimeout::Allow => "允许",
        _ => "拒绝",
    }
}

/// 启发式路径：无 request_id 时扫目录。
///
/// - corrupt 文件非空 → fail-closed（exit 1）：无法确认 Sieve 判定，保守拒绝
/// - 零 fresh pending → fail-open（exit 0）
/// - stale 文件 → 删除 + warn + fail-open（exit 0）
/// - 有 fresh pending → 合并显示所有 detection，TTY 弹窗确认，广播决策
///
/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）；known-issues-v1.4.md §P1-R3-#6。
fn run_heuristic(base: &std::path::Path) -> i32 {
    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);

    // 损坏文件优先检查：只要有损坏文件，立即 fail-closed，不管 fresh 有没有。
    // 因为损坏文件可能对应本次工具调用的 Sieve 拦截标记，无法安全放行。
    // 关联：P1-R3-#6（corrupt → fail-open 漏洞修复，与 lib::run_check_heuristic 行为对齐）。
    if !scan.corrupt_paths.is_empty() {
        eprintln!(
            "sieve-hook: pending file corrupt ({} files), refusing tool call to be safe",
            scan.corrupt_paths.len()
        );
        for p in &scan.corrupt_paths {
            eprintln!("  - {}", p.display());
        }
        return 1;
    }

    // 删除 stale 文件 + 打 warning。
    for stale_path in &scan.stale_paths {
        eprintln!(
            "sieve-hook: warning: stale pending file deleted: {}",
            stale_path.display()
        );
        let _ = std::fs::remove_file(stale_path);
    }

    if scan.fresh.is_empty() {
        // 零 pending（corrupt=[]，stale 已清理）：Sieve 代理未标记任何请求，fail-open。
        return 0;
    }

    // 合并所有 detection 到一个"虚拟"请求以统一显示。
    // timeout_seconds 和 default_on_timeout 取最严的策略（任一 Block/Redact → Deny）。
    let merged = merge_requests(&scan.fresh);
    print_summary(&merged);
    let outcome = prompt_user(&merged);

    // 广播决策给所有 pending request_id。
    for req in &scan.fresh {
        if let Err(e) = write_decision(req.request_id, &outcome, base) {
            eprintln!(
                "sieve-hook: failed to write decision for {}: {e}",
                req.request_id
            );
        }
    }

    match outcome {
        DecisionOutcome::Allow => 0,
        DecisionOutcome::Deny => 1,
    }
}

/// 合并多个 DecisionRequest 的 detection，取最严 default_on_timeout。
fn merge_requests(reqs: &[protocol::DecisionRequest]) -> protocol::DecisionRequest {
    let mut all_detections = Vec::new();
    let mut worst_timeout = protocol::DefaultOnTimeout::Allow;
    let mut min_timeout_secs = u32::MAX;

    for req in reqs {
        all_detections.extend(req.detections.clone());
        // 最严策略：Block/Redact > Allow。
        match req.default_on_timeout {
            protocol::DefaultOnTimeout::Allow => {}
            other => worst_timeout = other,
        }
        if req.timeout_seconds < min_timeout_secs {
            min_timeout_secs = req.timeout_seconds;
        }
    }

    let timeout_secs = if min_timeout_secs == u32::MAX {
        30
    } else {
        min_timeout_secs
    };

    protocol::DecisionRequest {
        // 启发式合并场景使用第一个请求的 id（仅用于日志）。
        request_id: reqs[0].request_id,
        created_at: reqs[0].created_at,
        timeout_seconds: timeout_secs,
        default_on_timeout: worst_timeout,
        detections: all_detections,
    }
}
