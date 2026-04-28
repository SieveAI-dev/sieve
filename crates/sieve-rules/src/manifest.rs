//! 规则包 manifest（关联 ADR-002 / data-model.md / PRD v1.4 §5.3 §5.4）。

use serde::{Deserialize, Serialize};

/// 规则包 manifest（rules-vN.manifest.json）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesManifest {
    /// schema 版本。
    pub schema_version: u32,
    /// 规则集版本（单调递增整数，如 1, 2, 3）。
    pub rules_version: u64,
    /// 生效日期（UTC ISO-8601）。
    pub effective_date: String,
    /// 规则条目列表。
    pub rules: Vec<RuleEntry>,
}

/// 单条规则。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEntry {
    /// 规则 ID（如 OUT-01）。
    pub id: String,
    /// 严重等级。
    pub severity: Severity,
    /// 处置动作。
    pub action: Action,
    /// 模式串（vectorscan 兼容 PCRE 子集）。
    pub pattern: String,
    /// 规则描述。
    pub description: String,
    /// 最低 Shannon entropy 阈值（None 表示不检查，关联 FP 控制）。
    #[serde(default)]
    pub entropy_min: Option<f32>,
    /// 快速预过滤关键词（命中后再走 vectorscan）。
    #[serde(default)]
    pub keywords: Vec<String>,
    /// 允许放行的正则列表（命中后检查，任一匹配则不定级 Critical）。
    #[serde(default)]
    pub allowlist_regexes: Vec<String>,
    /// 允许放行的停用词列表（命中后检查，任一出现则不定级 Critical）。
    #[serde(default)]
    pub allowlist_stopwords: Vec<String>,
    /// 处置形式（PRD v1.4 §5.4.1）。
    ///
    /// `None` 表示 TOML 未显式写，调用 [`RuleEntry::effective_disposition`] 获取
    /// 按 severity 保守推断的值：Critical → [`Disposition::GuiPopup`]，
    /// 其他 → [`Disposition::StatusBar`]。
    #[serde(default)]
    pub disposition: Option<Disposition>,
    /// 等待 GUI/hook 决策的超时秒数（`None` = 不超时，适用于 AutoRedact / StatusBar）。
    #[serde(default)]
    pub timeout_seconds: Option<u32>,
    /// 超时后的默认处置（PRD v1.4 §5.4.2）。
    #[serde(default = "default_on_timeout_block")]
    pub default_on_timeout: DefaultOnTimeout,
}

impl RuleEntry {
    /// 返回规则的最终处置形式（PRD v1.4 §5.4.1）。
    ///
    /// TOML 未显式写 `disposition` 时，按 severity 保守推断：
    /// - [`Severity::Critical`] → [`Disposition::GuiPopup`]
    /// - 其他 → [`Disposition::StatusBar`]
    pub fn effective_disposition(&self) -> Disposition {
        self.disposition.unwrap_or(match self.severity {
            Severity::Critical => Disposition::GuiPopup,
            _ => Disposition::StatusBar,
        })
    }
}

/// 规则触发后的处置形式（PRD v1.4 §5.4.1 / ADR-016）。
///
/// 决定命中后产物如何到达用户：自动改写、GUI 弹窗、hook 拦截还是静默通知。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    /// 自动脱敏改写 body bytes 后转发，不弹窗（OUT-01~05/12）。
    AutoRedact,
    /// hold 住 SSE 流，通过 IPC 通知 GUI 弹窗等待决策（IN-CR-01/05、IN-GEN-04、OUT-06~10）。
    GuiPopup,
    /// 不修改 SSE 流，写 IPC pending file，由 sieve-hook 在 PreToolUse 阶段拦截
    /// （IN-CR-02~04、IN-GEN-01~03）。
    HookTerminal,
    /// 状态栏通知，不打断用户流程（OUT-11、IN-GEN-05）。
    StatusBar,
}

/// 规则超时后的默认处置（PRD v1.4 §5.4.2）。
///
/// 当 GUI 弹窗或 hook 等待超过 `timeout_seconds` 后触发此动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultOnTimeout {
    /// 脱敏后发送（出站默认 fail-open 到脱敏）。
    Redact,
    /// 拒绝（入站默认 fail-closed）。
    Block,
    /// 允许通过（仅 IN-GEN Relaxed preset 用）。
    Allow,
}

fn default_on_timeout_block() -> DefaultOnTimeout {
    DefaultOnTimeout::Block
}

/// 严重等级。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// 低危。
    Low,
    /// 中危。
    Medium,
    /// 高危。
    High,
    /// 严重（PRD §9 FP < 0.5% 公理 12）。
    Critical,
}

