use oxc_minifier::CompressOptions;

use crate::{test, test_snapshot, test_with_options};

#[test]
fn addition_folding() {
    test("1 + 1", "2");
    test("1 + 1 + 1", "3");
    test("0 + true", "1");
    test("x+''", "x+''");
}

#[test]
fn typeof_folding() {
    test("typeof x === 'undefined'", "typeof x>'u'");
    test("'undefined' === typeof x", "typeof x>'u'");
}

#[test]
fn addition_folding_snapshots() {
    test_snapshot(
        "addition_folding",
        [
            "let x = 1 + 1",
            "function foo() { return 1 + 1; }",
            "'' + true",
            "'' + false",
            "'' + null",
            "false + null",
            "'1' + '1'",
            "NaN + NaN",
            "'' + NaN",
            // identifiers
            "let x = 1; let y = x + 1",
            "var x = 1; x + 1 === 2",
            "var y = 1; 1 + y === 2",
            "null - Number(1)",
            "1 + 1.0000001",
        ],
    );
}

#[test]
fn test_join_vars() {
    let options = CompressOptions { join_vars: false, ..CompressOptions::default() };
    test_with_options("var foo = 1; var bar = 2", "var foo=1;var bar=2", options);
    // join_vars: true
    let options = CompressOptions::default();
    test_with_options("var foo = 1; var bar = 2", "var foo=1,bar=2", options);
}
