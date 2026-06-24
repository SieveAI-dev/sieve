//! 入站 Hook 类路径（HookTerminal disposition）。
//!
//! 命中 IN-CR-02~04、IN-GEN-01~03 等 HookTerminal 规则时，写入 IPC pending 文件，
//! **不修改 SSE 流**——流由调用方（daemon）原样转发给客户端。
//! sieve-hook 二进制会在 PreToolUse 阶段读取 pending 文件并在 TTY 拦截。
//!
//! 关联：Hook 路径、SPEC-001（pending 文件写入规约）。

use sieve_ipc::{paths::sieve_home, pending_file::write_pending, DecisionRequest};
use thiserror::Error;
use uuid::Uuid;

/// Hook 路径专用错误。
#[derive(Debug, Error)]
pub enum HookError {
    /// IPC 操作失败（目录创建 / 文件写入 / 锁获取）。
    #[error("IPC error: {0}")]
    Ipc(#[from] sieve_ipc::IpcError),
}

/// 写入 IPC pending 文件，通知 sieve-hook 在 PreToolUse 阶段拦截。
///
/// # 行为
/// - 在 `~/.sieve/pending/<request_id>.json`（或 `$SIEVE_HOME`）写入 [`DecisionRequest`]；
/// - **不修改 SSE 流**——调用方负责原样转发；
/// - 返回 `Ok(())` 表示文件已写入，daemon 可继续转发。
///
/// # 错误
/// 目录创建或文件写入失败时返回 [`HookError::Ipc`]。
///
/// 关联：Hook 路径、SPEC-001 §3.1。
pub fn write_hook_pending(request_id: Uuid, req: &DecisionRequest) -> Result<(), HookError> {
    let _ = request_id; // request_id 已包含在 req.request_id 中，此参数保留供调用侧校验
    let base = sieve_home()?;
    write_pending(req, &base)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use sieve_ipc::{
        pending_file::read_pending,
        protocol::{DefaultOnTimeout, DetectionPayload, Disposition, Severity},
    };

    fn make_request(id: Uuid) -> DecisionRequest {
        DecisionRequest {
            request_id: id,
            created_at: Utc::now(),
            timeout_seconds: 30,
            default_on_timeout: DefaultOnTimeout::Block,
            detections: vec![DetectionPayload {
                rule_id: "IN-CR-02".to_owned(),
                severity: Severity::Critical,
                disposition: Disposition::HookTerminal,
                title: "危险 shell 命令".to_owned(),
                one_line_summary: "检测到 rm -rf 命令".to_owned(),
                details: serde_json::json!({ "command": "rm -rf /tmp" }),
                recommendation: None,
            }],
            source_agent: sieve_ipc::SourceAgent::Unknown,
            origin_chain: vec![],
            source_channel: None,
            explicit_chain_depth: None,
            allow_remember: false,
        }
    }

    #[test]
    fn write_and_read_pending_file() {
        // 使用独立 tmpdir 直接调用底层 write_pending / read_pending，
        // 避免并发测试互相污染 SIEVE_HOME 全局变量。
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        let id = Uuid::now_v7();
        let req = make_request(id);

        // 直接写入指定 base 目录
        sieve_ipc::pending_file::write_pending(&req, base).unwrap();

        // 验证文件内容正确
        let read_back = read_pending(id, base).unwrap();
        assert_eq!(read_back.request_id, id);
        assert_eq!(read_back.detections.len(), 1);
        assert_eq!(read_back.detections[0].rule_id, "IN-CR-02");
    }

    #[test]
    fn write_hook_pending_idempotent_on_same_id() {
        // 使用独立 tmpdir 避免污染 SIEVE_HOME 全局变量（并发测试安全）
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        let id = Uuid::now_v7();
        let req = make_request(id);

        // 写两次不应 panic
        sieve_ipc::pending_file::write_pending(&req, base).unwrap();
        sieve_ipc::pending_file::write_pending(&req, base).unwrap();

        // 最终文件可正常读取
        let read_back = read_pending(id, base).unwrap();
        assert_eq!(read_back.request_id, id);
    }
}
