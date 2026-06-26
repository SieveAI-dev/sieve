// SPEC-005 §14.1 fixture 测试：DecisionResponse v2.1 字段完整性验证。
//
// 三档 fixture 覆盖：
//   __minimal     —— 仅 required 字段，optional 字段全部缺失
//   __full        —— 全字段，包含 ui_phase_when_clicked 三种枚举值
//   __null_optional —— optional 字段显式设 null/false
//
// 关联：SPEC-005 §6.2.1, §5.10 / PROGRESS P1-4。

use sieve_ipc::protocol::{
    DecisionAction, DecisionResponse, MergedDecisionAction, MergedDecisionResponse, UiPhase,
};
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

#[test]
fn merged_all_deny_deserializes_and_aggregates() {
    let json = r#"{
      "request_id":"9c1d8b73-2a4f-4e6c-b5d8-3e7f1a9c2b04",
      "merged_decision":"all_deny",
      "per_issue":[
        {"issue_id":"i-1","decision":"deny","remember":false},
        {"issue_id":"i-2","decision":"deny","remember":false}
      ],
      "decided_at":"2026-05-03T12:02:00.000Z",
      "by_user":true
    }"#;
    let merged: MergedDecisionResponse = serde_json::from_str(json).expect("merged decode");
    assert_eq!(merged.merged_decision, MergedDecisionAction::AllDeny);
    let aggregate = merged.into_aggregate();
    assert_eq!(aggregate.decision, DecisionAction::Deny);
    assert!(aggregate.by_user);
}

/// 回归 fail-closed 聚合缺陷：含 Deny 的 Partial 必须 fail-closed 聚合为 Deny，绝不降级为
/// RedactAndAllow。入站把 RedactAndAllow 当 Allow——若用户拒掉 Critical(IN-CR-*)、放行
/// non-critical，旧实现会把被拒的 Critical 静默降级并转发危险 tool call（违反核心安全不变量：
/// Critical 决策任何情况不得降级，且是「解码失败→超时→Block→Deny」旧行为的安全回归）。
/// 聚合为 Deny 时 remember 亦强制 false。
#[test]
fn merged_partial_with_deny_aggregates_to_deny_fail_closed() {
    let json = r#"{
      "request_id":"9c1d8b73-2a4f-4e6c-b5d8-3e7f1a9c2b04",
      "merged_decision":"partial",
      "per_issue":[
        {"issue_id":"i-1","decision":"deny","remember":false},
        {"issue_id":"i-2","decision":"allow","remember":true,"context_hint":"safe"}
      ],
      "decided_at":"2026-05-03T12:02:00.000Z",
      "by_user":true
    }"#;
    let aggregate = serde_json::from_str::<MergedDecisionResponse>(json)
        .expect("merged decode")
        .into_aggregate();
    assert_eq!(
        aggregate.decision,
        DecisionAction::Deny,
        "Partial 含 Deny 必须 fail-closed 聚合为 Deny，绝不降级被拒的 Critical"
    );
    assert!(!aggregate.remember, "聚合为 Deny 时不得 remember 拒绝");
}

/// 无任何 Deny 的 Partial（混合 Allow + RedactAndAllow）→ 保守 RedactAndAllow：出站脱敏后
/// 转发、入站无危险项放行。守护「仅无 Deny 才脱敏转发」的另一半不变量。
#[test]
fn merged_partial_without_deny_aggregates_to_redact_and_allow() {
    let json = r#"{
      "request_id":"9c1d8b73-2a4f-4e6c-b5d8-3e7f1a9c2b04",
      "merged_decision":"partial",
      "per_issue":[
        {"issue_id":"i-1","decision":"redact_and_allow","remember":false},
        {"issue_id":"i-2","decision":"allow","remember":true,"context_hint":"safe"}
      ],
      "decided_at":"2026-05-03T12:02:00.000Z",
      "by_user":true
    }"#;
    let aggregate = serde_json::from_str::<MergedDecisionResponse>(json)
        .expect("merged decode")
        .into_aggregate();
    assert_eq!(aggregate.decision, DecisionAction::RedactAndAllow);
    assert!(aggregate.remember, "无 Deny 时 Allow+remember 折叠保留");
    assert_eq!(aggregate.context_hint.as_deref(), Some("safe"));
}

/// 防御：GUI 误报 merged_decision=all_allow 标签却夹带 Deny per_issue → 信任更保守的 per_issue，
/// fail-closed 聚合为 Deny（wire 标签与 per_issue 不一致时绝不放行被拒项）。
#[test]
fn merged_all_allow_label_with_stray_deny_per_issue_is_deny() {
    let json = r#"{
      "request_id":"9c1d8b73-2a4f-4e6c-b5d8-3e7f1a9c2b04",
      "merged_decision":"all_allow",
      "per_issue":[
        {"issue_id":"i-1","decision":"allow","remember":false},
        {"issue_id":"i-2","decision":"deny","remember":false}
      ],
      "decided_at":"2026-05-03T12:02:00.000Z",
      "by_user":true
    }"#;
    let aggregate = serde_json::from_str::<MergedDecisionResponse>(json)
        .expect("merged decode")
        .into_aggregate();
    assert_eq!(
        aggregate.decision,
        DecisionAction::Deny,
        "标签 all_allow 但 per_issue 夹带 deny → 信任 per_issue，fail-closed Deny"
    );
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
