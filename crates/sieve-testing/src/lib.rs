//! Sieve 共享集成测试 harness。
//!
//! 把分散在 `sieve-cli/tests/*` 里重复的 mock 上游 / daemon 进程管理 /
//! 原始 HTTP client / CLI 驱动 helper 抽取为可复用库，供 dogfood 自动化基建复用。
//!
//! 这些 helper lift 自 `crates/sieve-cli/tests/`：
//! - `outbound_block.rs`：`find_free_port` / `workspace_root` / `sieve_binary` /
//!   `spawn_mock_upstream` / `DaemonGuard` / `spawn_sieve_daemon*`
//! - `inbound_block.rs`：`spawn_mock_sse_upstream` / `decode_chunked` / `fetch_response_body_raw`
//! - `content_type_matrix.rs`：Anthropic / OpenAI 的 SSE 与 JSON 响应生成器
//!
//! .cursorrules §3.2：测试代码允许使用 `.unwrap()`。本 crate 是测试 harness，
//! 内部沿用 `.unwrap()`；面向调用方的失败点尽量在 doc 中标注 panic 条件。
//!
//! # 用法概览
//!
//! ```no_run
//! # async fn demo() {
//! use sieve_testing::upstream::{responses, spawn_mock_upstream};
//! use sieve_testing::daemon::{spawn_daemon, DaemonConfig};
//! use sieve_testing::http::http_post;
//!
//! // 1. 起一个 mock 上游，返回 Anthropic JSON "ok"
//! let mock = spawn_mock_upstream(|_req| async {
//!     responses::anthropic_json_response("ok")
//! })
//! .await;
//!
//! // 2. 起真实 daemon 指向 mock 上游
//! let guard = spawn_daemon(DaemonConfig {
//!     upstream_url: mock.url(),
//!     ..Default::default()
//! });
//!
//! // 3. 发请求，断言透传
//! let (status, _headers, body) = http_post(
//!     &guard.base_url(),
//!     "/v1/messages",
//!     &[("content-type", "application/json")],
//!     br#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"hi"}]}"#,
//! );
//! assert_eq!(status, 200);
//! assert!(String::from_utf8_lossy(&body).contains("ok"));
//! # }
//! ```

#![allow(clippy::missing_panics_doc)]

pub mod cli;
pub mod daemon;
pub mod http;
pub mod paths;
pub mod upstream;
