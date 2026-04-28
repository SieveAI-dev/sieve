//! 数据集 benchmark（PRD §10.1 Week 4 完成定义：200-500 攻击 + 50-100 benign）。
//!
//! 验证 PRD §9 #7 P99 < 20ms 硬约束。
//!
//! 跑法：
//! ```bash
//! cargo bench -p sieve-rules --bench dataset_bench
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

fn bench_data_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("bench-data");
    p
}

fn build_outbound_engine() -> VectorscanEngine {
    let path = rules_dir().join("outbound.toml");
    let rules = load_outbound_rules(&path).expect("load outbound.toml");
    VectorscanEngine::compile(rules).expect("compile outbound engine")
}

fn build_inbound_engine() -> VectorscanEngine {
    let path = rules_dir().join("inbound.toml");
    let rules = load_inbound_rules(&path).expect("load inbound.toml");
    // 过滤掉占位 pattern（IN-CR-01 / IN-CR-06），不参与 vectorscan 编译
    let filtered: Vec<_> = rules
        .into_iter()
        .filter(|r| {
            r.pattern != "__ADDRESS_GUARD_PLACEHOLDER__"
                && r.pattern != "__OPENCLAW_SKILL_GUARD_PLACEHOLDER__"
        })
        .collect();
    VectorscanEngine::compile(filtered).expect("compile inbound engine")
}

/// 读取目录下所有 .txt 文件内容，返回 (文件名, 内容) 列表。
fn read_dataset(dir: &PathBuf) -> Vec<(String, Vec<u8>)> {
    let mut samples = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("txt"))
            .map(|e| e.path())
            .collect();
        paths.sort(); // deterministic order
        for path in paths {
            if let Ok(content) = std::fs::read(&path) {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                samples.push((name, content));
            }
        }
    }
    samples
}

fn dataset_bench(c: &mut Criterion) {
    let outbound_engine = build_outbound_engine();
    let inbound_engine = build_inbound_engine();

    let attacks = read_dataset(&bench_data_dir().join("attacks"));
    let benigns = read_dataset(&bench_data_dir().join("benign"));

    assert!(!attacks.is_empty(), "attacks dataset must not be empty");
    assert!(!benigns.is_empty(), "benigns dataset must not be empty");

    // -------------------------------------------------------------------
    // 批量扫描攻击样本（吞吐量指标）
    // -------------------------------------------------------------------
    {
        let total_bytes: u64 = attacks.iter().map(|(_, c)| c.len() as u64).sum();
        let mut group = c.benchmark_group("dataset_attacks");
        group.throughput(Throughput::Bytes(total_bytes));
        group.bench_function("scan_all_attacks_outbound", |b| {
            b.iter(|| {
                let mut total = 0usize;
                for (_, content) in &attacks {
                    total += outbound_engine.scan(content).map(|h| h.len()).unwrap_or(0);
                }
                std::hint::black_box(total)
            });
        });
        group.bench_function("scan_all_attacks_inbound", |b| {
            b.iter(|| {
                let mut total = 0usize;
                for (_, content) in &attacks {
                    total += inbound_engine.scan(content).map(|h| h.len()).unwrap_or(0);
                }
                std::hint::black_box(total)
            });
        });
        group.finish();
    }

    // -------------------------------------------------------------------
    // 批量扫描 benign 样本（FP 延迟指标）
    // -------------------------------------------------------------------
    {
        let total_bytes: u64 = benigns.iter().map(|(_, c)| c.len() as u64).sum();
        let mut group = c.benchmark_group("dataset_benign");
        group.throughput(Throughput::Bytes(total_bytes));
        group.bench_function("scan_all_benign_outbound", |b| {
            b.iter(|| {
                let mut total = 0usize;
                for (_, content) in &benigns {
                    total += outbound_engine.scan(content).map(|h| h.len()).unwrap_or(0);
                }
                std::hint::black_box(total)
            });
        });
        group.bench_function("scan_all_benign_inbound", |b| {
            b.iter(|| {
                let mut total = 0usize;
                for (_, content) in &benigns {
                    total += inbound_engine.scan(content).map(|h| h.len()).unwrap_or(0);
                }
                std::hint::black_box(total)
            });
        });
        group.finish();
    }

    // -------------------------------------------------------------------
    // 单条延迟（P99 代理指标）——用前 5 个攻击样本
    // -------------------------------------------------------------------
    {
        let mut group = c.benchmark_group("single_sample_latency");
        for (i, (name, content)) in attacks.iter().take(5).enumerate() {
            group.bench_with_input(
                BenchmarkId::new("attack_outbound", format!("{i}_{name}")),
                content,
                |b, buf| {
                    b.iter(|| outbound_engine.scan(std::hint::black_box(buf)));
                },
            );
            group.bench_with_input(
                BenchmarkId::new("attack_inbound", format!("{i}_{name}")),
                content,
                |b, buf| {
                    b.iter(|| inbound_engine.scan(std::hint::black_box(buf)));
                },
            );
        }
        // benign 单条（验证 benign 不比 attack 慢）
        for (i, (name, content)) in benigns.iter().take(5).enumerate() {
            group.bench_with_input(
                BenchmarkId::new("benign_outbound", format!("{i}_{name}")),
                content,
                |b, buf| {
                    b.iter(|| outbound_engine.scan(std::hint::black_box(buf)));
                },
            );
        }
        group.finish();
    }

    // -------------------------------------------------------------------
    // 数据集统计（不是 benchmark，仅 println 方便 CI 日志查看）
    // -------------------------------------------------------------------
    println!(
        "\n[dataset_bench] attacks={} samples, benign={} samples",
        attacks.len(),
        benigns.len()
    );
}

criterion_group!(benches, dataset_bench);
criterion_main!(benches);
