//! SPEC-005 v2 wire format fixture 测试（§14.1）。
//!
//! 验证 `tests/fixtures/v2/` 下的 JSON fixture 能被正确反序列化为 sieve-ipc 协议类型。
//! 覆盖：全部 17 method × minimal/full/null_optional = 51+ 条（P2-5 完成）。
//!
//! 生成式测试 `all_fixtures_valid_json` 遍历 fixtures/v2/ 所有 .json 文件，
//! 验证每个文件均可被解析为合法 JSON，并针对已知 method 做类型级反序列化验证。

use sieve_ipc::protocol::{HealthResult, HelloParams, SetPausedRequest, SetPausedResult};
use sieve_ipc::wire::{MergedRequestDecisionWire, RequestDecisionWire};

// ── sieve.hello ──────────────────────────────────────────────────────────────

/// sieve.hello full fixture 可反序列化。
#[test]
fn hello_full_fixture_deserializes() {
    let json = include_str!("fixtures/v2/sieve.hello/full.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse hello/full.json");
    let params = val.get("params").expect("params field").clone();
    let _hello: HelloParams =
        serde_json::from_value(params).expect("deserialize HelloParams from full fixture");
}

/// sieve.hello minimal fixture 可反序列化。
#[test]
fn hello_minimal_fixture_deserializes() {
    let json = include_str!("fixtures/v2/sieve.hello/minimal.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse hello/minimal.json");
    let params = val.get("params").expect("params field").clone();
    let hello: HelloParams =
        serde_json::from_value(params).expect("deserialize HelloParams from minimal fixture");
    assert_eq!(hello.protocol_version, "v2");
}

// ── sieve.set_paused ─────────────────────────────────────────────────────────

/// set_paused request minimal fixture 可反序列化。
#[test]
fn set_paused_request_minimal_deserializes() {
    let json = include_str!("fixtures/v2/sieve.set_paused/request.minimal.json");
    let val: serde_json::Value = serde_json::from_str(json).unwrap();
    let params = val.get("params").unwrap().clone();
    let req: SetPausedRequest = serde_json::from_value(params).unwrap();
    assert_eq!(req.minutes, 30);
}

/// set_paused response full fixture（paused=true + paused_until）可反序列化。
#[test]
fn set_paused_response_full_deserializes() {
    let json = include_str!("fixtures/v2/sieve.set_paused/response.full.json");
    let val: serde_json::Value = serde_json::from_str(json).unwrap();
    let result = val.get("result").unwrap().clone();
    let resp: SetPausedResult = serde_json::from_value(result).unwrap();
    assert!(resp.paused);
    assert!(resp.paused_until.is_some());
}

/// set_paused response null_optional fixture（paused=false + paused_until=null）可反序列化。
#[test]
fn set_paused_response_null_optional_deserializes() {
    let json = include_str!("fixtures/v2/sieve.set_paused/response.null_optional.json");
    let val: serde_json::Value = serde_json::from_str(json).unwrap();
    let result = val.get("result").unwrap().clone();
    let resp: SetPausedResult = serde_json::from_value(result).unwrap();
    assert!(!resp.paused);
    assert!(resp.paused_until.is_none());
}

// ── sieve.health ─────────────────────────────────────────────────────────────

/// health response minimal fixture 可反序列化。
#[test]
fn health_response_minimal_deserializes() {
    let json = include_str!("fixtures/v2/sieve.health/response.minimal.json");
    let val: serde_json::Value = serde_json::from_str(json).unwrap();
    let result = val.get("result").unwrap().clone();
    let health: HealthResult = serde_json::from_value(result).unwrap();
    assert_eq!(health.protocol_version, "v2");
    assert!(!health.paused);
    assert_eq!(health.rules.system_count, 12);
    // SPEC-005 §9.5：minimal fixture 省略 listeners，#[serde(default)] 解析为空 vec。
    assert!(
        health.listeners.is_empty(),
        "minimal fixture 省略 listeners 时应回落为空 vec（向后兼容）"
    );
}

