//! 规则加载、校验和热替换。
use super::*;

/// 重新读取、lint 并编译用户规则，返回可立即 swap 的两个方向引擎（v2.1）。
///
/// 返回 `(outbound_engine, inbound_engine, rule_count)`，方向引擎均为 `Option<UserEngine>`：
/// - `None` 表示该方向无规则（文件不存在、或该方向 0 条），LayeredEngine 退化为纯系统引擎
/// - `Some(engine)` 即编译通过的用户引擎，调用方直接调用 `swap_user` 生效
///
/// 任何错误（lint 违规 / SIEVE_HOME 未设置）返回 `Err`（fail-safe：daemon 保留旧引擎）。
/// 用户规则 reload 一次的结果（reload_config）。
#[derive(Debug, Clone)]
pub(crate) struct ReloadOutcome {
    /// 当前调用方未消费 `success` 字段（成功 / 失败可由 `user_rules_errors.is_empty()` 间接判定），
    /// 但保留供未来扩展（如分级 audit）使用。
    #[allow(dead_code)]
    pub success: bool,
    pub rule_count: usize,
    pub user_rules_errors: Vec<String>,
}

/// 执行一次用户规则 reload 完整流程（lint + 编译 + hot swap + 广播 + audit）。
///
/// 既被 IPC `sieve.reload_user_rules` notification listener 调用（向后兼容），
/// 也被 control plane `sieve.reload_config` 直接同步调用（拿 errors 同步返回）。
///
/// 行为与原内联闭包等价：
/// - 成功 → swap_user + 推 `NotifyKind::UserRulesReloaded` + 写 audit success
/// - 失败 → 不动当前引擎 + 推 `NotifyKind::UserRulesLoadFailed` + 写 audit failure
///
/// 关联：用户规则 reload / fail-safe。
pub(crate) fn perform_user_rules_reload(
    user_rules_path: Option<&std::path::Path>,
    outbound_layered: &Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    inbound_layered: &Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    ipc: &Arc<sieve_ipc::IpcServer>,
    audit: &Arc<crate::audit::AuditStore>,
    trigger_id: Option<uuid::Uuid>,
) -> ReloadOutcome {
    let trigger_id_str = trigger_id.map(|id| id.to_string());
    tracing::info!(
        trigger_id = ?trigger_id_str,
        "执行用户规则 reload"
    );

    let reload_result = reload_user_engines(user_rules_path);
    let (notify_kind, notify_title, notify_detail, success, rule_count, err_msg) =
        match reload_result {
            Ok((outbound_eng, inbound_eng, count)) => {
                outbound_layered.swap_user(outbound_eng);
                inbound_layered.swap_user(inbound_eng);
                tracing::info!(rule_count = count, "用户规则 hot swap 完成");
                (
                    sieve_ipc::protocol::NotifyKind::UserRulesReloaded,
                    format!("用户规则已 hot reload（{count} 条）"),
                    Some("已立即生效，无需重启 daemon".to_owned()),
                    true,
                    count,
                    None,
                )
            }
            Err(e) => {
                tracing::warn!(error = %e, "用户规则重新加载失败（保留旧引擎）");
                (
                    sieve_ipc::protocol::NotifyKind::UserRulesLoadFailed,
                    "用户规则加载失败".to_owned(),
                    Some(e.to_string()),
                    false,
                    0,
                    Some(e.to_string()),
                )
            }
        };

    let notify = sieve_ipc::protocol::StatusBarNotify {
        notify_id: uuid::Uuid::now_v7(),
        created_at: chrono::Utc::now(),
        kind: notify_kind,
        title: notify_title,
        detail: notify_detail,
        rule_id: None,
        auto_dismiss_seconds: 5,
    };
    ipc.broadcast_status_bar(notify);

    // audit 写入（fail-soft）
    let event = crate::audit::AuditEvent::UserRulesReloaded {
        success,
        rule_count: if success { Some(rule_count) } else { None },
        error: err_msg.clone(),
        trigger_id: trigger_id_str,
    };
    let audit_clone = Arc::clone(audit);
    tokio::spawn(async move {
        // UserRulesReloaded 是 daemon 系统级事件，无 listener 上下文
        if let Err(e) = audit_clone
            .append(event, crate::audit::SYSTEM_PROVIDER_ID)
            .await
        {
            tracing::warn!(error = %e, "audit append UserRulesReloaded failed");
        }
    });

    ReloadOutcome {
        success,
        rule_count,
        user_rules_errors: err_msg.into_iter().collect(),
    }
}

