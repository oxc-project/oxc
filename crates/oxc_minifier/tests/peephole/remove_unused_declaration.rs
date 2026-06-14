use oxc_span::SourceType;

use crate::{
    CompressOptions, TreeShakeOptions, test_options, test_options_source_type, test_same_options,
    test_same_options_source_type, test_same_smallest, test_smallest,
};

#[test]
fn remove_unused_variable_declaration() {
    let options = CompressOptions::smallest();
    test_options("var x", "", &options);
    test_options("var x = 1", "", &options);
    test_options("var x = foo", "foo", &options);

    test_options("var [] = []", "", &options);
    test_options("var [] = [1]", "", &options);
    test_options("var [] = [foo]", "foo", &options);
    test_options("var [] = 'foo'", "", &options);
    test_same_options("export var f = () => { var [] = arguments }", &options);
    test_options(
        "export function f() { var [] = arguments }",
        "export function f() { arguments; }",
        &options,
    );
    test_options(
        "function foo() {return (()=>{ var []=arguments })()};foo()",
        "function foo() {arguments;} foo();",
        &options,
    );
    test_same_options_source_type(
        "globalThis.f = function () { var [] = arguments }",
        SourceType::cjs(),
        &options,
    );
    test_same_options("var [] = arguments", &options);
    test_same_options("var [] = null", &options);
    test_same_options("var [] = void 0", &options);
    test_same_options("var [] = 1", &options);
    test_same_options("var [] = a", &options);

    test_options("var {} = {}", "", &options);
    test_options("var {} = { a: 1 }", "", &options);
    test_options("var {} = { foo }", "foo", &options);
    test_same_options("var {} = null", &options);
    test_same_options("var {} = a", &options);
    test_same_options("var {} = null", &options);
    test_same_options("var {} = void 0", &options);

    test_same_options("var x; foo(x)", &options);
    test_same_options("export var x", &options);
    test_same_options("using x = foo", &options);
    test_same_options("await using x = foo", &options);

    test_options("for (var x; ; );", "for (; ;);", &options);
    test_options("for (var x = 1; ; );", "for (; ;);", &options);
    test_same_options("for (var x = foo; ; );", &options); // can be improved
}

