//! `sieve verify` 子命令（ADR-043）——红队 bypass 测试集的 headless 驱动入口。
//!
//! **headless：无 GUI、无网络。** 红队测试本身全程 hermetic（mock 上游 + 真 daemon，
//! `SIEVE_NO_UPDATE=1` / `SIEVE_NO_TELEMETRY=1`）。本命令只负责编排：驱动红队测试、
//! 按类别（inbound / outbound）汇总通过/失败，退出码供 CI 判定。
//!
//! **红队集是已知攻击手法的回归基线，不是检测能力的完备性证明。**
//! 规则包缺失时红队测试优雅 SKIP（不 panic、不 fail）；本命令把「全部 SKIP」如实
//! 报告并以退出码 0 收尾——公开仓无签名规则包时不误红。
//!
//! 实现取向：shell out 跑 `cargo test -p sieve-cli --test <name>`（或 `cargo nextest
//! run`，见 `--nextest`），逐类别捕获退出码。保持极简可编译，不内联断言逻辑。

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::cli::{VerifyArgs, VerifyCommand};

/// 红队测试类别（test target 名 → 人类可读标签）。
const REDTEAM_TARGETS: &[(&str, &str)] = &[
    (
        "redteam_inbound",
        "入站红队（地址替换 / 危险 shell × 四路由）",
    ),
    ("redteam_outbound", "出站红队（BIP39 / WIF / xprv 脱敏）"),
];

/// 从本二进制所在路径推断 workspace 根（含 Cargo.toml）。
///
/// 二进制通常在 `<root>/target/{debug,release}/sieve`；回溯两级取 root。
/// 推断失败时回退到当前工作目录（CI 一般在 workspace 根运行）。
fn workspace_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        // <root>/target/<profile>/sieve → 回溯三级到 <root>
        if let Some(root) = exe
            .parent() // <profile>/
            .and_then(|p| p.parent()) // target/
            .and_then(|p| p.parent())
        // <root>/
        {
            if root.join("Cargo.toml").exists() {
                return root.to_path_buf();
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// `sieve verify` 入口。
pub fn run(args: VerifyArgs) -> Result<()> {
    match args.command {
        VerifyCommand::Redteam { nextest } => run_redteam(nextest),
    }
}

/// 单类别运行结果。
struct CategoryResult {
    label: &'static str,
    target: &'static str,
    /// 子进程退出码（None = 无法获取，按失败处理）。
    code: Option<i32>,
}

impl CategoryResult {
    fn passed(&self) -> bool {
        self.code == Some(0)
    }
}

fn run_redteam(use_nextest: bool) -> Result<()> {
    let root = workspace_root();
    let manifest = root.join("Cargo.toml");

    eprintln!("sieve verify redteam — ADR-043 红队 bypass 回归基线（非完备性证明）");
    eprintln!("workspace: {}", root.display());
    eprintln!(
        "runner: {}",
        if use_nextest {
            "cargo nextest run"
        } else {
            "cargo test"
        }
    );
    eprintln!("注：规则包缺失时红队测试优雅 SKIP（公开仓无签名规则包），退出码仍为 0。\n");

    let mut results: Vec<CategoryResult> = Vec::new();

    for (target, label) in REDTEAM_TARGETS {
        eprintln!("── 运行 {label} [{target}] ──");
        let mut cmd = Command::new("cargo");
        if use_nextest {
            cmd.args(["nextest", "run"]);
        } else {
            cmd.arg("test");
        }
        cmd.arg("--manifest-path")
            .arg(&manifest)
            .args(["-p", "sieve-cli", "--test", target]);
        // 透传测试 stdout/stderr（含 SKIP 行），供 CI 日志留痕。
        let status = cmd
            .status()
            .with_context(|| format!("启动 cargo 运行 {target} 失败"))?;
        results.push(CategoryResult {
            label,
            target,
            code: status.code(),
        });
        eprintln!();
    }

    // 汇总。
    eprintln!("── 红队汇总 ──");
    let mut all_ok = true;
    for r in &results {
        let mark = if r.passed() { "PASS" } else { "FAIL" };
        if !r.passed() {
            all_ok = false;
        }
        eprintln!(
            "  [{mark}] {} (exit={})",
            r.label,
            r.code.map(|c| c.to_string()).unwrap_or_else(|| "?".into())
        );
    }

    if all_ok {
        eprintln!("\n红队 bypass 回归基线全过 ✓（含规则缺失时的优雅 SKIP）");
        Ok(())
    } else {
        let failed: Vec<&str> = results
            .iter()
            .filter(|r| !r.passed())
            .map(|r| r.target)
            .collect();
        anyhow::bail!("红队 bypass 回归基线失败：{}", failed.join(", "))
    }
}
