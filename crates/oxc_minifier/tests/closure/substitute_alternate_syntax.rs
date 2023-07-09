//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntaxTest.java>

use oxc_minifier::CompressOptions;

use crate::{test, test_with_options};

#[test]
fn fold_return_result() {
    test("function f(){return !1;}", "function f(){return !1}");
    test("function f(){return null;}", "function f(){return null}");
    test("function f(){return void 0;}", "function f(){return}");
    test("function f(){return void foo();}", "function f(){return void foo()}");
    test("function f(){return undefined;}", "function f(){return}");
    test("function f(){if(a()){return undefined;}}", "function f(){if(a())return}");
}

#[test]
fn undefined() {
    test("var x = undefined", "var x");
    test("let a = 5, b = undefined, c = true", "let a=5,b,c=!0");
    test("const x = undefined", "const x=void 0");
    test(
        "var undefined = 1;function f() {var undefined=2;var x = undefined;}",
        "var undefined=1;function f(){var undefined=2,x}",
    );
    test("function f(undefined) {}", "function f(undefined){}");
    test("try {} catch(undefined) {}", "try{}catch(undefined){}");
    test("for (undefined in {}) {}", "for(undefined in {}){}");
    test("undefined++", "undefined++");
    test("undefined += undefined;", "undefined+=void 0");
}

#[test]
fn console() {
    let opts = oxc_minifier::MinifierOptions {
        mangle: false,
        compress: CompressOptions { drop_console: true, ..Default::default() },
        ..Default::default()
    };
    test_with_options("let x = 5; console.log(x);", "let x=5", opts);
    test_with_options("let x = console.log('foo')", "let x", opts);
}
