//! Hermes Agent（Nous Research）的 `pre_tool_call` shell hook 协议层。
//!
//! 与 `codex` 子命令的根本区别在于**决策载体与失败语义**：
//! - Codex 用 **exit code**（且 `exit 1 = fail-open` 的 footgun），可用 `exit 2` 强制 fail-closed。
//! - **Hermes 用 stdout JSON 的 `decision` 字段表决策，退出码完全不影响 block**；且 Hermes
//!   是 **fail-OPEN**：畸形 JSON / 非零退出 / 超时一律仅 warn，"never abort the agent loop"，
//!   且**无 strict/required 配置可改**（官方文档核实 2026-06-22）。
//!
//! 因此本模块的"fail-closed 兜底"也只能**尽力输出 block JSON**——一旦 Hermes 已超时，
//! 本进程输出什么都可能无效。系统真正的 fail-closed 不变量由 daemon 网关 `inbound_hold`
//! 保证，本 hook 仅作 UX 层（终端可见 + 拿到执行边界的确切 tool 调用）。详见
//! ADR-014 §6 勘误（2026-06-22）与 memory `project_openclaw_hermes_have_hooks`。
//!
//! 契约（官方文档 hermes-agent.nousresearch.com/docs/user-guide/features/hooks 核实）：
//! - stdin：JSON `{ tool_name, tool_input, session_id }`
//! - stdout：Hermes-canonical `{"decision":"block","reason":...}`，**亦明确接受 Claude-Code 风格
//!   `{"action":"block","message":...}`**（内部归一化）；本模块统一发 Hermes-canonical 形态。
//! - 本模块只做「裁决 → 输出契约」纯映射，检测在 daemon（经 IPC `judge_tool_call`，与 codex 共用）。
//!   拆纯函数便于单测把 Hermes 的「退出码不 block」语义在测试里锁死。

use serde::Deserialize;
use serde_json::Value;

// 判危结果是 agent-neutral 的（只表达 allow/deny/rewrite），命名沿用 codex 模块的定义以
// 复用 `judge_tool_call`，避免为换名做无谓重构。
use crate::codex::CodexVerdict;

/// Hermes 经 stdin 传入的 `pre_tool_call` JSON。
///
/// 全部 `#[serde(default)]` 且不 `deny_unknown_fields`，容忍 Hermes 未来新增字段。
/// 注意：Hermes 的 payload **不含 `cwd`**（与 codex 不同），敏感路径判定缺此上下文；
/// 不阻塞——daemon 对 `tool_input` 做全文扫描，内容型规则不依赖 cwd。
#[derive(Deserialize, Debug, Default, Clone)]
pub struct HermesPreToolCallInput {
    /// 工具名。Hermes Function Calling 标准（`<tools>`/`<tool_call>`）下由模型产生。
    #[serde(default)]
    pub tool_name: String,
    /// 工具参数对象；入站危险检测的主输入。
    #[serde(default)]
    pub tool_input: Value,
    /// 会话/任务 ID，用作 daemon 请求关联键（代替 codex 的 `tool_use_id`）。
    #[serde(default)]
    pub session_id: String,
}

/// Hermes hook 的输出。
///
/// **不带 exit_code 字段是刻意的**：Hermes 退出码对 block 无效（fail-open），调用方固定
/// `exit 0`，决策完全经 `stdout` 传达。`stderr` 仅供日志/排障，不影响 Hermes 决策。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HermesOutput {
    pub stdout: String,
    pub stderr: String,
}

/// 把 agent-neutral 裁决映射成 Hermes 的 stdout JSON 契约。
///
/// **安全语义**：仅 `Deny` / `Rewrite` 产出 `{"decision":"block"}`；`Allow` 产
/// `{"decision":"approve"}`。即便 Hermes 未识别 `approve`（文档仅明确 block 形态），
/// 未识别 = 放行（fail-open 方向，无害）。`approve` 形态待真机 dogfood 复核。
pub fn emit_hermes_decision(verdict: &CodexVerdict) -> HermesOutput {
    match verdict {
        CodexVerdict::Allow => HermesOutput {
            stdout: serde_json::json!({ "decision": "approve" }).to_string(),
            stderr: String::new(),
        },
        CodexVerdict::Deny { reason } => HermesOutput {
            stdout: serde_json::json!({ "decision": "block", "reason": reason }).to_string(),
            stderr: String::new(),
        },
        // Hermes `pre_tool_call` 文档未给出 input 改写契约（结果改写走 `transform_tool_result`，
        // 那是 post 阶段）。出站脱敏（OUT-*）由 daemon 转发前完成，不应到达 hook；保险起见
        // 遇 Rewrite 保守 block（fail-safe），避免未脱敏内容因 hook 不支持改写而漏过。
        CodexVerdict::Rewrite { .. } => HermesOutput {
            stdout: serde_json::json!({
                "decision": "block",
                "reason": "sieve: outbound redaction required but hermes pre_tool_call cannot rewrite tool_input"
            })
            .to_string(),
            stderr: String::new(),
        },
    }
}

