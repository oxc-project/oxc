use cow_utils::CowUtils;
use oxc_span::SourceType;

use crate::{
    CompressOptions, CompressOptionsKeepNames, CompressOptionsUnused, default_options, test,
    test_options, test_options_source_type, test_same, test_same_options,
    test_same_options_source_type,
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
    test("do { } while (true)", "do;while(1)");
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

    test("b: { var x = 1; } x = 2;", "var x = 1; x = 2;");
    test("a: b: { var x = 1; } x = 2;", "var x = 1; x = 2;");
    test("foo:;", "");
}

#[test]
fn test_remove_unused_labels() {
    test("a: foo()", "foo()");
    test("a: { foo(); bar() }", "foo(), bar()");
    test("a: for (;;) foo()", "for (;;) foo()");
    test("a: { if (false) break a; foo() }", "foo()");
    test("a: { let x = foo(); bar(x, x) }", "{ let x = foo(); bar(x, x); }");

    test("foo: while (true) { break foo; }", "for (;;) break;");
    test("foo: for (x of y) { if (z) continue foo; bar() }", "for (x of y) z || bar();");
    test("a: do { if (x) continue a; y(); } while (z)", "do x || y(); while (z);");
    test("a: do { if (x) break a; y(); } while (z)", "do { if (x) break; y(); } while (z);");
    test("foo: switch (x) { case 1: a(); break foo; }", "x === 1 && a();");
    test("a: b: while (x) { if (y) break a; foo() }", "for (; x && !y;) foo();");
    test("a: b: while (x) { if (y) continue b; foo() }", "for (; x;) y || foo();");
    test(
        "foo: while (x) { try { if (a) break foo; } finally { b() } bar: { if (c) continue foo; d() } switch (y) { case 1: continue foo; } }",
        "for (; x;) { try { if (a) break; } finally { b(); } c || (d(), y); }",
    );

    test("foo: while (x) while (y) break foo;", "foo: for (; x;) for (; y;) break foo;");
    test(
        "foo: while (x) switch (y) { case 1: break foo; }",
        "foo: for (; x;) switch (y) { case 1: break foo; }",
    );
    test(
        "foo: while (x) { if (a) break foo; while (y) if (b) break foo; }",
        "foo: for (; x && !a;) for (; y;) if (b) break foo;",
    );
    test("foo: { while (x) break foo; }", "foo: { for (; x;) break foo; }");

    test("a: x = function () { a: for (;;) if (y) break a; }", "x = function() { for (; !y;); };");
    test(
        "a: x = function () { a: for (;;) for (;;) if (y) break a; }",
        "x = function() { a: for (;;) for (;;) if (y) break a; };",
    );
    test(
        "a: for (;;) { function g() { a: for (;;) for (;;) if (y) break a; } if (x) break a; h(g) }",
        "for (;;) { function g() { a: for (;;) for (;;) if (y) break a; } if (x) break; h(g); }",
    );
    test(
        "a: for (;;) { h(function () { a: for (;;) for (;;) if (y) break a; }); if (x) break a; }",
        "for (; h(function() { a: for (;;) for (;;) if (y) break a; }), !x;);",
    );
    test(
        "a: for (;;) { h(class { static { a: for (;;) for (;;) if (y) break a; } }); if (x) break a; }",
        "for (; h(class { static { a: for (;;) for (;;) if (y) break a; } }), !x;);",
    );

    let script = SourceType::cjs().with_script(true);
    test_same_options_source_type("a: function f() {}", script, &default_options());
    test_same_options_source_type(
        "{ let y = g(); a: function f() { h(y) } } f();",
        script,
        &default_options(),
    );
    test_options_source_type(
        "a: b: function f() {}",
        "b: function f() {}",
        script,
        &default_options(),
    );
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

    test("for (foo = bar; false;) {}", "for (foo = bar; 0;);");
    // test("l1:for(;false;) {  }", "");
}

