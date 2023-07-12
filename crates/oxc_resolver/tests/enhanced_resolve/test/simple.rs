//! <https://github.com/webpack/enhanced-resolve/blob/main/test/simple.test.js>

use std::env;

use crate::resolve;

#[test]
fn test() {
    // mimic `enhanced-resolve/test/simple.test.js`
    let dirname = env::current_dir().unwrap().join("tests/enhanced_resolve/test/");

    let paths = vec![(dirname.clone(), "../lib/index", "direct")];

    for (path, request, comment) in paths {
        let resolved = resolve(path, request);
        let expected = Ok(dirname.join("../lib/index.js").canonicalize().expect("file exists"));
        assert_eq!(resolved, expected, "{comment}");
    }
}
