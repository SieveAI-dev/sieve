//! `sieve setup` 命令实现（ADR-015 / SPEC-003 §setup）。
//!
//! 仅 macOS Phase 1。非 macOS 编译进友好错误 stub，不影响构建。
//!
//! 步骤：
//! 1. 检测 `~/.claude/settings.json` 是否存在
//! 2. 计算 diff（ANTHROPIC_BASE_URL + PreToolUse hook + launchd plist）
//! 3. dry-run 打印 diff，非 --yes 等待用户确认
//! 4. 备份原文件到 `~/.sieve/backups/<RFC3339>/`
//! 5. 写 `~/.sieve/sieve.toml`（默认配置，绝对路径）
//! 6. 修改 settings.json（解析失败则 abort，不写任何内容）
//! 7. 写 launchd plist（命令包含 `--config <abs_path>/sieve.toml`）+ `launchctl load -w`
//! 8. 写 setup.log（JSON Lines，含 created_new 字段）
//! 9. 自动调用 doctor 验证
//!
//! 错误恢复：任意步骤失败 → 反向回滚已做改动。

use crate::cli::SetupArgs;
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

    /// setup.log 每行的结构（JSON Lines）。
    ///
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
    }

    impl SetupLogEntry {
        pub(super) fn new(action: impl Into<String>) -> Self {
            Self {
                timestamp: Utc::now().to_rfc3339(),
                action: action.into(),
                path: None,
                detail: None,
                created_new: false,
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
    }

    /// setup 执行上下文，用于错误时反向回滚。
    struct SetupContext {
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

        /// 回滚所有已做改动（从备份目录恢复）。
        fn rollback(&self) {
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

    /// 运行 `sieve setup`。关联 ADR-015 / SPEC-003 §setup。
    pub fn run(args: SetupArgs) -> Result<()> {
        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
        let home_path = PathBuf::from(&home);

        let settings_path = home_path.join(".claude").join("settings.json");
        let sieve_home =
            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
        let backup_ts = Utc::now().to_rfc3339().replace(':', "-");
        let backup_dir = sieve_home.join("backups").join(&backup_ts);
        let plist_path = home_path
            .join("Library")
            .join("LaunchAgents")
            .join("com.sieve.daemon.plist");
        let setup_log_path = sieve_home.join("setup.log");

        // ── 1. 读取现有 settings.json（允许不存在；解析失败则 abort，不覆盖用户文件）
        let settings_existed_before = settings_path.exists();
        let existing_settings: Value = if settings_existed_before {
            let raw =
                fs::read_to_string(&settings_path).context("读取 ~/.claude/settings.json 失败")?;
            // Strip JSON 注释（简单处理：删除 // 行注释）
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
        // sieve.toml 将写入 ~/.sieve/sieve.toml（绝对路径）
        let sieve_toml_path = sieve_home.join("sieve.toml");

        // ── 2. 计算 diff
        let sieve_url = "http://127.0.0.1:11453";
        let hook_entry = serde_json::json!({
            "matcher": ".*",
            "hooks": [{"type": "command", "command": "sieve-hook check"}]
        });

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
        let plist_content = build_plist_content(&sieve_toml_path)?;

        // ── 3. 打印 diff
        println!("=== sieve setup diff ===");
        println!(
            "[settings.json] env.ANTHROPIC_BASE_URL: {:?} → {:?}",
            current_base_url, sieve_url
        );
        if has_hook {
            println!("[settings.json] hooks.PreToolUse: sieve-hook 已存在（幂等）");
        } else {
            println!("[settings.json] hooks.PreToolUse: 新增 sieve-hook check 条目");
        }
        if sieve_toml_path.exists() {
            println!(
                "[sieve.toml] {} 已存在，将覆盖（原文件备份到 backups/）",
                sieve_toml_path.display()
            );
        } else {
            println!("[sieve.toml] 新建 {}", sieve_toml_path.display());
        }
        println!(
            "[launchd] 写入 {} (含 --config {})",
            plist_path.display(),
            sieve_toml_path.display()
        );
        println!("[launchd] 执行 launchctl load -w");
        println!("========================");

        // ── 4. dry-run 直接返回
        if args.dry_run {
            println!("[dry-run] 未做任何改动。");
            return Ok(());
        }

        // ── 5. 等待用户确认
        if !args.yes {
            print!("继续执行以上操作？[y/N] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("已取消。");
                return Ok(());
            }
        }

        // ── 6. 备份
        fs::create_dir_all(&backup_dir)
            .with_context(|| format!("创建备份目录 {} 失败", backup_dir.display()))?;
        let mut ctx = SetupContext::new(backup_dir.clone());

        let result = do_setup(
            &mut ctx,
            &home_path,
            &settings_path,
            &plist_path,
            &sieve_toml_path,
            &setup_log_path,
            &backup_dir,
            existing_settings,
            settings_existed_before,
            sieve_url,
            hook_entry,
            plist_content,
        );

        if let Err(ref e) = result {
            eprintln!("[sieve setup] 失败: {e}");
            ctx.rollback();
            return result;
        }

        // ── 9. 自动跑 doctor 验证
        println!("\n[sieve setup] 正在验证安装…");
        doctor::run()?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn do_setup(
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

        // 写 setup.log（含 created_new 字段，供 uninstall 精确还原）
        {
            let entries: Vec<SetupLogEntry> = vec![
                SetupLogEntry::new("setup_complete")
                    .with_detail(format!("backup_dir={}", backup_dir.display())),
                SetupLogEntry::new("settings_updated")
                    .with_path(settings_path.to_string_lossy().to_string())
                    .with_detail("env.ANTHROPIC_BASE_URL + hooks.PreToolUse")
                    .with_created_new(!settings_existed_before),
                SetupLogEntry::new("sieve_toml_written")
                    .with_path(sieve_toml_path.to_string_lossy().to_string())
                    .with_created_new(!sieve_toml_existed_before),
                SetupLogEntry::new("launchd_loaded")
                    .with_path(plist_path.to_string_lossy().to_string()),
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

    // ── 测试 #3：SetupLogEntry 序列化 created_new 字段 ──────────────────────
    // 修复 #9 数据基础：setup.log 正确记录 created_new=true/false

    #[test]
    fn setup_log_entry_created_new_serializes_correctly() {
        let entry_new = SetupLogEntry::new("settings_updated")
            .with_path("/tmp/test.json".to_string())
            .with_created_new(true);
        let json = serde_json::to_string(&entry_new).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            v.get("created_new").and_then(|c| c.as_bool()),
            Some(true),
            "新建文件 created_new 应序列化为 true: {json}"
        );

        let entry_existing = SetupLogEntry::new("settings_updated")
            .with_path("/tmp/test.json".to_string())
            .with_created_new(false);
        let json2 = serde_json::to_string(&entry_existing).unwrap();
        let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
        assert_eq!(
            v2.get("created_new").and_then(|c| c.as_bool()),
            Some(false),
            "已有文件 created_new 应序列化为 false: {json2}"
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
