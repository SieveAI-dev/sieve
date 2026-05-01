//! `sieve rules` 用户规则管理子命令（PRD v2.0 §5.5.2 §5.5.5）。
//!
//! Phase A MVP 4 个子命令：
//! - `edit`：调用 `$EDITOR`（fallback vim/nano）+ lint pipeline + atomic backup/rename
//! - `list`：合并展示用户规则 + 系统规则数量
//! - `disable <id>` / `enable <id>`：切换 user.toml 中规则的 `enabled` 字段
//!
//! daemon hot-reload 推 Week 6（v2.0 Phase A 仅做文件级操作）。

use crate::cli::RulesCommand;
use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use sieve_policy::lint::lint;
use sieve_policy::loader::{load_user_rules, UserRulesFile};
use std::path::{Path, PathBuf};

const TEMPLATE_USER_TOML: &str = r#"# ~/.sieve/rules/user.toml
# Sieve 用户规则文件（PRD v2.0 §5.5）。
#
# 用户规则只能 ask / warn / mark / status_bar，不能 critical / block / hook_terminal。
# 系统规则的 Critical 永远先评估，用户规则不能 override 系统拦截。
#
# direction 字段（PRD v2.0 §5.5）：
#   "outbound" — 只扫出站请求（user prompt / system prompt）
#   "inbound"  — 只扫入站响应（assistant text / tool_use）
#   "both"     — 两侧都扫（默认值，旧规则文件缺省即 both）
#
# 详见 docs/guides/user-rules.md（待补）。

schema_version = 1
created_at = "__CREATED_AT__"
updated_at = "__UPDATED_AT__"

# 示例规则（首次编辑时删除并改成你自己的）：
#
# [[rules]]
# id = "MY-EXAMPLE-RULE"
# description = "示例：禁止常见钓鱼合约函数名"
# pattern = '''(?i)\b(claim|migrate|upgrade|verifyAndExecute)Wallet\b'''
# severity = "medium"
# action = "warn"
# keywords = ["claim", "migrate"]
# allowlist_stopwords = ["interface definition"]
# disposition = "status_bar"
# direction = "outbound"   # 只扫出站，不扫模型回复
# enabled = true
# added_at = "__CREATED_AT__"
# added_by = "manual"
"#;

/// 入口：根据子命令分发。
pub fn run(args: &crate::cli::RulesArgs) -> Result<()> {
    let path = user_rules_path()?;
    match &args.command {
        RulesCommand::Edit => run_edit_at(&path),
        RulesCommand::List => run_list_at(&path),
        RulesCommand::Disable { id } => run_toggle_at(&path, id, false),
        RulesCommand::Enable { id } => run_toggle_at(&path, id, true),
    }
}

// ─────────────────────────── 路径辅助 ──────────────────────────────

/// 返回 `~/.sieve/rules/user.toml` 路径。
fn user_rules_path() -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .ok_or_else(|| anyhow!("HOME 环境变量未设置，无法定位 ~/.sieve/rules/user.toml"))?;
    Ok(PathBuf::from(home)
        .join(".sieve")
        .join("rules")
        .join("user.toml"))
}

/// 确保 `~/.sieve/rules/` 目录存在并设为 0700。
fn ensure_rules_dir(path: &Path) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("rules 路径没有父目录: {}", path.display()))?;
    if !dir.exists() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("创建 rules 目录失败: {}", dir.display()))?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
            .with_context(|| format!("设目录 {} 权限 0700 失败", dir.display()))?;
    }
    Ok(())
}

/// 写入文件并设为 0600（Unix）。
fn write_user_toml(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content).with_context(|| format!("写入 {} 失败", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("设文件 {} 权限 0600 失败", path.display()))?;
    }
    Ok(())
}

/// 不存在则写入模板。返回是否新建。
fn ensure_user_toml(path: &Path) -> Result<bool> {
    ensure_rules_dir(path)?;
    if path.exists() {
        return Ok(false);
    }
    let now = Utc::now().to_rfc3339();
    let content = TEMPLATE_USER_TOML
        .replace("__CREATED_AT__", &now)
        .replace("__UPDATED_AT__", &now);
    write_user_toml(path, &content)?;
    Ok(true)
}

// ─────────────────────────── edit ──────────────────────────────

