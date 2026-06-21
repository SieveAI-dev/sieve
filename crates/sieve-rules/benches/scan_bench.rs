use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use sieve_rules::engine::{MatchEngine, VectorscanEngine};
use sieve_rules::manifest::{Action, DefaultOnTimeout, RuleEntry, Severity};

fn build_test_engine() -> VectorscanEngine {
    let rules = vec![RuleEntry {
        id: "OUT-01".to_string(),
        description: "Anthropic API key".to_string(),
        pattern: r"sk-ant-api03-[a-zA-Z0-9_\-]{93}AA".to_string(),
        severity: Severity::Critical,
        action: Action::Block,
        entropy_min: None,
        keywords: vec!["sk-ant-api03".to_string()],
        allowlist_regexes: vec![],
        allowlist_stopwords: vec![],
        disposition: None,
        fail_closed: None,
        timeout_seconds: None,
        default_on_timeout: DefaultOnTimeout::Block,
    }];
    VectorscanEngine::compile(rules).unwrap()
}

fn bench_scan_sizes(c: &mut Criterion) {
    let engine = build_test_engine();
    let mut group = c.benchmark_group("vectorscan_scan");

    for size in [1024usize, 100 * 1024, 1024 * 1024] {
        let buffer = vec![b'x'; size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(format!("size_{}", size), &buffer, |b, buf| {
            b.iter(|| engine.scan(buf).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, bench_scan_sizes);
criterion_main!(benches);