/// 系统规则热重载（updater 装入新签名包后调用，无需重启）。
///
/// 从签名包（`current.json`）重新加载出站/入站系统规则，编译后原子 `swap_system` 到
/// live 引擎；任一方向为空或编译失败时，保留两个方向的上一版引擎。编译时
/// [`VectorscanEngine::compile`] 同步刷新 fail-closed 运行时注册表（accumulate）。
/// 与 [`perform_user_rules_reload`] 对称——系统层 `ArcSwap` zero-downtime 热替换。
pub(crate) fn perform_rules_reload(
    pack_path: Option<&std::path::Path>,
    dev_outbound_path: &std::path::Path,
    dev_inbound_path: &std::path::Path,
    outbound_layered: &Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    inbound_layered: &Arc<
        sieve_rules::engine::LayeredEngine<
            sieve_rules::engine::SystemEngine,
            sieve_policy::engine::UserEngine,
        >,
    >,
    ipc: Option<&Arc<sieve_ipc::IpcServer>>,
) {
    let out = crate::reload_system_vectorscan(pack_path, dev_outbound_path, true);
    let inb = crate::reload_system_vectorscan(pack_path, dev_inbound_path, false);
    let out_count = out
        .as_ref()
        .map(sieve_rules::engine::MatchEngine::rule_count)
        .unwrap_or(0);
    let in_count = inb
        .as_ref()
        .map(sieve_rules::engine::MatchEngine::rule_count)
        .unwrap_or(0);
    let total = out_count + in_count;

    if out.is_none() || inb.is_none() {
        tracing::error!("系统规则更新失败或为空，保留上一版规则");
        if let Some(ipc) = ipc {
            ipc.broadcast_status_bar(sieve_ipc::protocol::StatusBarNotify {
                notify_id: uuid::Uuid::now_v7(),
                created_at: chrono::Utc::now(),
                kind: sieve_ipc::protocol::NotifyKind::Generic,
                title: "规则更新失败，继续使用上一版保护规则".into(),
                detail: None,
                rule_id: None,
                auto_dismiss_seconds: 0,
            });
        }
        return;
    }

    // 原子热替换系统层（已在进行中的 scan 持旧快照，结束后释放）。
    outbound_layered.swap_system(out);
    inbound_layered.swap_system(inb);

    tracing::info!(
        out_count,
        in_count,
        "系统规则热重载完成（swap_system，zero-downtime）"
    );

    if let Some(ipc) = ipc {
        let notify = sieve_ipc::protocol::StatusBarNotify {
            notify_id: uuid::Uuid::now_v7(),
            created_at: chrono::Utc::now(),
            kind: sieve_ipc::protocol::NotifyKind::Generic,
            title: format!("规则包已热加载（{total} 条）"),
            detail: Some("已立即生效，无需重启 daemon".to_owned()),
            rule_id: None,
            auto_dismiss_seconds: 5,
        };
        ipc.broadcast_status_bar(notify);
    }
}

fn reload_user_engines(
    user_rules_path: Option<&std::path::Path>,
) -> anyhow::Result<(
    Option<sieve_policy::engine::UserEngine>,
    Option<sieve_policy::engine::UserEngine>,
    usize,
)> {
    use sieve_policy::lint::lint;
    use sieve_policy::loader::load_user_rules;

    let path = user_rules_path
        .ok_or_else(|| anyhow::anyhow!("user rules path 未知（SIEVE_HOME 未设置）"))?;

    if !path.exists() {
        // 文件不存在：两个方向均 None（退化为纯系统规则），视为成功（0 条规则）
        return Ok((None, None, 0));
    }

    let file = load_user_rules(path).map_err(|e| anyhow::anyhow!("user.toml 解析失败: {e}"))?;

    let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
    let violations = lint(&file, file_size);
    if !violations.is_empty() {
        return Err(anyhow::anyhow!(
            "user.toml lint 失败（{} 条违规）：{}",
            violations.len(),
            violations[0].message
        ));
    }

    let total = file.rules.len();

    // 出站引擎（编译 direction=outbound/both 的规则）；该方向无规则时返回 None（fail-safe）
    let outbound_eng = sieve_policy::engine::UserEngine::compile_for_direction(
        file.rules.clone(),
        sieve_policy::loader::RuleDirection::Outbound,
    )
    .ok();

    // 入站引擎（编译 direction=inbound/both 的规则）；该方向无规则时返回 None（fail-safe）
    let inbound_eng = sieve_policy::engine::UserEngine::compile_for_direction(
        file.rules,
        sieve_policy::loader::RuleDirection::Inbound,
    )
    .ok();

    Ok((outbound_eng, inbound_eng, total))
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use sieve_rules::engine::{LayeredEngine, MatchEngine, SystemEngine};
    #[test]
    fn failed_update_preserves_both_previous_engines() {
        let directory = tempfile::tempdir().unwrap();
        let outbound = directory.path().join("out.toml");
        let inbound = directory.path().join("in.toml");
        let rule = |id: &str, pattern: &str| {
            format!("[[rules]]\nid = \"{id}\"\ndescription = \"test\"\npattern = '{pattern}'\nseverity = \"high\"\naction = \"warn\"\ndisposition = \"status_bar\"\n")
        };
        std::fs::write(&outbound, rule("OUT-OLD", "old_out")).unwrap();
        std::fs::write(&inbound, rule("IN-OLD", "old_in")).unwrap();
        let out = crate::reload_system_vectorscan(None, &outbound, true).unwrap();
        let ins = crate::reload_system_vectorscan(None, &inbound, false).unwrap();
        let out: Arc<LayeredEngine<SystemEngine, sieve_policy::engine::UserEngine>> =
            Arc::new(LayeredEngine::new(SystemEngine::new(Some(out)), None));
        let ins = Arc::new(LayeredEngine::new(SystemEngine::new(Some(ins)), None));
        std::fs::write(&outbound, rule("OUT-NEW", "new_out")).unwrap();
        std::fs::write(&inbound, "not valid toml").unwrap();
        perform_rules_reload(None, &outbound, &inbound, &out, &ins, None);
        assert_eq!(out.scan(b"old_out").unwrap()[0].rule_id, "OUT-OLD");
        assert_eq!(ins.scan(b"old_in").unwrap()[0].rule_id, "IN-OLD");
        assert!(out.scan(b"new_out").unwrap().is_empty());
    }
}
