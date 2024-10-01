use oxc_minifier::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options = CompressOptions::default();
    crate::test(source_text, expected, options);
}

fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[test]
fn test_disabled() {
    let options = CompressOptions { join_vars: false, ..Default::default() };
    crate::test("let a; let b;", "let a;let b;", options);
}

#[test]
fn join_lets() {
    test("let a;", "let a;");
    test("let a; let b;", "let a,b;");
    test("let a = 1; let b = 2;", "let a=1,b=2;");
    test("let a = 1; let b;", "let a=1,b;");
    test("let a; let b = 2;", "let a,b=2;");
}

#[test]
fn join_vars() {
    test("var a; var b;", "var a,b;");
    test("var a = 1; var b = 2;", "var a=1,b=2;");
    test("var a = 1; var b;", "var a=1,b;");
    test("var a; var b = 2;", "var a,b=2;");
}

#[test]
fn join_const() {
    test("const a; const b;", "const a,b;");
    test("const a = 1; const b = 2;", "const a=1,b=2;");
}

#[test]
fn no_join_require() {
    test_same("var a = require('a'); var b = require('b');");
    test_same("const a = require('a'); const b = require('b');");
    test_same("let a = require('a'); let b = require('b');");
    test_same("var a = 1; var b = require('b');");
    // Should these not merge?
    test("var a = require('a'); var b = 1;", "var a = require('a'), b = 1;");
    test(
        "var a = require('a'); var b = 1; var c = require('c');",
        "var a = require('a'), b = 1; var c = require('c');",
    );
}

#[test]
fn join_mixed_all_different() {
    // different
    test("let a = 1; var b = 2;", "let a=1;var b=2;");
    test("let a = 1; const b = 2;", "let a=1;const b=2;");
}

#[test]
fn join_mixed_interspersed() {
    test("let a = 1; var b = 2; let c = 3;", "let a=1;var b=2;let c=3;");
}

#[test]
fn join_mixed_same_adjacent() {
    test("let a = 1; var b = 2; var c = 3;", "let a=1;var b=2,c=3;");
    test("let a = 1; let b = 2; var c = 3;", "let a=1,b=2; var c=3;");
}

#[test]
fn interspersed_statements() {
    test_same("var x = 1; assert.equals(x, 1); var x = 2;");
    test("var x = 1; if (y) { x = 2 } var x = 2;", "var x = 1; if (y) x = 2; var x = 2;");
}

#[test]
fn interspersed_expressions() {
    test_same("var a = 1; a = 2; var b = 3;");
    test_same("var a = 1; a = 2; var a = 3;");
    test_same("var a = 1; a += 1; var b = 3;");
}
