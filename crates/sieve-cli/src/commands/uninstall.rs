//! `sieve uninstall` 命令实现（ADR-015 / SPEC-003 §uninstall / SPEC-004 §2.3）。
//!
//! 步骤：
//! 1. 读 `~/.sieve/setup.log` 反向遍历 entries（了解 backup_dir + created_new 标志）
//! 2. 按 `--agent` 过滤 entries（SPEC-004 §5.2）；`--all` 时不过滤
//! 3. dry-run 打印将恢复的内容
//! 4. 非 --yes 等待用户确认
//! 5. 按 setup.log 记录的 created_new 字段决定还原策略：
//!    - `created_new = true`：setup 前不存在，直接删除（恢复"原状"）
//!    - `created_new = false`：仅移除 Sieve entries（ANTHROPIC_BASE_URL + sieve-hook），
//!      保留用户 setup 后添加的其他配置
//! 6. `launchctl unload` 并删除 plist 文件（仅在 --all 或最后一家 agent 时）
//! 7. 提示用户手动删 `~/.sieve/`
//!
//! 不传 `--agent` 且不传 `--all` 时：输出提示并 exit 2（SPEC-004 §2.3）。
//!
//! 仅 macOS Phase 1 支持；非 macOS 编译进 stub。

use crate::cli::UninstallArgs;
use anyhow::Result;

#[cfg(target_os = "macos")]
pub use macos::run;

#[cfg(not(target_os = "macos"))]
pub use stub::run;

