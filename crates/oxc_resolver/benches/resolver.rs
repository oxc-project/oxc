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
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

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
        // query fragment
        (cwd.join("test/fixtures"), "./main1.js#fragment?query"),
        (cwd.join("test/fixtures"), "m1/a.js?query#fragment"),
        // extensions
        (cwd.join("test/fixtures/extensions"), "./foo"),
        (cwd.join("test/fixtures/extensions/module"), "module"),
        // browserField
        (cwd.join("test/fixtures/browser-module"), "./lib/replaced"),
        (cwd.join("test/fixtures/browser-module/lib"), "./replaced"),
        // extensionAlias
        (cwd.join("test/fixtures/extension-alias"), "./index.js"),
        // scoped
        (cwd.join("test/fixtures/scoped"), "@scope/pack1"),
        (cwd.join("test/fixtures/scoped"), "@scope/pack2/lib"),
    ]
}

fn nodejs_resolver() -> nodejs_resolver::Resolver {
    use nodejs_resolver::{Options, Resolver};
    Resolver::new(Options {
        browser_field: true,
        extension_alias: vec![
            (".js".into(), vec![".ts".into(), ".js".into()]),
            (".mjs".into(), vec![".mts".into()]),
        ],
        ..Options::default()
    })
}

fn oxc_resolver() -> oxc_resolver::Resolver {
    use oxc_resolver::{ResolveOptions, Resolver};
    Resolver::new(ResolveOptions {
        alias_fields: vec!["browser".into()],
        extension_alias: vec![
            (".js".into(), vec![".ts".into(), ".js".into()]),
            (".mjs".into(), vec![".mts".into()]),
        ],
        ..ResolveOptions::default()
    })
}

fn resolver_benchmark(c: &mut Criterion) {
    let data = data();

    // Check path is valid, re-initlize so it they don't use cache
    for (path, request) in &data {
        assert!(nodejs_resolver().resolve(path, request).is_ok(), "{path:?} {request}");
        assert!(oxc_resolver().resolve(path, request).is_ok(), "{path:?} {request}");
    }

    // Bench nodejs_resolver with cache
    c.bench_with_input(BenchmarkId::new("nodejs_resolver", ""), &data, |b, data| {
        let nodejs_resolver = nodejs_resolver();
        b.iter(|| {
            for (path, request) in data {
                _ = nodejs_resolver.resolve(path, request);
            }
        });
    });

    // Bench oxc_resolver with cache
    c.bench_with_input(BenchmarkId::new("oxc_resolver", ""), &data, |b, data| {
        let oxc_resolver = oxc_resolver();
        b.iter(|| {
            for (path, request) in data {
                _ = oxc_resolver.resolve(path, request);
            }
        });
    });
}

criterion_group!(benches, resolver_benchmark);
criterion_main!(benches);
