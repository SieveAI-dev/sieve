//! Sieve CLI 入口（关联 PRD §6.1 / ADR-001）。
//!
//! 子命令：
//! - `sieve start [--config <path>] [--dry-run]`：启动 daemon
//! - `sieve version`：打印版本号
//! - `sieve setup [--agent <name>] [--all-detected] [--dry-run] [--yes]`：配置 AI agent（仅 macOS，ADR-015 / SPEC-004）
//! - `sieve doctor [--agent <name>] [--all]`：诊断 Sieve 安装状态（仅 macOS）
//! - `sieve uninstall [--agent <name>] [--all] [--dry-run] [--yes]`：回滚 setup 改动（仅 macOS）

// unsafe_code 在生产代码中禁止（等效 forbid），测试代码通过 #[allow(unsafe_code)] 豁免
// 以支持 Rust 1.80+ 的 std::env::set_var 必须用 unsafe {} 的要求。
#![deny(unsafe_code)]

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

mod audit;
mod cli;
mod commands;
mod config;
mod daemon;
mod daemon_control_plane;
mod embedded_rules;
mod engine_adapter;
pub mod process_context;
mod upstream_routes;

use audit::AuditStore;
use cli::{Cli, Command};
use engine_adapter::{InboundAdapter, OutboundAdapter};
use sieve_core::detection::DefaultOnTimeout;
use sieve_core::pipeline::inbound::AddressGuardConfig;
use sieve_core::pipeline::outbound::OutboundFilter;
use sieve_rules::engine::{LayeredEngine, MatchEngine, VectorscanEngine};
use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};