#[test]
fn test_fold_dead_for_lexical_initializer() {
    for kind in ["let", "const"] {
        test_unused(
            &format!("function f(){{for({kind} x=console.log('effect');false;){{}}}} f();"),
            "function f(){console.log('effect')} f();",
        );
        test(
            &format!("for ({kind} x = foo(); false; update()) {{ body(); }}"),
            "{ let x = foo(); }",
        );
        test(
            &format!("for ({kind} x = foo(), y = bar(x); false;) {{}}"),
            "{ let x = foo(), y = bar(x); }",
        );
        test(
            &format!("let x = 1; for ({kind} x = foo(); false;) {{}} use(x);"),
            "let x = 1; { let x = foo(); } use(1);",
        );
        test(
            &format!("for ({kind} x = eval('x'); false;) {{}}"),
            &format!("{{ {kind} x = eval('x'); }}"),
        );
        test(&format!("for ({kind} x = x; false;) {{}}"), "{ let x = x; }");
        test(&format!("if (a) for ({kind} x = foo(); false;) {{}}"), "if (a) { let x = foo(); }");
        test(
            &format!("for ({kind} x = foo(); false;) {{ var y = bar(); }} use(y);"),
            "{ let x = foo(); var y; } use(y);",
        );
        // Destructuring can invoke getters, iterators, and default initializers.
        for pattern in ["{x}", "[x]", "{x = foo()}", "[x = foo()]"] {
            test_unused(
                &format!("for ({kind} {pattern} = source; false;) {{}}"),
                &format!("{{ let {pattern} = source; }}"),
            );
        }
    }
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

    // Leak regression: when the empty `try` drops, the catch arm's write-ref
    // to `x` must be walked into `PassChanges`, else the stale write blocks
    // constant inlining of `x`.
    let options = CompressOptions::smallest();
    test_options(
        "let x = 'initial'; try {} catch (e) { x = 'unexpected'; } console.log(x);",
        "console.log('initial');",
        &options,
    );
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

    // A kept function declaration (not a dead IIFE) so the unreachable `var a`
    // after `return` is preserved under `unused: Keep`.
    test("function f() { return; var a }", "function f() { return; var a }");
    test_unused("(function () { return; var a })()", "");

    // https://github.com/rolldown/rolldown/issues/10184
    // A statement that never completes normally also terminates the list:
    // a block pinned by its lexical declaration, or a try/catch where both
    // blocks jump.
    test(
        "function f() { { const a = g(); a.x = a; return a; } h(); }",
        "function f() { { let a = g(); return a.x = a, a; } }",
    );
    test(
        "function f() { try { return g(); } catch { return h(); } i(); }",
        "function f() { try { return g(); } catch { return h(); } }",
    );
    // Negative: the block can complete normally, so the tail stays.
    test_same("function f(c) { { let a = g(); if (c) return a; } return foo(); }");
    // Hoisting survivors trailing the jump inside the block — a kept
    // `function` declaration or a `var` stub re-emitted by `KeepVar` — don't
    // hide that the block terminates.
    test(
        "function f() { { const a = g(); a.x = a; return a; function g() { return {} } } h(); }",
        "function f() { { let a = g(); return a.x = a, a; function g() { return {} } } }",
    );
    test(
        "function f() { use(() => x); { let a = g(); use(a); return a; var x = h(); } tail(); }",
        "function f() { use(() => x); { let a = g(); return use(a), a; var x; } }",
    );
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

    // Regression: a >=3 element sequence in indirect-access position whose
    // second-to-last element is already `0` must converge. Re-wrapping the
    // already-`0` element re-records a mutation every iteration, spinning
    // the fixed-point loop into the 10-iteration debug_assert.
    test_same("(sideEffect(), 0, foo.bar)();");
    test("delete (sideEffect(), 0, foo.bar);", "sideEffect(), foo.bar;");

    test("typeof (0, foo);", "foo");
    test_same("v = typeof (0, foo);");
    test("var foo; typeof (0, foo);", "var foo;");
    test("v = function(foo) { return typeof (0, foo) }", "v = function(foo) { return typeof foo }");
    test("typeof 0", "");

    test("delete (0, foo);", "foo");
    test("delete (0, foo.#bar);", "foo.#bar");
    test("delete (0, foo.bar);", "foo.bar");
    test("delete (0, foo[bar]);", "foo[bar]");
    test("delete (0, foo?.bar);", "foo?.bar");
}