#[test]
fn remove_unused_pure_iife_init() {
    // https://github.com/oxc-project/oxc/issues/17480
    test_smallest("var x = /* @__PURE__ */ foo()", "");
    test_smallest("var x = /* @__PURE__ */ new Foo()", "");
    test_smallest("var x = /* @__PURE__ */ foo(a)", "a;");
    test_smallest("var x = /* @__PURE__ */ foo(bar())", "bar();");
    test_smallest("var x = /* @__PURE__ */ new Foo(bar())", "bar();");
    test_smallest("var x = /* @__PURE__ */ foo(/* @__PURE__ */ bar(z))", "z;");

    test_smallest("var x = /* @__PURE__ */ (() => foo())()", "");
    test_smallest("var x = /* @__PURE__ */ (() => new Foo())()", "");
    test_smallest("var x = /* @__PURE__ */ (() => { return foo() })()", "");
    test_smallest("var x = /* @__PURE__ */ (() => { foo() })()", "");

    test_smallest("var x = /* @__PURE__ */ (() => g.x)()", "");
    test_smallest("var x = /* @__PURE__ */ (() => g[k])()", "");
    test_smallest("var x = /* @__PURE__ */ (() => foo`tpl`)()", "");
    test_smallest("var x = /* @__PURE__ */ (() => [a, b])()", "");
    test_smallest("var x = /* @__PURE__ */ (() => ({ a }))()", "");
    test_smallest("var x = /* @__PURE__ */ (() => a + b)()", "");
    test_smallest("var x = /* @__PURE__ */ (() => `${a}`)()", "");
    test_smallest("var x = /* @__PURE__ */ (() => foo()?.bar())()", "");
    test_smallest("var x = /* @__PURE__ */ (() => a ? b : c)()", "");

    test_smallest("var x = /* @__PURE__ */ (function() { return foo() })()", "");

    test_smallest("let x = /* @__PURE__ */ (() => g.x)()", "");
    test_smallest("const x = /* @__PURE__ */ (() => g.x)()", "");

    test_smallest("var x = /* @__PURE__ */ foo(), y = bar(); use(y);", "var y = bar(); use(y);");

    // Referenced bindings keep the declarator — `symbol_is_unused` blocks
    // the drop. Propagation still inlines the IIFE body.
    test_same_smallest("var x = /* @__PURE__ */ foo(); use(x);");
    test_smallest(
        "var x = /* @__PURE__ */ (() => foo())(); use(x);",
        "var x = /* @__PURE__ */ foo(); use(x);",
    );
    test_smallest("var x = /* @__PURE__ */ (() => g.x)(); use(x);", "var x = g.x; use(x);");
    test_smallest(
        "var x = /* @__PURE__ */ (() => { return foo() })(); use(x);",
        "var x = /* @__PURE__ */ foo(); use(x);",
    );
    // Conditional body — propagation only fires on Call/New, so the
    // top-level conditional is inlined without an annotation.
    test_smallest(
        "var x = /* @__PURE__ */ (() => a ? b : c)(); use(x);",
        "var x = a ? b : c; use(x);",
    );

    // Exported bindings are cross-module reachable — the export-ancestor
    // check blocks the early drop.
    test_smallest("export var x = /* @__PURE__ */ foo()", "export var x = /* @__PURE__ */ foo();");
    test_smallest(
        "export const x = /* @__PURE__ */ (() => foo())();",
        "export const x = /* @__PURE__ */ foo();",
    );
    test_smallest("export const x = /* @__PURE__ */ (() => g.x)();", "export const x = g.x;");
    test_same_smallest("var x = /* @__PURE__ */ foo(); export { x }");

    test_smallest("var x = (() => g.x)();", "g.x;");

    // `using` runs `[Symbol.dispose]` at scope exit, so the declarator stays.
    test_smallest("using x = /* @__PURE__ */ (() => foo())()", "using x = /* @__PURE__ */ foo();");
    test_smallest(
        "await using x = /* @__PURE__ */ (() => foo())()",
        "await using x = /* @__PURE__ */ foo();",
    );

    // Function-local var inside an exported function — the export ancestor
    // walk must not be fooled by `f` being exported. `x` is a local.
    test_smallest(
        "export function f() { var x = /* @__PURE__ */ (() => foo())(); } f();",
        "export function f() {} f();",
    );

    // Empty async/generator IIFE in unused-var-init position now collapses
    // through `is_expression_result_unused` (which the widening newly covers).
    test_smallest("var x = (async () => {})()", "");
    test_smallest("var x = (function* () {})()", "");

    // `can_remove_unused_declarators` blocks top-level `var` drops in script
    // mode (the binding is an observable global). The IIFE inlines with
    // propagation as in any other position, but the declarator stays.
    test_options_source_type(
        "var x = /* @__PURE__ */ (() => stuff())()",
        "var x = /* @__PURE__ */ stuff();",
        SourceType::cjs().with_script(true),
        &CompressOptions::smallest(),
    );

    // Direct eval at the root scope blocks the drop — eval might reference
    // the binding even when static analysis sees no use.
    test_smallest(
        "eval('x'); var x = /* @__PURE__ */ (() => stuff())()",
        "eval('x'); var x = /* @__PURE__ */ stuff();",
    );
}

#[test]
fn remove_unused_function_declaration() {
    let options = CompressOptions::smallest();
    test_options("function foo() {}", "", &options);
    test_same_options("function foo() { bar } foo()", &options);
    test_same_options("export function foo() {} foo()", &options);
    test_same_options("function foo() { bar } eval('foo()')", &options);
}

///  Eval inside an unused declaration is execution-dead and must not block its removal.(#20992)
#[test]
fn remove_unused_declaration_with_direct_eval_inside_body() {
    let options = CompressOptions::smallest();
    test_options("function foo() { eval('foo()') }", "", &options);
    test_options("function foo() { eval('foo()'); var x = 1 }", "", &options);
    test_options("class C { m() { eval('C') } }", "", &options);
    test_same_options("function foo() { bar } eval('foo()')", &options);
    // `bar` is unused and its eval never runs, so `bar` is removed; then `foo` is removed too.
    test_options("function bar() { eval('foo()') } function foo() {}", "", &options);
    // Live call to `bar` keeps `foo` -> eval in `bar` may reference `foo` at runtime.
    test_same_options("function bar() { eval('foo()') } function foo() {} bar()", &options);
    test_options("class D { m() { eval('C') } } class C {}", "", &options);
    test_options("function outer() { function inner() { eval('x') } }", "", &options);
    test_options("function outer() { eval('x') } function inner() {}", "", &options);
    test_options(
        "(0, eval)('code'); function foo() { eval('foo()') }",
        "(0, eval)('code');",
        &options,
    );
    // Nested class with eval inside an unused function.
    test_options("function foo() { class Inner { m() { eval('foo()') } } }", "", &options);
    // Direct eval inside try/catch still lexically inside the function body.
    test_options("function foo() { try { eval('foo()') } catch (e) {} }", "", &options);
    // Reassigned eval is indirect -> must not block removal.
    test_options("function foo() { let e = eval; e('foo()') }", "", &options);
    // Optional chaining on eval is indirect (see `remove_unused_declaration_with_optional_eval`).
    test_options("function foo() { eval?.('foo()') }", "", &options);
    // Unused class with a static block containing only direct eval.
    test_options("class C { static { eval('C') } }", "", &options);
}