/// 入站规则中不送入 vectorscan 编译的占位 pattern 列表（R6-#6）。
///
/// IN-CR-01 使用 `__ADDRESS_GUARD_PLACEHOLDER__`，由运行时地址守卫逻辑处理；
/// IN-CR-06 使用 `__OPENCLAW_SKILL_GUARD_PLACEHOLDER__`，由 skill_install_guard 逻辑处理；
/// IN-CR-03-BIP39-INBOUND 使用 `__BIP39_SECOND_PASS_PLACEHOLDER__`，由 engine_adapter
///   inbound second-pass 处理（与 outbound OUT-09 共用 candidate_bip39_windows + verify_checksum）。
/// 字面量传入 vectorscan 会导致含该字符串的任意文本被误触发。
pub(crate) const INBOUND_PLACEHOLDER_PATTERNS: &[&str] = &[
    "__ADDRESS_GUARD_PLACEHOLDER__",
    "__OPENCLAW_SKILL_GUARD_PLACEHOLDER__",
    "__BIP39_SECOND_PASS_PLACEHOLDER__",
];

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();

    match cli.command {
        Command::Start {
            config: cfg_path,
            dry_run: cli_dry_run,
        } => {
            let mut cfg = config::Config::load(&cfg_path)
                .with_context(|| format!("failed to load config from {}", cfg_path.display()))?;

            // CLI --dry-run 出现（true）时覆盖 config 中的值；
            // 不出现（false）时沿用 config.dry_run（bool OR 语义符合预期：CLI 只能追加 true）。
            if cli_dry_run {
                cfg.dry_run = true;
            }

            cfg.enforce_safety_invariants(); // bind_addr 非 127.0.0.1 → exit(1)

            let audit_path = cfg.audit_db_path()?;
            let audit_store = Arc::new(
                AuditStore::init(&audit_path)
                    .with_context(|| format!("init audit store at {}", audit_path.display()))?,
            );

            // 加载出站规则（fail-closed：加载失败直接退出，不 fallback 到无规则模式，ADR-007）
            let rules_path = cfg.resolved_rules_path();
            tracing::info!(path = %rules_path.display(), "loading outbound rules");
            let rules = load_outbound_rules(&rules_path).with_context(|| {
                format!(
                    "failed to load outbound rules from {}; \
                     set rules_path in sieve.toml or ensure the default path exists",
                    rules_path.display()
                )
            })?;
            tracing::info!(count = rules.len(), "outbound rules loaded");

            // 编译出站 vectorscan db（fail-closed）
            let system_engine = VectorscanEngine::compile(rules.clone())
                .map_err(|e| anyhow::anyhow!("vectorscan compile: {e}"))?;

            // 加载用户规则（PRD §5.5 + §9 #14 fail-safe）
            let user_rules_path = sieve_ipc::paths::sieve_home()
                .map(|h| h.join("rules").join("user.toml"))
                .ok();

            // 出站用户规则引擎（只编译 direction=outbound/both 的规则，PRD v2.0 §5.5）
            let outbound_user_engine = load_user_engine_fail_safe(
                user_rules_path.as_deref(),
                sieve_policy::loader::RuleDirection::Outbound,
            );

            // 用 LayeredEngine 包装系统 + 用户规则（PRD §6.3 / §5.5.2.1）
            // 以 Arc 持有，同时给 OutboundAdapter 使用，并保留 Arc 引用供 reload hot swap（PRD §5.5.5 v2.1）
            let outbound_layered =
                Arc::new(LayeredEngine::new(system_engine, outbound_user_engine));
            let adapter = OutboundAdapter::new(Arc::clone(&outbound_layered), rules);

            // 加载 .sieveignore（出站 + 入站共用同一份）
            let sieveignore_path = cfg.resolved_sieveignore_path();
            let sieveignore = load_sieveignore(&sieveignore_path);
            tracing::info!(
                path = %sieveignore_path.display(),
                entries = sieveignore.len(),
                "sieveignore loaded"
            );
            let sieveignore_arc = Arc::new(sieveignore);

            let filter = Arc::new(OutboundFilter::new(
                Arc::new(adapter),
                Arc::clone(&sieveignore_arc),
            ));

            // 加载入站规则（fail-closed，ADR-007）
            let inbound_rules_path = cfg.resolved_inbound_rules_path();
            tracing::info!(path = %inbound_rules_path.display(), "loading inbound rules");
            let inbound_rules_raw = load_inbound_rules(&inbound_rules_path).with_context(|| {
                format!(
                    "failed to load inbound rules from {}; \
                         set inbound_rules_path in sieve.toml or ensure the default path exists",
                    inbound_rules_path.display()
                )
            })?;

            // 占位规则不传 vectorscan 编译（R6-#6：含 IN-CR-01 + IN-CR-06 两个 placeholder）
            let (placeholder_rules, vectorscan_rules): (Vec<_>, Vec<_>) = inbound_rules_raw
                .iter()
                .cloned()
                .partition(|r| INBOUND_PLACEHOLDER_PATTERNS.contains(&r.pattern.as_str()));
            tracing::info!(
                count = vectorscan_rules.len(),
                placeholders = placeholder_rules.len(),
                "inbound rules partitioned"
            );

            // 编译入站 vectorscan db（独立实例，fail-closed）
            let inbound_system_engine = VectorscanEngine::compile(vectorscan_rules)
                .map_err(|e| anyhow::anyhow!("inbound vectorscan compile: {e}"))?;

            // 入站用户规则引擎（只编译 direction=inbound/both 的规则，PRD v2.0 §5.5）
            let inbound_user_engine = load_user_engine_fail_safe(
                user_rules_path.as_deref(),
                sieve_policy::loader::RuleDirection::Inbound,
            );

            // 用 LayeredEngine 包装入站系统 + 用户规则
            // 以 Arc 持有，同时给 InboundAdapter 使用，并保留 Arc 引用供 reload hot swap（PRD §5.5.5 v2.1）
            let inbound_layered = Arc::new(LayeredEngine::new(
                inbound_system_engine,
                inbound_user_engine,
            ));
            // InboundAdapter 持有全量 rule_lookup（含 placeholder，用于反查元数据）
            let inbound_adapter =
                InboundAdapter::new(Arc::clone(&inbound_layered), inbound_rules_raw);

            // 从 IN-CR-01 RuleEntry 读取地址替换检测配置（修 R3-#5）。
            // 若未找到 IN-CR-01（不应发生），使用安全默认值（60s + fail-closed block）。
            let address_guard_config = placeholder_rules
                .iter()
                .find(|r| r.id == "IN-CR-01")
                .map(|r| {
                    let timeout = r.timeout_seconds.unwrap_or(60);
                    let dot = match r.default_on_timeout {
                        sieve_rules::manifest::DefaultOnTimeout::Redact => DefaultOnTimeout::Redact,
                        sieve_rules::manifest::DefaultOnTimeout::Block => DefaultOnTimeout::Block,
                        sieve_rules::manifest::DefaultOnTimeout::Allow => DefaultOnTimeout::Allow,
                    };
                    AddressGuardConfig {
                        timeout_seconds: timeout,
                        default_on_timeout: dot,
                    }
                })
                .unwrap_or_else(|| {
                    tracing::warn!(
                        "IN-CR-01 rule not found; using default AddressGuardConfig (60s + block)"
                    );
                    AddressGuardConfig::default()
                });
            tracing::info!(
                timeout_seconds = address_guard_config.timeout_seconds,
                "IN-CR-01 address guard config loaded"
            );

            // YOLO mode 运行时审计（防御性双保险）
            audit_yolo_disabled(&cfg)?;

            daemon::run(
                cfg,
                filter,
                Arc::new(inbound_adapter),
                Arc::clone(&sieveignore_arc),
                address_guard_config,
                audit_store,
                outbound_layered,
                inbound_layered,
            )
            .await?;
        }
        Command::Version => {
            println!("sieve {}", env!("CARGO_PKG_VERSION"));
        }
        Command::Setup(args) => {
            commands::setup::run(args)?;
        }
        Command::Doctor(args) => {
            // R4-#8：doctor 失败时返回非零 exit code，CI 脚本可捕获。
            if let Err(e) = commands::doctor::run(args) {
                eprintln!("sieve doctor: {e}");
                std::process::exit(1);
            }
        }
        Command::Uninstall(args) => {
            commands::uninstall::run(args)?;
        }
        Command::Rules(args) => {
            commands::rules::run(&args)?;
        }
    }

    Ok(())
}

