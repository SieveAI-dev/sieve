//! `sieve doctor` 命令实现（ADR-015 / SPEC-003 §doctor / SPEC-004 §6）。
//!
//! 6 项检查（Claude Code）：
//! 1. settings.json 中 ANTHROPIC_BASE_URL 是否为 http://127.0.0.1:11453
//! 2. hooks.PreToolUse 是否含 sieve-hook check
//! 3. daemon 是否在 :11453 监听（TCP 连接）
//! 4. launchd 状态（launchctl list | grep com.sieve.daemon）
//! 5. canary 本地引擎命中测试（OUT-01 规则 scan，不发真实网络请求）
//! 6. sieve-hook 二进制是否与 sieve 同目录存在（hook 注册但二进制漏装的缺口）
//!
//! `--agent openclaw` / `--agent hermes` 通过 setup::macos 桥接函数调用真实 adapter（R10-#5）。
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
//!
//! # R5-#2 修复说明
//!
//! 原实现 canary 规则路径列表硬编码，只看 `$HOME/.sieve/rules/outbound.toml`，
//! 不读 `SIEVE_HOME` env var / `sieve.toml` 的 `rules_path` 字段。
//!
//! 新实现通过 `resolve_rules_path()` 按 4 级优先级解析：
//! 1. `SIEVE_RULES_PATH` env var（显式覆盖，dev/CI 用）
//! 2. `$SIEVE_HOME/sieve.toml`（或 `~/.sieve/sieve.toml`）中的 `rules_path` 字段
//! 3. `$SIEVE_HOME/rules/outbound.toml`（env var 指定的 sieve home）
//! 4. `$HOME/.sieve/rules/outbound.toml`（最终 fallback）
//!
//! # R10-#5 修复说明
//!
//! 原实现 openclaw/hermes 分支调用本地 stub（只打印"⚠ 为 stub"），
//! all_passed 保持 true → 假绿。
//!
//! 新实现通过 setup::macos 中的 run_openclaw_doctor_check / run_hermes_doctor_check
//! 桥接函数调用真实 OpenClawAdapter::doctor_check / HermesAdapter::doctor_check：
//! - 先 detect 确认是否安装，未安装则跳过（友好提示）
//! - 安装了但检查失败 → run() 返回 Err，exit 1

