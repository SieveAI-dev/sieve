//! 驱动真实 sieve CLI 子命令（`audit query` / `decisions resolve` / `doctor` 等）。
//!
//! 用 [`crate::paths::sieve_binary`] 跑子命令并注入 `SIEVE_HOME`，让 harness 直接驱动真实
//! CLI，不重写 JSON-RPC。

use crate::paths::sieve_binary;
use std::path::Path;
use std::process::{Command, Output};

/// 用 sieve 二进制运行一个 CLI 子命令，返回 [`Output`]（含 status / stdout / stderr）。
///
/// `args` 是子命令参数，如 `["audit", "query", "--format", "jsonl"]` /
/// `["decisions", "resolve", id, "--block"]` / `["doctor"]`。
/// `env` 是额外环境变量（如 `SIEVE_HOME`）；同时默认注入 `SIEVE_NO_UPDATE=1` /
/// `SIEVE_NO_TELEMETRY=1`（ADR-030，可被 `env` 覆盖）。
///
/// # Panics
///
/// sieve 二进制不存在或进程 spawn 失败时 panic。
#[must_use]
pub fn run_sieve_cli(args: &[&str], env: &[(&str, &str)]) -> Output {
    let binary = sieve_binary();
    assert!(
        binary.exists(),
        "sieve binary not found at {}; run `cargo build` (or `--release`) first",
        binary.display()
    );

    let mut cmd = Command::new(&binary);
    cmd.args(args)
        .env("SIEVE_NO_UPDATE", "1")
        .env("SIEVE_NO_TELEMETRY", "1");
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.output().expect("run sieve cli")
}

/// 便捷封装：用指定 `SIEVE_HOME` 运行 CLI 子命令。
///
/// # Panics
///
/// 同 [`run_sieve_cli`]。
#[must_use]
pub fn run_sieve_cli_with_home(args: &[&str], sieve_home: &Path) -> Output {
    let home = sieve_home.to_string_lossy();
    run_sieve_cli(args, &[("SIEVE_HOME", home.as_ref())])
}
