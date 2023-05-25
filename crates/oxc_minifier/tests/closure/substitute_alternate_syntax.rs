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
