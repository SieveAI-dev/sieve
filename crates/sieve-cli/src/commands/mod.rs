//! Week 5 CLI 子命令模块（ADR-015 / SPEC-003）。
//!
//! - `setup`：自动配置 Claude Code 环境（仅 macOS）
//! - `doctor`：诊断 Sieve 安装状态（仅 macOS）
//! - `uninstall`：干净回滚 setup 改动（仅 macOS）

pub mod doctor;
pub mod setup;
pub mod uninstall;
