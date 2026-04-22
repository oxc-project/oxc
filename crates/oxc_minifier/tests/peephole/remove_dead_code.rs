use crate::{
    CompressOptions, CompressOptionsUnused, default_options, test, test_options, test_same,
    test_same_options,
};

#[track_caller]
fn test_unused(source_text: &str, expected: &str) {
    let options = CompressOptions { unused: CompressOptionsUnused::Remove, ..default_options() };
    test_options(source_text, expected, &options);
}

#[test]
fn test_fold_block() {
    test("{{foo()}}", "foo()");
    test("{foo();{}}", "foo()");
    test("{{foo()}{}}", "foo()");
    test("{{foo()}{bar()}}", "foo(), bar()");
    test("{if(false)foo(); {bar()}}", "bar()");
    test("{if(false)if(false)if(false)foo(); {bar()}}", "bar()");

    test("{'hi'}", "");
    test("{x==3}", "x");
    test("{`hello ${foo}`}", "`${foo}`");
    test("{ (function(){x++}) }", "");
    test("{ (function foo(){x++; foo()}) }", "");
    test("function f(){return;}", "function f(){}");
    test("function f(){return 3;}", "function f(){return 3}");
    test("function f(){if(x)return; x=3; return; }", "function f(){ x ||= 3; }");
    test("{x=3;;;y=2;;;}", "x=3, y=2");

    // Cases to test for empty block.
    // test("while(x()){x}", "while(x());");
    test("while(x()){x()}", "for(;x();)x()");
    // test("for(x=0;x<100;x++){x}", "for(x=0;x<100;x++);");
    // test("for(x in y){x}", "for(x in y);");
    // test("for (x of y) {x}", "for(x of y);");
    test("for (let x = 1; x <10; x++ ) {}", "for (let x = 1; x <10; x++ );");
    test("for (var x = 1; x <10; x++ ) {}", "for (var x = 1; x <10; x++ );");
    test("do { } while (true)", "do;while(!0)");
    test(
        "function z(a) {
          {
            for (var i = 0; i < a; i++) {}
            foo()
          }
          bar()
        }",
        "function z(a) {
          for (var i = 0; i < a; i++);
          foo(), bar()
        }",
    );
}

#[test]
fn test_remove_no_op_labelled_statement() {
    test("a: break a;", "");
    test("a: { break a; }", "");

    test("a: { break a; console.log('unreachable'); }", "");
    test("a: { break a; var x = 1; } x = 2;", "var x = 2;");

    test("b: { var x = 1; } x = 2;", "b: var x = 1; x = 2;");
    test("a: b: { var x = 1; } x = 2;", "a: b: var x = 1; x = 2;");
    test("foo:;", "");
}

#[test]
fn test_fold_useless_for() {
    test("for(;false;) { foo() }", "");
    test("for(;void 0;) { foo() }", "");
    test("for(;undefined;) { foo() }", "");
    test("for(;true;) foo() ", "for(;;) foo() ");
    test_same("for(;;) foo()");
    test("for(;false;) { var a = 0; }", "var a");
    test("for(;false;) { const a = 0; }", "");
    test("for(;false;) { let a = 0; }", "");

    // Make sure it plays nice with minimizing
    test("for(;false;) { foo(); continue }", "");

    test("for (var { c, x: [d] } = {}; 0;);", "var { c, x: [d] } = {};");
    test("for (var se = [1, 2]; false;);", "var se = [1, 2];");
    test("for (var se = [1, 2]; false;) { var a = 0; }", "var se = [1, 2], a;");

    test("for (foo = bar; false;) {}", "for (foo = bar; !1;);");
    // test("l1:for(;false;) {  }", "");
}

#[test]
fn test_minimize_loop_with_constant_condition_vanilla_for() {
    test("for(;true;) foo()", "for(;;) foo()");
    test("for(;0;) foo()", "");
    test("for(;0.0;) foo()", "");
    test("for(;NaN;) foo()", "");
    test("for(;null;) foo()", "");
    test("for(;undefined;) foo()", "");
    test("for(;'';) foo()", "");
}

#[test]
fn test_fold_try_statement() {
    test("try { throw 0 } catch (e) { foo() }", "try { throw 0 } catch { foo() }");
    test("try {} catch (e) { var foo }", "try {} catch { var foo }");
    test("try {} catch (e) { var foo; bar() } finally {}", "try {} catch { var foo }");
    test(
        "try {} catch (e) { var foo; bar() } finally { baz() }",
        "try {} catch { var foo } finally { baz() }",
    );
    test("try {} catch (e) { foo() }", "");
    test("try {} catch (e) { foo() } finally {}", "");
    test("try {} finally { foo() }", "foo()");
    test("try {} catch (e) { foo() } finally { bar() }", "bar()");
    test("try {} finally { var x = foo() }", "var x = foo()");
    test("try {} catch (e) { foo() } finally { var x = bar() }", "var x = bar()");
    test("try {} finally { let x = foo() }", "{ let x = foo() }");
    test("try {} catch (e) { foo() } finally { let x = bar() }", "{ let x = bar();}");
    test("try {} catch (e) { } finally {}", "");
    test("try { foo() } catch (e) { bar() } finally {}", "try { foo() } catch { bar() }");
    test_same("try { foo() } catch { bar() } finally { baz() }");
}

