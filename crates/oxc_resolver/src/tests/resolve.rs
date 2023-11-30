//! <https://github.com/webpack/enhanced-resolve/blob/main/test/resolve.test.js>

use crate::{ResolveOptions, Resolver};

#[test]
fn resolve() {
    let f = super::fixture();

    let resolver = Resolver::default();

    let main1_js_path = f.join("main1.js").to_string_lossy().to_string();

    #[rustfmt::skip]
    let pass = [
        ("absolute path", f.clone(), main1_js_path.as_str(), f.join("main1.js")),
        ("file with .js", f.clone(), "./main1.js", f.join("main1.js")),
        ("file without extension", f.clone(), "./main1", f.join("main1.js")),
        ("another file with .js", f.clone(), "./a.js", f.join("a.js")),
        ("another file without extension", f.clone(), "./a", f.join("a.js")),
        ("file in module with .js", f.clone(), "m1/a.js", f.join("node_modules/m1/a.js")),
        ("file in module without extension", f.clone(), "m1/a", f.join("node_modules/m1/a.js")),
        ("another file in module without extension", f.clone(), "complexm/step1", f.join("node_modules/complexm/step1.js")),
        ("from submodule to file in sibling module", f.join("node_modules/complexm"), "m2/b.js", f.join("node_modules/m2/b.js")),
        ("from nested directory to overwritten file in module", f.join("multiple_modules"), "m1/a.js", f.join("multiple_modules/node_modules/m1/a.js")),
        ("from nested directory to not overwritten file in module", f.join("multiple_modules"), "m1/b.js", f.join("node_modules/m1/b.js")),
        ("file with query", f.clone(), "./main1.js?query", f.join("main1.js?query")),
        ("file with fragment", f.clone(), "./main1.js#fragment", f.join("main1.js#fragment")),
        ("file with fragment and query", f.clone(), "./main1.js#fragment?query", f.join("main1.js#fragment?query")),
        ("file with query and fragment", f.clone(), "./main1.js?#fragment", f.join("main1.js?#fragment")),

        ("file with query (unicode)", f.clone(), "./测试.js?query", f.join("测试.js?query")),
        ("file with fragment (unicode)", f.clone(), "./测试.js#fragment", f.join("测试.js#fragment")),
        ("file with fragment and query (unicode)", f.clone(), "./测试.js#fragment?query", f.join("测试.js#fragment?query")),
        ("file with query and fragment (unicode)", f.clone(), "./测试.js?#fragment", f.join("测试.js?#fragment")),

        ("file in module with query", f.clone(), "m1/a?query", f.join("node_modules/m1/a.js?query")),
        ("file in module with fragment", f.clone(), "m1/a#fragment", f.join("node_modules/m1/a.js#fragment")),
        ("file in module with fragment and query", f.clone(), "m1/a#fragment?query", f.join("node_modules/m1/a.js#fragment?query")),
        ("file in module with query and fragment", f.clone(), "m1/a?#fragment", f.join("node_modules/m1/a.js?#fragment")),
        ("file in module with query and fragment", f.clone(), "m1/a?#fragment", f.join("node_modules/m1/a.js?#fragment")),
        ("differ between directory and file, resolve file", f.clone(), "./dirOrFile", f.join("dirOrFile.js")),
        ("differ between directory and file, resolve directory", f.clone(), "./dirOrFile/", f.join("dirOrFile/index.js")),
        ("find node_modules outside of node_modules", f.join("browser-module/node_modules"), "m1/a", f.join("node_modules/m1/a.js")),
        ("don't crash on main field pointing to self", f.clone(), "./main-field-self", f.join("./main-field-self/index.js")),
        ("don't crash on main field pointing to self (2)", f.clone(), "./main-field-self2", f.join("./main-field-self2/index.js")),
        // enhanced-resolve has `#` prepended with a `\0`, they are removed from the
        // following 3 expected test results.
        // See https://github.com/webpack/enhanced-resolve#escaping
        ("handle fragment edge case (no fragment)", f.clone(), "./no#fragment/#/#", f.join("no#fragment/#/#.js")),
        ("handle fragment edge case (fragment)", f.clone(), "./no#fragment/#/", f.join("no.js#fragment/#/")),
        ("handle fragment escaping", f.clone(), "./no\0#fragment/\0#/\0##fragment", f.join("no#fragment/#/#.js#fragment")),

    ];

    for (comment, path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(|r| r.full_path());
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}

#[test]
fn issue238_resolve() {
    let f = super::fixture().join("issue-238");
    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into(), ".jsx".into(), ".ts".into(), ".tsx".into()],
        modules: vec!["src/a".into(), "src/b".into(), "src/common".into(), "node_modules".into()],
        ..ResolveOptions::default()
    });
    let resolved_path =
        resolver.resolve(f.join("src/common"), "config/myObjectFile").map(|r| r.full_path());
    assert_eq!(resolved_path, Ok(f.join("src/common/config/myObjectFile.js")),);
}

#[test]
fn prefer_relative() {
    let f = super::fixture();

    let resolver =
        Resolver::new(ResolveOptions { prefer_relative: true, ..ResolveOptions::default() });

    #[rustfmt::skip]
    let pass = [
        ("should correctly resolve with preferRelative 1", "main1.js", f.join("main1.js")),
        ("should correctly resolve with preferRelative 2", "m1/a.js", f.join("node_modules/m1/a.js")),
    ];

    for (comment, request, expected) in pass {
        let resolved_path = resolver.resolve(&f, request).map(|r| r.full_path());
        assert_eq!(resolved_path, Ok(expected), "{comment} {request}");
    }
}

#[test]
fn resolve_to_context() {
    let f = super::fixture();
    let resolver =
        Resolver::new(ResolveOptions { resolve_to_context: true, ..ResolveOptions::default() });

    #[rustfmt::skip]
    let data = [
        ("context for fixtures", f.clone(), "./", f.clone()),
        ("context for fixtures/lib", f.clone(), "./lib", f.join("lib")),
        ("context for fixtures with ..", f.clone(), "./lib/../../fixtures/./lib/..", f.clone()),
        ("context for fixtures with query", f.clone(), "./?query", f.clone().with_file_name("fixtures?query")),
    ];

    for (comment, path, request, expected) in data {
        let resolved_path = resolver.resolve(&path, request).map(|r| r.full_path());
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}
