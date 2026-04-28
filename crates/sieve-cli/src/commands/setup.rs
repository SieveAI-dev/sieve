//! `sieve setup` 命令实现（ADR-015 / SPEC-003 §setup / SPEC-004）。
//!
//! 仅 macOS Phase 1。非 macOS 编译进友好错误 stub，不影响构建。
//!
//! ## 架构
//!
//! `AgentAdapter` trait 抽象每家 agent 的配置注入接口（SPEC-004 §4）：
//! - `ClaudeAdapter`：沿用 SPEC-003 已有逻辑（`~/.claude/settings.json` + launchd plist）
//! - `OpenClawAdapter`：stub + 完整接口；Week 7 实测后补真实写入（SPEC-004 §10 TBD-01）
//! - `HermesAdapter`：stub + 完整接口；Week 7 实测后补真实写入（SPEC-004 §10 TBD-02）
//!
//! ## 主流程（SPEC-004 §2.1）
//!
//! 1. 解析 agent 列表（`--agent` 重复 / `--all-detected` / 默认 claude）
//! 2. 每家 agent dry-run diff 打印
//! 3. 用户统一确认（除非 `--yes`）
//! 4. 顺序 apply（任一失败回滚该 agent；已成功其他 agent 不回滚）
//! 5. 跑 doctor 验证

