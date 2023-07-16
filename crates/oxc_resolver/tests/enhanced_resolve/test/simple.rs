//! <https://github.com/webpack/enhanced-resolve/blob/main/test/simple.test.js>

use std::env;

use oxc_resolver::{ResolveError, Resolver};

#[test]
fn simple() -> Result<(), ResolveError> {
    let resolver = Resolver::default();

    // mimic `enhanced-resolve/test/simple.test.js`
    let dirname = env::current_dir().unwrap().join("tests/enhanced_resolve/test/");

    let data = [
        ("direct", dirname.clone(), "../lib/index"),
        ("as directory", dirname.clone(), ".."),
        ("as module", dirname.join("../../").canonicalize().unwrap(), "./enhanced_resolve"),
    ];

    for (comment, path, request) in data {
        let resolution = resolver.resolve(&path, request)?;
        let resolved_path = resolution.path().canonicalize().unwrap();
        let expected = dirname.join("../lib/index.js").canonicalize().unwrap();
        assert_eq!(resolved_path, expected, "{comment} {path:?} {request}");
    }

    Ok(())
}