/// 防御性检查：确认配置中无任何试图禁用 Critical 检测的字段。
///
/// Phase 1 实现：`Config` 已用 `#[serde(deny_unknown_fields)]` 在反序列化时拒绝
/// 所有未知字段（含 `disable_critical` / `yolo` / `bypass` 等），此函数作为
/// 运行时第二道防线，仅记录审计日志。
///
/// # Errors
/// 当前实现不返回错误；签名保留 `Result<()>` 便于 Week 4 扩展检查逻辑。
fn audit_yolo_disabled(cfg: &config::Config) -> Result<()> {
    // dry_run 模式下 fail-closed 规则仍强制 Block（ADR-007 §2）
    if cfg.dry_run {
        tracing::warn!(
            "dry_run=true: non-fail-closed Critical detections will only be logged, \
             NOT blocked. Fail-closed rules (IN-CR-01/02/05/IN-GEN-01/03/OUT-01~12) \
             remain enforced regardless."
        );
    }
    tracing::info!("YOLO mode audit: passed (no critical-disable fields detected)");
    Ok(())
}

/// 从文件加载 `.sieveignore` fingerprint 白名单。
///
/// 文件不存在时静默返回空集合（正常状态）；读取失败时打印 WARN 并返回空集合。
/// 每行一个 fingerprint，支持 `#` 注释行和空行。
fn load_sieveignore(path: &Path) -> HashSet<String> {
    if !path.exists() {
        return HashSet::new();
    }
    match std::fs::read_to_string(path) {
        Ok(s) => s
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect(),
        Err(e) => {
            tracing::warn!(
                path = %path.display(),
                error = %e,
                "failed to load .sieveignore; proceeding with empty allowlist"
            );
            HashSet::new()
        }
    }
}

/// 加载并按方向编译用户规则引擎（PRD v2.0 §5.5 / §9 #14 fail-safe）。
///
/// 文件不存在时 `sieve_policy::loader::load_user_rules` 返回空 `UserRulesFile`，
/// 空规则列表（或按方向过滤后 0 条）导致 `UserEngine::compile_for_direction` 返回错误，
/// 此时 `load_user_engine_fail_safe` 返回 `None`，daemon 以纯系统规则正常启动。
///
/// `direction` 控制哪些规则被编译进该引擎实例（PRD §5.5）：
/// - `Outbound`：只编译 direction=outbound/both 的规则，挂出站侧
/// - `Inbound`：只编译 direction=inbound/both 的规则，挂入站侧
fn load_and_compile_user_engine(
    path: &std::path::Path,
    direction: sieve_policy::loader::RuleDirection,
) -> Result<sieve_policy::engine::UserEngine, anyhow::Error> {
    use sieve_policy::lint::lint;
    use sieve_policy::loader::load_user_rules;

    // 文件不存在时 load_user_rules 返回空 UserRulesFile（PRD §5.5.2.1）
    let file_size = if path.exists() {
        std::fs::metadata(path)?.len()
    } else {
        0u64
    };

    let file = load_user_rules(path)?;

    // 空规则 → 直接返错（调用方会降级为 None，效果等同于无用户规则）
    if file.rules.is_empty() {
        anyhow::bail!(
            "user rules file is empty or not present at {}",
            path.display()
        );
    }

    // lint 校验（PRD §5.5.3）
    let violations = lint(&file, file_size);
    if !violations.is_empty() {
        // PRD §9 #14：记录 + 返错（调用方把错降级为 warn + 用 None）
        let summary = violations
            .iter()
            .map(|v| format!("[{}] {:?}: {}", v.rule_id, v.kind, v.message))
            .collect::<Vec<_>>()
            .join("; ");
        anyhow::bail!("user rules lint failed: {summary}");
    }

    // 按方向过滤后编译（PRD §5.5）
    sieve_policy::engine::UserEngine::compile_for_direction(file.rules, direction)
        .map_err(|e| anyhow::anyhow!("compile user engine (direction={direction:?}): {e}"))
}

