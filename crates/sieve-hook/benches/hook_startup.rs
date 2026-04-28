// benchmark: sieve-hook 启动时延（pending 文件不存在路径，即最快路径）。
//
// 测量从 run_check() 入口到 return 0（PendingError::NotFound fail-open）的耗时。
// 目标 < 50ms（P99）。关联：SPEC-001 §5（启动时延约束）。

use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::TempDir;
use uuid::Uuid;

fn bench_pending_not_found(c: &mut Criterion) {
    let tmp = TempDir::new().unwrap();
    let base = tmp.path().to_owned();

    c.bench_function("hook_run_check_pending_not_found", |b| {
        b.iter(|| {
            let id = Uuid::now_v7();
            // pending 文件不存在 → fail-open → exit 0，最快路径。
            let exit_code = sieve_hook_lib::run_check(id, &base);
            assert_eq!(exit_code, 0);
        });
    });
}

criterion_group!(benches, bench_pending_not_found);
criterion_main!(benches);