/// 处置动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// 放行。
    Allow,
    /// 标记但不阻断。
    Mark,
    /// 弹出警告。
    Warn,
    /// 阻断。
    Block,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let json = r#"{
            "schema_version": 1,
            "rules_version": 1,
            "effective_date": "2026-04-27",
            "rules": []
        }"#;
        let m: RulesManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.schema_version, 1);
        assert!(m.rules.is_empty());
    }

    #[test]
    fn severity_serde() {
        let s = Severity::Critical;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"critical\"");
    }

    #[test]
    fn parse_manifest_with_rules() {
        let json = r#"{
            "schema_version": 1,
            "rules_version": 2,
            "effective_date": "2026-04-27",
            "rules": [
                {
                    "id": "OUT-01",
                    "severity": "critical",
                    "action": "block",
                    "pattern": "(?i)private[_\\s]?key",
                    "description": "检测输出中的私钥泄露"
                }
            ]
        }"#;
        let m: RulesManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.rules.len(), 1);
        assert_eq!(m.rules[0].id, "OUT-01");
        assert_eq!(m.rules[0].severity, Severity::Critical);
        assert_eq!(m.rules[0].action, Action::Block);
    }

    #[test]
    fn action_serde() {
        let a = Action::Block;
        let json = serde_json::to_string(&a).unwrap();
        assert_eq!(json, "\"block\"");
    }

    // -------------------------------------------------------------------------
    // PRD v1.4 §5.4 新字段测试
    // -------------------------------------------------------------------------

    /// 旧格式 TOML（无 disposition / timeout_seconds / default_on_timeout）
    /// 必须能正常解析，不 break 现有规则文件。
    #[test]
    fn old_toml_without_disposition_parses_ok() {
        let toml = r#"
[[rules]]
id = "OUT-01"
description = "test"
pattern = "secret"
severity = "critical"
action = "block"
"#;
        #[derive(serde::Deserialize)]
        struct F {
            rules: Vec<RuleEntry>,
        }
        let f: F = toml::from_str(toml).unwrap();
        let r = &f.rules[0];
        assert!(r.disposition.is_none());
        assert!(r.timeout_seconds.is_none());
        assert_eq!(r.default_on_timeout, DefaultOnTimeout::Block);
    }

    /// Critical 规则未写 disposition 时 effective_disposition → GuiPopup。
    #[test]
    fn effective_disposition_critical_defaults_to_gui_popup() {
        let toml = r#"
[[rules]]
id = "IN-CR-02"
description = "test"
pattern = "rm"
severity = "critical"
action = "block"
"#;
        #[derive(serde::Deserialize)]
        struct F {
            rules: Vec<RuleEntry>,
        }
        let f: F = toml::from_str(toml).unwrap();
        assert_eq!(
            f.rules[0].effective_disposition(),
            Disposition::GuiPopup,
            "Critical without explicit disposition must default to GuiPopup"
        );
    }

    /// 非 Critical 规则未写 disposition 时 effective_disposition → StatusBar。
    #[test]
    fn effective_disposition_non_critical_defaults_to_status_bar() {
        let toml = r#"
[[rules]]
id = "IN-GEN-02"
description = "test"
pattern = "img"
severity = "high"
action = "warn"
"#;
        #[derive(serde::Deserialize)]
        struct F {
            rules: Vec<RuleEntry>,
        }
        let f: F = toml::from_str(toml).unwrap();
        assert_eq!(
            f.rules[0].effective_disposition(),
            Disposition::StatusBar,
            "Non-critical without explicit disposition must default to StatusBar"
        );
    }

    /// 显式写了 disposition = "hook_terminal" 时必须正确解析。
    #[test]
    fn explicit_hook_terminal_disposition_parses() {
        let toml = r#"
[[rules]]
id = "IN-CR-02"
description = "test"
pattern = "rm"
severity = "critical"
action = "block"
disposition = "hook_terminal"
timeout_seconds = 30
default_on_timeout = "block"
"#;
        #[derive(serde::Deserialize)]
        struct F {
            rules: Vec<RuleEntry>,
        }
        let f: F = toml::from_str(toml).unwrap();
        let r = &f.rules[0];
        assert_eq!(r.effective_disposition(), Disposition::HookTerminal);
        assert_eq!(r.timeout_seconds, Some(30));
        assert_eq!(r.default_on_timeout, DefaultOnTimeout::Block);
    }

    /// disposition = "auto_redact" + default_on_timeout = "redact" 正确解析。
    #[test]
    fn auto_redact_disposition_parses() {
        let toml = r#"
[[rules]]
id = "OUT-01"
description = "test"
pattern = "sk-ant"
severity = "critical"
action = "block"
disposition = "auto_redact"
default_on_timeout = "redact"
"#;
        #[derive(serde::Deserialize)]
        struct F {
            rules: Vec<RuleEntry>,
        }
        let f: F = toml::from_str(toml).unwrap();
        let r = &f.rules[0];
        assert_eq!(r.effective_disposition(), Disposition::AutoRedact);
        assert_eq!(r.default_on_timeout, DefaultOnTimeout::Redact);
        assert!(r.timeout_seconds.is_none());
    }

    /// Disposition 枚举 serde snake_case 正确。
    #[test]
    fn disposition_serde_roundtrip() {
        for (d, expected) in [
            (Disposition::AutoRedact, "\"auto_redact\""),
            (Disposition::GuiPopup, "\"gui_popup\""),
            (Disposition::HookTerminal, "\"hook_terminal\""),
            (Disposition::StatusBar, "\"status_bar\""),
        ] {
            let json = serde_json::to_string(&d).unwrap();
            assert_eq!(json, expected);
            let back: Disposition = serde_json::from_str(&json).unwrap();
            assert_eq!(back, d);
        }
    }

    /// DefaultOnTimeout 枚举 serde snake_case 正确。
    #[test]
    fn default_on_timeout_serde_roundtrip() {
        for (d, expected) in [
            (DefaultOnTimeout::Redact, "\"redact\""),
            (DefaultOnTimeout::Block, "\"block\""),
            (DefaultOnTimeout::Allow, "\"allow\""),
        ] {
            let json = serde_json::to_string(&d).unwrap();
            assert_eq!(json, expected);
            let back: DefaultOnTimeout = serde_json::from_str(&json).unwrap();
            assert_eq!(back, d);
        }
    }
}
