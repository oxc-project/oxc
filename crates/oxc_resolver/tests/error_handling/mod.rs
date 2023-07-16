use std::env;

use oxc_resolver::{JSONError, ResolveError, Resolver};

#[test]
fn broken_json() {
    let dir = env::current_dir().unwrap().join("tests/error_handling/");
    let resolver = Resolver::default();
    let resolution = resolver.resolve(&dir, "./broken_package_json");
    let error = ResolveError::JSONError(JSONError {
        path: dir.join("broken_package_json").join("package.json"),
        message: String::from("expected value at line 1 column 1"),
        line: 1,
        column: 1,
    });
    assert_eq!(resolution, Err(error));
}
