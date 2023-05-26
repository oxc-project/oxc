//! <https://github.com/google/closure-compiler/blob/master/test/com/google/javascript/jscomp/PeepholeSubstituteAlternateSyntaxTest.java>

use crate::expect;

#[test]
fn fold_return_result() {
    expect("function f(){return !1;}", "function f(){return !1}");
    expect("function f(){return null;}", "function f(){return null}");
    expect("function f(){return void 0;}", "function f(){return}");
    expect("function f(){return void foo();}", "function f(){return void foo()}");
    expect("function f(){return undefined;}", "function f(){return}");
    expect("function f(){if(a()){return undefined;}}", "function f(){if(a()){return}}");
}

#[test]
fn undefined() {
    expect("var x = undefined", "var x=void 0");
    expect(
        "var undefined = 1;function f() {var undefined=2;var x = undefined;}",
        "var undefined=1;function f(){var undefined=2,x=undefined}",
    );
    expect("function f(undefined) {}", "function f(undefined){}");
    expect("try {} catch(undefined) {}", "try{}catch(undefined){}");
    expect("for (undefined in {}) {}", "for(undefined in {}){}");
    expect("undefined++", "undefined++");
    expect("undefined += undefined;", "undefined+=void 0");
}