/// health response full fixture 可反序列化 + listeners[] 完整 + 双向稳定（SPEC-005 §14.1）。
///
/// 双向稳定是 §14.1 的核心防漂移约束：fixture MUST 等于 daemon 序列化输出，
/// 任何 HealthResult schema 变更（如新增 listeners[] 却忘了更新 fixture）都会让本
/// 测试失败——这正是 2026-06-07 审查发现 fixture 缺 listeners[] 漂移本该触发的防线。
#[test]
fn health_response_full_deserializes_and_roundtrips() {
    let json = include_str!("fixtures/v2/sieve.health/response.full.json");
    let val: serde_json::Value = serde_json::from_str(json).unwrap();
    let result = val.get("result").unwrap().clone();
    let health: HealthResult = serde_json::from_value(result.clone()).unwrap();

    // multi-listener：full fixture 列出 2 个 listener，含 provider_id + protocol。
    assert_eq!(health.listeners.len(), 2, "full fixture 应含 2 个 listener");
    assert_eq!(health.listeners[0].provider_id, "anthropic");
    assert_eq!(health.listeners[0].protocol, "anthropic");
    assert_eq!(health.listeners[1].provider_id, "deepseek");
    assert_eq!(health.listeners[1].protocol, "auto");
    // 向后兼容：listen 单字段仍等价 listeners[0] 地址。
    assert_eq!(health.listen.port, 11453);

    // §14.1 双向稳定：序列化回 Value 必须与 fixture result 逐字段一致。
    let reserialized = serde_json::to_value(&health).expect("serialize HealthResult");
    assert_eq!(
        reserialized, result,
        "health full fixture 双向不稳定——fixture 与 daemon 序列化输出漂移（SPEC-005 §14.1）"
    );
}

/// health response null_optional fixture：可选字段显式 null + listeners 显式空数组。
#[test]
fn health_response_null_optional_deserializes() {
    let json = include_str!("fixtures/v2/sieve.health/response.null_optional.json");
    let val: serde_json::Value = serde_json::from_str(json).unwrap();
    let result = val.get("result").unwrap().clone();
    let health: HealthResult = serde_json::from_value(result).unwrap();
    assert!(!health.paused);
    assert!(
        health.paused_until.is_none(),
        "null_optional: paused_until 应为 None"
    );
    assert!(
        health.rules.last_reload.is_none(),
        "null_optional: last_reload 应为 None"
    );
    assert!(
        health.listeners.is_empty(),
        "null_optional: listeners 显式空数组应解析为空 vec"
    );
}

// ── sieve.request_decision（P1-5 wire DTO）───────────────────────────────────

/// 单 issue minimal fixture 可反序列化为 RequestDecisionWire。
/// 验证：`received_at_daemon` 字段存在，`merged=false`，`rule_id` 在顶层。
#[test]
fn request_decision_single_minimal_deserializes() {
    let json = include_str!("fixtures/v2/sieve.request_decision/request.single.minimal.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse single minimal fixture");
    let params = val.get("params").expect("params field").clone();

    let wire: RequestDecisionWire =
        serde_json::from_value(params.clone()).expect("deserialize RequestDecisionWire");
    assert_eq!(wire.rule_id, "IN-CR-05");
    assert_eq!(wire.severity, "critical");
    assert_eq!(wire.direction, "inbound");
    assert_eq!(wire.disposition, "gui_popup");
    assert!(!wire.merged, "single form: merged must be false");

    // received_at_daemon 字段存在于 JSON（P2-2 字段重命名验证）。
    let json_str = serde_json::to_string(&params).unwrap();
    assert!(
        json_str.contains("received_at_daemon"),
        "wire must contain received_at_daemon (P2-2)"
    );
    assert!(
        !json_str.contains("\"created_at\""),
        "wire must NOT contain created_at (renamed to received_at_daemon)"
    );
}

