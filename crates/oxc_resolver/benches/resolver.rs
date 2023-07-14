//! Resolver benchmark
//!
//! ```bash
//! git switch main
//! cargo bench --bench resolver -- --save-baseline main
//! git switch -
//! cargo bench --bench resolver -- --save-baseline pr
//! critcmp
//! ```
use std::env;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_resolver::Resolver;

fn criterion_benchmark(c: &mut Criterion) {
    let cwd = env::current_dir().unwrap().join("tests/enhanced_resolve/");
    let resolver = Resolver::new();
    c.bench_with_input(BenchmarkId::new("index", ""), &cwd, |b, cwd| {
        b.iter(|| {
            assert!(resolver.resolve(cwd, "./lib/index").is_ok());
            assert!(resolver.resolve(cwd, "./").is_ok());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
