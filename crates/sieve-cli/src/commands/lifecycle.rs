//! daemon 生命周期命令（`status` / `stop` / `restart`）。
//!
//! - `status`：调 `sieve.health` pretty 渲染 daemon 状态。
//! - `stop` / `restart`：launchd 封装。plist `KeepAlive=true` → SIGTERM 会被立即拉起，
//!   必须 `launchctl bootout`（老系统 fallback `launchctl unload`）。
//!
//! launchd 常量（Label / plist 路径）与 setup.rs 的 `build_plist_content` 保持一致
//! （Label = `com.sieve.daemon`，plist = `~/Library/LaunchAgents/com.sieve.daemon.plist`）。

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

use crate::cli::OutputFormat;
use crate::commands::ipc_client;

/// launchd job Label（与 setup.rs plist `Label` 一致）。
const LAUNCHD_LABEL: &str = "com.sieve.daemon";

/// launchd plist 路径 `~/Library/LaunchAgents/com.sieve.daemon.plist`。
fn plist_path() -> Result<PathBuf> {
    let home = dirs_home().context("获取 HOME 目录失败")?;
    Ok(home
        .join("Library")
        .join("LaunchAgents")
        .join("com.sieve.daemon.plist"))
}

/// 当前用户 HOME（不引入 dirs crate，读 `$HOME`）。
fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// 当前用户 UID（`launchctl` gui domain 需要）。用 `id -u`，避免 unsafe libc 调用。
fn current_uid() -> Result<String> {
    let out = Command::new("id")
        .arg("-u")
        .output()
        .context("执行 id -u 失败")?;
    if !out.status.success() {
        anyhow::bail!("id -u 返回非零");
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// launchd 服务目标标识 `gui/<uid>/com.sieve.daemon`。
fn service_target(uid: &str) -> String {
    format!("gui/{uid}/{LAUNCHD_LABEL}")
}

/// launchd job 是否已加载（`launchctl print <target>` exit 0）。
fn is_launchd_loaded(uid: &str) -> bool {
    Command::new("launchctl")
        .args(["print", &service_target(uid)])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ─────────────────────────── status ────────────────────────────────────────

/// `sieve status`：调 sieve.health pretty 渲染。daemon 不在线 → socket 路径 + exit 1。
pub async fn run_status(format: Option<OutputFormat>) -> Result<()> {
    let sock_path = ipc_client::ipc_socket_path()?;
    let result = match ipc_client::rpc_call_oneshot("sieve.health", serde_json::json!({})).await {
        Ok(r) => r,
        Err(_) => {
            eprintln!(
                "sieve status: daemon 未在线（无法连接 {}）",
                sock_path.display()
            );
            eprintln!("  检查 launchd job：launchctl print gui/$(id -u)/{LAUNCHD_LABEL}");
            eprintln!("  或前台启动：sieve start --config <path>");
            std::process::exit(1);
        }
    };

    if matches!(format, Some(OutputFormat::Jsonl)) {
        println!("{}", serde_json::to_string(&result)?);
        return Ok(());
    }

    // pretty 渲染。
    let version = result
        .get("daemon_version")
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let uptime = result
        .get("uptime_seconds")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let paused = result
        .get("paused")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let paused_until = result
        .get("paused_until")
        .and_then(|v| v.as_str())
        .unwrap_or("-");
    let preset_mode = result
        .get("preset")
        .and_then(|p| p.get("mode"))
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let inflight = result
        .get("ipc")
        .and_then(|i| i.get("total_decisions_inflight"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let clients = result
        .get("ipc")
        .and_then(|i| i.get("connected_clients"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let system_rules = result
        .get("rules")
        .and_then(|r| r.get("system_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let user_rules = result
        .get("rules")
        .and_then(|r| r.get("user_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let last_reload = result
        .get("rules")
        .and_then(|r| r.get("last_reload"))
        .and_then(|v| v.as_str())
        .unwrap_or("-");

    println!("sieve daemon 运行中");
    println!("  version:      {version}");
    println!("  uptime:       {uptime}s");
    println!(
        "  paused:       {}{}",
        paused,
        if paused {
            format!(" (until {paused_until})")
        } else {
            String::new()
        }
    );
    println!("  preset:       {preset_mode}");
    println!("  inflight:     {inflight} decisions");
    println!("  clients:      {clients}");
    println!("  rules:        system={system_rules} user={user_rules} last_reload={last_reload}");

    if let Some(listeners) = result.get("listeners").and_then(|v| v.as_array()) {
        println!("  listeners:");
        for l in listeners {
            let addr = l.get("addr").and_then(|v| v.as_str()).unwrap_or("?");
            let port = l.get("port").and_then(|v| v.as_u64()).unwrap_or(0);
            let provider = l.get("provider_id").and_then(|v| v.as_str()).unwrap_or("?");
            let protocol = l.get("protocol").and_then(|v| v.as_str()).unwrap_or("?");
            println!("    {addr}:{port} provider={provider} protocol={protocol}");
        }
    }
    Ok(())
}

// ─────────────────────────── stop / restart ────────────────────────────────

/// `sieve stop`：launchctl bootout（KeepAlive=true，SIGTERM 无效）。
pub async fn run_stop(yes: bool) -> Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = yes;
        eprintln!("sieve stop: 仅 macOS Phase 1 支持");
        std::process::exit(1);
    }

    #[cfg(target_os = "macos")]
    {
        let uid = current_uid()?;
        if !is_launchd_loaded(&uid) {
            eprintln!(
                "sieve stop: daemon 未被 launchd 管理（可能是前台 `sieve start`）；\
                 前台运行请用 Ctrl-C 停止"
            );
            std::process::exit(1);
        }

        if !yes && !confirm("将停止 launchd 管理的 sieve daemon。确认？输入 yes 继续：")?
        {
            eprintln!("已取消。");
            std::process::exit(1);
        }

        bootout(&uid)?;
        wait_socket_closed().await;
        println!("已停止 sieve daemon。");
        println!(
            "重新启动：launchctl bootstrap gui/{uid} {} 或 sieve setup",
            plist_display()?
        );
        Ok(())
    }
}

/// `sieve restart`：bootout + bootstrap/load（复用同一 launchd 封装）。
pub async fn run_restart(yes: bool) -> Result<()> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = yes;
        eprintln!("sieve restart: 仅 macOS Phase 1 支持");
        std::process::exit(1);
    }

    #[cfg(target_os = "macos")]
    {
        let uid = current_uid()?;
        let plist = plist_path()?;
        if !plist.exists() {
            eprintln!(
                "sieve restart: 未找到 launchd plist（{}）；请先 sieve setup",
                plist.display()
            );
            std::process::exit(1);
        }

        if !yes && !confirm("将重启 launchd 管理的 sieve daemon。确认？输入 yes 继续：")?
        {
            eprintln!("已取消。");
            std::process::exit(1);
        }

        if is_launchd_loaded(&uid) {
            bootout(&uid)?;
            wait_socket_closed().await;
        }
        bootstrap(&uid, &plist)?;
        println!("已重启 sieve daemon。");
        Ok(())
    }
}

/// `launchctl bootout gui/<uid>/com.sieve.daemon`，老系统 fallback `launchctl unload <plist>`。
#[cfg(target_os = "macos")]
fn bootout(uid: &str) -> Result<()> {
    let status = Command::new("launchctl")
        .args(["bootout", &service_target(uid)])
        .status()
        .context("执行 launchctl bootout 失败")?;
    if status.success() {
        return Ok(());
    }
    // fallback：老系统用 unload。
    let plist = plist_path()?;
    let status = Command::new("launchctl")
        .args(["unload", &plist.to_string_lossy()])
        .status()
        .context("执行 launchctl unload（fallback）失败")?;
    if !status.success() {
        anyhow::bail!("launchctl bootout / unload 均失败");
    }
    Ok(())
}

/// `launchctl bootstrap gui/<uid> <plist>`，老系统 fallback `launchctl load -w <plist>`。
#[cfg(target_os = "macos")]
fn bootstrap(uid: &str, plist: &std::path::Path) -> Result<()> {
    let status = Command::new("launchctl")
        .args(["bootstrap", &format!("gui/{uid}"), &plist.to_string_lossy()])
        .status()
        .context("执行 launchctl bootstrap 失败")?;
    if status.success() {
        return Ok(());
    }
    let status = Command::new("launchctl")
        .args(["load", "-w", &plist.to_string_lossy()])
        .status()
        .context("执行 launchctl load（fallback）失败")?;
    if !status.success() {
        anyhow::bail!("launchctl bootstrap / load 均失败");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn plist_display() -> Result<String> {
    Ok(plist_path()?.to_string_lossy().to_string())
}

/// 交互确认（从 stdin 读一行，等于 "yes" 才返回 true）。
#[cfg(target_os = "macos")]
fn confirm(prompt: &str) -> Result<bool> {
    use std::io::Write;
    eprint!("{prompt}");
    std::io::stderr().flush().ok();
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("读取确认输入失败")?;
    Ok(input.trim() == "yes")
}

/// 等待 socket 关闭（daemon 退出后 socket 文件不可连），最多 ~3s。
#[cfg(target_os = "macos")]
async fn wait_socket_closed() {
    use std::time::Duration;
    let Ok(sock) = ipc_client::ipc_socket_path() else {
        return;
    };
    for _ in 0..30 {
        if tokio::net::UnixStream::connect(&sock).await.is_err() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_target_format() {
        assert_eq!(service_target("501"), "gui/501/com.sieve.daemon");
    }

    #[test]
    fn plist_path_ends_correctly() {
        if std::env::var_os("HOME").is_none() {
            return;
        }
        let p = plist_path().expect("plist_path");
        assert!(p
            .to_string_lossy()
            .ends_with("Library/LaunchAgents/com.sieve.daemon.plist"));
    }
}
