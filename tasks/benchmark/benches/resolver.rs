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
        (cwd.clone(), "/absolute/path"),
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
        // alias
        (cwd.clone(), "aaa"),
        (cwd.clone(), "ggg"),
        (cwd.clone(), "rrr"),
        (cwd.clone(), "@"),
        (cwd, "@@@"),
    ]
}

fn oxc_resolver() -> oxc_resolver::Resolver {
    use oxc_resolver::{AliasValue, ResolveOptions, Resolver};
    let alias_value = AliasValue::Path("./".into());
    Resolver::new(ResolveOptions {
        extensions: vec![".ts".into(), ".js".into()],
        condition_names: vec!["webpack".into()],
        alias_fields: vec![vec!["browser".into()]],
        extension_alias: vec![
            (".js".into(), vec![".ts".into(), ".js".into()]),
            (".mjs".into(), vec![".mts".into()]),
        ],
        // Real projects LOVE setting these many aliases.
        // I saw them with my own eyes.
        alias: vec![
            ("/absolute/path".into(), vec![alias_value.clone()]),
            ("aaa".into(), vec![alias_value.clone()]),
            ("bbb".into(), vec![alias_value.clone()]),
            ("ccc".into(), vec![alias_value.clone()]),
            ("ddd".into(), vec![alias_value.clone()]),
            ("eee".into(), vec![alias_value.clone()]),
            ("fff".into(), vec![alias_value.clone()]),
            ("ggg".into(), vec![alias_value.clone()]),
            ("hhh".into(), vec![alias_value.clone()]),
            ("iii".into(), vec![alias_value.clone()]),
            ("jjj".into(), vec![alias_value.clone()]),
            ("kkk".into(), vec![alias_value.clone()]),
            ("lll".into(), vec![alias_value.clone()]),
            ("mmm".into(), vec![alias_value.clone()]),
            ("nnn".into(), vec![alias_value.clone()]),
            ("ooo".into(), vec![alias_value.clone()]),
            ("ppp".into(), vec![alias_value.clone()]),
            ("qqq".into(), vec![alias_value.clone()]),
            ("rrr".into(), vec![alias_value.clone()]),
            ("sss".into(), vec![alias_value.clone()]),
            ("@".into(), vec![alias_value.clone()]),
            ("@@".into(), vec![alias_value.clone()]),
            ("@@@".into(), vec![alias_value]),
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

    group.bench_with_input(BenchmarkId::from_parameter("multi-thread"), &data, |b, data| {
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
