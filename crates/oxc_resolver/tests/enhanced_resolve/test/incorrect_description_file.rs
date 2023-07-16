//! <https://github.com/webpack/enhanced-resolve/blob/main/test/incorrect-description-file.test.js>

use std::path::PathBuf;

use oxc_resolver::{JSONError, ResolveError, Resolver};

fn fixture() -> PathBuf {
    super::fixture().join("incorrect-package")
}

// TODO: add `ctx with fileDependencies and then check file dependencies

#[test]
fn incorrect_description_file_1() {
    // should not resolve main in incorrect description file #1
    let f = fixture();
    let resolution = Resolver::default().resolve(f.join("pack1"), ".");
    let error = ResolveError::JSON(JSONError {
        path: f.join("pack1/package.json"),
        message: String::from("EOF while parsing a value at line 3 column 0"),
        line: 3,
        column: 0,
    });
    assert_eq!(resolution, Err(error));
}

#[test]
fn incorrect_description_file_2() {
    // should not resolve main in incorrect description file #2
    let f = fixture();
    let resolution = Resolver::default().resolve(f.join("pack2"), ".");
    let error = ResolveError::JSON(JSONError {
        path: f.join("pack2/package.json"),
        message: String::from("EOF while parsing a value at line 1 column 0"),
        line: 1,
        column: 0,
    });
    assert_eq!(resolution, Err(error));
}

#[test]
fn incorrect_description_file_3() {
    // should not resolve main in incorrect description file #3
    let f = fixture();
    let resolution = Resolver::default().resolve(f.join("pack2"), ".");
    assert!(resolution.is_err());
}
