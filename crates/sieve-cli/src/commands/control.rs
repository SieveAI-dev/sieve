//! headless 控制面命令实现（`pause` / `resume` / `preset` / `graylist` / `reload`）。
//!
//! 均走既有 IPC 控制面方法（`sieve.set_paused` / `sieve.set_preset` / `sieve.health` /
//! `sieve.list_graylist` / `sieve.remove_graylist` / `sieve.reload_config`），CLI 与 GUI
//! 共用同一组方法，**不引入特权 endpoint**。IPC 封装见 [`crate::commands::ipc_client`]。

use anyhow::{Context, Result};

use crate::cli::{GraylistCommand, OutputFormat, PresetCommand, PresetMode};
use crate::commands::ipc_client::rpc_call_oneshot;

// ─────────────────────────── pause / resume ────────────────────────────────

/// `sieve pause --minutes N`：暂停非 Critical 弹窗 N 分钟。
pub async fn run_pause(minutes: u32) -> Result<()> {
    let result = rpc_call_oneshot(
        "sieve.set_paused",
        serde_json::json!({ "minutes": minutes }),
    )
    .await
    .context("调用 sieve.set_paused 失败")?;

    let paused = result
        .get("paused")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let until = result
        .get("paused_until")
        .and_then(|v| v.as_str())
        .unwrap_or("-");
    let applies_to = result
        .get("applies_to")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();

    if paused {
        println!("已暂停：paused_until={until} applies_to=[{applies_to}]");
    } else {
        println!("未进入暂停状态（paused=false）");
    }
    // 始终提示 Critical 不受影响（goal 硬要求）。
    println!("注意：Critical 锁规则不受暂停影响，签名/转账/敏感路径仍会强制确认。");
    Ok(())
}

/// `sieve resume`：清除暂停（set_paused minutes=0）。
pub async fn run_resume() -> Result<()> {
    let result = rpc_call_oneshot("sieve.set_paused", serde_json::json!({ "minutes": 0 }))
        .await
        .context("调用 sieve.set_paused 失败")?;
    let paused = result
        .get("paused")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if paused {
        println!("警告：daemon 仍报告 paused=true（应已清除）");
    } else {
        println!("已恢复：弹窗决策正常。");
    }
    Ok(())
}

// ─────────────────────────── preset ────────────────────────────────────────

/// `sieve preset get|set`。
pub async fn run_preset(cmd: PresetCommand) -> Result<()> {
    match cmd {
        PresetCommand::Get => run_preset_get().await,
        PresetCommand::Set { mode } => run_preset_set(mode).await,
    }
}

async fn run_preset_get() -> Result<()> {
    let result = rpc_call_oneshot("sieve.health", serde_json::json!({}))
        .await
        .context("查询 sieve.health 失败")?;
    let preset = result.get("preset");
    let mode = preset
        .and_then(|p| p.get("mode"))
        .and_then(|v| v.as_str())
        .unwrap_or("?");
    let overrides = preset
        .and_then(|p| p.get("overrides"))
        .and_then(|v| v.as_object())
        .map(|m| m.len())
        .unwrap_or(0);
    println!("preset mode={mode} custom_overrides={overrides}");
    Ok(())
}

async fn run_preset_set(mode: PresetMode) -> Result<()> {
    let result = rpc_call_oneshot(
        "sieve.set_preset",
        serde_json::json!({ "mode": mode.wire() }),
    )
    .await
    .context("调用 sieve.set_preset 失败")?;
    let applied_at = result
        .get("applied_at")
        .and_then(|v| v.as_str())
        .unwrap_or("-");
    println!("preset 已切换为 {} (applied_at={applied_at})", mode.wire());
    Ok(())
}

// ─────────────────────────── graylist ──────────────────────────────────────

/// `sieve graylist list|remove`。
pub async fn run_graylist(cmd: GraylistCommand) -> Result<()> {
    match cmd {
        GraylistCommand::List { format } => run_graylist_list(format).await,
        GraylistCommand::Remove { fingerprint } => run_graylist_remove(fingerprint).await,
    }
}

async fn run_graylist_list(format: Option<OutputFormat>) -> Result<()> {
    let result = rpc_call_oneshot("sieve.list_graylist", serde_json::json!({}))
        .await
        .context("查询 sieve.list_graylist 失败")?;
    let entries = result
        .get("entries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let pretty = matches!(format, Some(OutputFormat::Pretty));
    if pretty {
        if entries.is_empty() {
            println!("(灰名单为空)");
        } else {
            for e in &entries {
                let fp = e.get("fingerprint").and_then(|v| v.as_str()).unwrap_or("?");
                let rule = e.get("rule_id").and_then(|v| v.as_str()).unwrap_or("?");
                let kind = e.get("rule_kind").and_then(|v| v.as_str()).unwrap_or("?");
                let count = e
                    .get("match_count_since")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let hint = e.get("context_hint").and_then(|v| v.as_str()).unwrap_or("");
                println!("{fp} rule={rule} kind={kind} matches={count} hint={hint}");
            }
        }
    } else {
        // jsonl 默认（每行一个 entry，方便接 jq）。
        for e in &entries {
            println!("{}", serde_json::to_string(e)?);
        }
    }
    Ok(())
}

async fn run_graylist_remove(fingerprint: String) -> Result<()> {
    let result = rpc_call_oneshot(
        "sieve.remove_graylist",
        serde_json::json!({ "fingerprint": fingerprint }),
    )
    .await
    .context("调用 sieve.remove_graylist 失败")?;

    let removed = result
        .get("removed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if removed {
        println!("已移除灰名单条目：{fingerprint}");
        Ok(())
    } else {
        eprintln!("sieve graylist remove: fingerprint={fingerprint} 不存在或未移除");
        std::process::exit(1);
    }
}

// ─────────────────────────── reload ────────────────────────────────────────

/// `sieve reload`：重新加载用户规则与配置。
pub async fn run_reload() -> Result<()> {
    let result = rpc_call_oneshot("sieve.reload_config", serde_json::json!({}))
        .await
        .context("调用 sieve.reload_config 失败")?;
    let system = result
        .get("system_rules_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let user = result
        .get("user_rules_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let reloaded_at = result
        .get("reloaded_at")
        .and_then(|v| v.as_str())
        .unwrap_or("-");
    let errors = result
        .get("user_rules_errors")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    println!(
        "已 reload：system_rules={system} user_rules={user} lint_errors={errors} reloaded_at={reloaded_at}"
    );
    if errors > 0 {
        if let Some(errs) = result.get("user_rules_errors").and_then(|v| v.as_array()) {
            for e in errs {
                if let Some(s) = e.as_str() {
                    eprintln!("  lint: {s}");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_mode_wire_values() {
        assert_eq!(PresetMode::Strict.wire(), "strict");
        assert_eq!(PresetMode::Standard.wire(), "standard");
        assert_eq!(PresetMode::Relaxed.wire(), "relaxed");
        assert_eq!(PresetMode::Custom.wire(), "custom");
    }
}
