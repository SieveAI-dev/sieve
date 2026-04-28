//! `sieve doctor` 命令实现（ADR-015 / SPEC-003 §doctor）。
//!
//! 5 项检查：
//! 1. settings.json 中 ANTHROPIC_BASE_URL 是否为 http://127.0.0.1:11453
//! 2. hooks.PreToolUse 是否含 sieve-hook check
//! 3. daemon 是否在 :11453 监听（TCP 连接）
//! 4. launchd 状态（launchctl list | grep com.sieve.daemon）
//! 5. canary 本地引擎命中测试（OUT-01 规则 scan，不发真实网络请求）
//!
//! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。
//!
//! # R4-#7 修复说明
//!
//! 原实现向 daemon 发 HTTP 请求，检查响应里**不含**原始 canary token。
//! 该逻辑存在误报通过漏洞：daemon 未拦截（401/502 透传）时响应同样不含 canary token。
//!
//! 新实现改为**直接调用本地 sieve-rules 引擎**对 canary token 做 scan，
//! 确认规则引擎确实命中 OUT-01，不依赖 daemon 是否在线。
//! 同时独立检查 daemon TCP 监听（检查 3）。
//! 输出明确区分「规则引擎命中」与「daemon 在线」两个状态。
//!
//! # R4-#8 修复说明
//!
//! 原实现任一检查失败仍返回 `Ok(())`，导致 CI 假绿灯。
//! 新实现收集所有失败项，任一失败则返回 `Err`，含失败项名称列表。

use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::process::Command;

    /// 运行 `sieve doctor`。关联 ADR-015 / SPEC-003 §doctor。
    ///
    /// # Errors
    ///
    /// 任一检查项失败时返回 `Err`，错误信息含失败项名称列表（R4-#8）。
    pub fn run() -> Result<()> {
        let home = std::env::var("HOME").unwrap_or_default();
        let settings_path = std::path::PathBuf::from(&home)
            .join(".claude")
            .join("settings.json");

        // 收集每项检查的结果 (label, passed)
        let mut results: Vec<(&str, bool)> = Vec::new();

        // ── 检查 1: ANTHROPIC_BASE_URL
        let check1 = check_base_url(&settings_path);
        print_check(
            "settings.json: ANTHROPIC_BASE_URL = http://127.0.0.1:11453",
            check1,
        );
        results.push(("ANTHROPIC_BASE_URL 配置", check1));

        // ── 检查 2: PreToolUse hook
        let check2 = check_hook_registered(&settings_path);
        print_check(
            "settings.json: hooks.PreToolUse 含 sieve-hook check",
            check2,
        );
        results.push(("PreToolUse hook 配置", check2));

        // ── 检查 3: daemon 监听 :11453
        let check3 = check_daemon_listening();
        print_check("daemon 在 127.0.0.1:11453 监听", check3);
        results.push(("daemon 监听 :11453", check3));

        // ── 检查 4: launchd 状态
        let check4 = check_launchd();
        print_check("launchd com.sieve.daemon 已加载", check4);
        results.push(("launchd 服务已加载", check4));

        // ── 检查 5: canary 本地引擎命中测试（R4-#7 修复）
        //
        // 直接调用本地 sieve-rules 引擎扫描 canary token，
        // 确认 OUT-01 规则确实命中。不发真实网络请求，不依赖 daemon 是否在线。
        // 输出明确说明「仅验证规则引擎 + daemon listening；端到端验证需手动测」。
        let check5 = check_canary_local_engine();
        print_check(
            "canary 本地规则引擎命中 OUT-01（注：端到端需手动验证）",
            check5,
        );
        results.push(("canary 规则引擎命中 OUT-01", check5));

        // ── 汇总（R4-#8 修复）
        println!();
        let failures: Vec<&str> = results
            .iter()
            .filter_map(|(label, ok)| if *ok { None } else { Some(*label) })
            .collect();

        if failures.is_empty() {
            println!("✅ 所有检查通过，Sieve 运行正常。");
            Ok(())
        } else {
            println!("❌ 部分检查失败，请查看上方输出并运行 `sieve setup` 修复。");
            Err(anyhow::anyhow!(
                "{} 项检查失败：{}",
                failures.len(),
                failures.join("、")
            ))
        }
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

    /// Canary 本地规则引擎命中测试（R4-#7 修复）。
    ///
    /// 构造一个**精确匹配 OUT-01 规则格式**的 canary token，
    /// 直接调用 sieve-rules VectorscanEngine + 出站规则，验证至少 1 个 Detection 命中 OUT-01。
    ///
    /// 不发任何网络请求，不依赖 daemon 是否在线。
    ///
    /// # 为什么不发 HTTP 请求验证
    ///
    /// - daemon 不支持 runtime upstream override，无法将 canary 请求导向 fake upstream
    /// - 向真实 upstream 发请求需要有效 API key，doctor 不应持有密钥
    /// - 401/502 响应同样不含 canary token → 原逻辑误判通过（R4-#7 根本原因）
    /// - 本地引擎 scan 已足以验证检测链路最关键的一环（规则编译 + pattern 匹配）
    fn check_canary_local_engine() -> bool {
        use sieve_rules::engine::{MatchEngine as _, VectorscanEngine};
        use sieve_rules::loader::load_outbound_rules;

        // 定位 outbound.toml：相对二进制路径推断，或 fallback 到 workspace 路径。
        // 在测试环境中，从 CARGO_MANIFEST_DIR 推断；生产环境从二进制同级目录推断。
        let rules_candidates: Vec<std::path::PathBuf> = vec![
            // 生产：~/.sieve/rules/outbound.toml
            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_default())
                .join(".sieve")
                .join("rules")
                .join("outbound.toml"),
            // 开发：workspace 相对路径（通过 SIEVE_RULES_PATH 覆盖）
            std::path::PathBuf::from(std::env::var("SIEVE_RULES_PATH").unwrap_or_default()),
        ];

        let rules_path = rules_candidates
            .into_iter()
            .find(|p| !p.as_os_str().is_empty() && p.exists());

        let Some(rules_path) = rules_path else {
            // 规则文件不存在：canary 检查无法执行
            return false;
        };

        let Ok(rules) = load_outbound_rules(&rules_path) else {
            return false;
        };

        let Ok(engine) = VectorscanEngine::compile(rules) else {
            return false;
        };

        // 构造精确匹配 OUT-01 pattern `sk-ant-api03-[a-zA-Z0-9_\-]{93}AA` 的 canary token。
        // body = "canaryDOCTOR" (12) + "test" (4) + 'a'*77 = 93 字符，后跟 "AA"。
        // 整体格式符合真实 Anthropic API key 结构，确保 OUT-01 命中而非误判。
        let canary_token = "sk-ant-api03-canaryDOCTORtestaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaAA";

        let Ok(hits) = engine.scan(canary_token.as_bytes()) else {
            return false;
        };

        hits.iter().any(|h| h.rule_id == "OUT-01")
    }
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
