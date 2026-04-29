//! Sieve core library
//!
//! Phase 1: Anthropic Messages API only (PRD §6.1)。
//! UnifiedMessage 接口预留 OpenAI / Gemini variant，但仅 Anthropic 实现解析。
//!
//! crate 边界：不允许 CLI / TUI / 配置加载 (.cursorrules §3.3)。

#![forbid(unsafe_code)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]

pub mod address_guard;
pub mod detection;
pub mod error;
#[cfg(feature = "forwarder")]
pub mod forwarder;
pub mod fuzz_helpers;
pub mod pipeline;
pub mod protocol;
pub mod skill_install_guard;
pub mod sse;
pub mod tool_use_aggregator;

pub use detection::{fingerprint, Action, ContentSource, DefaultOnTimeout, Detection, Severity};
pub use error::{SieveCoreError, SieveCoreResult};
#[cfg(feature = "forwarder")]
pub use forwarder::Forwarder;
pub use protocol::unified_message::{
    ContentBlock, MessageMetadata, Role, ToolResultBlock, ToolUseBlock, UnifiedMessage,
    UpstreamProvider,
};
pub use tool_use_aggregator::{Aggregator, CompletedToolCall};