#[test]
fn test_fold_if_statement() {
    test("if (foo) {}", "foo");
    test("if (foo) {} else {}", "foo");
    test("if (false) {}", "");
    test("if (true) {}", "");
    test("if (false) { var a; console.log(a) }", "if (0) var a");
    test_unused("if (false) { var a; console.log(a) }", "");
}

#[test]
fn test_fold_conditional() {
    test("true ? foo() : bar()", "foo()");
    test("false ? foo() : bar()", "bar()");
    test_same("foo() ? bar() : baz()");
    test("foo && false ? foo() : bar()", "(foo, bar());");

    test("var a; (true ? a : 0)()", "var a; a()");
    test("var a; (true ? a.b : 0)()", "var a; (0, a.b)()");
    test("var a; (false ? 0 : a)()", "var a; a()");
    test("var a; (false ? 0 : a.b)()", "var a; (0, a.b)()");
}

#[test]
fn test_remove_empty_static_block() {
    test("class Foo { static {}; foo }", "class Foo { foo }");
    test_same("class Foo { static { foo() } }");
}

#[test]
fn keep_module_syntax() {
    test_same("throw foo; export let bar");
    test_same("throw foo; export default bar");
}

#[test]
fn remove_empty_spread_arguments() {
    test("foo(...[])", "foo()");
    test("new Foo(...[])", "new Foo()");
}

#[test]
fn remove_unreachable() {
    test("while(true) { break a; unreachable;}", "for(;;) break a");
    test("while(true) { continue a; unreachable;}", "for(;;) continue a");
    test("while(true) { throw a; unreachable;}", "for(;;) throw a");
    test("while(true) { return a; unreachable;}", "for(;;) return a");

    test("(function () { return; var a })()", "(function () { return; var a })()");
    test_unused("(function () { return; var a })()", "");
}

#[test]
fn remove_unused_expressions_in_sequence() {
    test("true, foo();", "foo();");
    test("(0, foo)();", "foo();");
    test("(0, foo)``;", "foo``;");
    test("(0, foo)?.();", "foo?.();");
    test_same("(0, eval)();"); // this can be compressed to `eval?.()`
    test_same("(0, eval)``;"); // this can be compressed to `eval?.()`
    test_same("(0, eval)?.();"); // this can be compressed to `eval?.()`
    test("var eval; (0, eval)();", "var eval; eval();");
    test_same("(0, foo.bar)();");
    test_same("(0, foo.bar)``;");
    test_same("(0, foo.bar)?.();");
    test("(true, foo.bar)();", "(0, foo.bar)();");
    test("(true, true, foo.bar)();", "(0, foo.bar)();");
    test("var foo; (true, foo.bar)();", "var foo; (0, foo.bar)();");
    test("var foo; (true, true, foo.bar)();", "var foo; (0, foo.bar)();");

    test("typeof (0, foo);", "foo");
    test_same("v = typeof (0, foo);");
    test("var foo; typeof (0, foo);", "var foo;");
    test("var foo; v = typeof (0, foo);", "var foo; v = typeof foo");
    test("typeof 0", "");

    test_same("delete (0, foo);");
    test_same("delete (0, foo.#bar);");
    test_same("delete (0, foo.bar);");
    test_same("delete (0, foo[bar]);");
    test_same("delete (0, foo?.bar);");
}

#[test]
fn remove_unused_expressions_in_for() {
    test("var i; for (i = 0, 0; i < 10; i++) foo(i);", "var i; for (i = 0; i < 10; i++) foo(i);");
    test(
        "var i; for (i = 0; i < 10; 0, i++, 0) foo(i);",
        "var i; for (i = 0; i < 10; i++) foo(i);",
    );
}

#[test]
fn remove_constant_value() {
    test("const foo = false; if (foo) { console.log('foo') }", "const foo = !1;");
}

#[test]
fn remove_empty_function() {
    let options = CompressOptions::smallest();
    test_options("function foo() {} foo()", "", &options);
    test_options("function foo() {} foo(); foo()", "", &options);
    test_options("var foo = () => {}; foo()", "", &options);
    test_options("var foo = () => {}; foo(a)", "a", &options);
    test_options("var foo = () => {}; foo(a, b)", "a, b", &options);
    test_options("var foo = () => {}; foo(...a, b)", "[...a], b", &options);
    test_options("var foo = () => {}; foo(...a, ...b)", "[...a], [...b]", &options);
    test_options("var foo = () => {}; x = foo()", "x = void 0", &options);
    test_options("var foo = () => {}; x = foo(a(), b())", "x = (a(), b(), void 0)", &options);
    test_options("var foo = function () {}; foo()", "", &options);

    test_same_options("function foo({}) {} foo()", &options);
    test_options("var foo = ({}) => {}; foo()", "(({}) => {})()", &options);
    test_options("var foo = function ({}) {}; foo()", "(function ({}) {})()", &options);

    test_same_options("async function foo({}) {} foo()", &options);
    test_options("var foo = async ({}) => {}; foo()", "(async ({}) => {})()", &options);
    test_options("var foo = async function ({}) {}; foo()", "(async function ({}) {})()", &options);

    test_same_options("function* foo({}) {} foo()", &options);
    test_options("var foo = function*({}) {}; foo()", "(function*({}) {})()", &options);
}
