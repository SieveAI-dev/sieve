// SPEC-005 §14.1 fixture 测试：DecisionResponse v2.1 字段完整性验证。
//
// 三档 fixture 覆盖：
//   __minimal     —— 仅 required 字段，optional 字段全部缺失
//   __full        —— 全字段，包含 ui_phase_when_clicked 三种枚举值
//   __null_optional —— optional 字段显式设 null/false
//
// 关联：SPEC-005 §6.2.1, §5.10 / PROGRESS P1-4。

use sieve_ipc::protocol::{DecisionAction, DecisionResponse, UiPhase};
use uuid::Uuid;

/// __minimal：仅含 required 字段，optional 字段在 JSON 中不存在。
/// 验证 serde default 兜底逻辑。
#[test]
fn decision_response_v2_minimal() {
    let rid = Uuid::now_v7();
    let json = format!(
        r#"{{
            "request_id": "{rid}",
            "decision": "allow",
            "decided_at": "2026-05-03T10:00:00Z",
            "by_user": true
        }}"#
    );

    let resp: DecisionResponse = serde_json::from_str(&json).expect("deserialize minimal");
    assert_eq!(resp.request_id, rid);
    assert_eq!(resp.decision, DecisionAction::Allow);
    assert!(resp.by_user);
    // optional 字段缺失时 serde 默认
    assert!(!resp.remember);
    assert!(resp.context_hint.is_none());
    assert!(resp.ui_phase_when_clicked.is_none());
}

/// __full：全字段，测试三种 UiPhase 枚举序列化 / 反序列化。
#[test]
fn decision_response_v2_full() {
    let rid = Uuid::now_v7();

    // blue
    let json_blue = format!(
        r#"{{
            "request_id": "{rid}",
            "decision": "deny",
            "decided_at": "2026-05-03T10:00:00Z",
            "by_user": true,
            "remember": false,
            "context_hint": "测试备注",
            "ui_phase_when_clicked": "blue"
        }}"#
    );
    let resp: DecisionResponse = serde_json::from_str(&json_blue).expect("deserialize blue");
    assert_eq!(resp.ui_phase_when_clicked, Some(UiPhase::Blue));
    assert_eq!(resp.context_hint.as_deref(), Some("测试备注"));
    assert_eq!(resp.decision, DecisionAction::Deny);

    // orange
    let json_orange = format!(
        r#"{{
            "request_id": "{rid}",
            "decision": "allow",
            "decided_at": "2026-05-03T10:00:00Z",
            "by_user": true,
            "remember": true,
            "context_hint": null,
            "ui_phase_when_clicked": "orange"
        }}"#
    );
    let resp: DecisionResponse = serde_json::from_str(&json_orange).expect("deserialize orange");
    assert_eq!(resp.ui_phase_when_clicked, Some(UiPhase::Orange));
    assert!(resp.remember);

    // red
    let json_red = format!(
        r#"{{
            "request_id": "{rid}",
            "decision": "redact_and_allow",
            "decided_at": "2026-05-03T10:00:00Z",
            "by_user": false,
            "ui_phase_when_clicked": "red"
        }}"#
    );
    let resp: DecisionResponse = serde_json::from_str(&json_red).expect("deserialize red");
    assert_eq!(resp.ui_phase_when_clicked, Some(UiPhase::Red));
    assert_eq!(resp.decision, DecisionAction::RedactAndAllow);
    assert!(!resp.by_user);

    // 序列化再反序列化 round-trip
    let serialized = serde_json::to_string(&resp).expect("serialize");
    let decoded: DecisionResponse = serde_json::from_str(&serialized).expect("round-trip");
    assert_eq!(decoded.ui_phase_when_clicked, Some(UiPhase::Red));
}

/// __null_optional：optional 字段显式传 null，验证不报错且值为 None。
#[test]
fn decision_response_v2_null_optional() {
    let rid = Uuid::now_v7();
    let json = format!(
        r#"{{
            "request_id": "{rid}",
            "decision": "deny",
            "decided_at": "2026-05-03T10:00:00Z",
            "by_user": false,
            "remember": false,
            "context_hint": null,
            "ui_phase_when_clicked": null
        }}"#
    );

    let resp: DecisionResponse = serde_json::from_str(&json).expect("deserialize null optional");
    assert!(resp.context_hint.is_none());
    assert!(resp.ui_phase_when_clicked.is_none());
    assert!(!resp.by_user);
    assert_eq!(resp.decision, DecisionAction::Deny);

    // 确保序列化输出中 ui_phase_when_clicked 字段存在（值为 null）
    let out: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&resp).expect("serialize"))
            .expect("parse back");
    assert!(
        out.get("ui_phase_when_clicked").is_some(),
        "ui_phase_when_clicked should be present in serialized output"
    );
}
