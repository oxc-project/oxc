//! <https://github.com/webpack/enhanced-resolve/blob/main/test/simple.test.js>

use std::env;

use crate::Resolver;

#[test]
fn simple() {
    // mimic `enhanced-resolve/test/simple.test.js`
    let f = env::current_dir().unwrap().join("tests/enhanced_resolve/test/");

    let resolver = Resolver::default();

    let data = [
        ("direct", f.clone(), "../lib/index"),
        ("as directory", f.clone(), ".."),
        ("as module", f.join("../../").canonicalize().unwrap(), "./enhanced_resolve"),
    ];

    for (comment, path, request) in data {
        let resolved_path =
            resolver.resolve(&path, request).map(|f| f.full_path().canonicalize().unwrap());
        let expected = f.join("../lib/index.js").canonicalize().unwrap();
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}