#[test]
fn keep_indirect_access_in_decorator() {
    test("@((0, a.b)) class C {}", "@(0, a.b) class C {}");
    test("@((0, a[b])) class C {}", "@(0, a[b]) class C {}");
    test("@((true, a.b)) class C {}", "@(0, a.b) class C {}");
    test("@((true, true, a.b)) class C {}", "@(0, a.b) class C {}");
    test_same("@(x(), a.b) class C {}");
    test("@(true ? a.b : c) class D {}", "@(0, a.b) class D {}");
    test("var x; @(x ? a.b : a.b) class D {}", "var x; @(0, a.b) class D {}");
    test("@(true && a.b) class C {}", "@(0, a.b) class C {}");
    test("@(null ?? a.b) class C {}", "@(0, a.b) class C {}");
    test("class E { @((0, a.b)) m() {} }", "class E { @(0, a.b) m() {} }");
    test("class E { @((0, a.b)) p = 1 }", "class E { @(0, a.b) p = 1 }");
    test("@((0, foo)) class C {}", "@foo class C {}");
    test("@((0, a?.b)) class C {}", "@(0, a?.b) class C {}");
    test("@((0, a?.b.c)) class C {}", "@(0, a?.b.c) class C {}");
    test("@(true && a?.b) class C {}", "@(0, a?.b) class C {}");
    test("@((0, a?.b())) class C {}", "@(a?.b()) class C {}");
}

#[test]
fn keep_names_indirect_anonymous_function_definition() {
    let keep_names =
        CompressOptions { keep_names: CompressOptionsKeepNames::all_true(), ..default_options() };
    let class_only =
        CompressOptions { keep_names: CompressOptionsKeepNames::class_only(), ..default_options() };
    let function_only = CompressOptions {
        keep_names: CompressOptionsKeepNames::function_only(),
        ..default_options()
    };

    // `(0, value)` stops NamedEvaluation, so removing `0,` would give the value a name.
    let named = [
        "x = (0, VALUE)",
        "x ||= (0, VALUE)",
        "var x = (0, VALUE)",
        "function f(a = (0, VALUE)) { return a }",
        "var { a = (0, VALUE) } = o",
        "[a = (0, VALUE)] = o",
        "({ a = (0, VALUE) } = o)",
        "x = { a: (0, VALUE) }",
        "x = { [a]: (0, VALUE) }",
        "x = class { a = (0, VALUE) }",
        "x = class { accessor a = (0, VALUE) }",
    ];
    // These positions never name the value, so `0,` can always be removed.
    let unnamed = [
        ("f((0, VALUE))", "f(VALUE)"),
        ("x.y = (0, VALUE)", "x.y = VALUE"),
        ("x[y] = (0, VALUE)", "x[y] = VALUE"),
        ("[x.y = (0, VALUE)] = o", "[x.y = VALUE] = o"),
        ("x = [(0, VALUE)]", "x = [VALUE]"),
    ];
    let values = ["class {}", "class extends v {}", "function () {}", "() => {}"];

    for value in values {
        let is_class = value.starts_with("class");
        for source in named {
            let source = source.cow_replace("VALUE", value);
            let folded = source.cow_replace(&format!("(0, {value})"), value);
            test_same_options(&source, &keep_names);
            if is_class {
                test_same_options(&source, &class_only);
                test_options(&source, &folded, &function_only);
            } else {
                test_same_options(&source, &function_only);
                test_options(&source, &folded, &class_only);
            }
            // Names are not guaranteed without `keepNames`, so the smaller output is kept.
            test(&source, &folded);
        }
        for (source, expected) in unnamed {
            let source = source.cow_replace("VALUE", value);
            let expected = expected.cow_replace("VALUE", value);
            test_options(&source, &expected, &keep_names);
            test(&source, &expected);
        }
    }

    // `export default (class {})` is also named `default`.
    test_same_options("export default (0, class {})", &keep_names);
    test_same_options("export default (0, function () {})", &keep_names);
    test_same_options("export default (0, () => {})", &keep_names);
    test("export default (0, class {})", "export default (class {})");
    test("export default (0, function () {})", "export default (function () {})");

    // Other side-effect-free leading elements are reduced to the `0,` placeholder.
    test_options("x = (1, class {})", "x = (0, class {})", &keep_names);
    test_options("x = (a(), 1, 2, class {})", "x = (a(), 0, class {})", &keep_names);
    test_options("x = (true && class {})", "x = (0, class {})", &keep_names);
    test_options("x = (true ? class {} : 0)", "x = (0, class {})", &keep_names);
    test("x = (true ? class {} : 0)", "x = class {}");
    // Named values and non-assigning operators are not affected.
    test_options("x = (0, class C {})", "x = class C {}", &keep_names);
    test_options("x = (0, function f() {})", "x = function f() {}", &keep_names);
    test_options("x += (0, class {})", "x += class {}", &keep_names);

    // Output of the class-properties transform for a lowered anonymous class (#29931).
    test_same_options(
        "var _Class; function make(v) { return _Class = (0, class extends v {}), _Class.r = 1, _Class }",
        &keep_names,
    );
    test(
        "var _Class; function make(v) { return _Class = (0, class extends v {}), _Class.r = 1, _Class }",
        "var _Class; function make(v) { return _Class = class extends v {}, _Class.r = 1, _Class }",
    );
}

