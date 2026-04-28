//! Critical 规则强制 fail-closed 名单（关联 ADR-007 / ADR-014 / PRD v1.4 §5.4）。
//!
//! ## 语义说明
//!
//! - [`FAIL_CLOSED_RULES`]：**不可关闭、不可永久白名单**的规则集合（所有 Critical），
//!   包括 Hook 类——Hook 的 fail-closed 由 sieve-hook 侧实现，但代理侧同样不允许绕过。
//! - [`HOOK_RULES`]：disposition=HookTerminal 的规则（IN-CR-02~04 + IN-GEN-01~03），
//!   命中后写 IPC pending file，由 sieve-hook 在 PreToolUse 阶段拦截。
//! - [`GUI_RULES`]：disposition=GuiPopup 的规则（IN-CR-01/05 + IN-GEN-04 + OUT-06~10），
//!   命中后 hold SSE 流并通过 IPC 弹出 GUI 等待决策。
//!
//! 变更需走 ADR（关联 ADR-007 §2 / ADR-014 §"disposition 矩阵"）。

use crate::manifest::Action;

/// fail-closed 规则 ID 清单。
///
/// 包含所有 Critical 规则（IN-CR-* + 出站 Critical OUT-*）。Hook 类规则的
/// fail-closed 由 sieve-hook 实现，但本清单同样列入以保证代理侧不可旁路。
/// 变更此清单需更新对应 ADR（ADR-007 §2）。
pub const FAIL_CLOSED_RULES: &[&str] = &[
    // IN-CR-01：地址替换（gui_popup，sieve-core::address_guard 实现）
    "IN-CR-01",
    // IN-CR-02：危险 shell 命令（hook_terminal）
    "IN-CR-02",
    "IN-CR-02-CURL-PIPE",
    "IN-CR-02-WGET-PIPE",
    "IN-CR-02-EVAL",
    "IN-CR-02-NC-REVERSE",
    "IN-CR-02-DD-WIPE",
    // IN-CR-04 持久化机制（hook_terminal，Week 4 落地，PRD §5.2 / US-07）
    "IN-CR-04-SHELL-RC-APPEND",
    "IN-CR-04-CRONTAB",
    "IN-CR-04-CRON-D-WRITE",
    "IN-CR-04-LAUNCHCTL",
    "IN-CR-04-LAUNCH-AGENT-PLIST",
    "IN-CR-04-SYSTEMCTL-ENABLE",
    "IN-CR-04-SYSTEMD-UNIT-WRITE",
    "IN-CR-04-FISH-CONFIG",
    "IN-CR-04-LOGIN-ITEMS",
    // IN-CR-05：签名工具（gui_popup，签名不可逆，PRD §9 #3）
    "IN-CR-05-EVM",
    "IN-CR-05-SOLANA",
    "IN-CR-05-BITCOIN",
    "IN-CR-05-MALFORMED", // P0-6: malformed tool_use partial_json fail-closed（PRD §9 #3）
    // IN-GEN-01/03：JS URI + bash -c（hook_terminal）
    "IN-GEN-01",
    "IN-GEN-03",
    // 出站 Critical（auto_redact 或 gui_popup，timeout default_on_timeout=block）
    "OUT-01",
    "OUT-02",
    "OUT-03",
    "OUT-04",
    "OUT-07",
    "OUT-08",
    "OUT-09",
    "OUT-10",
];

/// disposition=HookTerminal 的规则集合（PRD v1.4 §5.4.1 / ADR-014）。
///
/// 这些规则命中后，代理侧**不截断 SSE 流**，而是写 IPC pending file，
/// 由 sieve-hook 在 Claude Code PreToolUse 钩子阶段拦截决策。
pub const HOOK_RULES: &[&str] = &[
    // IN-CR-02：危险 shell 命令
    "IN-CR-02",
    "IN-CR-02-CURL-PIPE",
    "IN-CR-02-WGET-PIPE",
    "IN-CR-02-EVAL",
    "IN-CR-02-NC-REVERSE",
    "IN-CR-02-DD-WIPE",
    // IN-CR-03：敏感路径访问
    "IN-CR-03-SSH-PRIVATE",
    "IN-CR-03-SSH-DIR",
    "IN-CR-03-AWS-CREDS",
    "IN-CR-03-DOTENV",
    "IN-CR-03-ETH-KEYSTORE",
    "IN-CR-03-GPG-DIR",
    "IN-CR-03-NETRC",
    "IN-CR-03-MACOS-KEYCHAIN",
    "IN-CR-03-GCP-CREDS",
    "IN-CR-03-SOLANA-KEYPAIR",
    // IN-CR-04：持久化机制
    "IN-CR-04-SHELL-RC-APPEND",
    "IN-CR-04-CRONTAB",
    "IN-CR-04-CRON-D-WRITE",
    "IN-CR-04-LAUNCHCTL",
    "IN-CR-04-LAUNCH-AGENT-PLIST",
    "IN-CR-04-SYSTEMCTL-ENABLE",
    "IN-CR-04-SYSTEMD-UNIT-WRITE",
    "IN-CR-04-FISH-CONFIG",
    "IN-CR-04-LOGIN-ITEMS",
    // IN-GEN-01~03：JS URI + 外链 img + bash -c
    "IN-GEN-01",
    "IN-GEN-02",
    "IN-GEN-03",
];

