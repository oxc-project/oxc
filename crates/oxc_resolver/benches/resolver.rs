//! Resolver benchmark
//!
//! ```bash
//! git switch main
//! cargo bench --bench resolver -- --save-baseline main
//! git switch -
//! cargo bench --bench resolver -- --save-baseline pr
//! critcmp
//! ```

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{env, path::PathBuf};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn data() -> Vec<(PathBuf, &'static str)> {
    let cwd = env::current_dir().unwrap().join("tests/enhanced_resolve/");
    vec![
        (cwd.clone(), "./"),
        (cwd.clone(), "./lib/index"),
        (cwd.join("./test/fixtures/extensions"), "./foo"),
        (cwd.join("test/fixtures/extensions/module"), "module"),
    ]
}

fn resolver_benchmark(c: &mut Criterion) {
    let data = data();

    // bench nodejs_resolver
    {
        let resolver = nodejs_resolver::Resolver::new(nodejs_resolver::Options::default());
        // Check path is valid
        for (path, request) in &data {
            assert!(resolver.resolve(path, request).is_ok(), "{path:?} {request}");
        }
        c.bench_with_input(BenchmarkId::new("nodejs_resolver", ""), &data, |b, data| {
            b.iter(|| {
                for (path, request) in data {
                    _ = resolver.resolve(path, request);
                }
            });
        });
    }

    // bench oxc_resolver
    {
        let resolver = oxc_resolver::Resolver::default();
        // Check path is valid
        for (path, request) in &data {
            assert!(resolver.resolve(path, request).is_ok(), "{path:?} {request}");
        }
        c.bench_with_input(BenchmarkId::new("oxc_resolver", ""), &data, |b, data| {
            b.iter(|| {
                for (path, request) in data {
                    _ = resolver.resolve(path, request);
                }
            });
        });
    }
}

criterion_group!(benches, resolver_benchmark);
criterion_main!(benches);
