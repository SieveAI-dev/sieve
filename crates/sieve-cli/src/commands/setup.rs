//! `sieve setup` 命令实现（ADR-015 / SPEC-003 §setup / SPEC-004）。
//!
//! 仅 macOS Phase 1。非 macOS 编译进友好错误 stub，不影响构建。
//!
//! ## 架构
//!
//! `AgentAdapter` trait 抽象每家 agent 的配置注入接口（SPEC-004 §4）：
//! - `ClaudeAdapter`：`~/.claude/settings.json` + launchd plist（SPEC-003）
//! - `OpenClawAdapter`：`~/.openclaw/openclaw.json` + provider base_url 重写（SPEC-004 §6）
//! - `HermesAdapter`：`~/.hermes/config.yaml` 模型 base_url 重写（SPEC-004 §7）
//!
//! `skill_install_guard` 的真实样本固化仍在 dogfood 期跟进（v2-pending TODO-DOGFOOD-1），
//! 不影响 setup 配置写入路径本身。
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
pub(crate) mod macos {
    use super::*;
    use crate::commands::doctor;
    use crate::embedded_rules;
    use crate::upstream_routes::UpstreamRoutes;
    use anyhow::{anyhow, bail, Context};
    use chrono::Utc;
    use serde_json::Value;
    use serde_yaml;
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
        /// setup.log 路径，用于 append_log_entry。None 时跳过写入（如 daemon_ctx）。
        setup_log_path: Option<PathBuf>,
    }

    impl SetupContext {
        fn new(backup_dir: PathBuf) -> Self {
            Self {
                backup_dir,
                written_files: Vec::new(),
                launchd_loaded: None,
                setup_log_path: None,
            }
        }

        /// 设置 setup.log 路径（链式调用）。
        fn with_setup_log(mut self, path: PathBuf) -> Self {
            self.setup_log_path = Some(path);
            self
        }

        /// 追加一条 SetupLogEntry 到 setup.log（JSON Lines append）。
        ///
        /// setup_log_path 为 None 时静默跳过（daemon_ctx 不需要写 agent entry）。
        /// 关联 SPEC-004 §5.1 / known-issues-v1.4.md R10-#1。
        pub(super) fn append_log_entry(&mut self, entry: &SetupLogEntry) -> Result<()> {
            let Some(ref path) = self.setup_log_path else {
                return Ok(());
            };
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .with_context(|| format!("打开 setup.log 失败: {}", path.display()))?;
            let line = serde_json::to_string(entry)? + "\n";
            file.write_all(line.as_bytes())?;
            Ok(())
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
                setup_log_path: None,
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
        /// Week 8 dogfood 待验证的注意事项（当前未读取，预留供 dry-run diff 扩展）。
        #[allow(dead_code)]
        pub todo_notes: Vec<&'static str>,
    }

    // ──────────────────────────────── DoctorReport ─────────────────────────

    /// doctor 检查报告（SPEC-004 §6 / R10-#5）。
    ///
    /// 每项 adapter 的 doctor_check 返回此结构，doctor.rs 汇总后输出 ✅/❌ 列表。
    /// 字段将在 doctor.rs 汇总阶段使用（R10-#5 后续实现）。
    #[allow(dead_code)]
    pub struct DoctorReport {
        /// 归属 agent。
        pub agent: AgentKind,
        /// 各检查项 (名称, 是否通过)。
        pub checks: Vec<(String, bool)>,
        /// 全部通过则为 true。
        pub all_passed: bool,
    }

    impl DoctorReport {
        /// 所有检查均通过的简单构造函数（供既有 apply 后 doctor 调用路径使用）。
        pub(super) fn ok_for(agent: AgentKind) -> Self {
            Self {
                agent,
                checks: vec![],
                all_passed: true,
            }
        }

        /// 通用构造函数：从检查列表推导 all_passed（R10-#5 后续实现接入）。
        #[allow(dead_code)]
        pub fn from_checks(agent: AgentKind, checks: Vec<(String, bool)>) -> Self {
            let all_passed = checks.iter().all(|(_, ok)| *ok);
            Self {
                agent,
                checks,
                all_passed,
            }
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
            // F-3：daemon 共享安装（sieve.toml + 规则 + plist + launchd）已在 run() 主流程
            // 的 install_shared_daemon() 中完成，此处只做 Claude 特有：settings.json + hook 注入。
            do_claude_setup(
                ctx,
                &self.home_path,
                &self.settings_path,
                &self.setup_log_path,
                &self.backup_dir,
                existing_settings,
                settings_existed_before,
                self.sieve_url,
                hook_entry,
            )
        }

        fn doctor_check(&self) -> Result<DoctorReport> {
            // 委托给 doctor 模块的 Claude 检查逻辑
            let args = crate::cli::DoctorArgs {
                agent: Some(AgentKind::Claude),
                all: false,
            };
            doctor::run(args)?;
            Ok(DoctorReport::ok_for(AgentKind::Claude))
        }
    }

    // ──────────────────────────────── OpenClawAdapter ──────────────────────

    /// OpenClaw 适配器（SPEC-004 §4.2）。
    ///
    /// ## 调研结论（Week 7，基于 openclaw/openclaw 公开文档）
    ///
    /// - **TBD-01 已解决**：配置文件为 `~/.openclaw/openclaw.json`（JSON 格式，非 TOML）。
    ///   provider 字段路径：`models.providers.<id>.baseUrl`（camelCase）。
    ///   参考：openclaw/docs/concepts/model-providers.md
    /// - **TBD-03 已解决**：`openclaw doctor` 命令存在（AGENTS.md 明确记录）。
    ///   注意：`openclaw status` 也被提及，但 `doctor` 是官方诊断入口；
    ///   Week 8 dogfood 时验证哪个命令更准确。
    /// - **TBD-05 已解决（部分）**：OpenClaw 支持 `models.providers.<id>.headers` 字段，
    ///   可在配置里注入自定义 HTTP header。
    ///   setup 时写入 `X-Sieve-Source-Channel: <channel-id>` 到目标 provider 的 headers。
    ///   **限制**：channel 值在配置时静态写死，无法动态反映运行时的 WhatsApp/Slack channel；
    ///   IN-GEN-06 获得 header 存在的信号，但 channel 值只是一个占位符 "openclaw"。
    ///   Week 8 dogfood 时确认 OpenClaw 是否在转发请求时保留自定义 headers。
    pub struct OpenClawAdapter {
        home_path: PathBuf,
        sieve_url: &'static str,
    }

    impl OpenClawAdapter {
        pub fn new(home_path: PathBuf) -> Self {
            Self {
                home_path,
                sieve_url: "http://127.0.0.1:11453",
            }
        }

        /// 探测 OpenClaw 配置文件（按 SPEC-004 §3.2 候选路径顺序）。
        ///
        /// 调研结论：主配置文件为 `~/.openclaw/openclaw.json`。
        /// 备用路径：macOS Library/Application Support 目录（npm 全局安装可能写此处）。
        fn probe_config_path(&self) -> Option<PathBuf> {
            // 环境变量优先
            if let Ok(val) = std::env::var("OPENCLAW_CONFIG") {
                if !val.is_empty() {
                    return Some(PathBuf::from(val));
                }
            }
            let candidates = [
                // 主路径（文档明确：~/.openclaw/openclaw.json）
                self.home_path.join(".openclaw").join("openclaw.json"),
                // 备用路径：macOS Application Support（npm 全局安装可能写此处）
                self.home_path
                    .join("Library")
                    .join("Application Support")
                    .join("openclaw")
                    .join("openclaw.json"),
                // 旧版兼容：部分早期版本用 config.json
                self.home_path.join(".openclaw").join("config.json"),
            ];
            candidates.into_iter().find(|p| p.exists())
        }

        /// 解析 openclaw.json，返回 models.providers 对象（可能为空）。
        ///
        /// 字段路径：`models.providers` → `Record<string, { baseUrl?: string, headers?: Record<string, string>, ... }>`
        fn read_config(&self) -> Result<serde_json::Value> {
            let path = self
                .probe_config_path()
                .ok_or_else(|| anyhow::anyhow!("未找到 OpenClaw 配置文件（已尝试所有候选路径）"))?;
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("读取 {} 失败", path.display()))?;
            serde_json::from_str(&raw)
                .with_context(|| format!("解析 {} 失败（须为有效 JSON）", path.display()))
        }

        /// 修改所有 models.providers 条目的 baseUrl、注入 X-Sieve-Source-Channel
        /// 和 X-Sieve-Provider header，同时记录每个 provider 原始 baseUrl（F-1）。
        ///
        /// 返回 (修改后的 JSON Value, 被修改的 provider id 列表,
        ///        provider_id → 原始 baseUrl 映射表)。
        fn patch_config(
            &self,
            mut config: serde_json::Value,
        ) -> Result<(serde_json::Value, Vec<String>, UpstreamRoutes)> {
            let mut patched_ids: Vec<String> = Vec::new();
            let mut original_routes = UpstreamRoutes::default();

            // models.providers 可能不存在（新安装 openclaw 未配置任何 provider）
            if let Some(providers) = config
                .pointer_mut("/models/providers")
                .and_then(|v| v.as_object_mut())
            {
                for (id, provider) in providers.iter_mut() {
                    let obj = match provider.as_object_mut() {
                        Some(o) => o,
                        None => continue,
                    };

                    // 记录原始 baseUrl（F-1：在改写前捕获）
                    let original_base_url = obj
                        .get("baseUrl")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    // 幂等：已是目标 URL 则跳过（但仍记录路由）
                    let already_patched = original_base_url == self.sieve_url;
                    if !original_base_url.is_empty() && original_base_url != self.sieve_url {
                        original_routes.insert(id.clone(), original_base_url.clone());
                    }
                    if already_patched {
                        continue;
                    }

                    obj.insert("baseUrl".to_string(), serde_json::json!(self.sieve_url));

                    // TBD-05：注入 X-Sieve-Source-Channel header（静态值 "openclaw"）。
                    // OpenClaw 支持 models.providers.<id>.headers 字段（见调研结论）。
                    // 静态 channel 值让 IN-GEN-06 知道请求来源是 openclaw，
                    // 但无法区分具体 WhatsApp/Slack channel（需 OpenClaw 侧 PR）。
                    // Week 8 dogfood 时验证 headers 是否随请求转发。
                    //
                    // F-1：同时注入 X-Sieve-Provider: <id>，daemon 据此查路由表
                    // 选择原始上游，避免 404（OpenAI/DeepSeek/OpenRouter 等全部坏掉）。
                    let headers = obj
                        .entry("headers")
                        .or_insert_with(|| serde_json::json!({}));
                    if let Some(h) = headers.as_object_mut() {
                        h.insert(
                            "X-Sieve-Source-Channel".to_string(),
                            serde_json::json!("openclaw"),
                        );
                        // F-1：注入 provider id，daemon 据此路由到正确上游
                        h.insert(
                            "X-Sieve-Provider".to_string(),
                            serde_json::json!(id.as_str()),
                        );
                    }

                    patched_ids.push(id.clone());
                }
            } else {
                // models.providers 不存在：写一条占位 provider
                // 让用户知道 sieve 已配置，需要手动添加真实 provider
                tracing::warn!(
                    "openclaw.json 中未找到 models.providers，\
                     已创建占位 sieve-proxy provider。\
                     请在 OpenClaw 中添加真实 provider 后，\
                     其 baseUrl 将自动指向 Sieve。"
                );
                let providers_obj = serde_json::json!({
                    "sieve-proxy": {
                        "baseUrl": self.sieve_url,
                        "headers": {
                            "X-Sieve-Source-Channel": "openclaw",
                            "X-Sieve-Provider": "sieve-proxy"
                        }
                    }
                });
                if let Some(models) = config
                    .pointer_mut("/models")
                    .and_then(|v| v.as_object_mut())
                {
                    models.insert("providers".to_string(), providers_obj);
                    patched_ids.push("sieve-proxy".to_string());
                } else {
                    // models 字段也不存在，顶层写入
                    if let Some(root) = config.as_object_mut() {
                        root.insert(
                            "models".to_string(),
                            serde_json::json!({"providers": {
                                "sieve-proxy": {
                                    "baseUrl": self.sieve_url,
                                    "headers": {
                                        "X-Sieve-Source-Channel": "openclaw",
                                        "X-Sieve-Provider": "sieve-proxy"
                                    }
                                }
                            }}),
                        );
                        patched_ids.push("sieve-proxy".to_string());
                    }
                }
            }

            Ok((config, patched_ids, original_routes))
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

            // TBD-03 已解决：`openclaw doctor` 是官方诊断命令（AGENTS.md 确认）。
            // `openclaw status` 也存在但面向 chat session 内部使用，不适合 daemon 状态检查。
            // 调用 doctor 返回 exit 0 → OpenClaw 已安装且配置正常。
            // Week 8 dogfood 时验证 doctor 的实际退出码语义。
            let daemon_running = Command::new("openclaw")
                .arg("doctor")
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
                // TBD-01/03/05 已通过调研填上，Week 8 dogfood 时最终验证
                todo_notes: vec![
                    "Week 8 dogfood：验证 models.providers.<id>.headers 是否随请求转发（TBD-05）",
                    "Week 8 dogfood：确认 openclaw doctor 退出码语义（TBD-03）",
                ],
            })
        }

        fn dry_run_diff(&self) -> Result<String> {
            let detection = self.detect()?;
            let config_str = detection
                .config_path
                .as_deref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "未找到（候选：~/.openclaw/openclaw.json）".to_string());
            let daemon_str = match detection.daemon_running {
                Some(true) => "openclaw doctor 返回 exit 0（正常）",
                Some(false) => "openclaw doctor 返回非零（可能配置问题）",
                None => "openclaw 二进制未找到，跳过 doctor 检查",
            };

            // 尝试读取现有 config 显示当前 provider 状态
            let provider_preview = match self.read_config() {
                Ok(cfg) => {
                    let providers = cfg.pointer("/models/providers");
                    match providers.and_then(|p| p.as_object()) {
                        Some(obj) if !obj.is_empty() => {
                            let ids: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
                            format!(
                                "找到 {} 个 provider（{}），将全部修改 baseUrl → {}",
                                ids.len(),
                                ids.join(", "),
                                self.sieve_url,
                            )
                        }
                        _ => format!(
                            "models.providers 为空，将创建占位 sieve-proxy provider（baseUrl = {}）",
                            self.sieve_url
                        ),
                    }
                }
                Err(_) => format!(
                    "配置文件未找到，将创建 models.providers.sieve-proxy.baseUrl = {}",
                    self.sieve_url
                ),
            };

            Ok(format!(
                "[openclaw] 检测到：{}\n\
                 [openclaw] 配置文件：~/.openclaw/openclaw.json（JSON 格式）\n\
                 [openclaw] 当前配置：{}\n\
                 [openclaw] doctor 状态：{}\n\
                 [openclaw] 将修改：{}\n\
                 [openclaw] 将注入：models.providers.<id>.headers.X-Sieve-Source-Channel = \"openclaw\"\n\
                 [openclaw] 注意：X-Sieve-Source-Channel 为静态值；动态 channel 需 Week 8 验证（见 SPEC-004 §10 TBD-05）",
                if detection.installed { "已安装" } else { "未找到" },
                config_str,
                daemon_str,
                provider_preview,
            ))
        }

        fn apply(&self, ctx: &mut SetupContext) -> Result<()> {
            let config_path = self.probe_config_path().ok_or_else(|| {
                anyhow::anyhow!(
                    "未找到 OpenClaw 配置文件（已尝试以下路径）：\n\
                     - ~/.openclaw/openclaw.json\n\
                     - ~/Library/Application Support/openclaw/openclaw.json\n\
                     - ~/.openclaw/config.json\n\
                     请手动配置，或等待 Week 8 dogfood 验证后更新 sieve。"
                )
            })?;

            // 读取现有配置
            let raw = fs::read_to_string(&config_path)
                .with_context(|| format!("读取 {} 失败", config_path.display()))?;
            let config: serde_json::Value = serde_json::from_str(&raw)
                .with_context(|| format!("解析 {} 失败（须为有效 JSON）", config_path.display()))?;

            // 备份原始配置
            let home = std::env::var("HOME").unwrap_or_default();
            let rel = config_path.strip_prefix(&home).unwrap_or(&config_path);
            let backup_dest = ctx.backup_dir.join(rel);
            if let Some(parent) = backup_dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&config_path, &backup_dest)
                .with_context(|| format!("备份 {} 失败", config_path.display()))?;

            // patch config（F-1：同时捕获原始 baseUrl 映射表）
            let (patched_config, patched_ids, original_routes) = self.patch_config(config)?;

            if patched_ids.is_empty() {
                println!(
                    "[setup] OpenClaw：所有 provider baseUrl 已是 {}（幂等，跳过写入）",
                    self.sieve_url
                );
                return Ok(());
            }

            // 写回 openclaw.json
            let new_raw = serde_json::to_string_pretty(&patched_config)?;
            fs::write(&config_path, new_raw.as_bytes())
                .with_context(|| format!("写入 {} 失败", config_path.display()))?;
            ctx.written_files.push(config_path.clone());

            println!(
                "[setup] ✅ OpenClaw 配置已更新：{} 个 provider（{}）baseUrl → {}",
                patched_ids.len(),
                patched_ids.join(", "),
                self.sieve_url,
            );
            println!("[setup] ✅ 已注入 headers.X-Sieve-Source-Channel = \"openclaw\"（静态）");
            println!(
                "[setup] ✅ 已注入 headers.X-Sieve-Provider = <provider-id>（F-1：daemon 路由用）"
            );

            // F-1：写 upstream-routes.json，daemon 据此把 OpenAI/DeepSeek/OpenRouter 请求
            // 路由到正确上游，避免全部打到 Anthropic → 404
            let sieve_home = sieve_ipc::paths::sieve_home()
                .map_err(|e| anyhow::anyhow!("获取 sieve home 失败: {e}"))?;
            let routes_path = sieve_home.join("upstream-routes.json");
            if !original_routes.is_empty() {
                original_routes.save(&routes_path).with_context(|| {
                    format!(
                        "写入 upstream-routes.json 到 {} 失败",
                        routes_path.display()
                    )
                })?;
                println!(
                    "[setup] ✅ upstream-routes.json 写入 {}（{} 条路由）",
                    routes_path.display(),
                    original_routes.len()
                );
            }

            // R10-#1：将 openclaw setup 记录到 setup.log，供 uninstall 查找（SPEC-004 §5.1）。
            // setup_complete entry：让 uninstall 能找到此次 backup_dir。
            let backup_detail = format!("backup_dir={}", ctx.backup_dir.display());
            ctx.append_log_entry(
                &SetupLogEntry::new("setup_complete")
                    .with_detail(backup_detail)
                    .with_agent(AgentKind::Openclaw),
            )?;
            // config_modified entry：记录哪个文件被改写，供 uninstall 恢复（R10-#1）。
            ctx.append_log_entry(
                &SetupLogEntry::new("config_modified")
                    .with_path(config_path.to_string_lossy().to_string())
                    .with_detail(format!(
                        "patched providers: {}; fields: models.providers.*.baseUrl, \
                     models.providers.*.headers.X-Sieve-Source-Channel, \
                     models.providers.*.headers.X-Sieve-Provider",
                        patched_ids.join(", ")
                    ))
                    .with_created_new(false)
                    .with_agent(AgentKind::Openclaw),
            )?;
            Ok(())
        }

        fn doctor_check(&self) -> Result<DoctorReport> {
            // 1. daemon 监听检查（TCP connect 127.0.0.1:11453）
            let tcp_ok = std::net::TcpStream::connect_timeout(
                &"127.0.0.1:11453".parse().unwrap(),
                std::time::Duration::from_secs(2),
            )
            .is_ok();
            if !tcp_ok {
                eprintln!("[doctor] OpenClaw：Sieve daemon 未监听 127.0.0.1:11453");
                return Err(anyhow::anyhow!("Sieve daemon 未在 127.0.0.1:11453 监听"));
            }
            println!("[doctor] ✅ OpenClaw：Sieve daemon 在监听");

            // 2. 解析配置验证 provider baseUrl
            match self.read_config() {
                Ok(cfg) => {
                    let all_patched = cfg
                        .pointer("/models/providers")
                        .and_then(|p| p.as_object())
                        .map(|providers| {
                            providers.values().all(|v| {
                                v.pointer("/baseUrl")
                                    .and_then(|u| u.as_str())
                                    .map(|u| u == self.sieve_url)
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false);

                    if all_patched {
                        println!("[doctor] ✅ OpenClaw：所有 provider baseUrl 已指向 Sieve");
                    } else {
                        eprintln!(
                            "[doctor] ✗ OpenClaw：部分 provider baseUrl 未指向 {}",
                            self.sieve_url
                        );
                        return Err(anyhow::anyhow!(
                            "OpenClaw provider 配置不正确，请重新运行 sieve setup --agent openclaw"
                        ));
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "OpenClaw doctor：无法读取配置文件（{}），跳过 provider 验证",
                        e
                    );
                    println!("[doctor] ⚠ OpenClaw：无法读取配置文件，跳过 provider 验证");
                }
            }

            // 3. X-Sieve-Source-Channel 透传状态说明（Week 8 dogfood 验证）
            println!(
                "[doctor] ⚠ OpenClaw X-Sieve-Source-Channel：静态 header 已注入配置，\
                 Week 8 dogfood 时验证是否随请求转发（SPEC-004 §10 TBD-05）"
            );

            Ok(DoctorReport::ok_for(AgentKind::Openclaw))
        }
    }

    // ──────────────────────────────── HermesAdapter ────────────────────────

    /// Hermes 适配器（SPEC-004 §4.3）。
    ///
    /// ## 调研结论（Week 7，基于 NousResearch/hermes-agent 公开文档）
    ///
    /// - **TBD-02 已解决**：配置文件为 `~/.hermes/config.yaml`（YAML 格式，非 TOML）。
    ///   备用：`~/.hermes/.env`（存放 API key）。
    ///   provider 字段路径：顶层 `base_url`（覆盖 provider 路由）或 `custom_providers[].base_url`。
    ///   参考：hermes-agent.nousresearch.com/docs/integrations/providers
    /// - **TBD-04 已解决**：`hermes config providers list` 命令不存在。
    ///   实际命令：`hermes config`（查看配置），`hermes config check`（验证配置）。
    ///   Week 8 dogfood 时确认 `hermes config check` 退出码语义。
    /// - **TBD-06 已解决（降级）**：Hermes delegation 子进程**不**自动继承父进程环境变量。
    ///   文档明确：sub-agents 使用 delegation section 的配置，不透传 ANTHROPIC_DEFAULT_HEADERS。
    ///   **降级方案**：setup 时在 delegation.base_url 写入 Sieve URL，子进程的 LLM 请求也经过 Sieve。
    ///   X-Sieve-Origin header 由 Sieve daemon 端根据请求特征（如 model 字段差异）推断，
    ///   而非通过 env var 注入。PRD §6.7 sub-agent 嵌套场景 F 的完整 origin chain 在 Phase 1 后期实现。
    pub struct HermesAdapter {
        home_path: PathBuf,
        sieve_url: &'static str,
    }

    impl HermesAdapter {
        pub fn new(home_path: PathBuf) -> Self {
            Self {
                home_path,
                sieve_url: "http://127.0.0.1:11453",
            }
        }

        /// 探测 Hermes 配置文件（按 SPEC-004 §3.3 候选路径顺序）。
        ///
        /// 调研结论：主配置文件为 `~/.hermes/config.yaml`（YAML 格式）。
        fn probe_config_path(&self) -> Option<PathBuf> {
            // 环境变量优先
            if let Ok(val) = std::env::var("HERMES_CONFIG") {
                if !val.is_empty() {
                    return Some(PathBuf::from(val));
                }
            }
            let candidates = [
                // 主路径（文档明确：~/.hermes/config.yaml，YAML 格式）
                self.home_path.join(".hermes").join("config.yaml"),
                // 旧版兼容：部分文档提到 config.toml（TOML 格式）
                self.home_path.join(".hermes").join("config.toml"),
                // .env 备用（仅存 API key，不包含 base_url；仅用于检测安装，不修改）
                self.home_path.join(".hermes").join(".env"),
            ];
            candidates.into_iter().find(|p| p.exists())
        }

        /// 读取 Hermes config.yaml，返回解析后的 YAML Value。
        fn read_config(&self) -> Result<serde_yaml::Value> {
            let path = self.probe_config_path().ok_or_else(|| {
                anyhow::anyhow!("未找到 Hermes 配置文件（~/.hermes/config.yaml）")
            })?;

            // .env 文件不包含 base_url，不支持修改
            if path.ends_with(".env") {
                bail!(
                    "Hermes 仅找到 .env 文件（~/.hermes/.env），\
                     该文件只存 API key，不支持 base_url 注入。\n\
                     请先运行 hermes config edit 创建 config.yaml，\n\
                     或手动创建 ~/.hermes/config.yaml 并设置 base_url。"
                );
            }

            let raw = fs::read_to_string(&path)
                .with_context(|| format!("读取 {} 失败", path.display()))?;
            serde_yaml::from_str(&raw)
                .with_context(|| format!("解析 {} 失败（须为有效 YAML）", path.display()))
        }

        /// 修改 Hermes config.yaml 中的 base_url 字段（顶层 model.base_url 和 delegation.base_url）。
        ///
        /// Hermes YAML schema（调研结论）：
        /// ```yaml
        /// model:
        ///   provider: openrouter
        ///   base_url: ""       # 覆盖 provider，设为 Sieve URL
        /// delegation:
        ///   base_url: ""       # TBD-06 降级：子进程也经过 Sieve
        /// ```
        ///
        /// 返回 (修改后的 YAML Value, 修改说明列表)。
        fn patch_config(
            &self,
            mut config: serde_yaml::Value,
        ) -> Result<(serde_yaml::Value, Vec<String>)> {
            let mut changes: Vec<String> = Vec::new();

            // 顶层 model.base_url
            if let Some(model) = config.get_mut("model").and_then(|v| v.as_mapping_mut()) {
                let current = model
                    .get("base_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if current != self.sieve_url {
                    model.insert(
                        serde_yaml::Value::String("base_url".to_string()),
                        serde_yaml::Value::String(self.sieve_url.to_string()),
                    );
                    changes.push(format!(
                        "model.base_url: {:?} → {:?}",
                        current, self.sieve_url
                    ));
                }
            } else {
                // model 字段不存在，创建
                if let Some(root) = config.as_mapping_mut() {
                    let mut model_map = serde_yaml::Mapping::new();
                    model_map.insert(
                        serde_yaml::Value::String("base_url".to_string()),
                        serde_yaml::Value::String(self.sieve_url.to_string()),
                    );
                    root.insert(
                        serde_yaml::Value::String("model".to_string()),
                        serde_yaml::Value::Mapping(model_map),
                    );
                    changes.push(format!("model.base_url: (新建) → {:?}", self.sieve_url));
                }
            }

            // TBD-06 降级：delegation.base_url 也指向 Sieve，
            // 使 Hermes 委托 Claude Code 子进程时的流量也经过 Sieve。
            // X-Sieve-Origin header 在 Phase 1 后期通过 Sieve daemon 端推断实现。
            if let Some(delegation) = config
                .get_mut("delegation")
                .and_then(|v| v.as_mapping_mut())
            {
                let current = delegation
                    .get("base_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if current != self.sieve_url {
                    delegation.insert(
                        serde_yaml::Value::String("base_url".to_string()),
                        serde_yaml::Value::String(self.sieve_url.to_string()),
                    );
                    changes.push(format!(
                        "delegation.base_url: {:?} → {:?} (TBD-06 降级：子进程流量经过 Sieve)",
                        current, self.sieve_url
                    ));
                }
            } else {
                // delegation 字段不存在，不强制创建（避免影响 Hermes 默认 delegation 行为）
                tracing::warn!(
                    "Hermes config.yaml 中无 delegation 字段，跳过 delegation.base_url 注入。\
                     Hermes 委托 Claude Code 子进程的流量将**不经过** Sieve（见 SPEC-004 §10 TBD-06 降级说明）。"
                );
                changes.push(
                    "delegation.base_url: 字段不存在，跳过（TBD-06 降级：子进程流量不经过 Sieve）"
                        .to_string(),
                );
            }

            Ok((config, changes))
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

            // TBD-04 已解决：`hermes config providers list` 不存在。
            // 实际用 `hermes config check` 验证配置完整性（文档确认存在）。
            // Week 8 dogfood 时确认 check 的退出码语义。
            let daemon_running = Command::new("hermes")
                .args(["config", "check"])
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
                // TBD-02/04/06 已通过调研填上，Week 8 dogfood 时最终验证
                todo_notes: vec![
                    "Week 8 dogfood：确认 hermes config check 退出码语义（TBD-04）",
                    "Week 8 dogfood：确认 delegation.base_url 是否对所有子进程生效（TBD-06）",
                ],
            })
        }

        fn dry_run_diff(&self) -> Result<String> {
            let detection = self.detect()?;
            let config_str = detection
                .config_path
                .as_deref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "未找到（候选：~/.hermes/config.yaml）".to_string());
            let check_str = match detection.daemon_running {
                Some(true) => "hermes config check 返回 exit 0（正常）",
                Some(false) => "hermes config check 返回非零",
                None => "hermes 二进制未找到，跳过 config check",
            };

            // 尝试读取现有配置显示当前状态
            let field_preview = match self.read_config() {
                Ok(cfg) => {
                    let model_base_url = cfg
                        .get("model")
                        .and_then(|m| m.get("base_url"))
                        .and_then(|u| u.as_str())
                        .unwrap_or("<未设置>");
                    let delegation_base_url = cfg
                        .get("delegation")
                        .and_then(|d| d.get("base_url"))
                        .and_then(|u| u.as_str())
                        .unwrap_or("<未设置>");
                    format!(
                        "model.base_url: {:?} → {:?}\n\
                         [hermes] delegation.base_url: {:?} → {:?}（TBD-06 降级，子进程流量经过 Sieve）",
                        model_base_url,
                        self.sieve_url,
                        delegation_base_url,
                        self.sieve_url,
                    )
                }
                Err(_) => format!(
                    "config.yaml 未找到，将创建 model.base_url = {}",
                    self.sieve_url
                ),
            };

            Ok(format!(
                "[hermes] 检测到：{}\n\
                 [hermes] 配置文件：~/.hermes/config.yaml（YAML 格式）\n\
                 [hermes] 当前配置：{}\n\
                 [hermes] config check 状态：{}\n\
                 [hermes] 将修改：{}\n\
                 [hermes] ⚠ TBD-06 降级说明：Hermes delegation 子进程不继承父进程 env var，\n\
                 [hermes]   ANTHROPIC_DEFAULT_HEADERS 注入不可行。\n\
                 [hermes]   降级方案：delegation.base_url → Sieve，子进程流量经过 Sieve。\n\
                 [hermes]   X-Sieve-Origin header 在 Phase 1 后期由 Sieve 端推断。",
                if detection.installed {
                    "已安装"
                } else {
                    "未找到"
                },
                config_str,
                check_str,
                field_preview,
            ))
        }

        fn apply(&self, ctx: &mut SetupContext) -> Result<()> {
            let config_path = self.probe_config_path().ok_or_else(|| {
                anyhow::anyhow!(
                    "未找到 Hermes 配置文件（已尝试以下路径）：\n\
                     - ~/.hermes/config.yaml\n\
                     - ~/.hermes/config.toml\n\
                     - ~/.hermes/.env\n\
                     请先运行 hermes config edit 创建配置文件后重试。"
                )
            })?;

            // .env 文件不支持修改（只存 API key）
            if config_path.ends_with(".env") {
                bail!(
                    "Hermes 仅找到 .env 文件，不支持 base_url 注入。\n\
                     请先运行 hermes config edit 创建 config.yaml，\n\
                     或手动将 model.base_url 设为 {url}。",
                    url = self.sieve_url
                );
            }

            // 读取配置
            let config = self.read_config()?;

            // 备份
            let home = std::env::var("HOME").unwrap_or_default();
            let rel = config_path.strip_prefix(&home).unwrap_or(&config_path);
            let backup_dest = ctx.backup_dir.join(rel);
            if let Some(parent) = backup_dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&config_path, &backup_dest)
                .with_context(|| format!("备份 {} 失败", config_path.display()))?;

            // patch
            let (patched_config, changes) = self.patch_config(config)?;

            if changes.is_empty() {
                println!(
                    "[setup] Hermes：所有字段已是目标值 {}（幂等，跳过写入）",
                    self.sieve_url
                );
                return Ok(());
            }

            // 写回 YAML
            let new_raw =
                serde_yaml::to_string(&patched_config).context("序列化 Hermes config.yaml 失败")?;
            fs::write(&config_path, new_raw.as_bytes())
                .with_context(|| format!("写入 {} 失败", config_path.display()))?;
            ctx.written_files.push(config_path.clone());

            for change in &changes {
                println!("[setup] ✅ Hermes 配置：{}", change);
            }
            println!(
                "[setup] ⚠ Hermes TBD-06 降级：ANTHROPIC_DEFAULT_HEADERS 注入不可行，\
                 delegation.base_url 已指向 Sieve，子进程流量经过 Sieve。"
            );

            // R10-#1：将 hermes setup 记录到 setup.log，供 uninstall 查找（SPEC-004 §5.1）。
            // setup_complete entry：让 uninstall 能找到此次 backup_dir。
            let backup_detail_h = format!("backup_dir={}", ctx.backup_dir.display());
            ctx.append_log_entry(
                &SetupLogEntry::new("setup_complete")
                    .with_detail(backup_detail_h)
                    .with_agent(AgentKind::Hermes),
            )?;
            // config_modified entry：记录哪个文件被改写，供 uninstall 恢复（R10-#1）。
            ctx.append_log_entry(
                &SetupLogEntry::new("config_modified")
                    .with_path(config_path.to_string_lossy().to_string())
                    .with_detail(format!("fields: {}", changes.join("; ")))
                    .with_created_new(false)
                    .with_agent(AgentKind::Hermes),
            )?;
            Ok(())
        }

        fn doctor_check(&self) -> Result<DoctorReport> {
            // 1. hermes 二进制检查
            let version_ok = Command::new("hermes")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !version_ok {
                return Err(anyhow::anyhow!("hermes 二进制未找到或 --version 失败"));
            }
            println!("[doctor] ✅ Hermes：hermes --version 通过");

            // 2. daemon 监听检查
            let tcp_ok = std::net::TcpStream::connect_timeout(
                &"127.0.0.1:11453".parse().unwrap(),
                std::time::Duration::from_secs(2),
            )
            .is_ok();
            if !tcp_ok {
                eprintln!("[doctor] Hermes：Sieve daemon 未监听 127.0.0.1:11453");
                return Err(anyhow::anyhow!("Sieve daemon 未在 127.0.0.1:11453 监听"));
            }
            println!("[doctor] ✅ Hermes：Sieve daemon 在监听");

            // 3. 解析配置验证 model.base_url
            match self.read_config() {
                Ok(cfg) => {
                    let model_ok = cfg
                        .get("model")
                        .and_then(|m| m.get("base_url"))
                        .and_then(|u| u.as_str())
                        .map(|u| u == self.sieve_url)
                        .unwrap_or(false);
                    if model_ok {
                        println!("[doctor] ✅ Hermes：model.base_url 已指向 Sieve");
                    } else {
                        eprintln!(
                            "[doctor] ✗ Hermes：model.base_url 未指向 {}",
                            self.sieve_url
                        );
                        return Err(anyhow::anyhow!(
                            "Hermes model.base_url 配置不正确，请重新运行 sieve setup --agent hermes"
                        ));
                    }

                    let delegation_ok = cfg
                        .get("delegation")
                        .and_then(|d| d.get("base_url"))
                        .and_then(|u| u.as_str())
                        .map(|u| u == self.sieve_url)
                        .unwrap_or(false);
                    if delegation_ok {
                        println!(
                            "[doctor] ✅ Hermes：delegation.base_url 已指向 Sieve（TBD-06 降级）"
                        );
                    } else {
                        // delegation.base_url 未设置不是硬错误（delegation 字段可能不存在）
                        println!(
                            "[doctor] ⚠ Hermes：delegation.base_url 未指向 Sieve，\
                             Hermes 委托子进程的流量将不经过 Sieve（TBD-06 降级）"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Hermes doctor：无法读取配置文件（{}），跳过 provider 验证",
                        e
                    );
                    println!("[doctor] ⚠ Hermes：无法读取 config.yaml，跳过验证");
                }
            }

            // 4. X-Sieve-Origin header 说明
            println!(
                "[doctor] ⚠ Hermes X-Sieve-Origin：TBD-06 降级，\
                 ANTHROPIC_DEFAULT_HEADERS 注入不可行。\
                 sub-agent 调用链在 Phase 1 后期由 Sieve 端推断（SPEC-004 §10 TBD-06）"
            );

            Ok(DoctorReport::ok_for(AgentKind::Hermes))
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

    // ──────────────────────────────── install_shared_daemon ────────────────

    /// sentinel 文件路径：`~/.sieve/.daemon-installed`。
    ///
    /// setup 首次安装 daemon 后写入，防止多次 `sieve setup --agent <x>` 重复安装。
    fn daemon_sentinel_path(sieve_home: &Path) -> PathBuf {
        sieve_home.join(".daemon-installed")
    }

    /// 共享 daemon 安装（F-3）：写 sieve.toml + 规则文件 + launchd plist + launchctl load。
    ///
    /// 任何 agent 的首次 setup 都会调此函数。第二次调用（sentinel 存在）直接返回 Ok。
    ///
    /// ## 幂等设计
    ///
    /// 用 `~/.sieve/.daemon-installed` 作 sentinel：
    /// - 首次调用：安装并写 sentinel
    /// - 后续调用（sentinel 存在）：打印跳过日志，不重复安装
    ///
    /// ## 关联
    ///
    /// - known-issues-v1.4.md F-3（现已修复）
    /// - SPEC-004 §2.1
    fn install_shared_daemon(
        ctx: &mut SetupContext,
        home_path: &Path,
        sieve_home: &Path,
        backup_dir: &Path,
    ) -> Result<()> {
        let sentinel = daemon_sentinel_path(sieve_home);
        if sentinel.exists() {
            println!("[setup] daemon 已安装（sentinel 存在），跳过重复安装");
            return Ok(());
        }

        // F-2：部署内嵌规则文件到 ~/.sieve/rules/
        let rules_dir = sieve_home.join("rules");
        embedded_rules::install_to(&rules_dir)?;
        ctx.written_files.push(rules_dir.join("outbound.toml"));
        ctx.written_files.push(rules_dir.join("inbound.toml"));
        println!(
            "[setup] ✅ 规则文件写入 {}（outbound.toml + inbound.toml）",
            rules_dir.display()
        );

        let sieve_toml_path = sieve_home.join("sieve.toml");
        let plist_path = home_path
            .join("Library")
            .join("LaunchAgents")
            .join("com.sieve.daemon.plist");
        let setup_log_path = sieve_home.join("setup.log");

        let sieve_toml_existed_before = sieve_toml_path.exists();

        // 备份已有 sieve.toml
        if sieve_toml_existed_before {
            let rel = sieve_toml_path
                .strip_prefix(home_path)
                .unwrap_or(&sieve_toml_path);
            let backup_dest = backup_dir.join(rel);
            if let Some(parent) = backup_dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&sieve_toml_path, &backup_dest).context("备份 sieve.toml 失败")?;
        }

        if let Some(parent) = sieve_toml_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_content = build_default_sieve_toml(&sieve_toml_path)?;
        fs::write(&sieve_toml_path, toml_content.as_bytes()).context("写入 sieve.toml 失败")?;
        ctx.written_files.push(sieve_toml_path.clone());
        println!("[setup] ✅ sieve.toml 写入 {}", sieve_toml_path.display());

        // 写 launchd plist
        let plist_content = build_plist_content(&sieve_toml_path)?;
        if let Some(parent) = plist_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if plist_path.exists() {
            let rel = plist_path.strip_prefix(home_path).unwrap_or(&plist_path);
            let backup_dest = backup_dir.join(rel);
            if let Some(parent) = backup_dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&plist_path, &backup_dest).context("备份 plist 失败")?;
        }
        fs::write(&plist_path, plist_content.as_bytes()).context("写入 launchd plist 失败")?;
        ctx.written_files.push(plist_path.clone());
        println!("[setup] ✅ launchd plist 写入 {}", plist_path.display());

        // launchctl load
        let status = Command::new("launchctl")
            .args(["load", "-w", &plist_path.to_string_lossy()])
            .status()
            .context("执行 launchctl load 失败")?;
        if !status.success() {
            bail!("launchctl load 返回非零: {:?}", status.code());
        }
        ctx.launchd_loaded = Some(plist_path.clone());
        println!("[setup] ✅ launchd 服务已加载");

        // 写 setup.log
        {
            let entries: Vec<SetupLogEntry> = vec![
                SetupLogEntry::new("setup_complete")
                    .with_detail(format!("backup_dir={}", backup_dir.display()))
                    .with_agent(AgentKind::Claude),
                SetupLogEntry::new("rules_deployed")
                    .with_path(rules_dir.to_string_lossy().to_string())
                    .with_created_new(true)
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
                .open(&setup_log_path)
                .context("打开 setup.log 失败")?;
            for entry in &entries {
                let line = serde_json::to_string(entry)? + "\n";
                file.write_all(line.as_bytes())?;
            }
            println!("[setup] ✅ setup.log 写入 {}", setup_log_path.display());
        }

        // 写 sentinel，标记 daemon 已安装，防止重复安装
        fs::write(&sentinel, b"").context("写入 daemon sentinel 失败")?;
        ctx.written_files.push(sentinel);

        Ok(())
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

        // ── 4.5 F-3：安装共享 daemon（sieve.toml + 规则 + plist + launchctl）
        //
        // 任何 agent 的 setup 都需要 daemon 监听；以前只在 ClaudeAdapter::apply 里做，
        // 导致 --agent openclaw / hermes 时 daemon 根本没起。
        // install_shared_daemon 用 sentinel 文件防重复安装。
        {
            let mut daemon_ctx = SetupContext::new(backup_dir.clone());
            if let Err(e) =
                install_shared_daemon(&mut daemon_ctx, &home_path, &sieve_home, &backup_dir)
            {
                eprintln!("[setup] daemon 安装失败：{e}");
                daemon_ctx.rollback();
                return Err(anyhow!("daemon 安装失败，setup 已回滚：{}", e));
            }
        }

        // ── 5. 顺序 apply（SPEC-004 §7.1：单个失败只回滚该 agent，不影响其他已成功的）
        // 同时保留成功 apply 的 ctx，供后续 doctor 失败时回滚使用。
        let setup_log_path = sieve_home.join("setup.log");
        let mut any_failed = false;
        // (adapter_index, ctx) for successfully applied agents, in order
        let mut applied_ctxs: Vec<(AgentKind, SetupContext)> = Vec::new();
        for adapter in &adapters {
            let mut ctx =
                SetupContext::new(backup_dir.clone()).with_setup_log(setup_log_path.clone());
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

        // ── 6. 对每个已成功 apply 的 agent 跑 doctor 验证
        //
        // R10-#5 / P1-3：之前只验证 Claude，OpenClaw / Hermes 的写入不做同级 doctor，
        // 一旦配置写坏就只能等 dogfood 阶段才暴露。现在为每个 applied agent 重建对应
        // adapter，逐个 doctor_check。任一失败时**只回滚该 agent**（其他 agent 的成功
        // 不受影响），并返回非零；多个 agent 失败时累加错误信息。
        //
        // `SIEVE_SKIP_SETUP_DOCTOR=1`：仅集成测试用，跳过 doctor 让"配置注入"类
        // 单元测试不依赖 launchctl 实际能加载 daemon。生产路径**不应**设置此变量。
        if std::env::var("SIEVE_SKIP_SETUP_DOCTOR").as_deref() == Ok("1") {
            eprintln!("[setup] SIEVE_SKIP_SETUP_DOCTOR=1 检测到，跳过 doctor 验证（仅测试场景）");
            return Ok(());
        }
        let mut doctor_errors: Vec<String> = Vec::new();
        for (kind, ctx) in applied_ctxs.iter_mut() {
            let adapter: Box<dyn AgentAdapter> = match kind {
                AgentKind::Claude => {
                    Box::new(ClaudeAdapter::new(home_path.clone(), backup_dir.clone())?)
                }
                AgentKind::Openclaw => Box::new(OpenClawAdapter::new(home_path.clone())),
                AgentKind::Hermes => Box::new(HermesAdapter::new(home_path.clone())),
            };
            println!("\n[sieve setup] 正在验证 {} 安装…", kind);
            match adapter.doctor_check() {
                Ok(_report) => {
                    println!("[sieve setup] ✅ {} doctor 通过", kind);
                }
                Err(e) => {
                    eprintln!("[sieve setup] ❌ {} doctor 失败：{e}", kind);
                    eprintln!("[sieve setup] 正在回滚 {} 的改动…", kind);
                    ctx.rollback();
                    doctor_errors.push(format!("{kind}: {e}"));
                }
            }
        }

        if !doctor_errors.is_empty() {
            return Err(anyhow!(
                "setup 部分 agent doctor 验证失败，对应改动已自动回滚：\n  - {}\n\
                 其他 agent 的成功配置已保留；如需重试：sieve setup --agent <name>",
                doctor_errors.join("\n  - ")
            ));
        }

        Ok(())
    }

    // ──────────────────────────────── Claude setup 内部实现 ─────────────────

    /// Claude 特有配置注入：settings.json（ANTHROPIC_BASE_URL + PreToolUse hook）。
    ///
    /// F-3 重构后，daemon 共享安装（sieve.toml / 规则 / plist / launchd）由
    /// `install_shared_daemon()` 在主流程 run() 中统一完成；本函数只负责 Claude 特有部分。
    ///
    /// 9 个参数超过 clippy 默认阈值 7；已拆分共享 daemon 参数到 `install_shared_daemon`，
    /// 剩余参数均为调用方直接拥有的非结构化值，组成 struct 收益低于代价。
    #[allow(clippy::too_many_arguments)]
    fn do_claude_setup(
        ctx: &mut SetupContext,
        home_path: &Path,
        settings_path: &Path,
        setup_log_path: &Path,
        backup_dir: &Path,
        mut existing_settings: Value,
        settings_existed_before: bool,
        sieve_url: &str,
        hook_entry: Value,
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

        // 写 setup.log（Claude 特有条目）
        {
            let entries: Vec<SetupLogEntry> = vec![SetupLogEntry::new("settings_updated")
                .with_path(settings_path.to_string_lossy().to_string())
                .with_detail("env.ANTHROPIC_BASE_URL + hooks.PreToolUse")
                .with_created_new(!settings_existed_before)
                .with_agent(AgentKind::Claude)];
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(setup_log_path)
                .context("打开 setup.log 失败")?;
            for entry in &entries {
                let line = serde_json::to_string(entry)? + "\n";
                file.write_all(line.as_bytes())?;
            }
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
preset = "standard"
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

    // ── R10-#5 桥接：doctor.rs 复用 adapter doctor_check ────────────────────

    /// OpenClaw adapter 的 detect + doctor_check 桥接函数（R10-#5）。
    ///
    /// doctor.rs 通过此函数复用 OpenClawAdapter 的真实实现，
    /// 不需要直接引用 `pub(super)` 的结构体。
    ///
    /// 返回值：`None` = 未安装（跳过），`Some(Ok)` = 通过，`Some(Err)` = 失败。
    pub fn run_openclaw_doctor_check(home_path: std::path::PathBuf) -> Option<anyhow::Result<()>> {
        let adapter = OpenClawAdapter::new(home_path);
        let detection = match adapter.detect() {
            Ok(d) => d,
            Err(e) => return Some(Err(e)),
        };
        if !detection.installed {
            return None;
        }
        Some(adapter.doctor_check().map(|_| ()))
    }

    /// Hermes adapter 的 detect + doctor_check 桥接函数（R10-#5）。
    ///
    /// 同 `run_openclaw_doctor_check`，用途相同。
    pub fn run_hermes_doctor_check(home_path: std::path::PathBuf) -> Option<anyhow::Result<()>> {
        let adapter = HermesAdapter::new(home_path);
        let detection = match adapter.detect() {
            Ok(d) => d,
            Err(e) => return Some(Err(e)),
        };
        if !detection.installed {
            return None;
        }
        Some(adapter.doctor_check().map(|_| ()))
    }

    // ── 内部测试：SetupContext::rollback（直接访问私有结构）─────────────────────
    #[cfg(test)]
    mod tests_rollback {
        use super::*;
        use std::sync::Mutex;
        use tempfile::tempdir;

        // 模块级共享 ENV_LOCK：所有 HOME 环境变量改写测试串行执行。
        // 历史 bug：之前每个 `#[test]` 函数内各声明一个 `static ENV_LOCK`，但 Rust 中
        // fn 内 `static` 是 fn 自己的独立 item，跨 fn 不共享 → 并发 cargo test 时多个
        // 测试同时改 $HOME，CI 上偶发 setup_context_rollback_deletes_new_file 失败。
        // 修复：提到 mod 级别，所有测试共用同一把锁。
        static ENV_LOCK: Mutex<()> = Mutex::new(());

        // ── 测试 #5：rollback 确实恢复备份文件 ──────────────────────────────────
        // R5-#1 修复验证：backup 存在时 rollback 从备份恢复
        #[test]
        #[allow(unsafe_code)] // 测试隔离需要临时覆盖 HOME env var
        fn setup_context_rollback_restores_settings() {
            // env var 修改需要串行（共用 mod 级 ENV_LOCK，避免与 fn-内 static 不互斥）
            let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
            // 共用 mod 级 ENV_LOCK 与上一个测试串行，避免 $HOME 并发 race
            let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

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