/// disposition=GuiPopup 的规则集合（PRD v1.4 §5.4.1 / ADR-014）。
///
/// 这些规则命中后，代理侧 hold SSE 流，通过 IPC 通知 GUI 弹窗等待用户决策。
pub const GUI_RULES: &[&str] = &[
    // 入站 Critical：地址替换 + 签名工具
    "IN-CR-01",
    "IN-CR-05-EVM",
    "IN-CR-05-SOLANA",
    "IN-CR-05-BITCOIN",
    "IN-CR-05-MALFORMED",
    // IN-GEN-04：markdown exfil
    "IN-GEN-04",
    // 出站：JWT + PEM + Stripe + Slack + OpenSSH
    "OUT-06",
    "OUT-07",
    "OUT-08",
    "OUT-09",
    "OUT-10",
];

/// 检查给定 rule_id 是否在 fail-closed 名单中。
pub fn is_fail_closed(rule_id: &str) -> bool {
    FAIL_CLOSED_RULES.contains(&rule_id)
}

/// 检查给定 rule_id 是否为 HookTerminal 处置规则。
pub fn is_hook_rule(rule_id: &str) -> bool {
    HOOK_RULES.contains(&rule_id)
}

/// 检查给定 rule_id 是否为 GuiPopup 处置规则。
pub fn is_gui_rule(rule_id: &str) -> bool {
    GUI_RULES.contains(&rule_id)
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
        // IN-GEN-04 markdown exfil 是 high warn（gui_popup，不在 fail-closed 名单）
        assert!(!is_fail_closed("IN-GEN-04"));
        // 旧 ID 不再存在；显式断言以防回归
        assert!(!is_fail_closed("IN-CR-04"));
    }

    #[test]
    fn in_cr_04_persistence_fail_closed() {
        // Week 4：IN-CR-04 持久化机制全部 9 条进 fail-closed 名单
        assert!(is_fail_closed("IN-CR-04-SHELL-RC-APPEND"));
        assert!(is_fail_closed("IN-CR-04-CRONTAB"));
        assert!(is_fail_closed("IN-CR-04-CRON-D-WRITE"));
        assert!(is_fail_closed("IN-CR-04-LAUNCHCTL"));
        assert!(is_fail_closed("IN-CR-04-LAUNCH-AGENT-PLIST"));
        assert!(is_fail_closed("IN-CR-04-SYSTEMCTL-ENABLE"));
        assert!(is_fail_closed("IN-CR-04-SYSTEMD-UNIT-WRITE"));
        assert!(is_fail_closed("IN-CR-04-FISH-CONFIG"));
        assert!(is_fail_closed("IN-CR-04-LOGIN-ITEMS"));
    }

    #[test]
    fn enforce_overrides_action() {
        assert_eq!(enforce_action("OUT-01", Action::Allow), Action::Block);
        assert_eq!(enforce_action("UNKNOWN", Action::Mark), Action::Mark);
        // IN-CR-04 持久化即使 manifest 写 Warn 也强制 Block
        assert_eq!(
            enforce_action("IN-CR-04-CRONTAB", Action::Warn),
            Action::Block
        );
    }

    /// HOOK_RULES 与 GUI_RULES 不应有重叠（两个 disposition 互斥）。
    #[test]
    fn hook_and_gui_rules_are_disjoint() {
        for id in HOOK_RULES {
            assert!(
                !GUI_RULES.contains(id),
                "rule {id} is in both HOOK_RULES and GUI_RULES — disposition must be unique"
            );
        }
    }

    /// FAIL_CLOSED_RULES 必须包含所有 IN-CR-* Critical 规则。
    #[test]
    fn fail_closed_covers_all_in_cr() {
        let in_cr_critical = [
            "IN-CR-01",
            "IN-CR-02",
            "IN-CR-02-CURL-PIPE",
            "IN-CR-02-WGET-PIPE",
            "IN-CR-02-EVAL",
            "IN-CR-02-NC-REVERSE",
            "IN-CR-02-DD-WIPE",
            "IN-CR-04-SHELL-RC-APPEND",
            "IN-CR-04-CRONTAB",
            "IN-CR-04-CRON-D-WRITE",
            "IN-CR-04-LAUNCHCTL",
            "IN-CR-04-LAUNCH-AGENT-PLIST",
            "IN-CR-04-SYSTEMCTL-ENABLE",
            "IN-CR-04-SYSTEMD-UNIT-WRITE",
            "IN-CR-04-FISH-CONFIG",
            "IN-CR-04-LOGIN-ITEMS",
            "IN-CR-05-EVM",
            "IN-CR-05-SOLANA",
            "IN-CR-05-BITCOIN",
        ];
        for id in in_cr_critical {
            assert!(
                is_fail_closed(id),
                "Critical rule {id} must be in FAIL_CLOSED_RULES"
            );
        }
    }

    /// IN-CR-02 系列必须在 HOOK_RULES 中。
    #[test]
    fn in_cr_02_in_hook_rules() {
        for id in [
            "IN-CR-02",
            "IN-CR-02-CURL-PIPE",
            "IN-CR-02-WGET-PIPE",
            "IN-CR-02-EVAL",
            "IN-CR-02-NC-REVERSE",
            "IN-CR-02-DD-WIPE",
        ] {
            assert!(is_hook_rule(id), "{id} must be in HOOK_RULES");
            assert!(!is_gui_rule(id), "{id} must NOT be in GUI_RULES");
        }
    }

    /// IN-CR-05 系列必须在 GUI_RULES 中。
    #[test]
    fn in_cr_05_in_gui_rules() {
        for id in ["IN-CR-05-EVM", "IN-CR-05-SOLANA", "IN-CR-05-BITCOIN"] {
            assert!(is_gui_rule(id), "{id} must be in GUI_RULES");
            assert!(!is_hook_rule(id), "{id} must NOT be in HOOK_RULES");
        }
    }
}
