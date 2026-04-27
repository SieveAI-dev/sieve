//! Sieve rules library
//!
//! Phase 1: vectorscan 多模式正则 + Ed25519 规则包验签（关联 ADR-001 / ADR-002）。
//! Week 1 仅骨架，具体引擎实现见 Week 2 起的 OUT-* / IN-* 规则集。
//!
//! crate 边界：**禁止任何网络 IO**（.cursorrules §3.3）。

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]

pub mod ed25519;
pub mod engine;
pub mod error;
pub mod manifest;

pub use error::{SieveRulesError, SieveRulesResult};
