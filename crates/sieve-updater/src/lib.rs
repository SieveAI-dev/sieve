//! `sieve-updater` — update check + telemetry beacon for Sieve daemon.
//!
//! Implements ADR-030 §5 (客户端实现).
//!
//! ## Design
//! - Runs as a detached `tokio::spawn` task inside `sieve-cli`.
//! - Checks `<DEFAULT_MANIFEST_URL>` every [`runner::DEFAULT_INTERVAL_SECS`]
//!   (overridable via server `next_check_after_seconds` or
//!   `SIEVE_UPDATE_URL` / `[update]` TOML section).
//! - Stores a persistent `install-id` (UUIDv4) in the platform cache dir;
//!   used as the telemetry UID (omitted if `SIEVE_NO_TELEMETRY` is set).
//! - Never panics, never takes down the daemon on failure.

pub mod cache_dir;
pub mod download;
pub mod env;
pub mod error;
pub mod install;
pub mod install_id;
pub mod manifest;
pub mod runner;
pub mod signature;
mod tls;

pub use download::download_rules;
pub use error::UpdaterError;
pub use install::install_rules;
pub use runner::{
    DEFAULT_CHANNEL, DEFAULT_INTERVAL_SECS, DEFAULT_MANIFEST_URL, DEFAULT_RULES_DIR, MAX_RULES_SIZE,
};
