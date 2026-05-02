//! SPEC-005 v2 wire format fixture 测试（§14.1）。
//!
//! 验证 `tests/fixtures/v2/` 下的 JSON fixture 能被正确反序列化为 sieve-ipc 协议类型。
//! 当前覆盖：sieve.hello / sieve.set_paused / sieve.health 3 个 method × minimal/full/null_optional。
//!
//! TODO：扩充到 17 method × 3 = 51 条（见 fixtures/v2/README.md）。

use sieve_ipc::protocol::{HealthResult, HelloParams, SetPausedRequest, SetPausedResult};

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
