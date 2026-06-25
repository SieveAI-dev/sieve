//! Sieve rules library
//!
//! Phase 1: vectorscan 多模式正则 + Ed25519 规则包验签。
//! Week 2: 完整规则引擎 + BIP39 checksum + placeholder 黑名单 + toml loader。
//!
//! crate 边界：**禁止任何网络 IO**（.cursorrules §3.3）。

#![deny(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]

pub mod base58check;
pub mod bip39;
pub mod critical_lock;
pub mod ed25519;
pub mod engine;
pub mod error;
pub mod loader;
pub mod manifest;
pub mod placeholder;
pub mod wordlist;

pub use engine::{
    ContentKind, Direction, LayeredEngine, MatchEngine, MatchHit, Protocol, ScanReport,
    ScanRequest, SystemEngine,
};
pub use error::{SieveRulesError, SieveRulesResult};
