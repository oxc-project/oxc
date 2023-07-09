use crate::test;

#[test]
fn undefined_assignment() {
    test("let x = undefined", "let x");
    test("var x = undefined", "var x");
    test("const x = undefined", "const x=void 0");
    test("let x; x = undefined", "let x;x=void 0");
    test("function foo(a = undefined) {}", "function foo(a=void 0){}");
    test("let a = undefined; let b = 5; let c = undefined;", "let a,b=5,c");
}

#[test]
fn undefined_return() {
    test("function f(){return undefined;}", "function f(){return}");
    test("function f(){return void 0;}", "function f(){return}");
    test("function f(){return void foo();}", "function f(){return void foo()}");
    test("function f(){if(a()){return undefined;}}", "function f(){if(a())return}");
}
