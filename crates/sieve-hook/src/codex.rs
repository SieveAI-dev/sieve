//! Codex CLI 的 PreToolUse hook 协议层（与 Claude Code 的 `check` 子命令隔离）。
//!
//! 为什么独立模块、与 `check` 彻底隔离：**Codex 的 exit code 语义与 Claude 相反**——
//! 在 Codex 下 `exit 1 = fail-OPEN（工具继续执行！）`，而 `check` 子命令用 `exit 1` 表拒绝。
//! 混用必然把「拒绝」变成「放行」，是安全 footgun。本模块严格只产 `exit 0`（读 stdout 决策）
//! 或 `exit 2`（stderr 兜底拒绝），**绝不产 exit 1**。
//!
//! 协议契约对着 openai/codex 源码 + 本机 codex 0.137.0 实测核实（high confidence）：
//! - stdin：snake_case JSON（`PreToolUseCommandInput`）
//! - stdout：camelCase JSON，**deny_unknown_fields 严格**（多写未知字段 → codex 解析失败 → fail-open）
//! - exit：0 读 stdout / 2 读 stderr 强制 block / 1 与其他非零 = fail-open
//!
//! 本模块只做「裁决 → 输出契约」的纯映射,不含检测(检测在 daemon,经 IPC `judge_tool_call`)。
//! 拆成纯函数便于单元测试,把 exit-code footgun 在测试里锁死。

use serde::Deserialize;
use serde_json::Value;

/// Codex 经 stdin 传入的 PreToolUse JSON（snake_case，对 openai/codex 源码核实）。
///
/// 只强解析「检测必需」字段；全部 `#[serde(default)]` 且不 `deny_unknown_fields`，
/// 以容忍 codex 未来新增字段（codex 对 hook 的 stdin 无 schema 锁定，加字段不应破坏 hook）。
#[derive(Deserialize, Debug, Default, Clone)]
pub struct CodexPreToolUseInput {
    /// 工具名。⚠️ 大小写敏感：codex 命令执行工具实测为 `"exec_command"`（**非** Claude 的 `"Bash"`），
    /// 文件编辑为 `"apply_patch"`。命令字符串在 `tool_input.cmd`（非 Claude 的 `command`）。
    /// daemon 侧检测对整个 `tool_input` 做全文扫描，内容型规则字段名无关；但工具名 gate 规则与改写需 codex-aware。
    #[serde(default)]
    pub tool_name: String,
    /// 工具参数对象（Bash 时形如 `{"command": "..."}`）；入站危险检测的主输入。
    #[serde(default)]
    pub tool_input: Value,
    /// 对应 LLM response 的 tool_use_id，用作 daemon 请求关联键。
    #[serde(default)]
    pub tool_use_id: String,
    /// 工具执行的工作目录（敏感路径判定用）。
    #[serde(default)]
    pub cwd: String,
}

/// daemon 判定后的裁决，映射到 Codex 的输出契约。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexVerdict {
    /// 放行。
    Allow,
    /// 拒绝（IN-CR-* fail-closed）。`reason` 写进 codex 的 `permissionDecisionReason`。
    Deny { reason: String },
    /// 放行但改写工具输入（出站脱敏 OUT-*）。`updated_input` **整体替换** `tool_input`（非 merge）。
    Rewrite { updated_input: Value },
}

/// codex 子进程的输出三元组：退出码 + stdout + stderr。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// 把裁决映射成 Codex 期望的输出契约。
///
/// **安全不变量**：返回的 `exit_code` 只能是 `0`（正常，决策在 stdout）；**绝不返回 `1`**
/// （codex 下 exit 1 = fail-open 放行）。fail-closed 兜底用 [`emit_codex_fail_closed`]（exit 2）。
pub fn emit_codex_decision(verdict: &CodexVerdict) -> CodexOutput {
    match verdict {
        CodexVerdict::Allow => CodexOutput {
            exit_code: 0,
            // 空对象 = 放行（codex 解析 stdout，无 permissionDecision 即默认 allow）。
            stdout: "{}".to_string(),
            stderr: String::new(),
        },
        CodexVerdict::Deny { reason } => {
            let out = serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason,
                }
            });
            CodexOutput {
                exit_code: 0,
                stdout: out.to_string(),
                stderr: String::new(),
            }
        }
        CodexVerdict::Rewrite { updated_input } => {
            let out = serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "allow",
                    "updatedInput": updated_input,
                }
            });
            CodexOutput {
                exit_code: 0,
                stdout: out.to_string(),
                stderr: String::new(),
            }
        }
    }
}

