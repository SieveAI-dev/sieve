//! 规则包 manifest（关联 ADR-002 / data-model.md）。
//!
//! 实际 manifest schema 在 Week 2 完整实现，Week 1 占位以验证 serde 可用。

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
}
