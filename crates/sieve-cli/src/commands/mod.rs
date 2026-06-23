//! Week 5 CLI 子命令模块（ADR-015 / SPEC-003）。
//!
//! - `setup`：自动配置 Claude Code 环境（仅 macOS）
//! - `doctor`：诊断 Sieve 安装状态（仅 macOS）
//! - `uninstall`：干净回滚 setup 改动（仅 macOS）
//!
//! ADR-028 新增：
//! - `decisions`：headless decision CLI（TODO-4）
//! - `audit`：unix-pipeable 审计查询（TODO-5）
//!
//! 加密审计 / 本地用量核算新增：
//! - `audit_keys`：full 档加密审计密钥生命周期（keygen / rotate-key / decrypt，可选特性）
//! - `usage`：本地 token 用量与超额计费查询（永不上传，可选特性）

pub mod audit;
#[cfg(feature = "audit-crypto")]
pub mod audit_keys;
pub mod decisions;
pub mod doctor;
pub mod rules;
pub mod setup;
pub mod uninstall;
#[cfg(feature = "usage")]
pub mod usage;
