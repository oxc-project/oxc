//! <https://github.com/webpack/enhanced-resolve/blob/main/test/browserField.test.js>

use crate::{AliasValue, ResolveError, ResolveOptions, Resolver};

#[test]
fn ignore() {
    let f = super::fixture().join("browser-module");

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec![
            vec!["browser".into()],
            vec!["innerBrowser1".into()],
            vec!["innerBrowser2".into()],
        ],
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
        let expected = ResolveError::Ignored(expected);
        assert_eq!(resolution, Err(expected), "{path:?} {request}");
    }
}

#[test]
fn replace_file() {
    let f = super::fixture().join("browser-module");

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec![
            vec!["browser".into()],
            vec!["innerBrowser1".into(), "field2".into(), "browser".into()], // not presented
            vec!["innerBrowser1".into(), "field".into(), "browser".into()],
            vec!["innerBrowser2".into(), "browser".into()],
        ],
        // Not part of enhanced-resolve. Added to make sure no interaction between these two fields.
        main_fields: vec!["browser".into()],
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
        ("should resolve in nested property 1", f.clone(), "./lib/main1.js", f.join("lib/main.js")),
        ("should resolve in nested property 2", f.clone(), "./lib/main2.js", f.join("lib/browser.js")),
        ("should check only alias field properties", f.clone(), "./toString", f.join("lib/toString.js")),
        // not part of enhanced-resolve
        ("recursion", f.clone(), "module-c", f.join("node_modules/module-c.js")),
    ];

    for (comment, path, request, expected) in data {
        let resolved_path = resolver.resolve(&path, request).map(|r| r.full_path());
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}

#[test]
fn broken() {
    let f = super::fixture();

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec![vec!["browser".into()]],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let data = [
        // The browser field string value should be ignored
        (f.clone(), "browser-module-broken", f.join("node_modules/browser-module-broken/main.js")),
    ];

    for (path, request, expected) in data {
        let resolved_path = resolver.resolve(&path, request).map(|r| r.full_path());
        assert_eq!(resolved_path, Ok(expected), "{path:?} {request}");
    }
}

#[test]
fn crypto_js() {
    let f = super::fixture();

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec![vec!["browser".into()]],
        fallback: vec![(
            "crypto".into(),
            vec![AliasValue::Path(f.join("lib.js").to_string_lossy().to_string())],
        )],
        ..ResolveOptions::default()
    });

    let resolved_path = resolver.resolve(f.join("crypto-js"), "crypto").map(|r| r.full_path());
    assert_eq!(resolved_path, Err(ResolveError::Ignored(f.join("crypto-js"))));
}