#[test]
fn remove_unused_expressions_in_for() {
    test("var i; for (i = 0, 0; i < 10; i++) foo(i);", "for (var i = 0; i < 10; i++) foo(i);");
    test("var i; for (i = 0; i < 10; 0, i++, 0) foo(i);", "for (var i = 0; i < 10; i++) foo(i);");
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

    test_options("var foo = (a = 0) => {}; foo()", "", &options);
    test_options(
        "var foo = (a = side_effect()) => {}; foo()",
        "((a = side_effect()) => {})()",
        &options,
    );
    test_same_options("function foo(a = side_effect()) {} foo()", &options);

    test_same_options("function foo({}) {} foo()", &options);
    test_options("var foo = ({}) => {}; foo()", "(({}) => {})()", &options);
    test_options("var foo = function ({}) {}; foo()", "(function ({}) {})()", &options);

    test_same_options("async function foo({}) {} foo()", &options);
    test_options("var foo = async ({}) => {}; foo()", "(async ({}) => {})()", &options);
    test_options("var foo = async function ({}) {}; foo()", "(async function ({}) {})()", &options);

    test_same_options("function* foo({}) {} foo()", &options);
    test_options("var foo = function*({}) {}; foo()", "(function*({}) {})()", &options);
}

#[test]
fn pure_function_preserves_rest_destructuring() {
    test_same("function foo(...[v = side_effect()]) {} foo(), foo();");
    test_same("function foo(...[v = side_effect()]) {} use(foo(), foo());");
    test_same("var foo = function (...[v = side_effect()]) {}; foo(), foo();");
    test_same("var foo = function (...[v = side_effect()]) {}; use(foo(), foo());");
    test_same("var foo = (...[v = side_effect()]) => {}; foo(), foo();");
    test_same("var foo = (...[v = side_effect()]) => {}; use(foo(), foo());");
    test_same("function foo(...{0: v = side_effect()}) {} foo(), foo();");
    test_same("function foo(...{0: v = side_effect()}) {} use(foo(), foo());");
    test_same("function foo(...[[v]]) {} foo(), foo();");
    test_same("function foo(...[[v]]) {} use(foo(), foo());");
    test("function foo(...args) {} foo();", "function foo(...args) {}");
}

#[test]
fn redeclared_pure_function_is_not_folded_var() {
    test_same("var foo = (u) => {}; if (g) var foo = (a) => { console.log(a); }; foo('x');");
}

#[test]
fn redeclared_pure_function_is_not_folded_function_declaration() {
    test_same_options_source_type(
        "function foo(u) {} function foo(a) { console.log(a); } foo('x');",
        SourceType::cjs().with_script(true),
        &default_options(),
    );
}

#[test]
fn direct_eval_rebound_function_is_not_folded() {
    test(
        "function foo() {} eval(\"foo = () => side_effect()\"); foo();",
        "function foo() {} eval(\"foo = () => side_effect()\"), foo();",
    );
}

#[test]
fn nested_direct_eval_rebound_function_is_not_folded() {
    test(
        "function foo() {} function g() { eval(\"foo = () => side_effect()\") } g(); foo();",
        "function foo() {} function g() { eval(\"foo = () => side_effect()\") } g(), foo();",
    );
}

#[test]
fn removed_direct_eval_reenables_pure_function_folding() {
    test_options_source_type(
        "function foo() {} if (0) eval(\"foo = () => side_effect()\"); foo();",
        "",
        SourceType::cjs(),
        &CompressOptions::smallest(),
    );
}

#[test]
fn script_root_rebound_function_is_not_folded() {
    test_options_source_type(
        "function foo() {} globalThis.foo = () => side_effect(); foo();",
        "function foo() {} globalThis.foo = () => side_effect(), foo();",
        SourceType::cjs().with_script(true),
        &default_options(),
    );
}

#[test]
fn non_redeclared_pure_function_still_folds() {
    test("const foo = (u) => {}; foo(1)", "const foo = (u) => {};");
    test_options_source_type(
        "function foo() {} foo()",
        "function foo() {}",
        SourceType::cjs(),
        &default_options(),
    );
}
