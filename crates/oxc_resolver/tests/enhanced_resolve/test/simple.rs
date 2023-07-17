//! <https://github.com/webpack/enhanced-resolve/blob/main/test/simple.test.js>

use std::env;

use oxc_resolver::{ResolveError, Resolver};

#[test]
fn simple() -> Result<(), ResolveError> {
    // mimic `enhanced-resolve/test/simple.test.js`
    let f = env::current_dir().unwrap().join("tests/enhanced_resolve/test/");

    let resolver = Resolver::default();

    let data = [
        ("direct", f.clone(), "../lib/index"),
        ("as directory", f.clone(), ".."),
        ("as module", f.join("../../").canonicalize().unwrap(), "./enhanced_resolve"),
    ];

    for (comment, path, request) in data {
        let resolution = resolver.resolve(&path, request)?;
        let resolved_path = resolution.path().canonicalize().unwrap();
        let expected = f.join("../lib/index.js").canonicalize().unwrap();
        assert_eq!(resolved_path, expected, "{comment} {path:?} {request}");
    }

    Ok(())
}
