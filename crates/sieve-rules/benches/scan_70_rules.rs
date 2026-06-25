//! 70 条系统规则全量扫描 benchmark（独立测试 baseline）。
//!
//! 目标：5KB 输入 P99 < 1ms。
//!
//! 跑法：
//! ```bash
//! cargo bench -p sieve-rules --bench scan_70_rules
//! # 快速模式（CI 友好，< 5s）：
//! cargo bench -p sieve-rules --bench scan_70_rules -- --sample-size 10
//! ```

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sieve_rules::engine::{MatchEngine, VectorscanEngine};
use sieve_rules::loader::{load_inbound_rules, load_outbound_rules};
use std::path::PathBuf;

fn rules_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("rules");
    p
}

/// 加载并合并入站 + 出站系统规则，过滤掉占位 pattern。
fn build_system_engine() -> VectorscanEngine {
    let inbound_path = rules_dir().join("inbound.toml");
    let outbound_path = rules_dir().join("outbound.toml");

    let inbound = load_inbound_rules(&inbound_path).expect("load inbound.toml");
    let outbound = load_outbound_rules(&outbound_path).expect("load outbound.toml");

    // 过滤占位 pattern（IN-CR-01 / IN-CR-06 等由 Hook 层处理，不进 vectorscan）
    let placeholder_patterns = [
        "__ADDRESS_GUARD_PLACEHOLDER__",
        "__OPENCLAW_SKILL_GUARD_PLACEHOLDER__",
    ];

    let mut all_rules: Vec<_> = inbound
        .into_iter()
        .chain(outbound)
        .filter(|r| !placeholder_patterns.contains(&r.pattern.as_str()))
        .collect();

    // 去重（若 inbound + outbound 中有相同 ID，只保留第一条）
    let mut seen_ids = std::collections::HashSet::new();
    all_rules.retain(|r| seen_ids.insert(r.id.clone()));

    VectorscanEngine::compile(all_rules).expect("compile system engine")
}

/// 构造 5KB 测试输入：普通对话文本 + 典型 Claude Code 输入 + 少量触发词。
///
/// 混入 1 条会命中规则的样本，验证引擎在真实条件下的 P99 延迟。
fn make_5kb_input() -> Vec<u8> {
    // 正常对话文本（不命中任何规则）
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

    // 典型 Claude Code 工具调用上下文（不命中）
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

    // 1 条会命中入站规则的样本（IN-GEN 类，包含 mnemonic 相关词）
    // 使用 keywords 命中 BIP39 相关词，不含真实助记词
    let trigger_sample = "Note: phrase abandon able about above absent absorb abstract sample\n";

    let combined = format!("{normal_text}{tool_context}{trigger_sample}");

    // 确保达到 5KB
    let mut input = combined.into_bytes();
    while input.len() < 5 * 1024 {
        input.extend_from_slice(b"padding text for benchmark size requirements. ");
    }
    input.truncate(5 * 1024);
    input
}

fn bench_scan_70_rules(c: &mut Criterion) {
    let engine = build_system_engine();
    let input = make_5kb_input();

    let rule_count = engine.rule_count();
    let mut group = c.benchmark_group("scan_system_rules");
    group.throughput(Throughput::Bytes(input.len() as u64));

    // 主 benchmark：70 条系统规则 × 5KB 输入
    group.bench_with_input(
        BenchmarkId::new("70_rules_5kb", rule_count),
        &input,
        |b, buf| {
            b.iter(|| engine.scan(buf).unwrap());
        },
    );

    group.finish();
}

criterion_group!(benches, bench_scan_70_rules);
criterion_main!(benches);
