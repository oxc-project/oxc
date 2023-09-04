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
    let cwd = project_root().join("crates/oxc_resolver/tests/enhanced_resolve");
    let f = cwd.join("test/fixtures");
    vec![
        (cwd.clone(), "./"),
        (cwd.clone(), "./lib/index"),
        (cwd, "/absolute/path"),
        // query fragment
        (f.clone(), "./main1.js#fragment?query"),
        (f.clone(), "m1/a.js?query#fragment"),
        // browserField
        (f.join("browser-module"), "./lib/replaced"),
        (f.join("browser-module/lib"), "./replaced"),
        // exportsField
        (f.join("exports-field"), "exports-field"),
        (f.join("exports-field"), "exports-field/dist/main.js"),
        (f.join("exports-field"), "exports-field/dist/main.js?foo"),
        (f.join("exports-field"), "exports-field/dist/main.js#foo"),
        (f.join("exports-field"), "@exports-field/core"),
        (f.join("imports-exports-wildcard"), "m/features/f.js"),
        // extensionAlias
        (f.join("extension-alias"), "./index.js"),
        (f.join("extension-alias"), "./dir2/index.mjs"),
        // extensions
        (f.join("extensions"), "./foo"),
        (f.join("extensions"), "."),
        (f.join("extensions"), "./dir"),
        (f.join("extensions"), "module/"),
        // importsField
        (f.join("imports-field"), "#imports-field"),
        (f.join("imports-exports-wildcard/node_modules/m/"), "#internal/i.js"),
        // scoped
        (f.join("scoped"), "@scope/pack1"),
        (f.join("scoped"), "@scope/pack2/lib"),
        // dashed name
        (f.clone(), "dash"),
        (f.clone(), "dash-name"),
        (f.join("node_modules/dash"), "dash"),
        (f.join("node_modules/dash"), "dash-name"),
        (f.join("node_modules/dash-name"), "dash"),
        (f.join("node_modules/dash-name"), "dash-name"),
    ]
}

fn oxc_resolver() -> oxc_resolver::Resolver {
    use oxc_resolver::{AliasValue, ResolveOptions, Resolver};
    Resolver::new(ResolveOptions {
        extensions: vec![".ts".into(), ".js".into()],
        condition_names: vec!["webpack".into()],
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

    // check validity
    for (path, request) in &data {
        assert!(oxc_resolver().resolve(path, request).is_ok(), "{path:?} {request}");
    }

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
