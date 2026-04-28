// sieve-hook: Claude Code PreToolUse hook 二进制。
//
// 夹在 Claude Code tool_use 调用与实际执行之间，对命中 Critical 规则的工具调用
// 在 TTY 显示危险摘要并等待用户确认。
//
// 启动时延目标 < 50ms（依赖仅 serde_json + fd-lock + clap，无 tokio / vectorscan）。
// 关联：SPEC-001（hook 文件协议）、SPEC-002（弹窗行为规范）、ADR-014（双层防御）。

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use uuid::Uuid;

// 从 lib target 引入共享模块，避免重复定义。
use sieve_hook_lib::decision::{write_decision, DecisionOutcome};
use sieve_hook_lib::error::PendingError;
use sieve_hook_lib::pending::{read_pending_checked, scan_pending_dir};
use sieve_hook_lib::protocol;

const STALE_THRESHOLD_SECS: i64 = 600;

/// sieve-hook: PreToolUse 安全确认 hook（Phase 1 macOS）。
#[derive(Parser, Debug)]
#[command(name = "sieve-hook", about = "Sieve PreToolUse safety hook")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// 检查 pending 决策请求并请求用户确认。
    Check {
        /// 决策请求 ID（UUID）；未传则读 $SIEVE_REQUEST_ID。
        #[arg(long)]
        request_id: Option<String>,

        /// sieve home 目录；未传则读 $SIEVE_HOME，默认 $HOME/.sieve。
        #[arg(long)]
        sieve_home: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();
    let Command::Check {
        request_id,
        sieve_home,
    } = cli.command;

    // 解析 sieve_home：flag > env > default。
    let base = sieve_home
        .or_else(|| std::env::var("SIEVE_HOME").ok().map(PathBuf::from))
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".sieve"))
        })
        .unwrap_or_else(|| {
            eprintln!("sieve-hook: cannot determine sieve home directory ($HOME not set)");
            std::process::exit(1);
        });

    // 解析 request_id：优先级 1（flag）> 优先级 2（env）> 优先级 3（启发式扫目录）。
    // 优先级 3 是关键修复：Claude Code settings.json 注册静态命令时无法传 request_id，
    // 必须走启发式路径；零 pending 时 fail-open（exit 0），不阻断正常工具调用。
    // 关联：SPEC-001 §4.3（启发式查 pending 目录）。
    let explicit_id = request_id.or_else(|| std::env::var("SIEVE_REQUEST_ID").ok());

    let exit_code = match explicit_id {
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
    };

    std::process::exit(exit_code);
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
/// - 零 fresh pending → fail-open（exit 0）
/// - stale 文件 → 删除 + warn + fail-open（exit 0）
/// - 有 fresh pending → 合并显示所有 detection，TTY 弹窗确认，广播决策
///
/// 关联：SPEC-001 §4.3（启发式查 pending 目录最新文件）。
fn run_heuristic(base: &std::path::Path) -> i32 {
    let scan = scan_pending_dir(base, STALE_THRESHOLD_SECS);

    // 删除 stale 文件 + 打 warning。
    for stale_path in &scan.stale_paths {
        eprintln!(
            "sieve-hook: warning: stale pending file deleted: {}",
            stale_path.display()
        );
        let _ = std::fs::remove_file(stale_path);
    }

    if scan.fresh.is_empty() {
        // 零 pending：Sieve 代理未标记任何请求，fail-open。
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