fn run_edit_at(path: &Path) -> Result<()> {
    let created = ensure_user_toml(path)?;
    if created {
        println!("已创建模板 user.toml：{}", path.display());
    }

    // 1. 唤起 $EDITOR
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // fallback 顺序：vim → nano
            for candidate in ["vim", "nano"] {
                if which(candidate).is_some() {
                    return candidate.to_string();
                }
            }
            "vi".to_string()
        });

    let status = std::process::Command::new(&editor)
        .arg(path)
        .status()
        .with_context(|| format!("无法启动编辑器 `{}`", editor))?;
    if !status.success() {
        bail!(
            "编辑器 `{}` 退出非零（{}），保留原 user.toml 不动",
            editor,
            status
        );
    }

    // 2. 编辑后 lint
    let new_meta = path
        .metadata()
        .with_context(|| format!("无法读取编辑后文件元信息: {}", path.display()))?;
    let new_size = new_meta.len();

    // 编辑器可能以非 0600 权限保存（如 vim 的 fileformat 行为）；先重置回 0600 让 loader 通过。
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("重置 {} 权限 0600 失败", path.display()))?;
    }

    let parsed = match load_user_rules(path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("lint 失败（解析阶段）：{}", e);
            eprintln!(
                "已保留你刚编辑的内容（未做备份），重新运行 `sieve rules edit` 修复后再保存。"
            );
            std::process::exit(1);
        }
    };

    let violations = lint(&parsed, new_size);
    if !violations.is_empty() {
        eprintln!("lint 失败（{} 条违规）：", violations.len());
        for v in &violations {
            eprintln!("  [{}] {:?}: {}", v.rule_id, v.kind, v.message);
        }
        eprintln!("已保留你刚编辑的内容（未做备份），修复违规后重新运行 `sieve rules edit`。");
        std::process::exit(1);
    }

    // 3. backup 旧版本（保留最近 10 份）
    if let Err(e) = backup_old_versions(path) {
        // backup 失败不致命，仅警告
        tracing::warn!(err = %e, "user.toml backup failed (non-fatal)");
        eprintln!("⚠ 备份失败（不影响保存）：{}", e);
    }

    // 4. 通知 daemon 重新加载用户规则（PRD §5.5.5 步骤 4）
    // run_edit_at 是同步函数；用 tokio::runtime::Runtime::new() 跑一次 async 调用。
    // socket 不存在（daemon 未运行）时静默跳过，不致命。
    let socket_path = sieve_ipc::paths::sieve_home()
        .ok()
        .map(|h| sieve_ipc::paths::ipc_socket_path(&h));
    if let Some(ref sp) = socket_path {
        if sp.exists() {
            let trigger_id = uuid::Uuid::now_v7();
            let sp_clone = sp.clone();
            match tokio::runtime::Runtime::new() {
                Ok(rt) => match rt.block_on(sieve_ipc::send_reload_user_rules_oneshot(
                    &sp_clone,
                    Some(trigger_id),
                )) {
                    Ok(()) => {
                        println!(
                            "✅ 已通知 daemon 重新加载用户规则（trigger_id = {}）",
                            trigger_id
                        );
                    }
                    Err(e) => {
                        eprintln!("⚠ 无法通知 daemon 重新加载（daemon 可能未运行）：{}", e);
                    }
                },
                Err(e) => {
                    eprintln!("⚠ 无法创建 tokio runtime 用于 reload 通知：{}", e);
                }
            }
        }
    }

    println!(
        "✅ user.toml 通过 lint，{} 条规则已就绪",
        parsed.rules.len()
    );
    Ok(())
}

/// 返回 PATH 上 `cmd` 的位置（仅用于 fallback 探测，不返完整路径）。
fn which(cmd: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(cmd);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// 把 `user.toml` 复制为 `user.toml.bak.YYYYMMDD-HHMMSS`，并淘汰超过 10 份的最旧备份。
///
/// 注意：edit 流程是「编辑器直接修改原文件」，本函数在 lint 通过后立即把当前内容
/// 复制成时间戳备份，留作回滚证据。下次编辑前的内容会成为新备份。
fn backup_old_versions(path: &Path) -> Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("无父目录: {}", path.display()))?;
    let stem = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("user.toml");
    let ts = Utc::now().format("%Y%m%d-%H%M%S");
    let bak = dir.join(format!("{stem}.bak.{ts}"));
    std::fs::copy(path, &bak)
        .with_context(|| format!("备份 {} → {} 失败", path.display(), bak.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&bak, std::fs::Permissions::from_mode(0o600));
    }
    prune_old_backups(dir, stem, 10)?;
    Ok(())
}