use crate::cli::{AgentKind, SetupArgs};
use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use crate::commands::doctor;
    use anyhow::{anyhow, bail, Context};
    use chrono::Utc;
    use serde_json::Value;
    use std::fs;
    use std::io::{self, Write as IoWrite};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    // ──────────────────────────────── setup.log entry ───────────────────────

    /// setup.log 每行的结构（JSON Lines）。
    ///
    /// `agent`：归属 agent（SPEC-004 §5.1）。
    /// `created_new`：true 表示 setup 前该文件不存在，由 setup 新建；
    /// uninstall 时 `created_new=true` 的文件直接删除，`false` 的从备份恢复。
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct SetupLogEntry {
        pub timestamp: String,
        pub action: String,
        pub path: Option<String>,
        pub detail: Option<String>,
        /// setup 前该文件是否不存在（新建 vs 覆盖）。
        #[serde(default)]
        pub created_new: bool,
        /// 归属 agent（SPEC-004 §5.1）。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub agent: Option<String>,
    }

    impl SetupLogEntry {
        pub(super) fn new(action: impl Into<String>) -> Self {
            Self {
                timestamp: Utc::now().to_rfc3339(),
                action: action.into(),
                path: None,
                detail: None,
                created_new: false,
                agent: None,
            }
        }

        pub(super) fn with_path(mut self, path: impl Into<String>) -> Self {
            self.path = Some(path.into());
            self
        }

        pub(super) fn with_detail(mut self, detail: impl Into<String>) -> Self {
            self.detail = Some(detail.into());
            self
        }

        pub(super) fn with_created_new(mut self, created_new: bool) -> Self {
            self.created_new = created_new;
            self
        }

        pub(super) fn with_agent(mut self, agent: AgentKind) -> Self {
            self.agent = Some(agent.to_string());
            self
        }
    }

    // ──────────────────────────────── SetupContext ──────────────────────────

    /// setup 执行上下文，用于错误时反向回滚。
    pub(super) struct SetupContext {
        backup_dir: PathBuf,
        /// 已写入的文件路径，错误时按逆序恢复。
        written_files: Vec<PathBuf>,
        /// 已执行的 launchctl load，错误时需要 unload。
        launchd_loaded: Option<PathBuf>,
    }

    impl SetupContext {
        fn new(backup_dir: PathBuf) -> Self {
            Self {
                backup_dir,
                written_files: Vec::new(),
                launchd_loaded: None,
            }
        }

        /// 测试专用：构造含已写文件列表的 SetupContext，用于验证 rollback 行为。
        #[cfg(test)]
        pub(super) fn new_with_written_files(
            backup_dir: PathBuf,
            written_files: Vec<PathBuf>,
        ) -> Self {
            Self {
                backup_dir,
                written_files,
                launchd_loaded: None,
            }
        }

        /// 回滚所有已做改动（从备份目录恢复）。
        pub(super) fn rollback(&self) {
            eprintln!("[sieve setup] 回滚已做改动…");

            if let Some(plist) = &self.launchd_loaded {
                let _ = Command::new("launchctl")
                    .args(["unload", &plist.to_string_lossy()])
                    .status();
                eprintln!("  ↩ launchctl unload {}", plist.display());
            }

            for path in self.written_files.iter().rev() {
                // 计算备份中的相对路径：去掉 HOME 前缀
                let home = std::env::var("HOME").unwrap_or_default();
                let rel = path.strip_prefix(&home).unwrap_or(path.as_path());
                let backup_src = self.backup_dir.join(rel);
                if backup_src.exists() {
                    if let Err(e) = fs::copy(&backup_src, path) {
                        eprintln!("  ✗ 恢复 {} 失败: {}", path.display(), e);
                    } else {
                        eprintln!("  ↩ 恢复 {}", path.display());
                    }
                } else {
                    // 备份不存在说明是新建的，直接删除
                    let _ = fs::remove_file(path);
                    eprintln!("  ↩ 删除新建文件 {}", path.display());
                }
            }
        }
    }

    // ──────────────────────────────── AgentDetection ───────────────────────

    /// agent 检测结果（SPEC-004 §3）。
    pub struct AgentDetection {
        /// 是否检测到安装。
        pub installed: bool,
        /// 主配置文件路径（若已找到）。
        pub config_path: Option<PathBuf>,
        /// daemon 是否运行中（None = 未知 / 检测命令不可用）。
        pub daemon_running: Option<bool>,
        /// TBD 注意事项（实测前的未知字段，显示在 diff 中提示用户）。
        pub todo_notes: Vec<&'static str>,
    }

    // ──────────────────────────────── DoctorReport ─────────────────────────

    /// doctor 检查报告（SPEC-004 §6）。
    ///
    /// Phase 1 stub：只表示成功/失败，无详细项；Week 7 OpenClaw/Hermes 实测后扩展字段。
    pub struct DoctorReport;

    impl DoctorReport {
        fn ok() -> Self {
            Self
        }
    }

    // ──────────────────────────────── AgentAdapter trait ───────────────────

    /// 每家 agent 的配置注入接口（SPEC-004 §4）。
    ///
    /// 关联 SPEC-004 §4 / §6 / §7。
    pub(super) trait AgentAdapter {
        /// agent 类型标识。
        fn kind(&self) -> AgentKind;

        /// 检测 agent 是否已安装（SPEC-004 §3）。
        fn detect(&self) -> Result<AgentDetection>;

        /// 打印将做的改动（dry-run diff）。
        fn dry_run_diff(&self) -> Result<String>;

        /// 执行配置注入（SPEC-004 §4）。
        fn apply(&self, ctx: &mut SetupContext) -> Result<()>;

        /// 执行 doctor 检查（SPEC-004 §6）。
        fn doctor_check(&self) -> Result<DoctorReport>;

        /// 回滚本 agent 已做的改动（SPEC-004 §7）。
        ///
        /// apply() 失败时由主流程调用；`ctx` 中的 written_files 已由 apply 填入。
        fn rollback(&self, ctx: &mut SetupContext) {
            ctx.rollback();
        }
    }

    // ──────────────────────────────── ClaudeAdapter ────────────────────────

    /// Claude Code 适配器（SPEC-003 已有逻辑封装，语义不变）。
    ///
    /// 关联 SPEC-003 §setup / SPEC-004 §4.1。
    pub(super) struct ClaudeAdapter {
        home_path: PathBuf,
        settings_path: PathBuf,
        plist_path: PathBuf,
        sieve_toml_path: PathBuf,
        setup_log_path: PathBuf,
        backup_dir: PathBuf,
        sieve_url: &'static str,
    }

    impl ClaudeAdapter {
        fn new(home_path: PathBuf, backup_dir: PathBuf) -> Result<Self> {
            let sieve_home =
                sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
            Ok(Self {
                settings_path: home_path.join(".claude").join("settings.json"),
                plist_path: home_path
                    .join("Library")
                    .join("LaunchAgents")
                    .join("com.sieve.daemon.plist"),
                sieve_toml_path: sieve_home.join("sieve.toml"),
                setup_log_path: sieve_home.join("setup.log"),
                backup_dir,
                home_path,
                sieve_url: "http://127.0.0.1:11453",
            })
        }

        fn read_existing_settings(&self) -> Result<(Value, bool)> {
            let existed = self.settings_path.exists();
            let v = if existed {
                let raw = fs::read_to_string(&self.settings_path)
                    .context("读取 ~/.claude/settings.json 失败")?;
                let stripped = strip_json_comments(&raw);
                serde_json::from_str(&stripped).map_err(|e| {
                    anyhow!(
                        "无法解析 ~/.claude/settings.json：{}。\n\
                         请用 JSON 校验工具修复后重试。setup 已 abort，未做任何改动。",
                        e
                    )
                })?
            } else {
                serde_json::json!({})
            };
            Ok((v, existed))
        }
    }

    impl AgentAdapter for ClaudeAdapter {
        fn kind(&self) -> AgentKind {
            AgentKind::Claude
        }

        fn detect(&self) -> Result<AgentDetection> {
            let config_path = if self.settings_path.exists() {
                Some(self.settings_path.clone())
            } else {
                None
            };
            let binary_ok = Command::new("which")
                .arg("claude")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            let installed = config_path.is_some() || binary_ok;
            if config_path.is_some() && !binary_ok {
                eprintln!(
                    "[sieve setup] 警告：未找到 claude 二进制，setup 继续但请确认 Claude Code 已安装"
                );
            }
            Ok(AgentDetection {
                installed,
                config_path,
                daemon_running: None,
                todo_notes: vec![],
            })
        }

        fn dry_run_diff(&self) -> Result<String> {
            let (existing_settings, _) = self.read_existing_settings()?;
            let current_base_url = existing_settings
                .pointer("/env/ANTHROPIC_BASE_URL")
                .and_then(|v| v.as_str())
                .unwrap_or("<未设置>");
            let has_hook = existing_settings
                .pointer("/hooks/PreToolUse")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter().any(|item| {
                        item.pointer("/hooks/0/command")
                            .and_then(|c| c.as_str())
                            .map(|c| c.contains("sieve-hook"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            let hook_line = if has_hook {
                "[settings.json] hooks.PreToolUse: sieve-hook 已存在（幂等）".to_string()
            } else {
                "[settings.json] hooks.PreToolUse: 新增 sieve-hook check 条目".to_string()
            };
            let toml_line = if self.sieve_toml_path.exists() {
                format!(
                    "[sieve.toml] {} 已存在，将覆盖（原文件备份到 backups/）",
                    self.sieve_toml_path.display()
                )
            } else {
                format!("[sieve.toml] 新建 {}", self.sieve_toml_path.display())
            };

            Ok(format!(
                "[settings.json] env.ANTHROPIC_BASE_URL: {:?} → {:?}\n{}\n{}\n[launchd] 写入 {} (含 --config {})\n[launchd] 执行 launchctl load -w",
                current_base_url,
                self.sieve_url,
                hook_line,
                toml_line,
                self.plist_path.display(),
                self.sieve_toml_path.display(),
            ))
        }

        fn apply(&self, ctx: &mut SetupContext) -> Result<()> {
            let (existing_settings, settings_existed_before) = self.read_existing_settings()?;
            let hook_entry = serde_json::json!({
                "matcher": ".*",
                "hooks": [{"type": "command", "command": "sieve-hook check"}]
            });
            let plist_content = build_plist_content(&self.sieve_toml_path)?;
            do_claude_setup(
                ctx,
                &self.home_path,
                &self.settings_path,
                &self.plist_path,
                &self.sieve_toml_path,
                &self.setup_log_path,
                &self.backup_dir,
                existing_settings,
                settings_existed_before,
                self.sieve_url,
                hook_entry,
                plist_content,
            )
        }

        fn doctor_check(&self) -> Result<DoctorReport> {
            // 委托给 doctor 模块的 Claude 检查逻辑
            let args = crate::cli::DoctorArgs {
                agent: Some(AgentKind::Claude),
                all: false,
            };
            doctor::run(args)?;
            Ok(DoctorReport::ok())
        }
    }

    // ──────────────────────────────── OpenClawAdapter ──────────────────────

    /// OpenClaw 适配器（SPEC-004 §4.2；当前为 stub，Week 7 实测后补完）。
    ///
    /// **TBD-01**：实际配置路径与字段名需 Week 7 实测确认；见 SPEC-004 §10。
    pub(super) struct OpenClawAdapter {
        home_path: PathBuf,
    }

    impl OpenClawAdapter {
        fn new(home_path: PathBuf) -> Self {
            Self { home_path }
        }

        /// 探测 OpenClaw 配置文件（按 SPEC-004 §3.2 候选路径顺序）。
        ///
        /// **TBD-01**：路径列表需 Week 7 实测后调整。
        fn probe_config_path(&self) -> Option<PathBuf> {
            let candidates = [
                self.home_path.join(".openclaw").join("config.toml"),
                self.home_path
                    .join("Library")
                    .join("Application Support")
                    .join("openclaw")
                    .join("config.toml"),
            ];
            // 检查环境变量 OPENCLAW_CONFIG
            if let Ok(val) = std::env::var("OPENCLAW_CONFIG") {
                if !val.is_empty() {
                    return Some(PathBuf::from(val));
                }
            }
            candidates.into_iter().find(|p| p.exists())
        }
    }

    impl AgentAdapter for OpenClawAdapter {
        fn kind(&self) -> AgentKind {
            AgentKind::Openclaw
        }

        fn detect(&self) -> Result<AgentDetection> {
            let config_path = self.probe_config_path();
            let dir_exists = self.home_path.join(".openclaw").is_dir()
                || self
                    .home_path
                    .join("Library")
                    .join("Application Support")
                    .join("openclaw")
                    .is_dir();
            let binary_ok = Command::new("which")
                .arg("openclaw")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            // daemon 状态：TBD-03，先尝试 openclaw status
            let daemon_running = Command::new("openclaw")
                .arg("status")
                .output()
                .ok()
                .map(|o| o.status.success());

            let installed = config_path.is_some() || dir_exists || binary_ok;
            if !installed {
                eprintln!(
                    "未找到 OpenClaw 安装（~/.openclaw/ 和 openclaw 二进制均未找到）。\n\
                     跳过 OpenClaw 配置。如已安装，请先运行 openclaw 确认路径后重试。"
                );
            }
            Ok(AgentDetection {
                installed,
                config_path,
                daemon_running,
                todo_notes: vec![
                    "TBD-01: 配置文件路径需 Week 7 实测确认（SPEC-004 §10）",
                    "TBD-03: openclaw status 命令名需实测（SPEC-004 §10）",
                    "TBD-05: X-Sieve-Source-Channel header 注入需实测（SPEC-004 §10）",
                ],
            })
        }

        fn dry_run_diff(&self) -> Result<String> {
            let detection = self.detect()?;
            let config_str = detection
                .config_path
                .as_deref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "未找到（TBD-01）".to_string());
            let daemon_str = match detection.daemon_running {
                Some(true) => "运行中",
                Some(false) => "未运行",
                None => "未知（TBD-03）",
            };
            Ok(format!(
                "[openclaw] 检测到：{}\n\
                 [openclaw] 配置文件：{}\n\
                 [openclaw] daemon 状态：{}\n\
                 [openclaw] 将修改：provider base_url → http://127.0.0.1:11453（TBD-01：字段路径待实测）\n\
                 [openclaw] ⚠ 以下项目需 Week 7 实测后才能完整写入：\n\
                 {}",
                if detection.installed { "已安装" } else { "未找到" },
                config_str,
                daemon_str,
                detection.todo_notes.iter().map(|n| format!("  - {n}")).collect::<Vec<_>>().join("\n"),
            ))
        }

        fn apply(&self, _ctx: &mut SetupContext) -> Result<()> {
            // TBD-01：OpenClaw 配置注入需 Week 7 实测后实现。
            // 当前 stub 明确 bail 避免静默跳过，防止用户误以为已配置。
            // 实测后删除此 bail!，替换为实际 TOML 写入逻辑（SPEC-004 §4.2.3）。
            bail!(
                "OpenClaw 配置注入尚未实现：需 Week 7 实测确认配置路径和字段格式。\n\
                 见 SPEC-004 §10 TBD-01。\n\
                 如需手动配置，请将 OpenClaw provider base_url 设为 http://127.0.0.1:11453"
            )
        }

        fn doctor_check(&self) -> Result<DoctorReport> {
            // TODO（Week 7 实测后实现）：
            // 1. 检查 daemon 监听（TCP connect 127.0.0.1:11453）
            // 2. 解析 ~/.openclaw/config.toml，验证 provider base_url（TBD-01）
            // 3. Canary（OpenAI 协议）（TBD-05）
            // 见 SPEC-004 §6.2。
            eprintln!(
                "[doctor] OpenClaw 检查为 stub，待 Week 7 实测后实现（SPEC-004 §6.2 TBD-01/TBD-05）"
            );
            Ok(DoctorReport::ok())
        }
    }

    // ──────────────────────────────── HermesAdapter ────────────────────────

    /// Hermes 适配器（SPEC-004 §4.3；当前为 stub，Week 7 实测后补完）。
    ///
    /// **TBD-02**：实际配置路径与格式需 Week 7 实测确认；见 SPEC-004 §10。
    pub(super) struct HermesAdapter {
        home_path: PathBuf,
    }

    impl HermesAdapter {
        fn new(home_path: PathBuf) -> Self {
            Self { home_path }
        }

        /// 探测 Hermes 配置文件（按 SPEC-004 §3.3 候选路径顺序）。
        ///
        /// **TBD-02**：路径列表需 Week 7 实测后调整。
        fn probe_config_path(&self) -> Option<PathBuf> {
            // 检查环境变量 HERMES_CONFIG
            if let Ok(val) = std::env::var("HERMES_CONFIG") {
                if !val.is_empty() {
                    return Some(PathBuf::from(val));
                }
            }
            let candidates = [
                self.home_path.join(".hermes").join("config.toml"),
                self.home_path.join(".hermes").join(".env"),
            ];
            candidates.into_iter().find(|p| p.exists())
        }
    }

    impl AgentAdapter for HermesAdapter {
        fn kind(&self) -> AgentKind {
            AgentKind::Hermes
        }

        fn detect(&self) -> Result<AgentDetection> {
            let config_path = self.probe_config_path();
            let dir_exists = self.home_path.join(".hermes").is_dir();
            let binary_ok = Command::new("which")
                .arg("hermes")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            // daemon/provider 列表：TBD-04，先尝试 hermes config providers list
            let daemon_running = Command::new("hermes")
                .args(["config", "providers", "list"])
                .output()
                .ok()
                .map(|o| o.status.success());

            let installed = config_path.is_some() || dir_exists || binary_ok;
            if !installed {
                eprintln!(
                    "未找到 Hermes 安装（~/.hermes/ 和 hermes 二进制均未找到）。\n\
                     跳过 Hermes 配置。"
                );
            }
            Ok(AgentDetection {
                installed,
                config_path,
                daemon_running,
                todo_notes: vec![
                    "TBD-02: 配置文件路径需 Week 7 实测确认（SPEC-004 §10）",
                    "TBD-04: hermes config providers list 命令名需实测（SPEC-004 §10）",
                    "TBD-06: ANTHROPIC_DEFAULT_HEADERS 注入机制需实测（SPEC-004 §10）",
                ],
            })
        }

        fn dry_run_diff(&self) -> Result<String> {
            let detection = self.detect()?;
            let config_str = detection
                .config_path
                .as_deref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "未找到（TBD-02）".to_string());
            let daemon_str = match detection.daemon_running {
                Some(true) => "可用",
                Some(false) => "不可用",
                None => "未知（TBD-04）",
            };
            Ok(format!(
                "[hermes] 检测到：{}\n\
                 [hermes] 配置文件：{}\n\
                 [hermes] provider 列表命令：{}\n\
                 [hermes] 将修改：provider base_url → http://127.0.0.1:11453（TBD-02：字段路径待实测）\n\
                 [hermes] ⚠ 以下项目需 Week 7 实测后才能完整写入：\n\
                 {}",
                if detection.installed { "已安装" } else { "未找到" },
                config_str,
                daemon_str,
                detection.todo_notes.iter().map(|n| format!("  - {n}")).collect::<Vec<_>>().join("\n"),
            ))
        }

        fn apply(&self, _ctx: &mut SetupContext) -> Result<()> {
            // TBD-02：Hermes 配置注入需 Week 7 实测后实现。
            // 当前 stub 明确 bail 避免静默跳过。
            // 实测后删除此 bail!，替换为实际写入逻辑（SPEC-004 §4.3.3）。
            bail!(
                "Hermes 配置注入尚未实现：需 Week 7 实测确认配置路径和字段格式。\n\
                 见 SPEC-004 §10 TBD-02。\n\
                 如需手动配置，请将 Hermes provider base_url 设为 http://127.0.0.1:11453"
            )
        }

        fn doctor_check(&self) -> Result<DoctorReport> {
            // TODO（Week 7 实测后实现）：
            // 1. hermes --version 检查
            // 2. 解析 Hermes 配置文件（TBD-02），验证 provider base_url
            // 3. Canary（OpenAI 协议）
            // 4. X-Sieve-Origin header 注入（TBD-06）
            // 见 SPEC-004 §6.3。
            eprintln!(
                "[doctor] Hermes 检查为 stub，待 Week 7 实测后实现（SPEC-004 §6.3 TBD-02/TBD-06）"
            );
            Ok(DoctorReport::ok())
        }
    }

    // ──────────────────────────────── detect_all_agents ────────────────────

    /// 自动检测系统已安装的所有 agent（SPEC-004 §3）。
    fn detect_all_agents(
        home_path: &Path,
        backup_dir: &Path,
    ) -> Result<Vec<Box<dyn AgentAdapter>>> {
        let all_adapters: Vec<Box<dyn AgentAdapter>> = vec![
            Box::new(ClaudeAdapter::new(
                home_path.to_path_buf(),
                backup_dir.to_path_buf(),
            )?),
            Box::new(OpenClawAdapter::new(home_path.to_path_buf())),
            Box::new(HermesAdapter::new(home_path.to_path_buf())),
        ];
        let mut detected = Vec::new();
        for adapter in all_adapters {
            let detection = adapter.detect()?;
            if detection.installed {
                detected.push(adapter);
            }
        }
        Ok(detected)
    }

    // ──────────────────────────────── confirm_or_abort ─────────────────────

    fn confirm_or_abort() -> Result<()> {
        print!("继续执行以上操作？[y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("已取消。");
            std::process::exit(0);
        }
        Ok(())
    }

    // ──────────────────────────────── run() 主流程 ─────────────────────────

    /// 运行 `sieve setup`（SPEC-004 §2.1 主流程）。
    ///
    /// 关联 ADR-015 / SPEC-003 §setup / SPEC-004 §2.1。
    pub fn run(args: SetupArgs) -> Result<()> {
        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
        let home_path = PathBuf::from(&home);
        let sieve_home =
            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
        let backup_ts = Utc::now().to_rfc3339().replace(':', "-");
        let backup_dir = sieve_home.join("backups").join(&backup_ts);

        // ── 1. 解析 agent 列表（SPEC-004 §2.1）
        let adapters: Vec<Box<dyn AgentAdapter>> = if args.all_detected {
            // --all-detected：扫描系统已安装的所有 agent
            let detected = detect_all_agents(&home_path, &backup_dir)?;
            if detected.is_empty() {
                println!("未检测到任何已安装的 agent。请先安装 Claude Code / OpenClaw / Hermes。");
                return Ok(());
            }
            detected
        } else if args.agent.is_empty() {
            // 默认：仅 Claude（兼容 v1.4 行为）
            vec![Box::new(ClaudeAdapter::new(
                home_path.clone(),
                backup_dir.clone(),
            )?)]
        } else {
            // --agent <name>（可重复）
            let mut adapters: Vec<Box<dyn AgentAdapter>> = Vec::new();
            for kind in &args.agent {
                let adapter: Box<dyn AgentAdapter> = match kind {
                    AgentKind::Claude => {
                        Box::new(ClaudeAdapter::new(home_path.clone(), backup_dir.clone())?)
                    }
                    AgentKind::Openclaw => Box::new(OpenClawAdapter::new(home_path.clone())),
                    AgentKind::Hermes => Box::new(HermesAdapter::new(home_path.clone())),
                };
                adapters.push(adapter);
            }
            adapters
        };

        // ── 2. dry-run diff 打印（每家 agent 单独一段）
        println!("=== sieve setup diff ===");
        for adapter in &adapters {
            println!("--- {} ---", adapter.kind());
            println!("{}", adapter.dry_run_diff()?);
        }
        println!("========================");

        if args.dry_run {
            println!("[dry-run] 未做任何改动。");
            return Ok(());
        }

        // ── 3. 用户确认（除非 --yes）
        if !args.yes {
            confirm_or_abort()?;
        }

        // ── 4. 备份目录
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("创建备份目录 {} 失败", backup_dir.display()))?;

        // ── 5. 顺序 apply（SPEC-004 §7.1：单个失败只回滚该 agent，不影响其他已成功的）
        // 同时保留成功 apply 的 ctx，供后续 doctor 失败时回滚使用。
        let mut any_failed = false;
        // (adapter_index, ctx) for successfully applied agents, in order
        let mut applied_ctxs: Vec<(AgentKind, SetupContext)> = Vec::new();
        for adapter in &adapters {
            let mut ctx = SetupContext::new(backup_dir.clone());
            println!("\n[setup] 正在配置 {}…", adapter.kind());
            if let Err(e) = adapter.apply(&mut ctx) {
                eprintln!("[setup] {} 配置失败：{e}", adapter.kind());
                eprintln!("[setup] 正在回滚 {} 的改动…", adapter.kind());
                adapter.rollback(&mut ctx);
                any_failed = true;
                // 继续处理下一个 agent（SPEC-004 §7.2：部分失败不中止其他）
            } else {
                println!("[setup] ✅ {} 配置完成", adapter.kind());
                applied_ctxs.push((adapter.kind(), ctx));
            }
        }

        if any_failed {
            return Err(anyhow!(
                "部分 agent 配置失败（见上方输出）。成功的 agent 配置已保留。\n\
                 如需重试失败的 agent：sieve setup --agent <name>"
            ));
        }

        // ── 6. 跑 doctor 验证（仅对 Claude；其他 agent 为 stub，跳过）
        //
        // doctor 失败时，用保存的 ctx（含 written_files）回滚 Claude 的实际写入。
        let claude_ctx_idx = applied_ctxs
            .iter()
            .position(|(k, _)| *k == AgentKind::Claude);
        if let Some(idx) = claude_ctx_idx {
            println!("\n[sieve setup] 正在验证 Claude Code 安装…");
            let claude_adapter = ClaudeAdapter::new(home_path.clone(), backup_dir.clone())?;
            if let Err(doctor_err) = claude_adapter.doctor_check() {
                eprintln!("[sieve setup] doctor 验证失败，正在自动回滚 Claude…");
                applied_ctxs[idx].1.rollback();
                return Err(anyhow!(
                    "setup 已自动回滚（doctor 验证失败：{}）；请检查 doctor 报告",
                    doctor_err
                ));
            }
        }

        Ok(())
    }

    // ──────────────────────────────── Claude setup 内部实现 ─────────────────

    #[allow(clippy::too_many_arguments)]
    fn do_claude_setup(
        ctx: &mut SetupContext,
        home_path: &Path,
        settings_path: &Path,
        plist_path: &Path,
        sieve_toml_path: &Path,
        setup_log_path: &Path,
        backup_dir: &Path,
        mut existing_settings: Value,
        settings_existed_before: bool,
        sieve_url: &str,
        hook_entry: Value,
        plist_content: String,
    ) -> Result<()> {
        // 备份 settings.json（仅在文件已存在时）
        if settings_existed_before {
            let rel = settings_path
                .strip_prefix(home_path)
                .unwrap_or(settings_path);
            let backup_dest = backup_dir.join(rel);
            if let Some(parent) = backup_dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(settings_path, &backup_dest).context("备份 settings.json 失败")?;
        }

        // 修改 settings.json
        {
            let env = existing_settings
                .get_mut("env")
                .and_then(|v| v.as_object_mut())
                .map(|obj| {
                    obj.insert(
                        "ANTHROPIC_BASE_URL".to_string(),
                        serde_json::json!(sieve_url),
                    );
                })
                .is_some();
            if !env {
                let obj = existing_settings
                    .as_object_mut()
                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
                obj.insert(
                    "env".to_string(),
                    serde_json::json!({"ANTHROPIC_BASE_URL": sieve_url}),
                );
            }

            // 追加 PreToolUse hook（幂等：已存在则跳过）
            let hooks_obj = existing_settings
                .get_mut("hooks")
                .and_then(|v| v.as_object_mut());
            if let Some(hooks) = hooks_obj {
                let pre_tool = hooks
                    .entry("PreToolUse")
                    .or_insert_with(|| serde_json::json!([]));
                if let Some(arr) = pre_tool.as_array_mut() {
                    let already = arr.iter().any(|item| {
                        item.pointer("/hooks/0/command")
                            .and_then(|c| c.as_str())
                            .map(|c| c.contains("sieve-hook"))
                            .unwrap_or(false)
                    });
                    if !already {
                        arr.push(hook_entry);
                    }
                }
            } else {
                let obj = existing_settings
                    .as_object_mut()
                    .ok_or_else(|| anyhow!("settings.json 根必须是 object"))?;
                obj.insert(
                    "hooks".to_string(),
                    serde_json::json!({"PreToolUse": [hook_entry]}),
                );
            }

            // 确保父目录存在
            if let Some(parent) = settings_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json_str = serde_json::to_string_pretty(&existing_settings)?;
            fs::write(settings_path, json_str.as_bytes()).context("写入 settings.json 失败")?;
            ctx.written_files.push(settings_path.to_path_buf());
            println!("[setup] ✅ settings.json 已更新");
        }

        // 写 ~/.sieve/sieve.toml（绝对路径配置，供 launchd plist 引用）
        let sieve_toml_existed_before = sieve_toml_path.exists();
        {
            if sieve_toml_existed_before {
                // 备份已有 sieve.toml
                let rel = sieve_toml_path
                    .strip_prefix(home_path)
                    .unwrap_or(sieve_toml_path);
                let backup_dest = backup_dir.join(rel);
                if let Some(parent) = backup_dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(sieve_toml_path, &backup_dest).context("备份 sieve.toml 失败")?;
            }
            if let Some(parent) = sieve_toml_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml_content = build_default_sieve_toml(sieve_toml_path)?;
            fs::write(sieve_toml_path, toml_content.as_bytes()).context("写入 sieve.toml 失败")?;
            ctx.written_files.push(sieve_toml_path.to_path_buf());
            println!("[setup] ✅ sieve.toml 写入 {}", sieve_toml_path.display());
        }

        // 写 launchd plist
        {
            if let Some(parent) = plist_path.parent() {
                fs::create_dir_all(parent)?;
            }
            // 备份已有 plist
            if plist_path.exists() {
                let rel = plist_path.strip_prefix(home_path).unwrap_or(plist_path);
                let backup_dest = backup_dir.join(rel);
                if let Some(parent) = backup_dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(plist_path, &backup_dest).context("备份 plist 失败")?;
            }
            fs::write(plist_path, plist_content.as_bytes()).context("写入 launchd plist 失败")?;
            ctx.written_files.push(plist_path.to_path_buf());
            println!("[setup] ✅ launchd plist 写入 {}", plist_path.display());
        }

        // launchctl load
        {
            let status = Command::new("launchctl")
                .args(["load", "-w", &plist_path.to_string_lossy()])
                .status()
                .context("执行 launchctl load 失败")?;
            if !status.success() {
                bail!("launchctl load 返回非零: {:?}", status.code());
            }
            ctx.launchd_loaded = Some(plist_path.to_path_buf());
            println!("[setup] ✅ launchd 服务已加载");
        }

        // 写 setup.log（含 agent + created_new 字段，供 uninstall 精确还原）
        {
            let entries: Vec<SetupLogEntry> = vec![
                SetupLogEntry::new("setup_complete")
                    .with_detail(format!("backup_dir={}", backup_dir.display()))
                    .with_agent(AgentKind::Claude),
                SetupLogEntry::new("settings_updated")
                    .with_path(settings_path.to_string_lossy().to_string())
                    .with_detail("env.ANTHROPIC_BASE_URL + hooks.PreToolUse")
                    .with_created_new(!settings_existed_before)
                    .with_agent(AgentKind::Claude),
                SetupLogEntry::new("sieve_toml_written")
                    .with_path(sieve_toml_path.to_string_lossy().to_string())
                    .with_created_new(!sieve_toml_existed_before)
                    .with_agent(AgentKind::Claude),
                SetupLogEntry::new("launchd_loaded")
                    .with_path(plist_path.to_string_lossy().to_string())
                    .with_agent(AgentKind::Claude),
            ];
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(setup_log_path)
                .context("打开 setup.log 失败")?;
            for entry in &entries {
                let line = serde_json::to_string(entry)? + "\n";
                file.write_all(line.as_bytes())?;
            }
            println!("[setup] ✅ setup.log 写入 {}", setup_log_path.display());
        }

        Ok(())
    }

    // ──────────────────────────────── 工具函数 ──────────────────────────────

    /// 构建 launchd plist 内容（使用当前 sieve 二进制路径 + 绝对路径 --config）。
    ///
    /// plist 中 ProgramArguments 必须使用绝对路径，且 --config 指向绝对配置文件，
    /// 否则 launchd 从根目录启动时找不到相对路径规则文件，daemon 会立即退出。
    /// WorkingDirectory 兜底设置为 sieve_home（~/.sieve）。
    pub(super) fn build_plist_content(sieve_toml_path: &Path) -> Result<String> {
        let sieve_bin = std::env::current_exe().context("获取当前二进制路径失败")?;
        let sieve_home =
            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
        let log_path = sieve_home.join("daemon.log");
        let err_path = sieve_home.join("daemon.err");
        // config 路径必须是绝对路径
        let config_abs = if sieve_toml_path.is_absolute() {
            sieve_toml_path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(sieve_toml_path)
        };

        Ok(format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>com.sieve.daemon</string>
  <key>ProgramArguments</key>
  <array>
    <string>{bin}</string>
    <string>start</string>
    <string>--config</string>
    <string>{config}</string>
  </array>
  <key>WorkingDirectory</key>
  <string>{work_dir}</string>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>{log}</string>
  <key>StandardErrorPath</key>
  <string>{err}</string>
</dict>
</plist>
"#,
            bin = sieve_bin.display(),
            config = config_abs.display(),
            work_dir = sieve_home.display(),
            log = log_path.display(),
            err = err_path.display(),
        ))
    }

    /// 构建默认 sieve.toml 内容（所有路径使用绝对路径）。
    ///
    /// 生成的内容与 [`crate::config::Config`] 的扁平字段完全匹配（`deny_unknown_fields`），
    /// 可直接被 `toml::from_str::<Config>()` 反序列化而不报错。
    pub(super) fn build_default_sieve_toml(sieve_toml_path: &Path) -> Result<String> {
        let sieve_home = sieve_toml_path
            .parent()
            .ok_or_else(|| anyhow!("sieve.toml 路径无父目录"))?;
        let rules_path = sieve_home.join("rules").join("outbound.toml");
        let inbound_rules_path = sieve_home.join("rules").join("inbound.toml");
        let audit_db = sieve_home.join("audit.db");
        let ipc_socket = sieve_home.join("ipc.sock");
        let pending_dir = sieve_home.join("pending");
        let decisions_dir = sieve_home.join("decisions");
        let home = std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| sieve_home.to_path_buf());
        let launchd_plist = home
            .join("Library")
            .join("LaunchAgents")
            .join("com.sieve.daemon.plist");

        Ok(format!(
            r#"# sieve.toml — 由 `sieve setup` 自动生成，所有路径为绝对路径
# 修改后需重启 daemon：launchctl kickstart -k gui/$(id -u)/com.sieve.daemon

upstream_url = "https://api.anthropic.com"
port = 11453
bind_addr = "127.0.0.1"
tls_verify_upstream = true
dry_run = false
preset = "default"
gui_socket_enabled = false

# 出站规则文件路径（绝对路径，launchd 从 / 启动时不依赖 cwd）
rules_path = "{rules_path}"

# 入站规则文件路径
inbound_rules_path = "{inbound_rules_path}"

# 审计日志数据库路径（绝对路径）
audit_db_path = "{audit_db}"

# IPC Unix socket 路径
ipc_socket_path = "{ipc_socket}"

# 待决策 / 已决策文件目录
pending_dir = "{pending_dir}"
decisions_dir = "{decisions_dir}"

# launchd plist 路径（macOS）
launchd_plist_path = "{launchd_plist}"
"#,
            rules_path = rules_path.display(),
            inbound_rules_path = inbound_rules_path.display(),
            audit_db = audit_db.display(),
            ipc_socket = ipc_socket.display(),
            pending_dir = pending_dir.display(),
            decisions_dir = decisions_dir.display(),
            launchd_plist = launchd_plist.display(),
        ))
    }

    /// 简单去除 `// ...` 行注释（不处理字符串内的 `//`，够用于 settings.json）。
    pub(super) fn strip_json_comments(s: &str) -> String {
        s.lines()
            .map(|line| {
                // 找到不在引号内的 `//`
                let mut in_string = false;
                let mut escaped = false;
                let mut comment_start = None;
                let chars: Vec<char> = line.chars().collect();
                let mut i = 0;
                while i < chars.len() {
                    if escaped {
                        escaped = false;
                    } else if chars[i] == '\\' && in_string {
                        escaped = true;
                    } else if chars[i] == '"' {
                        in_string = !in_string;
                    } else if !in_string
                        && chars[i] == '/'
                        && i + 1 < chars.len()
                        && chars[i + 1] == '/'
                    {
                        comment_start = Some(i);
                        break;
                    }
                    i += 1;
                }
                if let Some(pos) = comment_start {
                    line[..pos].to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    // ── 内部测试：SetupContext::rollback（直接访问私有结构）─────────────────────
    #[cfg(test)]
    mod tests_rollback {
        use super::*;
        use tempfile::tempdir;

        // ── 测试 #5：rollback 确实恢复备份文件 ──────────────────────────────────
        // R5-#1 修复验证：backup 存在时 rollback 从备份恢复
        #[test]
        #[allow(unsafe_code)] // 测试隔离需要临时覆盖 HOME env var
        fn setup_context_rollback_restores_settings() {
            use std::sync::Mutex;

            // env var 修改需要串行
            static ENV_LOCK: Mutex<()> = Mutex::new(());
            let _guard = ENV_LOCK.lock().unwrap();

            let dir = tempdir().unwrap();
            let backup_dir = dir.path().join("backups").join("2026-01-01");
            fs::create_dir_all(&backup_dir).unwrap();

            let original_content = r#"{"env": {"ORIGINAL_KEY": "original_value"}}"#;
            let home_root = dir.path().join("home");
            let claude_dir = home_root.join(".claude");
            fs::create_dir_all(&claude_dir).unwrap();
            let settings_path = claude_dir.join("settings.json");

            // 写入备份（模拟 setup 前的备份）
            let backup_settings = backup_dir.join(".claude").join("settings.json");
            fs::create_dir_all(backup_settings.parent().unwrap()).unwrap();
            fs::write(&backup_settings, original_content).unwrap();

            // 写入已改的文件（模拟 setup 修改后）
            fs::write(
                &settings_path,
                r#"{"env": {"ANTHROPIC_BASE_URL": "http://127.0.0.1:11453"}}"#,
            )
            .unwrap();

            let ctx = SetupContext::new_with_written_files(
                backup_dir.clone(),
                vec![settings_path.clone()],
            );

            let orig_home = std::env::var("HOME").unwrap_or_default();
            unsafe {
                std::env::set_var("HOME", home_root.to_str().unwrap());
            }
            ctx.rollback();
            unsafe {
                std::env::set_var("HOME", &orig_home);
            }

            let restored = fs::read_to_string(&settings_path).unwrap();
            assert_eq!(
                restored, original_content,
                "rollback 后 settings.json 应恢复为原始内容"
            );
        }

        // ── 测试 #6：新建文件回滚时被删除（无备份 → 删文件）────────────────────
        #[test]
        #[allow(unsafe_code)] // 测试隔离需要临时覆盖 HOME env var
        fn setup_context_rollback_deletes_new_file() {
            use std::sync::Mutex;

            static ENV_LOCK: Mutex<()> = Mutex::new(());
            let _guard = ENV_LOCK.lock().unwrap();

            let dir = tempdir().unwrap();
            let backup_dir = dir.path().join("backups").join("2026-01-01");
            fs::create_dir_all(&backup_dir).unwrap();

            let home_root = dir.path().join("home");
            let claude_dir = home_root.join(".claude");
            fs::create_dir_all(&claude_dir).unwrap();
            let new_file = claude_dir.join("settings.json");

            fs::write(&new_file, r#"{"env": {}}"#).unwrap();
            assert!(new_file.exists());

            let ctx =
                SetupContext::new_with_written_files(backup_dir.clone(), vec![new_file.clone()]);

            let orig_home = std::env::var("HOME").unwrap_or_default();
            unsafe {
                std::env::set_var("HOME", home_root.to_str().unwrap());
            }
            ctx.rollback();
            unsafe {
                std::env::set_var("HOME", &orig_home);
            }

            assert!(!new_file.exists(), "无备份的新建文件在 rollback 后应被删除");
        }
    }
}

// ──────────────────────────────── 非 macOS stub ─────────────────────────────

#[cfg(not(target_os = "macos"))]
mod stub {
    use super::*;

    /// `sieve setup` 非 macOS 占位实现。
    /// Phase 1 仅支持 macOS；Linux/Windows 在 Phase 2 规划（ADR-015）。
    pub fn run(_args: SetupArgs) -> Result<()> {
        anyhow::bail!(
            "sieve setup is macOS only in Phase 1. \
             Linux/Windows support is planned for Phase 2."
        )
    }
}

// ──────────────────────────────── 单元测试 ──────────────────────────────────

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::macos::{
        build_default_sieve_toml, build_plist_content, strip_json_comments, SetupLogEntry,
    };
    use tempfile::tempdir;

    // ── 测试 #1：plist 包含 --config <绝对路径>/sieve.toml ──────────────────
    // 修复 #6 验证：launchd plist 必须含绝对路径 --config 和 WorkingDirectory

    #[test]
    fn plist_contains_absolute_config_flag() {
        let dir = tempdir().unwrap();
        let sieve_toml = dir.path().join("sieve.toml");
        let plist = build_plist_content(&sieve_toml).unwrap();

        assert!(
            plist.contains("<string>--config</string>"),
            "plist 必须包含 --config 参数: {plist}"
        );
        let config_str = sieve_toml.to_string_lossy();
        assert!(
            plist.contains(config_str.as_ref()),
            "plist 必须包含 sieve.toml 绝对路径 {config_str}: {plist}"
        );
        assert!(
            plist.contains("<key>WorkingDirectory</key>"),
            "plist 必须包含 WorkingDirectory: {plist}"
        );
    }

    // ── 测试 #2：解析失败的 JSON 返回 Err（不 fallback 到空对象）──────────────
    // 修复 #8 核心：strip_json_comments + serde_json::from_str 失败路径

    #[test]
    fn bad_json_parse_returns_error_not_empty_object() {
        // 尾逗号是无效 JSON，strip_json_comments 无法修复
        let bad_json = r#"{"env": {"SOME_KEY": "value",},}"#;
        let stripped = strip_json_comments(bad_json);
        let result: Result<serde_json::Value, _> = serde_json::from_str(&stripped);

        // 修复前是 unwrap_or_else(|_| {}) 导致覆盖用户数据；修复后必须返回 Err
        assert!(
            result.is_err(),
            "尾逗号 JSON 应解析失败，不得 fallback 到空对象"
        );
    }

    // ── 测试 #3：SetupLogEntry 序列化 created_new + agent 字段 ──────────────
    // SPEC-004 §5.1：每条 entry 含 agent 字段

    #[test]
    fn setup_log_entry_created_new_and_agent_serialize_correctly() {
        use crate::cli::AgentKind;

        let entry_new = SetupLogEntry::new("settings_updated")
            .with_path("/tmp/test.json".to_string())
            .with_created_new(true)
            .with_agent(AgentKind::Claude);
        let json = serde_json::to_string(&entry_new).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            v.get("created_new").and_then(|c| c.as_bool()),
            Some(true),
            "新建文件 created_new 应序列化为 true: {json}"
        );
        assert_eq!(
            v.get("agent").and_then(|a| a.as_str()),
            Some("claude"),
            "agent 字段应序列化为 'claude': {json}"
        );

        let entry_existing = SetupLogEntry::new("settings_updated")
            .with_path("/tmp/test.json".to_string())
            .with_created_new(false)
            .with_agent(AgentKind::Openclaw);
        let json2 = serde_json::to_string(&entry_existing).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        assert_eq!(
            v2.get("created_new").and_then(|c| c.as_bool()),
            Some(false),
            "已有文件 created_new 应序列化为 false: {json2}"
        );
        assert_eq!(
            v2.get("agent").and_then(|a| a.as_str()),
            Some("openclaw"),
            "agent 字段应序列化为 'openclaw': {json2}"
        );
    }

    // ── sieve.toml 使用绝对路径 ─────────────────────────────────────────────

    #[test]
    fn default_sieve_toml_has_absolute_paths() {
        let dir = tempdir().unwrap();
        let sieve_toml = dir.path().join("sieve.toml");
        let content = build_default_sieve_toml(&sieve_toml).unwrap();

        assert!(
            content.contains(&format!("rules_path = \"{}", dir.path().display())),
            "rules_path 必须是绝对路径: {content}"
        );
        assert!(
            content.contains(&format!("audit_db_path = \"{}", dir.path().display())),
            "audit_db_path 必须是绝对路径: {content}"
        );
    }

    #[test]
    fn default_sieve_toml_parses_as_config() {
        // R2-#2：build_default_sieve_toml 生成的内容必须能被 Config 反序列化
        use crate::config::Config;

        let dir = tempdir().unwrap();
        let sieve_toml = dir.path().join("sieve.toml");
        let content = build_default_sieve_toml(&sieve_toml).unwrap();
        let cfg: Config = toml::from_str(&content).unwrap_or_else(|e| {
            panic!("build_default_sieve_toml 生成的 TOML 解析失败: {e}\n---\n{content}")
        });
        assert_eq!(cfg.port, 11453);
        assert_eq!(cfg.bind_addr, "127.0.0.1");
        assert_eq!(cfg.upstream_url, "https://api.anthropic.com");
        assert!(cfg.audit_db_path.is_some(), "audit_db_path 应有绝对路径");
        assert!(cfg.rules_path.is_some(), "rules_path 应有绝对路径");
    }
}
