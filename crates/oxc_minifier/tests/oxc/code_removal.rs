use crate::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options = CompressOptions::all_true();
    crate::test(source_text, expected, options);
}

#[test]
fn undefined_assignment() {
    test("let x = undefined", "let x");
    test("var x = undefined", "var x");
    test("const x = undefined", "const x=void 0");
    test("let x; x = undefined", "let x;x=void 0");
    test("function foo(a = undefined) {}", "function foo(a=void 0){}");
    test("let a = undefined; let b = 5; let c = undefined", "let a,b=5,c");
}

#[test]
fn undefined_return() {
    test("function f(){return undefined;}", "function f(){return}");
    test("function f(){return void 0;}", "function f(){return}");
    test("function f(){return void foo();}", "function f(){return void foo()}");
    test("function f(){if(a()){return undefined;}}", "function f(){if(a())return}");
}

#[test]
fn console_removal() {
    let options = CompressOptions { drop_console: true, ..CompressOptions::default() };
    crate::test("console.log('hi')", "", options);
    crate::test("let x = console.error('oops')", "let x", options);
    crate::test("function f() { return console.warn('problem') }", "function f(){return}", options);

    // console isn't removed when drop_console is `false`. This is also the
    // default value.
    let options = CompressOptions::default();
    crate::test("console.log('hi')", "console.log('hi')", options);
}