/// fail-closed 兜底：daemon 不可达 / 解析失败 / 超时等错误路径，**尽力**输出 block JSON。
///
/// ⚠️ Hermes fail-OPEN：若错误源于已超过 Hermes 的 hook timeout，则 Hermes 可能已放行，
/// 本输出未必生效。真正的 fail-closed 由 daemon 网关 `inbound_hold` 兜底（ADR-014 §6）。
/// `reason` 同时进 `stderr` 便于排障。
pub fn emit_hermes_fail_closed(reason: &str) -> HermesOutput {
    HermesOutput {
        stdout: serde_json::json!({ "decision": "block", "reason": reason }).to_string(),
        stderr: reason.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_emits_decision_approve() {
        let o = emit_hermes_decision(&CodexVerdict::Allow);
        let v: Value = serde_json::from_str(&o.stdout).unwrap();
        assert_eq!(v["decision"], "approve");
        assert!(o.stderr.is_empty());
    }

    #[test]
    fn deny_emits_decision_block_with_reason() {
        let o = emit_hermes_decision(&CodexVerdict::Deny {
            reason: "含硬编码私钥，拒绝执行".to_string(),
        });
        let v: Value = serde_json::from_str(&o.stdout).unwrap();
        assert_eq!(v["decision"], "block");
        assert_eq!(v["reason"], "含硬编码私钥，拒绝执行");
    }

    /// Rewrite 在 Hermes 不支持改写，必须保守 block（fail-safe），绝不静默 approve。
    #[test]
    fn rewrite_falls_back_to_block() {
        let o = emit_hermes_decision(&CodexVerdict::Rewrite {
            updated_input: serde_json::json!({"command": "echo redacted"}),
        });
        let v: Value = serde_json::from_str(&o.stdout).unwrap();
        assert_eq!(v["decision"], "block", "Rewrite 不可降级为 approve");
    }

    #[test]
    fn fail_closed_emits_block() {
        let o = emit_hermes_fail_closed("daemon 不可达，fail-closed 尽力 block");
        let v: Value = serde_json::from_str(&o.stdout).unwrap();
        assert_eq!(v["decision"], "block");
        assert_eq!(o.stderr, "daemon 不可达，fail-closed 尽力 block");
    }

    /// 穷举不变量：除 Allow 外的任何裁决与兜底，stdout 的 decision 必须是 "block"，
    /// 绝不出现"未拦截却放行"。Allow 才 approve。
    #[test]
    fn only_allow_yields_approve_everything_else_blocks() {
        let approve = emit_hermes_decision(&CodexVerdict::Allow);
        assert_eq!(
            serde_json::from_str::<Value>(&approve.stdout).unwrap()["decision"],
            "approve"
        );
        let blockers = [
            emit_hermes_decision(&CodexVerdict::Deny {
                reason: "r".to_string(),
            }),
            emit_hermes_decision(&CodexVerdict::Rewrite {
                updated_input: serde_json::json!({}),
            }),
            emit_hermes_fail_closed("r"),
        ];
        for o in &blockers {
            assert_eq!(
                serde_json::from_str::<Value>(&o.stdout).unwrap()["decision"],
                "block"
            );
        }
    }

    #[test]
    fn parse_stdin_extracts_required_fields() {
        let raw = r#"{"tool_name":"terminal","tool_input":{"cmd":"rm -rf /"},"session_id":"s-1"}"#;
        let inp: HermesPreToolCallInput = serde_json::from_str(raw).unwrap();
        assert_eq!(inp.tool_name, "terminal");
        assert_eq!(inp.session_id, "s-1");
        assert_eq!(inp.tool_input["cmd"], "rm -rf /");
    }

    /// Hermes 未来加字段 / 缺字段不破坏解析（容忍未知 + default）。
    #[test]
    fn parse_stdin_tolerates_unknown_and_missing() {
        let raw = r#"{"tool_name":"terminal","tool_input":{},"future_field":1}"#;
        let inp: HermesPreToolCallInput = serde_json::from_str(raw).unwrap();
        assert_eq!(inp.tool_name, "terminal");
        assert_eq!(inp.session_id, "");
    }
}