#[test]
fn remove_unused_class_static_block_shadowed_eval() {
    let options = CompressOptions::smallest();
    // Shadowed `eval` is not direct eval; class is unused so the static block never runs.
    test_options("class C { static { function eval() {} eval('x') } }", "", &options);
}

/// Eval inside a named function *expression* still runs when the expression is evaluated.
#[test]
fn remove_unused_declaration_eval_in_named_function_expression() {
    let options = CompressOptions::smallest();
    test_same_options("(function helper() { eval('target()') })(); function target() {}", &options);
}

/// Edge cases where eval runs eagerly, not inside the function/class body (#20992).
#[test]
fn remove_unused_declaration_direct_eval_edge_cases() {
    let options = CompressOptions::smallest();
    // Default-parameter scope is nested -> eval does not escape.
    test_options("function foo(x = eval('foo()')) {}", "", &options);
    // Computed method key runs at class definition -> keep the eval, drop the class.
    test_options("class C { [eval('C')]() {} }", "eval('C');", &options);
    // `extends` runs at class definition -> keep the super expression.
    test_options("class C extends eval('Base') {}", "eval('Base');", &options);
}

#[test]
fn remove_unused_declaration_after_dead_direct_eval() {
    let options = CompressOptions::smallest();
    test_options("function f(){if(false)eval('x');var x}f()", "", &options);
    // Live eval still keeps `var x` alive after the refresh.
    test_same_options("function f(){eval('x');var x}f()", &options);
    // Parenthesized eval is still direct eval; the wrapped form must keep `var x` alive.
    test_options(
        "function f(){if(false)y;(eval)('x');var x}f()",
        "function f(){(eval)('x');var x}f()",
        &options,
    );
    // Eval nested inside another call's arguments still keeps `var x` alive.
    // The dead `if(false)y` triggers a peephole change so the refresh actually runs.
    test_options(
        "function f(){if(false)y;foo(eval('x'));var x}f()",
        "function f(){foo(eval('x'));var x}f()",
        &options,
    );
    // Eval in a nested scope: clearing must propagate up through both nested and
    // outer chains, then re-set only what's still live (here, nothing).
    test_options(
        "function outer(){function inner(){if(false)eval('x')}inner();var x}outer()",
        "",
        &options,
    );
    // Live eval at the program root keeps an otherwise-unused `var x` alive
    // (the root flag is the global witness checked by `can_remove_unused_declarators`).
    test_same_options("eval('x');var x", &options);
}

#[test]
fn remove_unused_declaration_with_optional_eval() {
    let options = CompressOptions::smallest();
    test_options("function f(){if(false)eval?.('x');var x}f()", "", &options);
    // Live optional eval is indirect — it doesn't set `DirectEval`, so `var x` is
    // removable even though the call itself stays as a side-effectful expression.
    // Contrast with the live-direct-eval root case above, where `var x` is kept.
    test_options("eval?.('x');var x", "eval?.('x');", &options);
}

#[test]
fn remove_unused_class_declaration() {
    let options = CompressOptions::smallest();
    test_options("class C {}", "", &options);
    test_same_options("export class C {}", &options);
    test_options("class C {} C", "", &options);
    test_same_options("class C {} eval('C')", &options);

    // extends
    test_options("class C {}", "", &options);
    test_options("class C extends Foo {}", "Foo", &options);

    // static block
    test_options("class C { static {} }", "", &options);
    test_same_options("class C { static { foo } }", &options);

    // method
    test_options("class C { foo() {} }", "", &options);
    test_options("class C { [foo]() {} }", "foo", &options);
    test_options("class C { static foo() {} }", "", &options);
    test_options("class C { static [foo]() {} }", "foo", &options);
    test_options("class C { [1]() {} }", "", &options);
    test_options("class C { static [1]() {} }", "", &options);

    // property
    test_options("class C { foo }", "", &options);
    test_options("class C { foo = bar }", "", &options);
    test_options("class C { foo = 1 }", "", &options);
    // TODO: would be nice if this is removed but the one with `this` is kept.
    test_same_options("class C { static foo = bar }", &options);
    test_same_options("class C { static foo = this.bar = {} }", &options);
    test_options("class C { static foo = 1 }", "", &options);
    test_options("class C { [foo] = bar }", "foo", &options);
    test_options("class C { [foo] = 1 }", "foo", &options);
    test_same_options("class C { static [foo] = bar }", &options);
    test_options("class C { static [foo] = 1 }", "foo", &options);

    // accessor
    test_options("class C { accessor foo = 1 }", "", &options);
    test_options("class C { accessor [foo] = 1 }", "foo", &options);

    // order
    test_options("class _ extends A { [B] = C; [D]() {} }", "A, B, D", &options);

    // decorators
    test_same_options("class C { @dec foo() {} }", &options);
    test_same_options("@dec class C {}", &options);

    // TypeError
    test_same_options("class C extends (() => {}) {}", &options);
}

