//! 系统规则 + 用户规则（LayeredEngine）benchmark（PRD v2.0 §6.3.2 压力测试）。
//!
//! 目标：70 系统规则 + 30 用户规则，LayeredEngine overhead < 20%。
//!
//! 跑法：
//! ```bash
//! cargo bench -p sieve-rules --bench scan_with_user_rules
//! # 快速模式（CI 友好，< 5s）：
//! cargo bench -p sieve-rules --bench scan_with_user_rules -- --sample-size 10
//! ```

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use sieve_rules::engine::{LayeredEngine, MatchEngine, VectorscanEngine};
use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};
use sieve_rules::manifest::{Action, DefaultOnTimeout, RuleEntry, Severity};
use std::path::PathBuf;

fn rules_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("rules");
    p
}

/// 加载系统规则（入站 + 出站合并，过滤占位 pattern）。
fn build_system_engine() -> VectorscanEngine {
    let inbound_path = rules_dir().join("inbound.toml");
    let outbound_path = rules_dir().join("outbound.toml");

    let inbound = load_inbound_rules(&inbound_path).expect("load inbound.toml");
    let outbound = load_outbound_rules(&outbound_path).expect("load outbound.toml");

    let placeholder_patterns = [
        "__ADDRESS_GUARD_PLACEHOLDER__",
        "__OPENCLAW_SKILL_GUARD_PLACEHOLDER__",
    ];

    let mut all_rules: Vec<_> = inbound
        .into_iter()
        .chain(outbound)
        .filter(|r| !placeholder_patterns.contains(&r.pattern.as_str()))
        .collect();

    let mut seen_ids = std::collections::HashSet::new();
    all_rules.retain(|r| seen_ids.insert(r.id.clone()));

    VectorscanEngine::compile(all_rules).expect("compile system engine")
}

/// 构造 30 条 dummy 用户规则（模拟真实用户规则负载）。
///
/// 使用简单字面量 pattern（MY-DUMMY-\d+ 形式），不含复杂正则，
/// 避免 ReDoS 影响 benchmark 公正性。
fn build_user_engine() -> VectorscanEngine {
    let user_rules: Vec<RuleEntry> = (0..30)
        .map(|i| RuleEntry {
            id: format!("MY-DUMMY-{:02}", i),
            description: format!("Dummy user rule #{}", i),
            // 交替使用不同 pattern 类型：字面量 / 简单字符类
            pattern: if i % 3 == 0 {
                format!("my-dummy-pattern-{}", i)
            } else if i % 3 == 1 {
                format!("DUMMY[_-]SECRET[_-]{}", i)
            } else {
                format!("user_rule_{:04}", i)
            },
            severity: Severity::High,
            action: Action::Warn,
            entropy_min: None,
            keywords: vec![format!("dummy-{}", i)],
            allowlist_regexes: vec![],
            allowlist_stopwords: vec![],
            disposition: None,
            timeout_seconds: None,
            default_on_timeout: DefaultOnTimeout::Allow,
        })
        .collect();

    VectorscanEngine::compile(user_rules).expect("compile user engine")
}

/// 构造 5KB 测试输入（与 scan_70_rules.rs 保持一致，便于对比 overhead）。
fn make_5kb_input() -> Vec<u8> {
    let normal_text = r#"
Hello! I'm working on a Rust project and need some help with async programming.
The function needs to handle multiple concurrent requests efficiently.
Here's the current implementation using tokio:

async fn handle_request(req: Request) -> Result<Response, Error> {
    let data = fetch_data(&req.id).await?;
    let processed = process(data).await?;
    Ok(Response::new(processed))
}

I'm seeing some issues with error propagation when the upstream service is slow.
The timeout is set to 30 seconds but sometimes the request hangs indefinitely.

Let me look at the configuration file to understand the current settings:
- max_connections: 100
- timeout_ms: 30000
- retry_count: 3
- backoff_base_ms: 500

The deployment is on a Linux server with 8 cores and 16GB RAM.
We're using Docker containers with resource limits set appropriately.

For the database connection, we're using PostgreSQL with connection pooling.
The pool size is currently set to 20 connections per pod.

The API endpoint returns JSON data which we parse and transform.
Error handling follows the standard pattern with proper logging.

We need to ensure all sensitive data is properly redacted before logging.
The configuration should not contain any hardcoded credentials.

Let me check the Cargo.toml for current dependencies:
[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
tracing = "0.1"
thiserror = "1"

The build passes clippy with -D warnings and all tests are green.
We should also check the integration tests are covering the edge cases.
    "#;

    let tool_context = r#"
<tool_call>
  <name>Bash</name>
  <parameters>
    <command>cargo test --workspace --locked 2>&1 | tail -20</command>
  </parameters>
</tool_call>

Running tests for sieve-core...
test sse_parser::tests::handles_partial_chunk ... ok
test sse_parser::tests::handles_multi_event ... ok
test unified_message::tests::roundtrip ... ok

All tests passed. Build is clean.
    "#;

    let trigger_sample = "Note: phrase abandon able about above absent absorb abstract sample\n";

    let combined = format!("{normal_text}{tool_context}{trigger_sample}");
    let mut input = combined.into_bytes();
    while input.len() < 5 * 1024 {
        input.extend_from_slice(b"padding text for benchmark size requirements. ");
    }
    input.truncate(5 * 1024);
    input
}

fn bench_layered_engine(c: &mut Criterion) {
    let system = build_system_engine();
    let user = build_user_engine();

    let system_rule_count = system.rule_count();
    let user_rule_count = user.rule_count();

    // 系统引擎单独跑（baseline，用于对比 overhead）
    let system_only_engine = build_system_engine();

    // LayeredEngine = 系统引擎 + 用户引擎
    let layered: LayeredEngine<VectorscanEngine, VectorscanEngine> =
        LayeredEngine::new(system, Some(user));

    let input = make_5kb_input();

    let mut group = c.benchmark_group("scan_layered_engine");
    group.throughput(Throughput::Bytes(input.len() as u64));

    // baseline：仅系统规则（与 scan_70_rules bench 保持一致）
    group.bench_function("system_only_baseline", |b| {
        b.iter(|| system_only_engine.scan(&input).unwrap());
    });

    // 压力测试：LayeredEngine（70 系统 + 30 用户规则）
    group.bench_function(
        format!(
            "layered_{sys}sys_{usr}usr",
            sys = system_rule_count,
            usr = user_rule_count
        ),
        |b| {
            b.iter(|| layered.scan(&input).unwrap());
        },
    );

    group.finish();
}

criterion_group!(benches, bench_layered_engine);
criterion_main!(benches);