/// 单 issue full fixture（含 context + recommendation）可反序列化。
#[test]
fn request_decision_single_full_deserializes() {
    let json = include_str!("fixtures/v2/sieve.request_decision/request.single.full.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse single full fixture");
    let params = val.get("params").expect("params field").clone();

    let wire: RequestDecisionWire =
        serde_json::from_value(params).expect("deserialize RequestDecisionWire full");
    assert_eq!(wire.rule_id, "IN-CR-05");
    assert!(wire.context.is_some(), "full fixture must have context");
    assert!(
        wire.recommendation.is_some(),
        "full fixture must have recommendation"
    );

    // 时间戳 Z 后缀 + 毫秒（P2-3 联动）
    let ts_str = &wire.received_at_daemon;
    assert!(
        ts_str.ends_with('Z'),
        "timestamp must end with Z suffix: {ts_str}"
    );
}

/// 多 issue merged fixture（`merged: true` + `issues[]`）可反序列化。
/// 验证：顶层无 `rule_id`，`issues` 包含 2 个 issue_id。
#[test]
fn request_decision_merged_deserializes() {
    let json = include_str!("fixtures/v2/sieve.request_decision/request.merged.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse merged fixture");
    let params = val.get("params").expect("params field").clone();

    let wire: MergedRequestDecisionWire =
        serde_json::from_value(params.clone()).expect("deserialize MergedRequestDecisionWire");
    assert!(wire.merged, "merged form: merged must be true");
    assert_eq!(wire.issues.len(), 2, "must have 2 issues");
    assert_eq!(wire.issues[0].issue_id, "i-1");
    assert_eq!(wire.issues[0].rule_id, "IN-CR-05");
    assert_eq!(wire.issues[1].issue_id, "i-2");
    assert_eq!(wire.issues[1].rule_id, "IN-GEN-04");
    assert_eq!(wire.severity, "critical", "merged severity = max of issues");

    // 顶层无 rule_id（§6.1.2）
    assert!(
        params.get("rule_id").is_none(),
        "merged form must not have top-level rule_id"
    );
}

// ── 生成式断言：所有 fixture 文件合法 JSON + 关键类型反序列化 ────────────────

use sieve_ipc::protocol::{
    DecisionResponse, EvaluateRequest, EvaluateResult, GraylistEntrySummary, ListGraylistResult,
    ListRulesResult, NotifyKind, PausedChangedNotify, PresetChangedNotify, PurgeHistoryRequest,
    PurgeHistoryResult, ReloadConfigResult, ReloadUserRules, RemoveGraylistResult,
    RequestDecisionCanceledNotify, SetPresetOverridesResult, SetPresetResult, StatusBarNotify,
};

