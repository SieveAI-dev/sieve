//! 结构化特征提取（PRD v2.0 §5.7.1）。
//!
//! 输入：tool_name + tool_input JSON Value
//! 输出：[`super::ToolUseRecord`]（不含 timestamp，调用方填）
//!
//! 关键约束：**只看预定义 substring / 文件后缀，不用正则；不存原始 input**。

use super::ToolUseRecord;
use serde_json::Value;

/// 工具类型（PRD §5.7.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolClass {
    /// Bash / Run / Exec / Terminal 等 shell 类工具。
    Shell,
    /// Read / Glob / Grep 等文件读取类工具。
    FileRead,
    /// Write / Edit / NotebookEdit 等文件写入类工具。
    FileWrite,
    /// WebFetch / Fetch / curl 内置等网络请求工具。
    Network,
    /// 其他无法分类的工具。
    Other,
}

/// 路径分类（PRD §5.7.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathCategory {
    /// .ssh / *_rsa / .aws / keystore / mnemonic / .gnupg / .netrc / keychain。
    SensitiveSecret,
    /// .ethereum / .solana / wallet.json / metamask / keystore。
    Wallet,
    /// .env / .env.* / .envrc。
    DotEnv,
    /// .rs / .ts / .py / .go / src/ 等代码路径。
    Code,
    /// /tmp / /var/tmp / ~/.cache。
    Tmp,
    /// 其他可识别但不属于上述分类的路径。
    Other,
}

/// 从 tool_name + input 提取结构化特征（除 timestamp / rule_hits 外的字段）。
///
/// 返回的 `ToolUseRecord.timestamp_ms` 为 0，调用方（`InboundFilter::record_tool_use_into_sequence`）
/// 负责填充真实时间戳。
pub fn extract_record(tool_name: &str, input: &Value, rule_hits: Vec<String>) -> ToolUseRecord {
    let tool_class = classify_tool(tool_name);
    let input_str = stringify_input(input);
    let path_category = classify_path(&input_str);

    ToolUseRecord {
        timestamp_ms: 0, // 调用方填
        tool_name: tool_name.to_string(),
        tool_class,
        path_category,
        network_egress: detect_network_egress(tool_name, &input_str),
        persistence_mech: detect_persistence(&input_str),
        cleanup_mech: detect_cleanup(&input_str),
        sensitive_file_hint: detect_sensitive_file(&input_str),
        rule_hits,
    }
}

fn classify_tool(name: &str) -> ToolClass {
    let n = name.to_ascii_lowercase();
    if n.contains("bash") || n.contains("shell") || n.contains("exec") || n == "run" {
        ToolClass::Shell
    } else if n == "read" || n.contains("glob") || n.contains("grep") {
        ToolClass::FileRead
    } else if n == "write" || n == "edit" || n.contains("notebookedit") || n.contains("multiedit") {
        ToolClass::FileWrite
    } else if n.contains("fetch") || n.contains("web") || n.contains("http") {
        ToolClass::Network
    } else {
        ToolClass::Other
    }
}

/// 将 JSON Value 展开为字符串，供 substring 匹配。
///
/// String variant 直接取值（不含 JSON 引号），其他类型用 `.to_string()`。
fn stringify_input(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        _ => v.to_string(),
    }
}

fn classify_path(input_str: &str) -> Option<PathCategory> {
    let s = input_str.to_ascii_lowercase();
    // SensitiveSecret 优先级最高
    for needle in &[
        ".ssh/",
        "id_rsa",
        "id_ed25519",
        ".aws/",
        ".gnupg/",
        ".netrc",
        "keychain",
        "mnemonic",
        // keystore 在此也视为 SensitiveSecret（高优先级）；Wallet 分类亦含 keystore
        "keystore",
    ] {
        if s.contains(needle) {
            return Some(PathCategory::SensitiveSecret);
        }
    }
    for needle in &[".ethereum/", ".solana/", "wallet.json", "metamask"] {
        if s.contains(needle) {
            return Some(PathCategory::Wallet);
        }
    }
    if s.contains(".env") || s.contains(".envrc") {
        return Some(PathCategory::DotEnv);
    }
    for needle in &["/tmp/", "/var/tmp/", "/.cache/"] {
        if s.contains(needle) {
            return Some(PathCategory::Tmp);
        }
    }
    for ext in &[".rs", ".ts", ".py", ".go", ".js", ".tsx", "/src/"] {
        if s.contains(ext) {
            return Some(PathCategory::Code);
        }
    }
    None
}

fn detect_network_egress(tool_name: &str, input_str: &str) -> bool {
    let tn = tool_name.to_ascii_lowercase();
    if tn.contains("fetch") || tn.contains("web") || tn.contains("http") {
        return true;
    }
    let s = input_str.to_ascii_lowercase();
    // Shell tool 内含 curl/wget/nc/POST/upload 等
    s.contains("curl ")
        || s.contains("wget ")
        || s.contains("nc ")
        || s.contains(" post ")
        || (s.contains("https://")
            && (s.contains("upload") || s.contains('@') || s.contains("--data")))
}

