//! <https://github.com/webpack/enhanced-resolve/blob/main/test/browserField.test.js>

use std::path::PathBuf;

use crate::{Resolution, ResolveError, ResolveOptions, Resolver};

fn fixture() -> PathBuf {
    super::fixture().join("browser-module")
}

#[test]
fn ignore() {
    let f = fixture();

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec!["browser".into(), "innerBrowser1".into(), "innerBrowser2".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let data = [
        (f.clone(), "./lib/ignore", f.join("lib/ignore.js")),
        (f.clone(), "./lib/ignore.js", f.join("lib/ignore.js")),
        (f.join("lib"), "./ignore", f.join("lib/ignore.js")),
        (f.join("lib"), "./ignore.js", f.join("lib/ignore.js")),
    ];

    for (path, request, expected) in data {
        let resolution = resolver.resolve(&path, request);
        let expected = ResolveError::Ignored(expected.into());
        assert_eq!(resolution, Err(expected), "{path:?} {request}");
    }
}

#[test]
fn replace_file() {
    let f = fixture();

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec!["browser".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let data = [
        ("should replace a file 1", f.clone(), "./lib/replaced", f.join("lib/browser.js")),
        ("should replace a file 2", f.clone(), "./lib/replaced.js", f.join("lib/browser.js")),
        ("should replace a file 3", f.join("lib"), "./replaced", f.join("lib/browser.js")),
        ("should replace a file 4", f.join("lib"), "./replaced.js", f.join("lib/browser.js")),
        ("should replace a module with a file 1", f.clone(), "module-a", f.join("browser/module-a.js")),
        ("should replace a module with a file 2", f.join("lib"), "module-a", f.join("browser/module-a.js")),
        ("should replace a module with a module 1", f.clone(), "module-b", f.join("node_modules/module-c.js")),
        ("should replace a module with a module 2", f.join("lib"), "module-b", f.join("node_modules/module-c.js")),
        // TODO: resolve `innerBrowser1` field in `browser-module/pakckage.json`
        // ("should resolve in nested property 1", f.clone(), "./lib/main1.js", f.join("lib/main.js")),
        // TODO: resolve `innerBrowser2` field in `browser-module/pakckage.json`
        // ("should resolve in nested property 2", f.clone(), "./lib/main2.js", f.join("lib/browser.js")),
        ("should check only alias field properties", f.clone(), "./toString", f.join("lib/toString.js")),
    ];

    for (comment, path, request, expected) in data {
        let resolved_path = resolver.resolve(&path, request).map(Resolution::full_path);
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}
