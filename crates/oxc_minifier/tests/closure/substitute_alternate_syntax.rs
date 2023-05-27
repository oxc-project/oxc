//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntaxTest.java>

use crate::test;

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
    test("var x = undefined", "var x=void 0");
    test(
        "var undefined = 1;function f() {var undefined=2;var x = undefined;}",
        "var undefined=1;function f(){var undefined=2,x=undefined}",
    );
    test("function f(undefined) {}", "function f(undefined){}");
    test("try {} catch(undefined) {}", "try{}catch(undefined){}");
    test("for (undefined in {}) {}", "for(undefined in {}){}");
    test("undefined++", "undefined++");
    test("undefined += undefined;", "undefined+=void 0");
}
