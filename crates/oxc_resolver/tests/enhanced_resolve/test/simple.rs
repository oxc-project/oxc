//! <https://github.com/webpack/enhanced-resolve/blob/main/test/simple.test.js>

use std::env;

use oxc_resolver::{ResolveError, Resolver};

#[test]
fn test() -> Result<(), ResolveError> {
    // mimic `enhanced-resolve/test/simple.test.js`
    let dirname = env::current_dir().unwrap().join("tests/enhanced_resolve/test/");

    let data = [
        (dirname.clone(), "../lib/index", "direct"),
        (dirname.clone(), "..", "as directory"),
        (dirname.join("../../").canonicalize().unwrap(), "./enhanced_resolve", "as module"),
    ];

    let resolver = Resolver::default();

    for (path, request, comment) in data {
        let resolution = resolver.resolve(&path, request)?;
        let resolved_path = resolution.path().canonicalize().unwrap();
        let expected = dirname.join("../lib/index.js").canonicalize().unwrap();
        assert_eq!(resolved_path, expected, "{comment} {path:?} {request}");
    }

    Ok(())
}
