//! 从 toml（公开仓 dev 源 / 编译期嵌入源）或 JSON 签名规则包加载规则集。

use crate::error::{SieveRulesError, SieveRulesResult};
use crate::manifest::{RuleEntry, RulesManifest};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct OutboundRulesFile {
    rules: Vec<RuleEntry>,
}

/// 从 toml 文件加载出站规则。
pub fn load_outbound_rules(path: &Path) -> SieveRulesResult<Vec<RuleEntry>> {
    let s = std::fs::read_to_string(path)
        .map_err(|e| SieveRulesError::Manifest(format!("read {}: {e}", path.display())))?;
    let f: OutboundRulesFile = toml::from_str(&s)
        .map_err(|e| SieveRulesError::Manifest(format!("parse {}: {e}", path.display())))?;
    Ok(f.rules)
}

/// 加载入站规则集（toml schema 与出站一致）。
pub fn load_inbound_rules(path: &Path) -> SieveRulesResult<Vec<RuleEntry>> {
    load_outbound_rules(path) // schema 同，直接复用
}

/// 从 JSON 签名规则包 manifest 加载规则（updater 安装的 `current.json`）。
///
/// 这是签名规则包的运行时加载入口：规则经更新通道下发，由 `sieve-updater::install`
/// 验签后落盘为 `current.json`。
/// 与 [`load_outbound_rules`] 的 TOML 路径并存——TOML 留作 dev 源 + 编译期嵌入的
/// 最小集；JSON [`RulesManifest`] 是签名包的 wire 格式，带 `schema_version` /
/// `rules_version` / `effective_date` 元数据。[`RuleEntry`] 的 `#[serde(default)]` 字段
/// 保证 JSON 与 TOML 等价解析（见 `manifest.rs` 双向序列化测试）。
///
/// # Errors
///
/// 文件读取失败或 JSON 反序列化失败时返回 [`SieveRulesError::Manifest`]。
pub fn load_rules_from_manifest_json(path: &Path) -> SieveRulesResult<Vec<RuleEntry>> {
    let s = std::fs::read_to_string(path)
        .map_err(|e| SieveRulesError::Manifest(format!("read {}: {e}", path.display())))?;
    let m: RulesManifest = serde_json::from_str(&s)
        .map_err(|e| SieveRulesError::Manifest(format!("parse json {}: {e}", path.display())))?;
    Ok(m.rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_minimal_toml() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"[[rules]]
id = "OUT-TEST"
description = "test rule"
pattern = "hello"
severity = "critical"
action = "block"
"#
        )
        .unwrap();
        let rules = load_outbound_rules(f.path()).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "OUT-TEST");
    }

    #[test]
    fn loads_rule_with_optional_fields() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            f,
            r#"[[rules]]
id = "OUT-02"
description = "with optional fields"
pattern = "secret"
severity = "high"
action = "warn"
entropy_min = 3.5
keywords = ["secret", "key"]
allowlist_regexes = ["(?i)example"]
allowlist_stopwords = ["test"]
"#
        )
        .unwrap();
        let rules = load_outbound_rules(f.path()).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].entropy_min, Some(3.5));
        assert_eq!(rules[0].keywords.len(), 2);
        assert_eq!(rules[0].allowlist_regexes.len(), 1);
        assert_eq!(rules[0].allowlist_stopwords.len(), 1);
    }

    #[test]
    fn returns_error_on_missing_file() {
        let result = load_outbound_rules(Path::new("/nonexistent/path.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_on_invalid_toml() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "not valid toml [[[").unwrap();
        let result = load_outbound_rules(f.path());
        assert!(result.is_err());
    }

    #[test]
    fn loads_rules_from_manifest_json_ok() {
        use crate::manifest::Severity;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(
            f,
            r#"{{"schema_version":1,"rules_version":3,"effective_date":"2026-06-21","rules":[{{"id":"IN-CR-TEST","severity":"critical","action":"block","pattern":"danger","description":"signed pack rule"}}]}}"#
        )
        .unwrap();
        let rules = load_rules_from_manifest_json(f.path()).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "IN-CR-TEST");
        assert_eq!(rules[0].severity, Severity::Critical);
    }

    #[test]
    fn manifest_json_returns_error_on_invalid() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "{{not json").unwrap();
        assert!(load_rules_from_manifest_json(f.path()).is_err());
    }
}