/// 保留最近 `keep` 份 backup，淘汰更旧的。
fn prune_old_backups(dir: &Path, stem: &str, keep: usize) -> Result<()> {
    let prefix = format!("{stem}.bak.");
    let mut backups: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.starts_with(&prefix))
                .unwrap_or(false)
        })
        .collect();
    backups.sort_by_key(|e| e.file_name());
    if backups.len() > keep {
        let drop_count = backups.len() - keep;
        for entry in backups.into_iter().take(drop_count) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    Ok(())
}

// ─────────────────────────── list ──────────────────────────────

fn run_list_at(path: &Path) -> Result<()> {
    let user_file = if path.exists() {
        load_user_rules(path).with_context(|| format!("加载 {} 失败", path.display()))?
    } else {
        UserRulesFile::default()
    };

    println!("# 用户规则（{}）", path.display());
    if user_file.rules.is_empty() {
        println!("  （空，运行 `sieve rules edit` 添加）");
    } else {
        for r in &user_file.rules {
            let state = if r.enabled { "enabled " } else { "disabled" };
            let dir = format!("{:?}", r.direction).to_lowercase();
            println!(
                "  [{}] dir={:<8} user:{:<40} severity={} action={} desc={}",
                state, dir, r.id, r.severity, r.action, r.description
            );
        }
        println!("  小计：{} 条用户规则", user_file.rules.len());
    }

    // 系统规则数量摘要（调 sieve-cli 内部 embedded_rules 加载入口）
    let inbound_count = crate::embedded_rules::INBOUND_RULES
        .matches("[[rules]]")
        .count();
    let outbound_count = crate::embedded_rules::OUTBOUND_RULES
        .matches("[[rules]]")
        .count();
    println!();
    println!("# 系统规则（编译进二进制，不可编辑）");
    println!("  入站：约 {} 条（IN-CR-* / IN-GEN-*）", inbound_count);
    println!("  出站：约 {} 条（OUT-*）", outbound_count);
    println!("  详见 docs/glossary.md 和 PRD §5");
    Ok(())
}

// ─────────────────────────── disable / enable ──────────────────────────────

fn run_toggle_at(path: &Path, id: &str, enable: bool) -> Result<()> {
    if !path.exists() {
        bail!(
            "user.toml 不存在（{}），先运行 `sieve rules edit` 创建并添加规则",
            path.display()
        );
    }

    let mut file =
        load_user_rules(path).with_context(|| format!("加载 {} 失败", path.display()))?;

    let mut found = false;
    for rule in &mut file.rules {
        if rule.id == id {
            if rule.enabled == enable {
                let state = if enable { "已启用" } else { "已禁用" };
                println!("规则 {} {state}，无需变更", id);
                return Ok(());
            }
            rule.enabled = enable;
            found = true;
            break;
        }
    }
    if !found {
        bail!(
            "未在 user.toml 找到规则 ID `{}`，运行 `sieve rules list` 查看可用 ID",
            id
        );
    }

    file.updated_at = Utc::now();

    // 重新序列化并 atomic 写入
    let new_content =
        toml::to_string_pretty(&file).with_context(|| "重新序列化 user.toml 失败".to_string())?;

    // 写 .tmp + rename 实现 atomic
    let tmp = path.with_extension("toml.tmp");
    write_user_toml(&tmp, &new_content)?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} → {} 失败", tmp.display(), path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("rename 后重置 {} 权限 0600 失败", path.display()))?;
    }

    let action = if enable { "启用" } else { "禁用" };
    println!("✅ 已{action}规则 {}", id);
    println!("⚠ daemon hot-reload 待 Week 6 落地；本次改动需重启 daemon 才生效");
    Ok(())
}

