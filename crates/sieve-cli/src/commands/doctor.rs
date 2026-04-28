//! `sieve doctor` 命令实现（ADR-015 / SPEC-003 §doctor）。
//!
//! 5 项检查：
//! 1. settings.json 中 ANTHROPIC_BASE_URL 是否为 http://127.0.0.1:11453
//! 2. hooks.PreToolUse 是否含 sieve-hook check
//! 3. daemon 是否在 :11453 监听（TCP 连接）
//! 4. launchd 状态（launchctl list | grep com.sieve.daemon）
//! 5. canary 拦截测试（构造 OUT-01 已知字符串，验证 daemon 改写）
//!
//! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。

use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::io::Write as IoWrite;
    use std::process::Command;

    /// 运行 `sieve doctor`。关联 ADR-015 / SPEC-003 §doctor。
    pub fn run() -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_default();
        let settings_path = std::path::PathBuf::from(&home)
            .join(".claude")
            .join("settings.json");

        let mut all_ok = true;

        // ── 检查 1: ANTHROPIC_BASE_URL
        let check1 = check_base_url(&settings_path);
        print_check(
            "settings.json: ANTHROPIC_BASE_URL = http://127.0.0.1:11453",
            check1,
        );
        all_ok &= check1;

        // ── 检查 2: PreToolUse hook
        let check2 = check_hook_registered(&settings_path);
        print_check(
            "settings.json: hooks.PreToolUse 含 sieve-hook check",
            check2,
        );
        all_ok &= check2;

        // ── 检查 3: daemon 监听 :11453
        let check3 = check_daemon_listening();
        print_check("daemon 在 127.0.0.1:11453 监听", check3);
        all_ok &= check3;

        // ── 检查 4: launchd 状态
        let check4 = check_launchd();
        print_check("launchd com.sieve.daemon 已加载", check4);
        all_ok &= check4;

        // ── 检查 5: canary 拦截测试
        let check5 = check_canary();
        print_check("canary 拦截测试（OUT-01 脱敏）", check5);
        all_ok &= check5;

        // ── 汇总
        println!();
        if all_ok {
            println!("✅ 所有检查通过，Sieve 运行正常。");
        } else {
            println!("❌ 部分检查失败，请查看上方输出并运行 `sieve setup` 修复。");
        }

        Ok(())
    }

    fn print_check(label: &str, ok: bool) {
        let icon = if ok { "✅" } else { "❌" };
        println!("  {} {}", icon, label);
    }

    /// 检查 settings.json 中 ANTHROPIC_BASE_URL。
    fn check_base_url(path: &std::path::Path) -> bool {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return false;
        };
        let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(&raw) else {
            return false;
        };
        v.pointer("/env/ANTHROPIC_BASE_URL")
            .and_then(|x| x.as_str())
            .map(|s| s == "http://127.0.0.1:11453")
            .unwrap_or(false)
    }

    /// 检查 PreToolUse hook 是否含 sieve-hook check。
    fn check_hook_registered(path: &std::path::Path) -> bool {
        let Ok(raw) = std::fs::read_to_string(path) else {
            return false;
        };
        let Ok(v): Result<serde_json::Value, _> = serde_json::from_str(&raw) else {
            return false;
        };
        v.pointer("/hooks/PreToolUse")
            .and_then(|arr| arr.as_array())
            .map(|arr| {
                arr.iter().any(|item| {
                    item.pointer("/hooks/0/command")
                        .and_then(|c| c.as_str())
                        .map(|c| c.contains("sieve-hook"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    /// 尝试 TCP 连接 127.0.0.1:11453，成功则 daemon 在监听。
    fn check_daemon_listening() -> bool {
        use std::net::TcpStream;
        use std::time::Duration;
        TcpStream::connect_timeout(
            &"127.0.0.1:11453".parse().unwrap(),
            Duration::from_millis(500),
        )
        .is_ok()
    }

    /// 检查 launchctl list 是否含 com.sieve.daemon。
    fn check_launchd() -> bool {
        let Ok(output) = Command::new("launchctl").arg("list").output() else {
            return false;
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("com.sieve.daemon")
    }

    /// Canary 拦截测试：向 daemon 发送含 OUT-01 特征的请求，
    /// 验证响应中已脱敏（不含原始 sk- token）。
    ///
    /// 注意：此测试仅在 daemon 运行时有意义；daemon 未运行时直接返回 false。
    fn check_canary() -> bool {
        use std::io::{Read, Write};
        use std::net::TcpStream;
        use std::time::Duration;

        // daemon 未运行直接 false
        let Ok(mut stream) = TcpStream::connect_timeout(
            &"127.0.0.1:11453".parse().unwrap(),
            Duration::from_millis(500),
        ) else {
            return false;
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));

        // 构造含已知 OUT-01 特征（sk-ant-api03-... 格式）的请求体
        // 注意：这里使用测试用虚假 token，格式符合 OUT-01 模式
        let canary_token = "sk-ant-api03-canary-test-aaaabbbbccccdddd-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX_AA";
        let body = serde_json::json!({
            "model": "claude-3-5-haiku-20241022",
            "max_tokens": 1,
            "messages": [{
                "role": "user",
                "content": format!("hello {canary_token}")
            }]
        })
        .to_string();

        let request = format!(
            "POST /v1/messages HTTP/1.1\r\n\
             Host: 127.0.0.1:11453\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             x-api-key: test\r\n\
             anthropic-version: 2023-06-01\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            body.len(),
            body
        );

        if stream.write_all(request.as_bytes()).is_err() {
            return false;
        }

        let mut response = String::new();
        let _ = stream.read_to_string(&mut response);

        // 验证响应中不含原始 canary token（已被脱敏/拦截）
        !response.contains(canary_token)
    }

    // 抑制 IoWrite 未使用警告
    const _: fn() = || {
        let _ = std::io::stdout().flush();
    };
}

// ──────────────────────────────── 非 macOS stub ─────────────────────────────

#[cfg(not(target_os = "macos"))]
mod stub {
    use super::*;

    /// `sieve doctor` 非 macOS 占位实现。
    pub fn run() -> Result<()> {
        anyhow::bail!(
            "sieve doctor is macOS only in Phase 1. \
             Linux/Windows support is planned for Phase 2."
        )
    }
}