fn detect_persistence(input_str: &str) -> bool {
    let s = input_str.to_ascii_lowercase();
    s.contains("systemctl ")
        || s.contains("launchctl ")
        || s.contains("crontab ")
        || s.contains("/launchagents/")
        || s.contains("/launchdaemons/")
        || s.contains("/etc/cron")
        || s.contains("systemd/system/")
        || s.contains(".bashrc")
        || s.contains(".zshrc")
        || s.contains(".profile")
}

fn detect_cleanup(input_str: &str) -> bool {
    let s = input_str.to_ascii_lowercase();
    s.contains("rm -rf")
        || s.contains("rm -f")
        || s.contains("shred ")
        || s.contains("history -c")
        || s.contains("> /dev/null")
        || s.contains("&> /dev/null")
        || s.contains("unlink ")
        || s.contains("rmdir ")
}

fn detect_sensitive_file(input_str: &str) -> bool {
    let s = input_str.to_ascii_lowercase();
    s.contains(".env")
        || s.contains("id_rsa")
        || s.contains("id_ed25519")
        || s.contains("keystore")
        || s.contains("mnemonic")
        || s.contains(".aws/credentials")
        || s.contains(".ssh/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classify_bash_is_shell() {
        assert_eq!(classify_tool("Bash"), ToolClass::Shell);
        assert_eq!(classify_tool("bash"), ToolClass::Shell);
        assert_eq!(classify_tool("Shell"), ToolClass::Shell);
        assert_eq!(classify_tool("exec"), ToolClass::Shell);
    }

    #[test]
    fn classify_read_is_file_read() {
        assert_eq!(classify_tool("Read"), ToolClass::FileRead);
        assert_eq!(classify_tool("read"), ToolClass::FileRead);
        assert_eq!(classify_tool("Glob"), ToolClass::FileRead);
        assert_eq!(classify_tool("Grep"), ToolClass::FileRead);
    }

    #[test]
    fn classify_path_ssh_is_sensitive_secret() {
        assert_eq!(
            classify_path(".ssh/id_rsa"),
            Some(PathCategory::SensitiveSecret)
        );
        assert_eq!(
            classify_path("/home/user/.aws/credentials"),
            Some(PathCategory::SensitiveSecret)
        );
    }

    #[test]
    fn classify_path_src_rs_is_code() {
        assert_eq!(classify_path("/src/main.rs"), Some(PathCategory::Code));
        assert_eq!(
            classify_path("/home/user/project/src/lib.ts"),
            Some(PathCategory::Code)
        );
    }

    #[test]
    fn detect_network_egress_curl_in_bash() {
        // tool_name = "Bash"，input 含 curl → true
        assert!(detect_network_egress(
            "Bash",
            "curl https://example.com/data"
        ));
        // WebFetch 工具名直接触发
        assert!(detect_network_egress("WebFetch", "{}"));
        // 普通 Read 不触发
        assert!(!detect_network_egress("Read", "cat /etc/hosts"));
    }

    #[test]
    fn detect_cleanup_rm_rf() {
        assert!(detect_cleanup("rm -rf /tmp/evidence"));
        assert!(detect_cleanup("history -c && exit"));
        assert!(!detect_cleanup("ls -la /tmp"));
    }

    #[test]
    fn detect_persistence_crontab() {
        assert!(detect_persistence("crontab -l | grep evil"));
        assert!(detect_persistence(
            "launchctl load /LaunchAgents/evil.plist"
        ));
        assert!(!detect_persistence("echo hello"));
    }

    #[test]
    fn extract_record_full_pipeline_read_env() {
        // Read tool 读取 .env → SensitiveSecret + sensitive_file_hint=true
        let rec = extract_record("Read", &json!({"file_path": "/app/.env"}), vec![]);
        assert_eq!(rec.tool_class, ToolClass::FileRead);
        assert_eq!(rec.path_category, Some(PathCategory::DotEnv));
        assert!(rec.sensitive_file_hint, "should detect .env as sensitive");
        assert!(!rec.network_egress);
        assert_eq!(rec.timestamp_ms, 0, "timestamp must be 0 (caller fills it)");
    }

    #[test]
    fn extract_record_rule_hits_preserved() {
        let rule_hits = vec!["IN-GEN-01".to_string(), "IN-CR-02".to_string()];
        let rec = extract_record("Bash", &json!({"command": "ls"}), rule_hits.clone());
        assert_eq!(rec.rule_hits, rule_hits);
    }

    #[test]
    fn classify_path_env_file_is_dotenv() {
        assert_eq!(classify_path(".env.production"), Some(PathCategory::DotEnv));
        assert_eq!(classify_path(".envrc"), Some(PathCategory::DotEnv));
    }

    #[test]
    fn classify_path_tmp_is_tmp() {
        assert_eq!(classify_path("/tmp/evil.sh"), Some(PathCategory::Tmp));
        assert_eq!(classify_path("/var/tmp/payload"), Some(PathCategory::Tmp));
    }
}