// ──────────────────────────────── macOS 实现 ────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use anyhow::{anyhow, Context};
    use std::fs;
    use std::io::{self, Write as IoWrite};
    use std::path::PathBuf;
    use std::process::Command;

    /// setup.log entry 镜像（只读取需要的字段）。
    #[derive(serde::Deserialize)]
    struct SetupLogEntry {
        action: String,
        path: Option<String>,
        detail: Option<String>,
        #[serde(default)]
        created_new: bool,
        /// 归属 agent（SPEC-004 §5.1）。
        #[serde(default)]
        agent: Option<String>,
    }

    /// 记录 setup 写入文件的还原策略。
    pub(super) struct FileRestoreInfo {
        /// 文件绝对路径。
        pub(super) path: PathBuf,
        /// true → setup 前不存在，uninstall 时删除；false → 仅移除 Sieve entries。
        pub(super) created_new: bool,
    }

    /// 运行 `sieve uninstall`。关联 ADR-015 / SPEC-003 §uninstall / SPEC-004 §2.3。
    pub fn run(args: UninstallArgs) -> Result<()> {
        // ── 0. 参数校验：必须传 --agent 或 --all（SPEC-004 §2.3）
        if args.agent.is_none() && !args.all {
            eprintln!("请指定 --agent <name> 或 --all。");
            eprintln!("示例：sieve uninstall --agent claude");
            eprintln!("      sieve uninstall --all");
            std::process::exit(2);
        }

        let home = std::env::var("HOME").map_err(|_| anyhow!("HOME 环境变量未设置"))?;
        let home_path = PathBuf::from(&home);
        let sieve_home =
            sieve_ipc::paths::sieve_home().map_err(|e| anyhow!("获取 sieve home 失败: {e}"))?;
        let setup_log_path = sieve_home.join("setup.log");
        let plist_path = home_path
            .join("Library")
            .join("LaunchAgents")
            .join("com.sieve.daemon.plist");
        let backups_root = sieve_home.join("backups");

        // ── 1. 读取 setup.log，按 agent 过滤，找到 backup_dir + 各文件 created_new 标志
        let agent_filter: Option<String> = args.agent.map(|a| a.to_string());
        let (latest_backup, file_restore_infos) =
            read_setup_log(&setup_log_path, &backups_root, agent_filter.as_deref());

        // R6-#1：--agent <非 claude> 且无匹配 entry → 直接提示并退出，避免误恢复 Claude 文件
        if latest_backup.is_none()
            && file_restore_infos.is_empty()
            && matches!(agent_filter.as_deref(), Some(f) if f != "claude")
        {
            let name = agent_filter.as_deref().unwrap_or("unknown");
            eprintln!("no setup record found for --agent {name}; nothing to uninstall");
            return Ok(());
        }

        // ── 2. 打印将要恢复的内容
        let agent_label = args
            .agent
            .map(|a| format!(" (agent: {})", a))
            .unwrap_or_else(|| " (--all)".to_string());
        println!("=== sieve uninstall 预览{} ===", agent_label);
        if !file_restore_infos.is_empty() {
            for info in &file_restore_infos {
                if info.created_new {
                    println!("[restore] 删除（setup 新建）: {}", info.path.display());
                } else {
                    println!("[restore] 移除 Sieve entries: {}", info.path.display());
                }
            }
        } else if let Some(ref bd) = latest_backup {
            println!("[restore] 从备份目录恢复: {}", bd.display());
            list_backup_files(bd);
        } else {
            println!("[restore] 未找到 setup.log 记录，将跳过文件恢复");
        }

        // daemon plist：仅 --all 或 Claude agent 时处理（daemon 共享资源，SPEC-004 §5.2）
        let should_unload_plist = args.all
            || args
                .agent
                .map(|a| matches!(a, crate::cli::AgentKind::Claude))
                .unwrap_or(false);
        if should_unload_plist && plist_path.exists() {
            println!("[launchd] launchctl unload {}", plist_path.display());
            println!("[launchd] 删除 {}", plist_path.display());
        }
        println!("[提示] ~/.sieve/ 目录将保留（含审计日志），请手动删除：");
        println!("       rm -rf {}", sieve_home.display());
        println!("=============================");

        if args.dry_run {
            println!("[dry-run] 未做任何改动。");
            return Ok(());
        }

        // ── 3. 等待用户确认
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

        // ── 4. 按 created_new 标志决定还原策略
        if !file_restore_infos.is_empty() {
            restore_files(&file_restore_infos, &home_path, latest_backup.as_deref())?;
        } else if let Some(ref bd) = latest_backup {
            // 旧格式 setup.log（无 created_new），退回全量备份恢复
            restore_from_backup(bd, &home_path)?;
        }

        // ── 5. 卸载 launchd（仅 --all 或 Claude agent）
        if should_unload_plist && plist_path.exists() {
            let status = Command::new("launchctl")
                .args(["unload", &plist_path.to_string_lossy()])
                .status();
            match status {
                Ok(s) if s.success() => println!("[uninstall] ✅ launchd 服务已卸载"),
                Ok(s) => eprintln!("[uninstall] ⚠ launchctl unload 返回: {:?}", s.code()),
                Err(e) => eprintln!("[uninstall] ⚠ launchctl unload 失败: {e}"),
            }
            if let Err(e) = fs::remove_file(&plist_path) {
                eprintln!("[uninstall] ⚠ 删除 plist 失败: {e}");
            } else {
                println!("[uninstall] ✅ plist 已删除");
            }
        }

        // ── 6. 提示手动删除
        println!();
        println!("✅ 卸载完成。");
        println!("提示：审计日志和备份文件保留在 {}", sieve_home.display());
        println!("如需彻底清除，请手动运行：");
        println!("  rm -rf {}", sieve_home.display());

        Ok(())
    }

    /// 从 setup.log 读取最新 backup_dir 和文件还原信息。
    ///
    /// `agent_filter`：Some("claude") 时只处理该 agent 的 entry；None（--all）时处理全部。
    ///
    /// 返回 (latest_backup_dir, file_restore_infos)。
    /// file_restore_infos 为空时表示 setup.log 是旧格式，退回全量备份恢复。
    #[cfg(test)]
    pub(super) fn read_setup_log_for_test(
        setup_log: &std::path::Path,
        backups_root: &std::path::Path,
        agent_filter: Option<&str>,
    ) -> (Option<PathBuf>, Vec<FileRestoreInfo>) {
        read_setup_log(setup_log, backups_root, agent_filter)
    }

    fn read_setup_log(
        setup_log: &std::path::Path,
        backups_root: &std::path::Path,
        agent_filter: Option<&str>,
    ) -> (Option<PathBuf>, Vec<FileRestoreInfo>) {
        let Ok(raw) = fs::read_to_string(setup_log) else {
            // setup.log 不存在：仅在 --all 或 --agent claude 时 fallback 到全局备份目录，
            // 避免 --agent openclaw 等非 Claude agent 误恢复 Claude 文件（R7-#4）。
            let backup = if matches!(agent_filter, None | Some("claude")) {
                find_latest_backup_dir(backups_root)
            } else {
                None
            };
            return (backup, vec![]);
        };

        let entries: Vec<SetupLogEntry> = raw
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        // 找最新 setup_complete entry 的 backup_dir（按 agent 过滤）
        let latest_backup = entries
            .iter()
            .rev()
            .find(|e| e.action == "setup_complete" && agent_matches(&e.agent, agent_filter))
            .and_then(|e| e.detail.as_deref())
            .and_then(|d| d.strip_prefix("backup_dir="))
            .map(PathBuf::from);

        // 收集文件 action（settings_updated / sieve_toml_written），取最新一次 setup 的记录
        // 策略：找最后一个 setup_complete 之后的所有文件 action
        let last_setup_idx = entries
            .iter()
            .rposition(|e| e.action == "setup_complete" && agent_matches(&e.agent, agent_filter))
            .unwrap_or(0);

        // config_modified：OpenClaw / Hermes apply 写入的 action（R10-#1）。
        let file_actions = ["settings_updated", "sieve_toml_written", "config_modified"];
        let infos: Vec<FileRestoreInfo> = entries[last_setup_idx..]
            .iter()
            .filter(|e| {
                file_actions.contains(&e.action.as_str()) && agent_matches(&e.agent, agent_filter)
            })
            .filter_map(|e| {
                let path_str = e.path.as_deref()?;
                Some(FileRestoreInfo {
                    path: PathBuf::from(path_str),
                    created_new: e.created_new,
                })
            })
            .collect();

        // 如果没有文件记录（旧格式 setup.log），返回空 infos 触发备份恢复兜底。
        //
        // fallback 到全局备份仅允许在 --all 或 --agent claude 时触发，
        // 避免 --agent openclaw / --agent hermes 等单 agent 误恢复 Claude 文件（R6-#1）。
        let backup = latest_backup.or_else(|| {
            // `agent_filter = None` 表示 --all；Some("claude") 允许旧格式 fallback（v1.4 兼容）
            if matches!(agent_filter, None | Some("claude")) {
                find_latest_backup_dir(backups_root)
            } else {
                None
            }
        });
        (backup, infos)
    }

    /// 判断 entry 的 agent 字段是否匹配过滤条件。
    ///
    /// - `agent_filter = None`（--all）：匹配所有
    /// - `agent_filter = Some("claude")`：只匹配 agent == "claude"
    ///
    /// 旧格式 entry（无 agent 字段，`entry_agent = None`）默认归属 "claude"——
    /// v1.4 只支持 Claude，因此旧 entry 必然是 Claude 的改动（SPEC-004 §5.2）。
    pub(super) fn agent_matches(entry_agent: &Option<String>, filter: Option<&str>) -> bool {
        match filter {
            None => true, // --all：不过滤
            Some(f) => {
                // 无 agent 字段的旧格式 entry 默认归 claude
                let agent = entry_agent.as_deref().unwrap_or("claude");
                agent == f
            }
        }
    }

    /// 扫描 backups/ 下最新目录（按名称字典序，RFC3339 时间戳排序正确）。
    fn find_latest_backup_dir(backups_root: &std::path::Path) -> Option<PathBuf> {
        if !backups_root.exists() {
            return None;
        }
        let mut entries: Vec<PathBuf> = fs::read_dir(backups_root)
            .ok()?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect();
        entries.sort();
        entries.into_iter().next_back()
    }

    /// 按 created_new 标志还原文件。
    ///
    /// - `created_new = true`：setup 前不存在，直接删除
    /// - `created_new = false`：仅从文件内移除 Sieve entries（保留用户其他配置）
    pub(super) fn restore_files(
        infos: &[FileRestoreInfo],
        _home_path: &std::path::Path,
        backup_dir: Option<&std::path::Path>,
    ) -> Result<()> {
        for info in infos {
            if !info.path.exists() {
                println!("[uninstall] 跳过（文件不存在）: {}", info.path.display());
                continue;
            }

            if info.created_new {
                // setup 前不存在 → 删除整个文件
                fs::remove_file(&info.path)
                    .with_context(|| format!("删除 setup 新建文件 {} 失败", info.path.display()))?;
                println!("[uninstall] ✅ 删除（setup 新建）: {}", info.path.display());
            } else {
                // setup 前已存在 → 仅移除 Sieve entries，保留用户其他配置
                // 对 settings.json：移除 env.ANTHROPIC_BASE_URL + hooks.PreToolUse 中 sieve-hook 条目
                // 对其他文件（openclaw.json / hermes config.yaml 等）：从备份恢复（R10-#1）
                let extension = info.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                // settings.json（Claude Code 配置）：精确移除 Sieve entries，保留用户其他配置。
                // 其他 JSON（openclaw.json 等）：从备份恢复整个文件（R10-#1）。
                let is_settings_json = info
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n == "settings.json")
                    .unwrap_or(false);
                if extension == "json" && is_settings_json {
                    match remove_sieve_entries_from_settings(&info.path) {
                        Ok(()) => {
                            println!("[uninstall] ✅ 移除 Sieve entries: {}", info.path.display());
                        }
                        Err(e) => {
                            // 移除 entries 失败，退回备份恢复
                            eprintln!("[uninstall] ⚠ 移除 entries 失败: {e}，尝试从备份恢复");
                            if let Some(bd) = backup_dir {
                                restore_file_from_backup(bd, &info.path)?;
                            }
                        }
                    }
                } else if extension == "json" {
                    // 非 settings.json 的 JSON（openclaw.json 等）：从备份恢复整个文件（R10-#1）
                    if let Some(bd) = backup_dir {
                        restore_file_from_backup(bd, &info.path)?;
                    } else {
                        eprintln!("[uninstall] ⚠ 无备份可恢复: {}", info.path.display());
                    }
                } else if extension == "toml" {
                    // toml 文件同样按 created_new 判断：
                    // - created_new=false → setup 前用户已有该文件，从备份恢复
                    // - created_new=true  → setup 新建，但 created_new=true 分支在上面已处理
                    // 此处 created_new 必定为 false（else 分支），从备份恢复用户原文件。
                    if let Some(bd) = backup_dir {
                        restore_file_from_backup(bd, &info.path)?;
                    } else {
                        // 无备份可恢复：只能删除（避免残留 Sieve 配置影响用户）
                        fs::remove_file(&info.path).with_context(|| {
                            format!("删除 {} 失败（无备份）", info.path.display())
                        })?;
                        println!("[uninstall] ✅ 删除（无备份）: {}", info.path.display());
                    }
                } else {
                    // 其他文件：从备份恢复
                    if let Some(bd) = backup_dir {
                        restore_file_from_backup(bd, &info.path)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 从 settings.json 中移除 Sieve 注入的 entries，保留用户其他配置。
    ///
    /// 移除：
    /// - `env.ANTHROPIC_BASE_URL`（若值为 `http://127.0.0.1:11453`）
    /// - `hooks.PreToolUse` 数组中包含 `sieve-hook` 的条目
    pub(super) fn remove_sieve_entries_from_settings(
        settings_path: &std::path::Path,
    ) -> Result<()> {
        let raw = fs::read_to_string(settings_path)
            .with_context(|| format!("读取 {} 失败", settings_path.display()))?;
        let mut v: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("解析 {} 失败", settings_path.display()))?;

        // 移除 env.ANTHROPIC_BASE_URL（仅当值为 sieve url 时）
        if let Some(env) = v.get_mut("env").and_then(|e| e.as_object_mut()) {
            if env
                .get("ANTHROPIC_BASE_URL")
                .and_then(|u| u.as_str())
                .map(|s| s == "http://127.0.0.1:11453")
                .unwrap_or(false)
            {
                env.remove("ANTHROPIC_BASE_URL");
                // 如果 env 对象变空，也一并移除（避免留下空对象）
                if env.is_empty() {
                    v.as_object_mut().map(|obj| obj.remove("env"));
                }
            }
        }

        // 移除 hooks.PreToolUse 中含 sieve-hook 的条目
        if let Some(pre_tool) = v
            .pointer_mut("/hooks/PreToolUse")
            .and_then(|a| a.as_array_mut())
        {
            pre_tool.retain(|item| {
                !item
                    .pointer("/hooks/0/command")
                    .and_then(|c| c.as_str())
                    .map(|c| c.contains("sieve-hook"))
                    .unwrap_or(false)
            });
        }
        // 如果 hooks.PreToolUse 变空，移除该 key
        let pre_tool_empty = v
            .pointer("/hooks/PreToolUse")
            .and_then(|a| a.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false);
        if pre_tool_empty {
            if let Some(hooks) = v.get_mut("hooks").and_then(|h| h.as_object_mut()) {
                hooks.remove("PreToolUse");
                if hooks.is_empty() {
                    v.as_object_mut().map(|obj| obj.remove("hooks"));
                }
            }
        }

        let json_str = serde_json::to_string_pretty(&v)?;
        fs::write(settings_path, json_str.as_bytes())
            .with_context(|| format!("写入 {} 失败", settings_path.display()))?;
        Ok(())
    }

    /// 从备份目录恢复单个文件。
    fn restore_file_from_backup(
        backup_dir: &std::path::Path,
        target: &std::path::Path,
    ) -> Result<()> {
        // 计算 backup 中的对应路径（target 的绝对路径去掉 HOME 前缀）
        let home = std::env::var("HOME").unwrap_or_default();
        let rel = target.strip_prefix(&home).unwrap_or(target);
        let backup_src = backup_dir.join(rel);
        if backup_src.exists() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&backup_src, target).with_context(|| {
                format!(
                    "从备份恢复 {} → {} 失败",
                    backup_src.display(),
                    target.display()
                )
            })?;
            println!("[uninstall] ✅ 从备份恢复: {}", target.display());
        } else {
            eprintln!("[uninstall] ⚠ 备份文件不存在: {}", backup_src.display());
        }
        Ok(())
    }

    /// 打印备份目录中的文件列表。
    fn list_backup_files(backup_dir: &std::path::Path) {
        if let Ok(walker) = fs::read_dir(backup_dir) {
            for entry in walker.flatten() {
                println!("  - {}", entry.path().display());
            }
        }
    }

    /// 将备份目录中的文件逐一恢复到 home 下对应路径（旧格式 setup.log 兜底）。
    fn restore_from_backup(
        backup_dir: &std::path::Path,
        home_path: &std::path::Path,
    ) -> Result<()> {
        restore_dir_recursive(backup_dir, backup_dir, home_path)
    }

    fn restore_dir_recursive(
        root: &std::path::Path,
        current: &std::path::Path,
        home_path: &std::path::Path,
    ) -> Result<()> {
        for entry in fs::read_dir(current)
            .with_context(|| format!("读取备份目录 {} 失败", current.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                restore_dir_recursive(root, &path, home_path)?;
            } else {
                // 计算目标路径：backup_root/rel → home/rel
                let rel = path.strip_prefix(root).unwrap_or(path.as_path());
                let dest = home_path.join(rel);
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&path, &dest).with_context(|| {
                    format!("恢复 {} → {} 失败", path.display(), dest.display())
                })?;
                println!("[uninstall] ✅ 恢复 {}", dest.display());
            }
        }
        Ok(())
    }
}