#[test]
fn keep_in_script_mode() {
    let options = CompressOptions::smallest();
    let source_type = SourceType::cjs().with_script(true);
    test_same_options_source_type("var x = 1; x = 2;", source_type, &options);
    test_same_options_source_type("var x = 1; x = 2, foo(x)", source_type, &options);
    test_options_source_type("var x = 1; x = 2;", "", SourceType::cjs(), &options);

    test_options_source_type("class C {}", "class C {}", source_type, &options);
}

#[test]
fn remove_unused_import_specifiers() {
    let options = CompressOptions::smallest();

    test_options("import a from 'a'", "import 'a';", &options);
    test_options("import a from 'a'; foo()", "import 'a'; foo();", &options);
    test_same_options(
        "import a from 'a'",
        &CompressOptions {
            treeshake: TreeShakeOptions {
                invalid_import_side_effects: true,
                ..TreeShakeOptions::default()
            },
            ..CompressOptions::smallest()
        },
    );

    test_options("import { a } from 'a'", "import 'a';", &options);
    test_options("import { a, b } from 'a'", "import 'a';", &options);

    test_options("import * as a from 'a'", "import 'a';", &options);

    test_options("import a, { b } from 'a'", "import 'a';", &options);
    test_options("import a, * as b from 'a'", "import 'a';", &options);

    test_same_options("import a from 'a'; foo(a);", &options);
    test_same_options("import { a } from 'a'; foo(a);", &options);
    test_same_options("import * as a from 'a'; foo(a);", &options);
    test_same_options("import a, { b } from 'a'; foo(a, b);", &options);

    test_options("import { a, b } from 'a'; foo(a);", "import { a } from 'a'; foo(a);", &options);
    test_options(
        "import { a, b, c } from 'a'; foo(b);",
        "import { b } from 'a'; foo(b);",
        &options,
    );
    test_options("import a, { b } from 'a'; foo(a);", "import a from 'a'; foo(a);", &options);
    test_options("import a, { b } from 'a'; foo(b);", "import { b } from 'a'; foo(b);", &options);

    test_options(
        "import a from 'a'; import { b } from 'b'; if (false) { console.log(b) }",
        "import 'a'; import 'b';",
        &options,
    );

    test_same_options("import 'a';", &options);

    test_options("import {} from 'a'", "import 'a';", &options);

    test_options(
        "import a from 'a' with { type: 'json' }",
        "import 'a' with { type: 'json' };",
        &options,
    );
    test_options(
        "import {} from 'a' with { type: 'json' }",
        "import 'a' with { type: 'json' };",
        &options,
    );

    test_options("import { a as b } from 'a'", "import 'a';", &options);
    test_same_options("import { a as b } from 'a'; foo(b);", &options);

    test_same_options("import { a } from 'a'; export { a };", &options);
    // Keep imports when direct eval is present
    test_same_options("import { a } from 'a'; eval('a');", &options);
    test_same_options("import a from 'a'; eval('a');", &options);
    test_same_options("import * as a from 'a'; eval('a');", &options);
    // Eval inside an unused function never runs -> `f` and the `a` import binding can go.
    test_options("import { a } from 'a'; function f() { eval('a'); }", "import 'a';", &options);
    // Live `f` keeps the import: eval in `f` may reference `a` at runtime.
    test_same_options("import { a } from 'a'; function f() { eval('a'); } f();", &options);
}

#[test]
fn remove_unused_import_source_statement() {
    let options = CompressOptions::smallest();

    test_options("import source a from 'a'", "", &options);
    test_options("import source a from 'a'; if (false) { console.log(a) }", "", &options);
    test_same_options("import source a from 'a'; foo(a);", &options);
    test_same_options(
        "import source a from 'a'",
        &CompressOptions {
            treeshake: TreeShakeOptions {
                invalid_import_side_effects: true,
                ..TreeShakeOptions::default()
            },
            ..CompressOptions::smallest()
        },
    );
}

#[test]
fn remove_unused_import_defer_statements() {
    let options = CompressOptions::smallest();

    test_options("import defer * as a from 'a'", "", &options);
    test_options("import defer * as a from 'a'; if (false) { console.log(a.foo) }", "", &options);
    test_same_options("import defer * as a from 'a'; foo(a);", &options);
    test_same_options("import defer * as a from 'a'; foo(a.bar);", &options);
    test_same_options(
        "import defer * as a from 'a'",
        &CompressOptions {
            treeshake: TreeShakeOptions {
                invalid_import_side_effects: true,
                ..TreeShakeOptions::default()
            },
            ..CompressOptions::smallest()
        },
    );
}
