//! 从 toml 加载出站规则集。

use crate::error::{SieveRulesError, SieveRulesResult};
use crate::manifest::RuleEntry;
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
}
