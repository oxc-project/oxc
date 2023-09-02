#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::path::PathBuf;

use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_tasks_common::project_root;

use rayon::prelude::*;

fn data() -> Vec<(PathBuf, &'static str)> {
    let cwd = project_root().join("crates/oxc_resolver/tests/enhanced_resolve/");
    vec![
        (cwd.clone(), "./"),
        (cwd.clone(), "./lib/index"),
        // query fragment
        (cwd.join("test/fixtures"), "./main1.js#fragment?query"),
        (cwd.join("test/fixtures"), "m1/a.js?query#fragment"),
        // extensions
        (cwd.join("test/fixtures/extensions"), "./foo"),
        (cwd.join("test/fixtures/extensions/module"), "module/"),
        // browserField
        (cwd.join("test/fixtures/browser-module"), "./lib/replaced"),
        (cwd.join("test/fixtures/browser-module/lib"), "./replaced"),
        // extensionAlias
        (cwd.join("test/fixtures/extension-alias"), "./index.js"),
        // scoped
        (cwd.join("test/fixtures/scoped"), "@scope/pack1"),
        (cwd.join("test/fixtures/scoped"), "@scope/pack2/lib"),
        // alias
        (cwd.clone(), "/absolute/path"),
    ]
}

fn oxc_resolver() -> oxc_resolver::Resolver {
    use oxc_resolver::{AliasValue, ResolveOptions, Resolver};
    Resolver::new(ResolveOptions {
        alias: vec![("/absolute/path".into(), vec![AliasValue::Path("./".into())])],
        alias_fields: vec![vec!["browser".into()]],
        extension_alias: vec![
            (".js".into(), vec![".ts".into(), ".js".into()]),
            (".mjs".into(), vec![".mts".into()]),
        ],
        ..ResolveOptions::default()
    })
}

fn bench_resolver(c: &mut Criterion) {
    let data = data();

    let mut group = c.benchmark_group("resolver");

    group.bench_with_input(BenchmarkId::from_parameter("single-thread"), &data, |b, data| {
        let oxc_resolver = oxc_resolver();
        b.iter(|| {
            for (path, request) in data {
                _ = oxc_resolver.resolve(path, request);
            }
        });
    });

    group.bench_with_input(BenchmarkId::from_parameter("single-thread"), &data, |b, data| {
        let oxc_resolver = oxc_resolver();
        b.iter(|| {
            data.par_iter().for_each(|(path, request)| {
                _ = oxc_resolver.resolve(path, request);
            });
        });
    });
}

criterion_group!(resolver, bench_resolver);
criterion_main!(resolver);