use crate::cli::{AgentKind, DoctorArgs};
use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::path::PathBuf;
    use std::process::Command;

    /// 按 4 级优先级解析出站规则路径（R5-#2）。
    ///
    /// 优先级（高 → 低）：
    /// 1. `SIEVE_RULES_PATH` env var（显式覆盖，dev/CI 用）
    /// 2. `$SIEVE_HOME/sieve.toml`（或 `~/.sieve/sieve.toml`）中的 `rules_path` 字段
    /// 3. `$SIEVE_HOME/rules/outbound.toml`（env var 指定的 sieve home）
    /// 4. `$HOME/.sieve/rules/outbound.toml`（最终 fallback）
    ///
    /// # Errors
    ///
    /// 所有候选路径均未找到有效文件时返回 `Err`，含每个候选尝试情况的说明。
    pub fn resolve_rules_path() -> Result<PathBuf> {
        // ── 优先级 1：SIEVE_RULES_PATH 显式覆盖 ────────────────────────────
        if let Ok(val) = std::env::var("SIEVE_RULES_PATH") {
            if !val.is_empty() {
                return Ok(PathBuf::from(val));
            }
        }

        // ── 优先级 2：从 sieve.toml 读 rules_path 字段 ────────────────���────
        let sieve_home = resolve_sieve_home();
        let toml_path = sieve_home.join("sieve.toml");
        if toml_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&toml_path) {
                // 只解析 rules_path 字段，容忍其他字段（避免引入 config::Config 循环依赖）
                if let Ok(table) = raw.parse::<toml::Table>() {
                    if let Some(toml::Value::String(p)) = table.get("rules_path") {
                        if !p.is_empty() {
                            return Ok(PathBuf::from(p));
                        }
                    }
                }
            }
        }

        // ── 优先级 3：$SIEVE_HOME/rules/outbound.toml ────────────��─────────
        let sieve_home_rules = sieve_home.join("rules").join("outbound.toml");

        // ── 优先级 4：$HOME/.sieve/rules/outbound.toml（fallback）──────────
        let home_rules = PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".sieve")
            .join("rules")
            .join("outbound.toml");

        // 优先级 3 和 4 可能相同（当 SIEVE_HOME 未设置时），只在文件存在时返回
        if sieve_home_rules.exists() {
            return Ok(sieve_home_rules);
        }
        if home_rules.exists() {
            return Ok(home_rules);
        }

        // 所有候选均失败：返回明确的 Err
        Err(anyhow::anyhow!(
            "出站规则文件未找到，尝试过的候选路径：\n\
             1. SIEVE_RULES_PATH（未设置或为空）\n\
             2. {toml} 中的 rules_path 字段（文件{toml_status}）\n\
             3. {sieve_home_rules}\n\
             4. {home_rules}",
            toml = toml_path.display(),
            toml_status = if toml_path.exists() {
                "存在但无 rules_path 字段"
            } else {
                "不存在"
            },
            sieve_home_rules = sieve_home_rules.display(),
            home_rules = home_rules.display(),
        ))
    }

    /// 解析 sieve home 目录：`$SIEVE_HOME` env var，否则 `$HOME/.sieve`。
    fn resolve_sieve_home() -> PathBuf {
        if let Ok(val) = std::env::var("SIEVE_HOME") {
            if !val.is_empty() {
                return PathBuf::from(val);
            }
        }
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".sieve")
    }

    /// 按 4 级优先级解析**入站**规则路径（ADR-041 canary 自检用）。
    ///
    /// 与 [`resolve_rules_path`]（出站）对称，但解析 `inbound_rules_path` 字段
    /// 与 `rules/inbound.toml` 文件：
    /// 1. `SIEVE_INBOUND_RULES_PATH` env var（显式覆盖，dev/CI 用）
    /// 2. `$SIEVE_HOME/sieve.toml`（或 `~/.sieve/sieve.toml`）的 `inbound_rules_path` 字段
    /// 3. `$SIEVE_HOME/rules/inbound.toml`
    /// 4. `$HOME/.sieve/rules/inbound.toml`（最终 fallback）
    ///
    /// # Errors
    ///
    /// 所有候选路径均未找到有效文件时返回 `Err`（canary 自检据此优雅降级，不 fail）。
    fn resolve_inbound_rules_path() -> Result<PathBuf> {
        if let Ok(val) = std::env::var("SIEVE_INBOUND_RULES_PATH") {
            if !val.is_empty() {
                return Ok(PathBuf::from(val));
            }
        }

        let sieve_home = resolve_sieve_home();
        let toml_path = sieve_home.join("sieve.toml");
        if toml_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&toml_path) {
                if let Ok(table) = raw.parse::<toml::Table>() {
                    if let Some(toml::Value::String(p)) = table.get("inbound_rules_path") {
                        if !p.is_empty() {
                            return Ok(PathBuf::from(p));
                        }
                    }
                }
            }
        }

        let sieve_home_rules = sieve_home.join("rules").join("inbound.toml");
        let home_rules = PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".sieve")
            .join("rules")
            .join("inbound.toml");

        if sieve_home_rules.exists() {
            return Ok(sieve_home_rules);
        }
        if home_rules.exists() {
            return Ok(home_rules);
        }

        Err(anyhow::anyhow!(
            "入站规则文件未找到（canary 自检需要）；候选：\n\
             1. SIEVE_INBOUND_RULES_PATH（未设置或为空）\n\
             2. {toml} 的 inbound_rules_path 字段\n\
             3. {sieve_home_rules}\n\
             4. {home_rules}",
            toml = toml_path.display(),
            sieve_home_rules = sieve_home_rules.display(),
            home_rules = home_rules.display(),
        ))
    }

    /// 运行 `sieve doctor`。关联 ADR-015 / SPEC-003 §doctor / SPEC-004 §6 / R10-#5。
    ///
    /// - `args.agent` 指定时只检查该 agent
    /// - 不传参数时：Claude 直接跑；OpenClaw/Hermes 先 detect，未装则跳过 + 友好提示
    ///
    /// # Errors
    ///
    /// 任一检查项失败时返回 `Err`，错误信息含失败项名称列表（R4-#8）。
    pub fn run(args: DoctorArgs) -> Result<()> {
        let home = PathBuf::from(std::env::var("HOME").unwrap_or_default());

        // 确定要检查的 agent 列表
        let agents: Vec<AgentKind> = if let Some(a) = args.agent {
            vec![a]
        } else {
            vec![
                AgentKind::Claude,
                AgentKind::Openclaw,
                AgentKind::Hermes,
                AgentKind::Codex,
            ]
        };

        let mut all_passed = true;

        for agent in &agents {
            match agent {
                AgentKind::Claude => {
                    if let Err(e) = run_claude_checks() {
                        eprintln!("[doctor] Claude Code 检查失败：{e}");
                        all_passed = false;
                    }
                }
                AgentKind::Openclaw => {
                    // R10-#5：调用真实 adapter，先 detect 确认是否安装
                    match run_openclaw_doctor(&home) {
                        RunAdapterResult::NotInstalled => {
                            println!(
                                "[doctor] ⚠ OpenClaw 未检测到安装，跳过检查（未找到 ~/.openclaw/ 或 openclaw 二进制）"
                            );
                        }
                        RunAdapterResult::Passed => {}
                        RunAdapterResult::Failed(e) => {
                            eprintln!("[doctor] OpenClaw 检查失败：{e}");
                            all_passed = false;
                        }
                    }
                }
                AgentKind::Hermes => {
                    // R10-#5：调用真实 adapter，先 detect 确认是否安装
                    match run_hermes_doctor(&home) {
                        RunAdapterResult::NotInstalled => {
                            println!(
                                "[doctor] ⚠ Hermes 未检测到安装，跳过检查（未找到 ~/.hermes/ 或 hermes 二进制）"
                            );
                        }
                        RunAdapterResult::Passed => {}
                        RunAdapterResult::Failed(e) => {
                            eprintln!("[doctor] Hermes 检查失败：{e}");
                            all_passed = false;
                        }
                    }
                }
                AgentKind::Codex => {
                    // 调用真实 CodexAdapter，先 detect 确认是否安装
                    match run_codex_doctor(&home) {
                        RunAdapterResult::NotInstalled => {
                            println!(
                                "[doctor] ⚠ Codex 未检测到安装，跳过检查（未找到 ~/.codex/ 或 codex 二进制）"
                            );
                        }
                        RunAdapterResult::Passed => {}
                        RunAdapterResult::Failed(e) => {
                            eprintln!("[doctor] Codex 检查失败：{e}");
                            all_passed = false;
                        }
                    }
                }
            }
        }

        // ── canary 诱饵自检（ADR-041 步骤 5；与具体 agent 无关，只跑一次）
        //
        // 优雅降级：诱饵缺失 / 规则包未安装均只打印诊断，**不**翻转 all_passed。
        // 诱饵是纵深防御补充，不是主防线；规则包随更新通道分发，本地可能尚未安装。
        report_canary_selfcheck(&home);

        if all_passed {
            Ok(())
        } else {
            Err(anyhow::anyhow!("doctor 检查未全部通过，见上方输出"))
        }
    }

    /// adapter doctor_check 的三态结果：未安装（跳过）/ 通过 / 失败。
    enum RunAdapterResult {
        NotInstalled,
        Passed,
        Failed(anyhow::Error),
    }

    /// 调用 setup::macos::run_openclaw_doctor_check 桥接函数（R10-#5：不再走 stub）。
    fn run_openclaw_doctor(home: &std::path::Path) -> RunAdapterResult {
        use crate::commands::setup::macos::run_openclaw_doctor_check;
        match run_openclaw_doctor_check(home.to_path_buf()) {
            None => RunAdapterResult::NotInstalled,
            Some(Ok(())) => RunAdapterResult::Passed,
            Some(Err(e)) => RunAdapterResult::Failed(e),
        }
    }

    /// 调用 setup::macos::run_hermes_doctor_check 桥接函数（R10-#5：不再走 stub）。
    fn run_hermes_doctor(home: &std::path::Path) -> RunAdapterResult {
        use crate::commands::setup::macos::run_hermes_doctor_check;
        match run_hermes_doctor_check(home.to_path_buf()) {
            None => RunAdapterResult::NotInstalled,
            Some(Ok(())) => RunAdapterResult::Passed,
            Some(Err(e)) => RunAdapterResult::Failed(e),
        }
    }

    /// 调用 setup::macos::run_codex_doctor_check 桥接函数。
    fn run_codex_doctor(home: &std::path::Path) -> RunAdapterResult {
        use crate::commands::setup::macos::run_codex_doctor_check;
        match run_codex_doctor_check(home.to_path_buf()) {
            None => RunAdapterResult::NotInstalled,
            Some(Ok(())) => RunAdapterResult::Passed,
            Some(Err(e)) => RunAdapterResult::Failed(e),
        }
    }

    /// Claude Code 5 项检查（SPEC-003 §doctor / SPEC-004 §6.1）。
    fn run_claude_checks() -> Result<()> {
        println!("=== Claude Code doctor 检查 ===");

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

        // ── 检查 3b（ADR-026 Stage F）: multi-listener 体检
        // 读 ~/.sieve/sieve.toml 解析 upstreams，逐个 TCP 探测。
        // 配置无 multi-listener（旧 schema 或无 sieve.toml）时跳过此项。
        let (total, ml_failures) = check_all_listeners_from_config();
        if total > 1 {
            // 仅 multi-listener 时打印（避免单 listener 重复 check 3 信息）
            let ml_passed = ml_failures.is_empty();
            let label = format!(
                "ADR-026 multi-listener 全部端口可达（{} 个 listener）",
                total
            );
            print_check(&label, ml_passed);
            if !ml_passed {
                println!("    失败的 listener: {}", ml_failures.join(", "));
            }
            results.push(("multi-listener 全部端口可达", ml_passed));
        }

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

        // ── 检查 6: sieve-hook 二进制就位（与 sieve 同目录存在）
        // 仅查 settings.json 字符串不够——hook 注册了但二进制没装（install.sh 漏装）会静默失效。
        let check6 = check_hook_binary_present();
        print_check("sieve-hook 二进制与 sieve 同目录存在", check6);
        results.push(("sieve-hook 二进制就位", check6));

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

    /// 检查 sieve-hook 二进制是否与当前 sieve 二进制同目录存在。
    ///
    /// install.sh 把 sieve + sieve-hook 装在同一目录；hook 注册用绝对路径
    /// （`setup::macos::sieve_hook_bin_path`）。只查 settings.json 字符串不足以发现
    /// 「hook 注册了但二进制漏装」——本检查补上这个缺口。
    fn check_hook_binary_present() -> bool {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("sieve-hook")))
            .map(|hook| hook.is_file())
            .unwrap_or(false)
    }

    /// 尝试 TCP 连接 127.0.0.1:11453，成功则 daemon 在监听。
    ///
    /// 兼容性检查：仅探测默认端口 11453。ADR-026 multi-listener 配置下，
    /// 用户可能配多个端口（如 11453 / 11454 / 11455），本函数只覆盖默认端口。
    /// 完整的 multi-listener 体检见 [`check_all_listeners_from_config`]。
    fn check_daemon_listening() -> bool {
        use std::net::TcpStream;
        use std::time::Duration;
        TcpStream::connect_timeout(
            &"127.0.0.1:11453".parse().unwrap(),
            Duration::from_millis(500),
        )
        .is_ok()
    }

    /// ADR-026 multi-listener 体检：读 sieve.toml 解析 upstreams，逐个 TCP 探测。
    ///
    /// 返回 (总数, 失败的 listener 列表)。配置文件不存在或解析失败时返回 (0, vec\[\])
    /// 表示 "没有 multi-listener 配置"，由调用方决定是否打印诊断。
    ///
    /// 关联：ADR-026 §决策 7 / Stage F doctor 升级。
    fn check_all_listeners_from_config() -> (usize, Vec<String>) {
        use std::net::TcpStream;
        use std::time::Duration;

        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return (0, vec![]),
        };
        let cfg_path = std::path::PathBuf::from(&home)
            .join(".sieve")
            .join("sieve.toml");

        let cfg = match crate::config::Config::load(&cfg_path) {
            Ok(c) => c,
            Err(_) => return (0, vec![]),
        };

        let upstreams = cfg.resolved_upstreams();
        let total = upstreams.len();
        let mut failures = Vec::new();
        for u in &upstreams {
            let addr_str = format!("127.0.0.1:{}", u.port);
            let addr = match addr_str.parse::<std::net::SocketAddr>() {
                Ok(a) => a,
                Err(_) => {
                    failures.push(format!("port {} (invalid addr)", u.port));
                    continue;
                }
            };
            let ok = TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok();
            let provider_id = u.resolved_provider_id();
            if !ok {
                failures.push(format!("port {} ({})", u.port, provider_id));
            }
        }
        (total, failures)
    }

    /// 检查 launchctl list 是否含 com.sieve.daemon。
    fn check_launchd() -> bool {
        let Ok(output) = Command::new("launchctl").arg("list").output() else {
            return false;
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains("com.sieve.daemon")
    }

    /// Canary 本地规则引擎命中测试（R4-#7 修复 / R5-#2 修复）。
    ///
    /// 构造一个**精确匹配 OUT-01 规则格式**的 canary token，
    /// 直接调用 sieve-rules VectorscanEngine + 出站规则，验证至少 1 个 Detection 命中 OUT-01。
    ///
    /// 不发任何网络请求，不依赖 daemon 是否在线。
    /// 规则路径通过 `resolve_rules_path()` 按 4 级优先级解析（R5-#2）。
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

        // R5-#2：按 4 级优先级解析规则路径（SIEVE_RULES_PATH > sieve.toml > SIEVE_HOME > HOME）
        let rules_path = match resolve_rules_path() {
            Ok(p) => {
                println!("  canary using rules from: {}", p.display());
                p
            }
            Err(e) => {
                println!("  canary 规则路径解析失败：{e}");
                return false;
            }
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

    /// Canary 诱饵自检（ADR-041 步骤 5）。
    ///
    /// 两部分，**全部本地完成、不发任何网络请求**（沿用检查 5 的「本地 scan canary
    /// token」范式）：
    ///
    /// 1. **布放体检**：诱饵文件是否在当前已存在的凭据 / 钱包目录里（复用 setup 的
    ///    [`canary_deploy_status`](crate::commands::setup::macos::canary_deploy_status)）。
    /// 2. **规则命中体检**：直接调本地 sieve-rules 引擎对固定 magic 串
    ///    [`CANARY_MAGIC`](crate::commands::setup::macos::CANARY_MAGIC) 做**入站**扫描，
    ///    断言命中 `IN-CR-CANARY`。
    ///
    /// **优雅降级**：诱饵缺失（用户删了 / 没跑 setup）或规则包未安装（签名规则包随
    /// 更新通道分发，本地可能尚未到位）均只打印诊断，**不让 doctor fail**——诱饵是
    /// 纵深防御补充，主防线（危险工具 gate）不依赖它。
    fn report_canary_selfcheck(home: &std::path::Path) {
        use crate::commands::setup::macos::{canary_deploy_status, CANARY_MAGIC};

        println!();
        println!("=== canary 诱饵自检（ADR-041）===");

        // ── ① 布放体检 ──────────────────────────────────────────────────────
        let (existing_dirs, deployed, missing) = canary_deploy_status(home);
        if existing_dirs == 0 {
            println!(
                "  ⚠ 未发现凭据 / 钱包目录（~/.ssh、~/.aws、~/.ethereum/keystore、\
                 ~/.config/solana 均不存在）——无处布放，跳过布放体检"
            );
        } else if missing == 0 {
            println!("  ✅ 诱饵已布放：{deployed}/{existing_dirs} 个已存在的敏感目录均就位");
        } else {
            println!(
                "  ⚠ 诱饵部分缺失：{deployed}/{existing_dirs} 已布放，{missing} 个目录缺诱饵\
                 （运行 `sieve setup` 补齐；删除诱饵不影响其他检测）"
            );
        }

        // ── ② 规则命中体检（本地 inbound 引擎 scan magic 串，不发网络）─────────
        match check_canary_rule_hits() {
            CanaryRuleCheck::Hit => {
                println!("  ✅ 本地入站规则引擎命中 IN-CR-CANARY（magic「{CANARY_MAGIC}」）");
            }
            CanaryRuleCheck::RulesAbsent(why) => {
                println!(
                    "  ⚠ 入站规则包未安装，跳过 IN-CR-CANARY 命中体检（优雅降级，不计失败）：{why}"
                );
            }
            CanaryRuleCheck::NoHit => {
                println!(
                    "  ⚠ 入站规则已加载但未命中 IN-CR-CANARY——规则包可能不含 canary 规则\
                     或版本过旧（优雅降级，不计失败）"
                );
            }
        }
    }

    /// canary 规则命中体检的三态结果。
    enum CanaryRuleCheck {
        /// 命中 IN-CR-CANARY。
        Hit,
        /// 规则文件 / 引擎不可用（规则包未安装等）——优雅降级。
        RulesAbsent(String),
        /// 规则已加载但未命中 IN-CR-CANARY——优雅降级。
        NoHit,
    }

    /// 直接调本地 sieve-rules 入站引擎对 magic 串 scan，判定是否命中 IN-CR-CANARY。
    ///
    /// 不发任何网络请求（ADR-041 §硬约束「绝不联网做 verifier」）。规则路径经
    /// [`resolve_inbound_rules_path`] 4 级解析；任一环节不可用返回 `RulesAbsent`。
    fn check_canary_rule_hits() -> CanaryRuleCheck {
        use crate::commands::setup::macos::CANARY_MAGIC;
        use sieve_rules::engine::{MatchEngine as _, VectorscanEngine};
        use sieve_rules::loader::load_inbound_rules;

        let rules_path = match resolve_inbound_rules_path() {
            Ok(p) => p,
            Err(e) => return CanaryRuleCheck::RulesAbsent(e.to_string()),
        };

        let rules = match load_inbound_rules(&rules_path) {
            Ok(r) => r,
            Err(e) => {
                return CanaryRuleCheck::RulesAbsent(format!(
                    "加载 {} 失败：{e}",
                    rules_path.display()
                ))
            }
        };

        let engine = match VectorscanEngine::compile(rules) {
            Ok(e) => e,
            Err(e) => return CanaryRuleCheck::RulesAbsent(format!("规则编译失败：{e}")),
        };

        match engine.scan(CANARY_MAGIC.as_bytes()) {
            Ok(hits) if hits.iter().any(|h| h.rule_id == "IN-CR-CANARY") => CanaryRuleCheck::Hit,
            Ok(_) => CanaryRuleCheck::NoHit,
            Err(e) => CanaryRuleCheck::RulesAbsent(format!("scan 失败：{e}")),
        }
    }
}

// ──────────────────────────────── 非 macOS stub ─────────────────────────────

#[cfg(not(target_os = "macos"))]
mod stub {
    use super::*;

    /// `sieve doctor` 非 macOS 占位实现。
    pub fn run(_args: DoctorArgs) -> Result<()> {
        anyhow::bail!(
            "sieve doctor is macOS only in Phase 1. \
             Linux/Windows support is planned for Phase 2."
        )
    }
}