/// fail-closed 兜底：daemon 不可达 / 内部错误且语义上必须拒绝时，用 `exit 2` + stderr。
///
/// 比 stdout JSON 路径更鲁棒——codex 看到 exit 2 直接 block，从 stderr 读原因，
/// 不依赖 stdout JSON 正确。**同样绝不返回 exit 1**。
pub fn emit_codex_fail_closed(reason: &str) -> CodexOutput {
    CodexOutput {
        exit_code: 2,
        stdout: String::new(),
        stderr: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_is_exit0_empty_object() {
        let o = emit_codex_decision(&CodexVerdict::Allow);
        assert_eq!(o.exit_code, 0);
        assert_eq!(o.stdout, "{}");
        assert!(o.stderr.is_empty());
    }

    #[test]
    fn deny_is_exit0_with_permission_decision_deny() {
        let o = emit_codex_decision(&CodexVerdict::Deny {
            reason: "含硬编码私钥，拒绝执行".to_string(),
        });
        // ⚠️ deny 走 exit 0 + stdout，不是 exit 1（exit 1 在 codex = 放行）。
        assert_eq!(o.exit_code, 0);
        let v: Value = serde_json::from_str(&o.stdout).unwrap();
        assert_eq!(v["hookSpecificOutput"]["hookEventName"], "PreToolUse");
        assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
        assert_eq!(
            v["hookSpecificOutput"]["permissionDecisionReason"],
            "含硬编码私钥，拒绝执行"
        );
        assert!(o.stderr.is_empty());
    }

    /// 安全关键回归测试：deny 绝不能产 exit 1（codex 下 exit 1 = fail-open）。
    #[test]
    fn deny_never_uses_exit_1() {
        let o = emit_codex_decision(&CodexVerdict::Deny {
            reason: "x".to_string(),
        });
        assert_ne!(
            o.exit_code, 1,
            "exit 1 在 codex = 放行；deny 绝不能用 exit 1"
        );
    }

    #[test]
    fn rewrite_is_exit0_allow_with_updated_input() {
        let ui = serde_json::json!({"command": "echo redacted"});
        let o = emit_codex_decision(&CodexVerdict::Rewrite {
            updated_input: ui.clone(),
        });
        assert_eq!(o.exit_code, 0);
        let v: Value = serde_json::from_str(&o.stdout).unwrap();
        assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "allow");
        assert_eq!(v["hookSpecificOutput"]["updatedInput"], ui);
    }

    #[test]
    fn fail_closed_is_exit2_with_stderr_reason() {
        let o = emit_codex_fail_closed("daemon 不可达，fail-closed 拒绝");
        assert_eq!(o.exit_code, 2);
        assert!(o.stdout.is_empty());
        assert_eq!(o.stderr, "daemon 不可达，fail-closed 拒绝");
    }

    #[test]
    fn fail_closed_never_uses_exit_1() {
        let o = emit_codex_fail_closed("x");
        assert_ne!(o.exit_code, 1);
    }

    /// 任何裁决路径产出的 exit_code 必须 ∈ {0, 2}，绝不为 1（穷举不变量）。
    #[test]
    fn no_path_ever_emits_exit_1() {
        let verdicts = [
            CodexVerdict::Allow,
            CodexVerdict::Deny {
                reason: "r".to_string(),
            },
            CodexVerdict::Rewrite {
                updated_input: serde_json::json!({}),
            },
        ];
        for v in &verdicts {
            assert_ne!(emit_codex_decision(v).exit_code, 1);
        }
        assert_ne!(emit_codex_fail_closed("r").exit_code, 1);
    }

    #[test]
    fn parse_stdin_extracts_required_tool_fields() {
        let raw = r#"{"session_id":"s","turn_id":"t","cwd":"/proj","hook_event_name":"PreToolUse",
            "model":"m","permission_mode":"default","tool_name":"Bash","tool_use_id":"tu-1",
            "tool_input":{"command":"rm -rf /"}}"#;
        let inp: CodexPreToolUseInput = serde_json::from_str(raw).unwrap();
        assert_eq!(inp.tool_name, "Bash");
        assert_eq!(inp.tool_use_id, "tu-1");
        assert_eq!(inp.cwd, "/proj");
        assert_eq!(inp.tool_input["command"], "rm -rf /");
    }

    /// codex 未来加字段不能破坏解析；缺失字段走 default（不 panic、不报错）。
    #[test]
    fn parse_stdin_tolerates_unknown_and_missing_fields() {
        let raw = r#"{"tool_name":"Bash","tool_input":{},"some_future_field":123}"#;
        let inp: CodexPreToolUseInput = serde_json::from_str(raw).unwrap();
        assert_eq!(inp.tool_name, "Bash");
        assert_eq!(inp.tool_use_id, ""); // 缺失 → default
        assert_eq!(inp.cwd, "");
    }
}
