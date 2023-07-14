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
    let resolver = Resolver::default();

    let data = [
        (cwd.clone(), "./"),
        (cwd.clone(), "./lib/index"),
        (cwd.join("./test/fixtures/extensions"), "./foo"),
        (cwd.join("test/fixtures/extensions/module"), "module"),
    ];

    // Check path is valid
    for (path, request) in &data {
        assert!(resolver.resolve(path, request).is_ok(), "{path:?} {request}");
    }

    c.bench_with_input(BenchmarkId::new("resolve", ""), &data, |b, data| {
        b.iter(|| {
            for (path, request) in data {
                _ = resolver.resolve(path, request);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
