//! sieve-testing harness 自验证：证明整条链路（mock 上游 → 真实 daemon → HTTP client）通。
//!
//! 需要 sieve 二进制存在：先 `cargo build -p sieve-cli`（或 `--release`），再
//! `cargo test -p sieve-testing`。daemon 会 fallback 到 debug 二进制。
//!
//! .cursorrules §3.2：测试代码允许 `.unwrap()`。

use sieve_testing::daemon::{spawn_daemon, DaemonConfig};
use sieve_testing::http::http_post;
use sieve_testing::upstream::{responses, spawn_mock_upstream};

/// mock 上游返回 Anthropic JSON "ok" → daemon 透传 benign 请求 → 客户端收到 200 + body 含 "ok"。
#[tokio::test]
async fn harness_end_to_end_passthrough() {
    // 1. mock 上游：benign 响应 Anthropic JSON "ok-from-upstream"
    let mock = spawn_mock_upstream(|_req| async {
        responses::anthropic_json_response("ok-from-upstream")
    })
    .await;

    // 2. 真实 daemon 指向 mock 上游（dry_run=false，自动隔离 SIEVE_HOME）
    let Some(guard) = spawn_daemon(DaemonConfig::new(mock.url())) else {
        eprintln!("SKIP harness_end_to_end_passthrough: 规则文件不存在（需安装签名规则包），跳过");
        return;
    };

    // 3. 发 benign /v1/messages（无 Critical 命中 → 应透传）
    let base = guard.base_url();
    let body = br#"{"model":"claude-sonnet-4-5","max_tokens":16,"messages":[{"role":"user","content":"hello world, tell me a joke"}]}"#;
    let (status, _headers, resp_body) = tokio::task::spawn_blocking(move || {
        http_post(
            &base,
            "/v1/messages",
            &[("content-type", "application/json")],
            body,
        )
    })
    .await
    .unwrap();

    // 4. 断言透传：daemon 把请求转发给 mock，mock 200 + body 含 "ok"
    assert_eq!(status, 200, "benign 请求应透传，daemon 返回上游 200");
    let body_str = String::from_utf8_lossy(&resp_body);
    assert!(
        body_str.contains("ok-from-upstream"),
        "响应 body 应含 mock 上游内容 'ok-from-upstream'，实际: {body_str}"
    );
}
