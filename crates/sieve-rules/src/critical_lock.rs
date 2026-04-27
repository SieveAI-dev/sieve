//! Critical 规则强制 fail-closed 名单（关联 ADR-007）。
//!
//! 此清单中的规则，无论 config 如何设置（包括 dry_run = true），
//! 命中时 action 强制为 Block，无视 manifest 中的 action 字段。

use crate::manifest::Action;

/// fail-closed 规则 ID 清单。变更需走 ADR（关联 ADR-007 §2）。
pub const FAIL_CLOSED_RULES: &[&str] = &[
    // 入站
    "IN-CR-01",
    "IN-CR-02",
    "IN-CR-02-CURL-PIPE",
    "IN-CR-02-WGET-PIPE",
    "IN-CR-02-EVAL",
    "IN-CR-02-NC-REVERSE",
    "IN-CR-02-DD-WIPE",
    "IN-CR-05-EVM",
    "IN-CR-05-SOLANA",
    "IN-CR-05-BITCOIN",
    "IN-CR-05-MALFORMED", // P0-6: malformed tool_use partial_json fail-closed（PRD §9 #3）
    "IN-GEN-01",
    "IN-GEN-03",
    // 出站（全部 OUT-01~12）
    "OUT-01",
    "OUT-02",
    "OUT-03",
    "OUT-04",
    "OUT-05",
    "OUT-06",
    "OUT-07",
    "OUT-08",
    "OUT-09",
    "OUT-10",
    "OUT-11",
    "OUT-12",
];

/// 检查给定 rule_id 是否在 fail-closed 名单中。
pub fn is_fail_closed(rule_id: &str) -> bool {
    FAIL_CLOSED_RULES.contains(&rule_id)
}

/// 强制覆盖 action：fail-closed 规则一律返回 Block。
pub fn enforce_action(rule_id: &str, requested: Action) -> Action {
    if is_fail_closed(rule_id) {
        Action::Block
    } else {
        requested
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_critical_rules_in_list() {
        assert!(is_fail_closed("OUT-01"));
        assert!(is_fail_closed("IN-CR-05-EVM"));
        assert!(is_fail_closed("IN-CR-02-CURL-PIPE"));
    }

    #[test]
    fn unknown_rule_not_failclosed() {
        assert!(!is_fail_closed("UNKNOWN-RULE"));
        assert!(!is_fail_closed("IN-CR-04")); // markdown exfil 是 warn
    }

    #[test]
    fn enforce_overrides_action() {
        assert_eq!(enforce_action("OUT-01", Action::Allow), Action::Block);
        assert_eq!(enforce_action("UNKNOWN", Action::Mark), Action::Mark);
    }
}
