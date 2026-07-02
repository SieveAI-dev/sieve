//! Week 5 CLI 子命令模块（关联 SPEC-003）。
//!
//! - `setup`：自动配置 Claude Code 环境（仅 macOS）
//! - `doctor`：诊断 Sieve 安装状态（仅 macOS）
//! - `uninstall`：干净回滚 setup 改动（仅 macOS）
//!
//! headless 决策接口新增：
//! - `decisions`：headless decision CLI
//! - `audit`：unix-pipeable 审计查询
//!
//! 加密审计 / 本地用量核算新增：
//! - `audit_keys`：full 档加密审计密钥生命周期（keygen / rotate-key / decrypt，可选特性）
//! - `usage`：本地 token 用量与超额计费查询（永不上传，可选特性）

/// `SIEVE_SKIP_LAUNCHCTL=1`：仅集成测试用，跳过一切**变更类** launchctl 调用
/// （load / unload / bootstrap / bootout）。
///
/// launchd 会话按 UID 归属、不随 `$HOME` 走：集成测试用临时 HOME 跑真实
/// `launchctl load` 时，KeepAlive 服务会被注册进**真实用户会话**——测试结束后
/// 泄漏 daemon 被 kill 即复活，并以空规则集直通占用真实 IPC socket 与代理端口。
/// 只读调用（`launchctl list` / `launchctl print`）不受影响。
/// 生产路径**不应**设置此变量（设置后 daemon 不会注册 launchd 自启）。
pub(crate) fn launchctl_mutations_skipped() -> bool {
    std::env::var("SIEVE_SKIP_LAUNCHCTL").as_deref() == Ok("1")
}

pub mod audit;
#[cfg(feature = "audit-crypto")]
pub mod audit_keys;
pub mod control;
pub mod decisions;
pub mod doctor;
pub mod ipc_client;
pub mod lifecycle;
pub mod rules;
pub mod setup;
pub mod uninstall;
#[cfg(feature = "usage")]
pub mod usage;
