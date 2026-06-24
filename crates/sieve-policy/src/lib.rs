//! Sieve policy crate（Phase A）。
//!
//! 提供用户规则系统（§5.5）和灰名单管理（§5.4.2）的核心能力：
//!
//! - [`loader`]：加载 `~/.sieve/rules/user.toml`，含文件系统安全校验
//! - [`lint`]：11 类用户规则安全约束校验
//! - [`engine`]：[`engine::UserEngine`]——包装 vectorscan，hits 携带 `user:` 前缀
//! - [`graylist`]：灰名单 add / lookup / remove，含 Critical 锁
//! - [`error`]：[`error::PolicyError`] 错误类型
//!
//! # 依赖边界
//!
//! 只依赖 `sieve-rules`；**禁止依赖** `sieve-cli` / `sieve-core` / `sieve-ipc`
//! （crate 边界约束，CLAUDE.md §五个 Crate）。

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod engine;
pub mod error;
pub mod graylist;
pub mod lint;
pub mod loader;

pub use error::{PolicyError, PolicyResult};
