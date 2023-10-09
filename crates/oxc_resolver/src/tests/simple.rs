//! <https://github.com/webpack/enhanced-resolve/blob/main/test/simple.test.js>

use std::env;

use crate::Resolver;

#[test]
fn simple() {
    // mimic `enhanced-resolve/test/simple.test.js`
    let dirname = env::current_dir().unwrap().join("fixtures");
    let f = dirname.join("enhanced_resolve/test");

    let resolver = Resolver::default();

    let data = [
        ("direct", f.clone(), "../lib/index"),
        ("as directory", f, ".."),
        ("as module", dirname.clone(), "./enhanced_resolve"),
    ];

    for (comment, path, request) in data {
        let resolved_path = resolver.resolve(&path, request).map(|f| f.full_path());
        let expected = dirname.join("enhanced_resolve/lib/index.js");
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}

#[test]
fn dashed_name() {
    let f = super::fixture();

    let resolver = Resolver::default();

    let data = [
        (f.clone(), "dash", f.join("node_modules/dash/index.js")),
        (f.clone(), "dash-name", f.join("node_modules/dash-name/index.js")),
        (f.join("node_modules/dash"), "dash", f.join("node_modules/dash/index.js")),
        (f.join("node_modules/dash"), "dash-name", f.join("node_modules/dash-name/index.js")),
        (f.join("node_modules/dash-name"), "dash", f.join("node_modules/dash/index.js")),
        (f.join("node_modules/dash-name"), "dash-name", f.join("node_modules/dash-name/index.js")),
    ];

    for (path, request, expected) in data {
        let resolved_path = resolver.resolve(&path, request).map(|f| f.full_path());
        assert_eq!(resolved_path, Ok(expected), "{path:?} {request}");
    }
}
