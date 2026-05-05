//! Week 5 CLI 子命令模块（ADR-015 / SPEC-003）。
//!
//! - `setup`：自动配置 Claude Code 环境（仅 macOS）
//! - `doctor`：诊断 Sieve 安装状态（仅 macOS）
//! - `uninstall`：干净回滚 setup 改动（仅 macOS）
//!
//! ADR-028 新增：
//! - `decisions`：headless decision CLI（TODO-4）
//! - `audit`：unix-pipeable 审计查询（TODO-5）

pub mod audit;
pub mod decisions;
pub mod doctor;
pub mod rules;
pub mod setup;
pub mod uninstall;