// ──────────────────────────────── 非 macOS stub ─────────────────────────────

#[cfg(not(target_os = "macos"))]
mod stub {
    use super::*;

    /// `sieve uninstall` 非 macOS 占位实现。
    pub fn run(_args: UninstallArgs) -> Result<()> {
        anyhow::bail!(
            "sieve uninstall is macOS only in Phase 1. \
             Linux/Windows support is planned for Phase 2."
        )
    }
}

// ──────────────────────────────── 单元测试 ──────────────────────────────────

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::macos::{restore_files, FileRestoreInfo};
    use std::fs;
    use tempfile::tempdir;

    // ── 测试 #4：uninstall 在 created_new=true entry 上删除整个文件 ─────────

    #[test]
    fn uninstall_created_new_true_deletes_file() {
        let dir = tempdir().unwrap();
        let settings = dir.path().join("settings.json");
        fs::write(
            &settings,
            r#"{"env":{"ANTHROPIC_BASE_URL":"http://127.0.0.1:11453"}}"#,
        )
        .unwrap();

        let infos = vec![FileRestoreInfo {
            path: settings.clone(),
            created_new: true,
        }];

        restore_files(&infos, dir.path(), None).unwrap();

        assert!(
            !settings.exists(),
            "created_new=true 时 uninstall 应删除整个文件"
        );
    }

    // ── 测试 #5：uninstall 在 created_new=false entry 上仅移除 Sieve entries ─

    #[test]
    fn uninstall_created_new_false_removes_sieve_entries_only() {
        let dir = tempdir().unwrap();
        // 模拟 Claude Code 的真实路径：settings.json 位于 .claude 目录下
        let claude_dir = dir.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        let settings = claude_dir.join("settings.json");

        // 模拟 setup 后的 settings.json：包含 Sieve entries 和用户原有配置
        let content = serde_json::json!({
            "env": {
                "ANTHROPIC_BASE_URL": "http://127.0.0.1:11453",
                "USER_VAR": "user_value"
            },
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": ".*",
                        "hooks": [{"type": "command", "command": "sieve-hook check"}]
                    },
                    {
                        "matcher": ".*",
                        "hooks": [{"type": "command", "command": "user-hook"}]
                    }
                ]
            },
            "model": "claude-opus-4-5"
        });
        fs::write(&settings, serde_json::to_string_pretty(&content).unwrap()).unwrap();

        let infos = vec![FileRestoreInfo {
            path: settings.clone(),
            created_new: false,
        }];

        restore_files(&infos, dir.path(), None).unwrap();

        assert!(settings.exists(), "created_new=false 时文件应保留");

        let result: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&settings).unwrap()).unwrap();

        // Sieve entries 应被移除
        assert!(
            result.pointer("/env/ANTHROPIC_BASE_URL").is_none(),
            "ANTHROPIC_BASE_URL 应被移除"
        );
        // 用户原有字段应保留
        assert_eq!(
            result.pointer("/env/USER_VAR").and_then(|v| v.as_str()),
            Some("user_value"),
            "用户 env 变量应保留"
        );
        // 用户的其他 hook 应保留
        let pre_tool = result
            .pointer("/hooks/PreToolUse")
            .and_then(|a| a.as_array())
            .unwrap();
        assert_eq!(pre_tool.len(), 1, "只应剩 1 个用户 hook");
        assert!(
            pre_tool[0]
                .pointer("/hooks/0/command")
                .and_then(|c| c.as_str())
                .map(|c| c.contains("user-hook"))
                .unwrap_or(false),
            "用户 hook 应保留"
        );
        // model 等其他字段应保留
        assert_eq!(
            result.get("model").and_then(|v| v.as_str()),
            Some("claude-opus-4-5"),
            "model 字段应保留"
        );
    }

    // ── R2-#5：toml 文件按 created_new 分流测试 ─────────────────────────────

    #[test]
    fn uninstall_toml_created_new_true_deletes_file() {
        // sieve.toml 由 setup 新建（created_new=true）→ uninstall 应删除整个文件
        let dir = tempdir().unwrap();
        let sieve_toml = dir.path().join("sieve.toml");
        fs::write(
            &sieve_toml,
            "upstream_url = \"https://api.anthropic.com\"\nport = 11453\n",
        )
        .unwrap();

        let infos = vec![FileRestoreInfo {
            path: sieve_toml.clone(),
            created_new: true,
        }];

        restore_files(&infos, dir.path(), None).unwrap();

        assert!(
            !sieve_toml.exists(),
            "created_new=true 时 sieve.toml 应被删除"
        );
    }

    #[test]
    fn uninstall_toml_created_new_false_restores_from_backup() {
        // 用户 setup 前已有 sieve.toml（created_new=false）→ 从备份恢复
        let dir = tempdir().unwrap();

        // 模拟 home_dir（充当 HOME）和 backup_dir
        let home_dir = dir.path().join("home");
        fs::create_dir_all(&home_dir).unwrap();

        let backup_dir = dir.path().join("backup");
        fs::create_dir_all(&backup_dir).unwrap();

        // sieve.toml 实际路径（在 home_dir 下）
        let sieve_toml = home_dir.join("sieve.toml");

        // 用户原始内容存放在 backup_dir/sieve.toml
        // restore_file_from_backup: target.strip_prefix(HOME) = "sieve.toml"
        // → backup_dir.join("sieve.toml") = backup_dir/sieve.toml ✓
        let original_content =
            "# 用户原始配置\nupstream_url = \"https://api.anthropic.com\"\nport = 9999\n";
        fs::write(backup_dir.join("sieve.toml"), original_content).unwrap();

        // 当前文件（被 setup 覆盖后的内容）
        let sieve_content_after_setup =
            "upstream_url = \"https://api.anthropic.com\"\nport = 11453\n";
        fs::write(&sieve_toml, sieve_content_after_setup).unwrap();

        let infos = vec![FileRestoreInfo {
            path: sieve_toml.clone(),
            created_new: false,
        }];

        // 临时设 HOME 让 restore_file_from_backup 正确 strip 前缀
        std::env::set_var("HOME", &home_dir);
        restore_files(&infos, &home_dir, Some(backup_dir.as_path())).unwrap();

        // 文件应仍存在，内容从备份恢复
        assert!(sieve_toml.exists(), "sieve.toml 应存在（从备份恢复）");
        let restored = fs::read_to_string(&sieve_toml).unwrap();
        assert_eq!(
            restored, original_content,
            "sieve.toml 内容应从备份恢复为用户原始内容"
        );
    }

    // ── A2-#4：agent_matches 旧格式 entry 默认归 claude ──────────────────────

    use super::macos::agent_matches;

    /// 旧 entry（无 agent 字段）+ --agent claude → 匹配（默认归 claude）
    #[test]
    fn agent_matches_legacy_entry_matches_claude() {
        assert!(
            agent_matches(&None, Some("claude")),
            "无 agent 字段的旧格式 entry 应归 claude，--agent claude 应匹配"
        );
    }

    /// 旧 entry（无 agent 字段）+ --agent openclaw → 不匹配（修复关键 case）
    #[test]
    fn agent_matches_legacy_entry_does_not_match_openclaw() {
        assert!(
            !agent_matches(&None, Some("openclaw")),
            "无 agent 字段的旧格式 entry 不应被 --agent openclaw 误匹配"
        );
    }

    /// 旧 entry（无 agent 字段）+ --agent hermes → 不匹配
    #[test]
    fn agent_matches_legacy_entry_does_not_match_hermes() {
        assert!(
            !agent_matches(&None, Some("hermes")),
            "无 agent 字段的旧格式 entry 不应被 --agent hermes 误匹配"
        );
    }

    /// 旧 entry（无 agent 字段）+ --all（filter=None）→ 匹配
    #[test]
    fn agent_matches_legacy_entry_matches_all() {
        assert!(
            agent_matches(&None, None),
            "--all 时不过滤，旧格式 entry 应匹配"
        );
    }

    /// 新 entry agent="openclaw" + --agent openclaw → 匹配（无回归）
    #[test]
    fn agent_matches_new_openclaw_matches_openclaw() {
        assert!(
            agent_matches(&Some("openclaw".to_string()), Some("openclaw")),
            "新格式 entry agent=openclaw 应被 --agent openclaw 匹配"
        );
    }

    /// 新 entry agent="claude" + --agent openclaw → 不匹配（无回归）
    #[test]
    fn agent_matches_new_claude_does_not_match_openclaw() {
        assert!(
            !agent_matches(&Some("claude".to_string()), Some("openclaw")),
            "新格式 entry agent=claude 不应被 --agent openclaw 匹配"
        );
    }

    // ── R6-#1 测试：uninstall --agent openclaw 无 entry → 不触发 fallback 备份 ──

    use super::macos::read_setup_log_for_test;

    /// R6-#1 场景 A：setup.log 仅含 Claude entry，--agent openclaw → backup 和 infos 均为 None/empty
    ///
    /// 修复关键：不应 fallback 到全局 backups/ 目录。
    #[test]
    fn uninstall_openclaw_no_entry_returns_none_no_fallback() {
        let dir = tempdir().unwrap();
        let setup_log = dir.path().join("setup.log");
        let backups_root = dir.path().join("backups");

        // setup.log 只含 claude entry（模拟旧版只装了 Claude 的用户）
        let log_entry = serde_json::json!({
            "action": "setup_complete",
            "detail": "backup_dir=/tmp/backup_2026",
            "agent": "claude"
        });
        fs::write(&setup_log, format!("{}\n", log_entry)).unwrap();

        // backups/ 中放一个 fake 备份目录（如果 fallback 生效，就会被误用）
        fs::create_dir_all(backups_root.join("2026-04-27T00:00:00")).unwrap();
        fs::write(
            backups_root
                .join("2026-04-27T00:00:00")
                .join("settings.json"),
            r#"{"env":{}}"#,
        )
        .unwrap();

        let (backup, infos) = read_setup_log_for_test(&setup_log, &backups_root, Some("openclaw"));

        assert!(
            backup.is_none(),
            "--agent openclaw 无匹配 entry 时不应 fallback 到全局备份，得到 backup={backup:?}"
        );
        assert!(
            infos.is_empty(),
            "--agent openclaw 无匹配 entry 时 infos 应为空"
        );
    }

    // ── R7-#4 测试：setup.log 完全不存在时的 agent_filter 保护 ──────────────

    /// R7-#4 场景 A：setup.log 不存在 + --agent openclaw → backup=None，不 fallback
    ///
    /// 修复 R7-#4：早期缺失分支无条件返回 find_latest_backup_dir，忽略 agent_filter。
    #[test]
    fn uninstall_no_setup_log_openclaw_no_fallback() {
        let dir = tempdir().unwrap();
        let setup_log = dir.path().join("setup.log"); // 不创建，文件不存在
        let backups_root = dir.path().join("backups");

        // 构造 backups/ 含 Claude 文件（如果 fallback 生效，就会被误用）
        fs::create_dir_all(backups_root.join("2026-04-27T00:00:00Z")).unwrap();
        fs::write(
            backups_root
                .join("2026-04-27T00:00:00Z")
                .join("settings.json"),
            r#"{"env":{}}"#,
        )
        .unwrap();

        let (backup, infos) = read_setup_log_for_test(&setup_log, &backups_root, Some("openclaw"));

        assert!(
            backup.is_none(),
            "setup.log 缺失 + --agent openclaw 不应 fallback 到全局备份，backup={backup:?}"
        );
        assert!(
            infos.is_empty(),
            "setup.log 缺失 + --agent openclaw 时 infos 应为空"
        );
    }

    /// R7-#4 场景 B：setup.log 不存在 + --agent claude → 仍允许 fallback（无回归）
    #[test]
    fn uninstall_no_setup_log_claude_still_fallbacks() {
        let dir = tempdir().unwrap();
        let setup_log = dir.path().join("setup.log"); // 不创建
        let backups_root = dir.path().join("backups");

        let backup_dir = backups_root.join("2026-04-27T00:00:00Z");
        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("settings.json"), r#"{"env":{}}"#).unwrap();

        let (backup, _infos) = read_setup_log_for_test(&setup_log, &backups_root, Some("claude"));

        assert!(
            backup.is_some(),
            "setup.log 缺失 + --agent claude 应允许 fallback 到全局备份（v1.4 老用户兼容），backup={backup:?}"
        );
    }

    /// R7-#4 场景 C：setup.log 不存在 + --all（filter=None）→ 仍允许 fallback（无回归）
    #[test]
    fn uninstall_no_setup_log_all_still_fallbacks() {
        let dir = tempdir().unwrap();
        let setup_log = dir.path().join("setup.log"); // 不创建
        let backups_root = dir.path().join("backups");

        let backup_dir = backups_root.join("2026-04-27T00:00:00Z");
        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("settings.json"), r#"{"env":{}}"#).unwrap();

        let (backup, _infos) = read_setup_log_for_test(&setup_log, &backups_root, None);

        assert!(
            backup.is_some(),
            "setup.log 缺失 + --all 应允许 fallback 到全局备份，backup={backup:?}"
        );
    }

    /// R6-#1 场景 B：旧格式 setup.log（无 agent 字段）+ --agent claude → 仍允许 fallback（无回归）
    ///
    /// v1.4 老用户只有 Claude，旧 setup.log 无 agent 字段，--agent claude 应能找到 backup。
    #[test]
    fn uninstall_claude_legacy_setup_log_fallback_works() {
        let dir = tempdir().unwrap();
        let setup_log = dir.path().join("setup.log");
        let backups_root = dir.path().join("backups");

        // 旧格式：无 agent 字段，且没有 setup_complete 中带 backup_dir
        // （最老的 setup.log 格式，只有 settings_updated 记录，没有 setup_complete）
        // → latest_backup = None，fallback 到 find_latest_backup_dir
        let log_entry = serde_json::json!({
            "action": "settings_updated",
            "path": "/tmp/home/.claude/settings.json",
            "created_new": true
            // 注意：无 agent 字段（旧格式）
        });
        fs::write(&setup_log, format!("{}\n", log_entry)).unwrap();

        // backups/ 有一个全局备份
        let backup_dir = backups_root.join("2026-04-27T00:00:00Z");
        fs::create_dir_all(&backup_dir).unwrap();
        fs::write(backup_dir.join("settings.json"), r#"{"env":{}}"#).unwrap();

        let (backup, _infos) = read_setup_log_for_test(&setup_log, &backups_root, Some("claude"));

        assert!(
            backup.is_some(),
            "--agent claude 配合旧格式 setup.log 应允许 fallback 到全局备份，backup={backup:?}"
        );
    }
}
