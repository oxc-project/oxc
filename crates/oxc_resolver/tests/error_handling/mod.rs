use std::env;

use oxc_resolver::{JSONError, ResolveError};

use crate::resolve;

#[test]
fn broken_json() {
    let dir = env::current_dir().unwrap().join("tests/error_handling/");
    let resolved = resolve(&dir, "./broken_package_json");
    let error = ResolveError::JSONError(JSONError {
        path: dir.join("broken_package_json").join("package.json"),
        message: String::from("expected value at line 1 column 1"),
        line: 1,
        column: 1,
    });
    assert_eq!(resolved, Err(error));
}