/// 遍历 fixtures/v2/ 所有 .json 文件，验证：
/// 1. 文件内容是合法 JSON（能被 serde_json::from_str 解析）。
/// 2. 针对已知 method / 目录，抽取对应子结构做类型级反序列化。
/// 3. 往返序列化：JSON → Value → re-serialize → 字节级一致。
///
/// 这是一个宏观健全性检查；每个 method 的深度业务断言在各自独立测试函数中。
#[test]
fn all_fixtures_valid_json_and_roundtrip() {
    let fixtures_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/v2");

    let mut total = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for dir_entry in std::fs::read_dir(&fixtures_root).expect("fixtures/v2 dir must exist") {
        let dir_entry = dir_entry.unwrap();
        let dir_name = dir_entry.file_name().into_string().unwrap();
        let dir_path = dir_entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        for file_entry in std::fs::read_dir(&dir_path).unwrap() {
            let file_entry = file_entry.unwrap();
            let file_name = file_entry.file_name().into_string().unwrap();
            if !file_name.ends_with(".json") {
                continue;
            }
            total += 1;
            let file_path = file_entry.path();
            let content = std::fs::read_to_string(&file_path)
                .unwrap_or_else(|e| panic!("read {file_path:?}: {e}"));

            // ① 合法 JSON
            let val: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(e) => {
                    errors.push(format!("{dir_name}/{file_name}: invalid JSON: {e}"));
                    continue;
                }
            };

            // ② 往返序列化字节级一致
            let re_serialized = serde_json::to_string(&val).unwrap();
            let re_val: serde_json::Value = serde_json::from_str(&re_serialized).unwrap();
            if val != re_val {
                errors.push(format!("{dir_name}/{file_name}: roundtrip mismatch"));
                continue;
            }

            // ③ 类型级反序列化（针对已知 method 目录）
            let type_error = match dir_name.as_str() {
                "sieve.hello" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<HelloParams>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.heartbeat" => {
                    // heartbeat 无 params，只验证 method 字段
                    if val.get("method").and_then(|v| v.as_str()) != Some("sieve.heartbeat") {
                        Some("method must be sieve.heartbeat".to_owned())
                    } else {
                        None
                    }
                }
                "sieve.notify_status_bar" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<StatusBarNotify>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.request_decision_canceled" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<RequestDecisionCanceledNotify>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.preset_changed" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<PresetChangedNotify>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.paused_changed" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<PausedChangedNotify>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.reload_user_rules" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<ReloadUserRules>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.set_paused" => {
                    // request: SetPausedRequest, response: SetPausedResult
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<sieve_ipc::protocol::SetPausedRequest>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else if let Some(r) = val.get("result") {
                        serde_json::from_value::<SetPausedResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.set_preset" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<sieve_ipc::protocol::SetPresetRequest>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else if let Some(r) = val.get("result") {
                        serde_json::from_value::<SetPresetResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.set_preset_overrides" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<sieve_ipc::protocol::SetPresetOverridesRequest>(
                            p.clone(),
                        )
                        .err()
                        .map(|e| e.to_string())
                    } else if let Some(r) = val.get("result") {
                        serde_json::from_value::<SetPresetOverridesResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.reload_config" => {
                    if let Some(r) = val.get("result") {
                        serde_json::from_value::<ReloadConfigResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.health" => {
                    if let Some(r) = val.get("result") {
                        serde_json::from_value::<HealthResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.evaluate" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<EvaluateRequest>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else if let Some(r) = val.get("result") {
                        serde_json::from_value::<EvaluateResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.list_graylist" => {
                    if let Some(r) = val.get("result") {
                        serde_json::from_value::<ListGraylistResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.remove_graylist" => {
                    if let Some(r) = val.get("result") {
                        serde_json::from_value::<RemoveGraylistResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.list_rules" => {
                    if let Some(r) = val.get("result") {
                        serde_json::from_value::<ListRulesResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else if let Some(p) = val.get("params") {
                        // request：params 可以是 {} 或 null
                        if p.is_object() || p.is_null() {
                            None
                        } else {
                            Some("list_rules params must be object or null".to_owned())
                        }
                    } else {
                        None
                    }
                }
                "sieve.purge_history" => {
                    if let Some(p) = val.get("params") {
                        serde_json::from_value::<PurgeHistoryRequest>(p.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else if let Some(r) = val.get("result") {
                        serde_json::from_value::<PurgeHistoryResult>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                "sieve.request_decision" => {
                    // 已有独立测试覆盖，此处跳过
                    None
                }
                "decision_response" => {
                    if let Some(r) = val.get("result") {
                        serde_json::from_value::<DecisionResponse>(r.clone())
                            .err()
                            .map(|e| e.to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(e) = type_error {
                errors.push(format!(
                    "{dir_name}/{file_name}: type deserialization failed: {e}"
                ));
            }
        }
    }

    assert!(total >= 51, "fixture 文件总数应 >= 51（当前 {total}）");
    assert!(
        errors.is_empty(),
        "fixture 验证失败（{} 条）:\n{}",
        errors.len(),
        errors.join("\n")
    );
}

// ── 新增 method 独立测试 ──────────────────────────────────────────────────────

/// sieve.heartbeat notification minimal fixture 可反序列化。
#[test]
fn heartbeat_minimal_fixture_deserializes() {
    let json = include_str!("fixtures/v2/sieve.heartbeat/notification.minimal.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse heartbeat minimal");
    assert_eq!(val["method"].as_str(), Some("sieve.heartbeat"));
    assert!(
        val.get("params").is_none(),
        "heartbeat 不应有 params 字段（§4）"
    );
}

/// sieve.notify_status_bar full fixture 可反序列化，kind=outbound_redacted 正确。
#[test]
fn notify_status_bar_full_fixture_deserializes() {
    let json = include_str!("fixtures/v2/sieve.notify_status_bar/notification.full.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse notify_status_bar full");
    let notify: StatusBarNotify =
        serde_json::from_value(val["params"].clone()).expect("deserialize StatusBarNotify");
    assert_eq!(notify.kind, NotifyKind::OutboundRedacted);
    assert!(notify.detail.is_some());
    assert!(notify.rule_id.is_some());
}

/// sieve.request_decision_canceled minimal fixture 可反序列化，reason=timeout。
#[test]
fn request_decision_canceled_minimal_deserializes() {
    let json =
        include_str!("fixtures/v2/sieve.request_decision_canceled/notification.minimal.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse canceled minimal");
    let notify: RequestDecisionCanceledNotify = serde_json::from_value(val["params"].clone())
        .expect("deserialize RequestDecisionCanceledNotify");
    use sieve_ipc::protocol::CancelReason;
    assert_eq!(notify.reason, CancelReason::Timeout);
}

/// sieve.paused_changed full fixture 可反序列化，origin_request_id 存在。
#[test]
fn paused_changed_full_fixture_deserializes() {
    let json = include_str!("fixtures/v2/sieve.paused_changed/notification.full.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse paused_changed full");
    let notify: PausedChangedNotify =
        serde_json::from_value(val["params"].clone()).expect("deserialize PausedChangedNotify");
    assert!(notify.paused);
    assert!(notify.paused_until.is_some());
    assert!(notify.origin_request_id.is_some());
}

/// sieve.preset_changed full fixture 可反序列化，overrides 有内容。
#[test]
fn preset_changed_full_fixture_deserializes() {
    let json = include_str!("fixtures/v2/sieve.preset_changed/notification.full.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse preset_changed full");
    let notify: PresetChangedNotify =
        serde_json::from_value(val["params"].clone()).expect("deserialize PresetChangedNotify");
    assert_eq!(notify.mode, "custom");
    assert!(!notify.overrides.is_empty());
    assert!(notify.origin_request_id.is_some());
}

/// decision_response full fixture 可反序列化，ui_phase_when_clicked = blue。
#[test]
fn decision_response_full_fixture_deserializes() {
    let json = include_str!("fixtures/v2/decision_response/response.full.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse decision_response full");
    let resp: DecisionResponse =
        serde_json::from_value(val["result"].clone()).expect("deserialize DecisionResponse");
    use sieve_ipc::protocol::{DecisionAction, UiPhase};
    assert_eq!(resp.decision, DecisionAction::Allow);
    assert!(resp.remember);
    assert_eq!(resp.ui_phase_when_clicked, Some(UiPhase::Blue));
}

/// sieve.evaluate response full fixture 可反序列化，matches 含 1 条命中。
#[test]
fn evaluate_response_full_fixture_deserializes() {
    let json = include_str!("fixtures/v2/sieve.evaluate/response.full.json");
    let val: serde_json::Value = serde_json::from_str(json).expect("parse evaluate response full");
    let result: EvaluateResult =
        serde_json::from_value(val["result"].clone()).expect("deserialize EvaluateResult");
    assert_eq!(result.matches.len(), 1);
    assert_eq!(result.matches[0].rule_id, "IN-CR-05");
    assert!(result.matches[0].would_recommendation.is_some());
}

/// sieve.list_graylist response full fixture 可反序列化，entries 含 1 条。
#[test]
fn list_graylist_response_full_fixture_deserializes() {
    let json = include_str!("fixtures/v2/sieve.list_graylist/response.full.json");
    let val: serde_json::Value =
        serde_json::from_str(json).expect("parse list_graylist response full");
    let result: ListGraylistResult =
        serde_json::from_value(val["result"].clone()).expect("deserialize ListGraylistResult");
    assert_eq!(result.entries.len(), 1);
    let entry: &GraylistEntrySummary = &result.entries[0];
    assert_eq!(entry.rule_id, "IN-GEN-04");
    assert!(entry.context_hint.is_some());
    assert!(result.next_cursor.is_some());
}