/// fail-safe 包装：将 `load_and_compile_user_engine` 的失败降级为 `None`（PRD §9 #14）。
///
/// daemon 必须在用户规则损坏时正常启动，系统规则不受影响。
/// `direction` 参数同时作为日志标识和过滤条件（PRD v2.0 §5.5）。
fn load_user_engine_fail_safe(
    path: Option<&std::path::Path>,
    direction: sieve_policy::loader::RuleDirection,
) -> Option<sieve_policy::engine::UserEngine> {
    let path = path?;
    let side = format!("{direction:?}").to_lowercase();
    match load_and_compile_user_engine(path, direction) {
        Ok(eng) => {
            tracing::info!(
                side = %side,
                path = %path.display(),
                rule_count = eng.rule_count(),
                "用户规则加载成功（PRD §5.5）"
            );
            Some(eng)
        }
        Err(e) => {
            // 文件不存在是正常状态，降低日志级别
            let msg = e.to_string();
            if msg.contains("empty or not present") || msg.contains("No rules to compile") {
                tracing::debug!(
                    side = %side,
                    path = %path.display(),
                    "用户规则文件不存在或该方向无规则，以纯系统规则启动（PRD §9 #14）"
                );
            } else {
                tracing::warn!(
                    side = %side,
                    path = %path.display(),
                    error = %e,
                    "用户规则加载失败，以纯系统规则继续启动（PRD §9 #14 fail-safe）"
                );
            }
            None
        }
    }
}

fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter = EnvFilter::try_from_env("SIEVE_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .init();
}

// ──────────────────────────────── 单元测试 ──────────────────────────────────

#[cfg(test)]
mod tests {
    use super::INBOUND_PLACEHOLDER_PATTERNS;

    /// R6-#6 测试 4：PLACEHOLDER_PATTERNS 常量至少含 IN-CR-01 和 IN-CR-06 两个占位（R6-#6）
    ///
    /// 保证未来新增 placeholder 时不会漏掉添加到常量列表。
    #[test]
    fn inbound_placeholder_patterns_contains_both_known_placeholders() {
        assert!(
            INBOUND_PLACEHOLDER_PATTERNS.contains(&"__ADDRESS_GUARD_PLACEHOLDER__"),
            "INBOUND_PLACEHOLDER_PATTERNS 应含 IN-CR-01 的 __ADDRESS_GUARD_PLACEHOLDER__"
        );
        assert!(
            INBOUND_PLACEHOLDER_PATTERNS.contains(&"__OPENCLAW_SKILL_GUARD_PLACEHOLDER__"),
            "INBOUND_PLACEHOLDER_PATTERNS 应含 IN-CR-06 的 __OPENCLAW_SKILL_GUARD_PLACEHOLDER__"
        );
        assert!(
            INBOUND_PLACEHOLDER_PATTERNS.len() >= 2,
            "INBOUND_PLACEHOLDER_PATTERNS 应至少包含 2 个 placeholder（IN-CR-01 + IN-CR-06）"
        );
    }

    /// R6-#6 测试 3：partition 后含 placeholder 字面量的文本不被 vectorscan 命中
    ///
    /// 直接验证 partition 逻辑将两个 placeholder pattern 都过滤出去，
    /// 确保 vectorscan 不编译这两个字面量（否则任何含该字符串的文本会被误触发）。
    #[test]
    fn placeholder_patterns_are_excluded_from_vectorscan_partition() {
        use sieve_rules::loader::load_inbound_rules;

        // 定位 inbound.toml（相对于 CARGO_MANIFEST_DIR）
        let rules_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("sieve-rules")
            .join("rules")
            .join("inbound.toml");

        if !rules_path.exists() {
            // CI 环境中规则文件路径可能不同，跳过
            eprintln!("跳过：inbound.toml 未找到（{:?}）", rules_path);
            return;
        }

        let rules = load_inbound_rules(&rules_path).expect("load inbound rules");

        // 用 INBOUND_PLACEHOLDER_PATTERNS partition
        let (placeholder_rules, vectorscan_rules): (Vec<_>, Vec<_>) = rules
            .iter()
            .cloned()
            .partition(|r| INBOUND_PLACEHOLDER_PATTERNS.contains(&r.pattern.as_str()));

        // 两个占位规则都应被 partition 出
        let ph_ids: Vec<&str> = placeholder_rules.iter().map(|r| r.id.as_str()).collect();
        assert!(
            ph_ids.contains(&"IN-CR-01"),
            "IN-CR-01 应被 partition 到 placeholder_rules，ph_ids={ph_ids:?}"
        );
        assert!(
            ph_ids.contains(&"IN-CR-06"),
            "IN-CR-06 应被 partition 到 placeholder_rules，ph_ids={ph_ids:?}"
        );

        // vectorscan_rules 中不含任何 placeholder pattern
        for r in &vectorscan_rules {
            assert!(
                !INBOUND_PLACEHOLDER_PATTERNS.contains(&r.pattern.as_str()),
                "vectorscan_rules 中不应有 placeholder pattern，rule_id={} pattern={}",
                r.id,
                r.pattern
            );
        }
    }
}
