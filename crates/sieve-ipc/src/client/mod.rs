//! IPC client helper（UDS 连接 + ndjson 帧编解码）。
//!
//! 供 sieve-cli 命令侧及测试 mock 使用。生产主路径的 GUI 是独立进程
//!（sieve-gui-macos），不使用本模块。关联：ADR-013、ADR-028。

pub mod connection;

pub use connection::{send_reload_user_rules_oneshot, IpcClient};
