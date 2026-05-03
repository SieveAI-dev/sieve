//! SPEC-005 v2 wire format fixture 测试（§14.1）。
//!
//! 验证 `tests/fixtures/v2/` 下的 JSON fixture 能被正确反序列化为 sieve-ipc 协议类型。
//! 当前覆盖：sieve.hello / sieve.set_paused / sieve.health / sieve.request_decision（P1-5）。
//!
//! TODO：扩充到 17 method × 3 = 51 条（见 fixtures/v2/README.md）。

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