// ─────────────────────────── tests ──────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// 在 tmp 目录下分配一个 user.toml 路径（不依赖 HOME，避免并行测试竞态）。
    fn alloc_path(tmp: &TempDir) -> PathBuf {
        tmp.path().join(".sieve").join("rules").join("user.toml")
    }

    #[test]
    fn ensure_user_toml_creates_template_when_missing() {
        let tmp = TempDir::new().unwrap();
        let path = alloc_path(&tmp);
        let created = ensure_user_toml(&path).unwrap();
        assert!(created);
        assert!(path.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir_mode = std::fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777;
            let file_mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(dir_mode, 0o700);
            assert_eq!(file_mode, 0o600);
        }
        let parsed = load_user_rules(&path).unwrap();
        assert_eq!(parsed.schema_version, 1);
        assert!(parsed.rules.is_empty());
    }

    #[test]
    fn ensure_user_toml_idempotent_when_exists() {
        let tmp = TempDir::new().unwrap();
        let path = alloc_path(&tmp);
        let _ = ensure_user_toml(&path).unwrap();
        let created2 = ensure_user_toml(&path).unwrap();
        assert!(!created2, "second call must not recreate");
    }

    #[test]
    fn toggle_on_missing_file_errors() {
        let tmp = TempDir::new().unwrap();
        let path = alloc_path(&tmp);
        let err = run_toggle_at(&path, "FOO", false).unwrap_err();
        assert!(err.to_string().contains("user.toml 不存在"));
    }

    #[test]
    fn toggle_disables_then_enables_round_trip() {
        let tmp = TempDir::new().unwrap();
        let path = alloc_path(&tmp);
        ensure_rules_dir(&path).unwrap();

        let toml = r#"
schema_version = 1
created_at = "2026-05-01T00:00:00Z"
updated_at = "2026-05-01T00:00:00Z"

[[rules]]
id = "MY-RULE"
description = "test"
pattern = "secret"
severity = "high"
action = "warn"
keywords = ["secret"]
enabled = true
added_at = "2026-05-01T00:00:00Z"
added_by = "manual"
"#;
        write_user_toml(&path, toml).unwrap();

        run_toggle_at(&path, "MY-RULE", false).unwrap();
        let after_disable = load_user_rules(&path).unwrap();
        assert!(!after_disable.rules[0].enabled);

        run_toggle_at(&path, "MY-RULE", true).unwrap();
        let after_enable = load_user_rules(&path).unwrap();
        assert!(after_enable.rules[0].enabled);
    }

    #[test]
    fn toggle_unknown_id_errors() {
        let tmp = TempDir::new().unwrap();
        let path = alloc_path(&tmp);
        ensure_rules_dir(&path).unwrap();
        write_user_toml(
            &path,
            "schema_version = 1\ncreated_at = \"2026-05-01T00:00:00Z\"\nupdated_at = \"2026-05-01T00:00:00Z\"\n",
        )
        .unwrap();
        let err = run_toggle_at(&path, "NOPE", true).unwrap_err();
        assert!(err.to_string().contains("未在 user.toml 找到规则 ID"));
    }

    #[test]
    fn backup_keeps_only_last_10() {
        let tmp = TempDir::new().unwrap();
        let path = alloc_path(&tmp);
        ensure_rules_dir(&path).unwrap();
        write_user_toml(
            &path,
            "schema_version = 1\ncreated_at = \"2026-05-01T00:00:00Z\"\nupdated_at = \"2026-05-01T00:00:00Z\"\n",
        )
        .unwrap();

        let dir = path.parent().unwrap();
        for i in 0..12 {
            std::fs::write(
                dir.join(format!("user.toml.bak.2026010{:01}-000000", i)),
                "old",
            )
            .unwrap();
        }
        prune_old_backups(dir, "user.toml", 10).unwrap();

        let count = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.starts_with("user.toml.bak."))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(count, 10, "应保留 10 份");
    }

    #[test]
    fn list_at_missing_file_prints_empty_and_succeeds() {
        let tmp = TempDir::new().unwrap();
        let path = alloc_path(&tmp);
        // path 不存在也不报错
        run_list_at(&path).unwrap();
    }

    #[test]
    fn list_at_existing_file_succeeds() {
        let tmp = TempDir::new().unwrap();
        let path = alloc_path(&tmp);
        ensure_rules_dir(&path).unwrap();
        write_user_toml(
            &path,
            "schema_version = 1\ncreated_at = \"2026-05-01T00:00:00Z\"\nupdated_at = \"2026-05-01T00:00:00Z\"\n",
        )
        .unwrap();
        run_list_at(&path).unwrap();
    }
}
