use oxc_span::SourceType;

use crate::{
    CompressOptions, CompressOptionsUnused, TreeShakeOptions, test_options,
    test_options_once_with_iterations, test_options_source_type, test_options_with_iterations,
    test_same_options, test_same_options_source_type, test_same_smallest, test_smallest,
};

// Leak regression: dropping an unused declarator must walk the whole
// declarator, not just the init — references can also live in the binding's
// TS type annotation (e.g. computed keys in a type literal). A leaked type
// ref makes the symbol look used, blocking its own removal.
#[test]
fn remove_unused_declarator_walks_type_annotation_refs() {
    let options = CompressOptions::smallest();
    test_options_source_type(
        "function f() { const a = Symbol('a'); const b = Symbol('b'); const reg: { [a]: string; [b]: string } = { foo: 1, bar: 2 }; return 1; } g(f());",
        "function f() { return 1; } g(f());",
        SourceType::ts(),
        &options,
    );
}

// Leak regression (single-use inlining, `stmts.pop()` site): after the lone
// declarator's init is inlined into the next statement, the whole declaration
// statement is popped — the discarded declarator's type annotation still holds
// a ref to `a`.
#[test]
fn single_use_inline_pop_walks_type_annotation_refs() {
    let options = CompressOptions::smallest();
    test_options_source_type(
        "function f() { const a = Symbol('a'); const x: { [a]: string } = g(); return x; } h(f());",
        "function f() { return g(); } h(f());",
        SourceType::ts(),
        &options,
    );
}

// Leak regression (single-use inlining, `declarations.truncate()` site): only
// the tail declarator `x` is inlined; the truncate discards it while `keep`
// survives — `x`'s type annotation still holds a ref to `a`.
#[test]
fn single_use_inline_truncate_walks_type_annotation_refs() {
    let options = CompressOptions::smallest();
    test_options_source_type(
        "function f() { const a = Symbol('a'); const keep = g(), x: { [a]: string } = h(); return [keep, keep, x]; } j(f());",
        "function f() { let keep = g(); return [keep, keep, h()]; } j(f());",
        SourceType::ts(),
        &options,
    );
}

// Leak regression (single-use inlining, `declarations.drain()` site): `x` is
// inlined into the sibling declarator `y`'s init within the same declaration;
// the drain discards `x`'s declarator — its type annotation still holds a ref
// to `a`.
#[test]
fn single_use_inline_drain_walks_type_annotation_refs() {
    let options = CompressOptions::smallest();
    test_options_source_type(
        "function f() { const a = Symbol('a'); const x: { [a]: string } = g(), y = [x]; return y; } j(f());",
        "function f() { return [g()]; } j(f());",
        SourceType::ts(),
        &options,
    );
}

// Leak regression (dead-code identity-drop site): an init-less `var` after
// `return` is classified as an identity drop (KeepVar re-emits it), skipping
// the drop walk — but KeepVar's re-emit strips the type annotation, so the
// annotation's ref to `b` leaks.
#[test]
fn dead_code_identity_drop_checks_type_annotation() {
    let options = CompressOptions::smallest();
    test_options_source_type(
        "function f() { const b = Symbol('b'); return 1; var a: { [b]: string }; } g(f());",
        "function f() { return 1; } g(f());",
        SourceType::ts(),
        &options,
    );
}

// Near-miss: dropping the annotated declarator must only kill the annotation's
// own ref — `a`'s other (value) uses keep `const a = Symbol('a')` alive.
#[test]
fn type_annotation_drop_keeps_symbol_used_elsewhere() {
    let options = CompressOptions::smallest();
    test_options_source_type(
        "function f() { const a = Symbol('a'); const x: { [a]: string } = g(); return [x, a, a]; } h(f());",
        "function f() { let a = Symbol('a'); return [g(), a, a]; } h(f());",
        SourceType::ts(),
        &options,
    );
}

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

// ---- Graph removal of dead recursive cycles ----

// #13105: a declaration whose every reference lives inside its own body (or
// inside the bodies of a cycle it belongs to) can never execute — no live
// code can reach it, so the whole group is removable. Reference counting
// alone can't see this: the internal references keep the count above zero.
#[test]
fn remove_recursive_unused_function_declaration() {
    // Self-recursion.
    test_smallest("function f() { f() }", "");
    // Side effects inside the dead body never run.
    test_smallest("function f() { console.log(1); f() }", "");
    // Mutual recursion.
    test_smallest("function c() { d() } function d() { c() }", "");
    // Self-reference as a value.
    test_smallest("function f() { return f }", "");
    test_smallest("function f() { g(f) }", "");
    // The cycle's only external reference is inside dead code.
    test_smallest("if (false) c(); function c() { d() } function d() { c() }", "");
}

// Ownership is determined from each reference's current semantic scope, not
// from traversal frames. References nested in unregistered scopes still
// belong to their nearest enclosing function declaration.
#[test]
fn remove_recursive_functions_through_nested_scope_kinds() {
    test_smallest(
        "function a(p = b) { { return () => function () { return class { m() { b() } } } } } function b() { a() }",
        "",
    );
}

#[test]
fn keep_recursive_functions_referenced_outside_registered_functions() {
    test_smallest(
        "function a() { b() } function b() { a() } use(() => a, function () { b() }, class { m() { a() } });",
        "function a() {\n\tb();\n}\nfunction b() {\n\ta();\n}\nuse(() => a, function() {\n\tb();\n}, class {\n\tm() {\n\t\ta();\n\t}\n});",
    );
}

#[test]
fn remove_recursive_unused_nested_in_live_function() {
    // Dead recursion inside a used function: statement-level tree shaking
    // (rolldown's linker) cannot see inside bodies, so this must be handled
    // here.
    test_smallest(
        "function live() { function inner() { inner() } return 1; } g(live());",
        "function live() { return 1; } g(live());",
    );
}

// ---- Site-local self-recursive declarators ----

#[test]
fn remove_self_recursive_function_valued_declarators() {
    test_smallest("var f = function() { f() };", "");
    test_smallest("const f = () => f();", "");
    test_smallest("let f = function(value = f()) {};", "");
    test_smallest("let f = (value = f()) => value;", "");
}

#[test]
fn keep_reachable_self_recursive_function_valued_declarators() {
    test_same_smallest("const f = function() { f() }; use(f);");
    test_same_smallest("let f = () => f(); f = other;");
    test_same_smallest("export const f = () => f();");
    test_same_smallest("const f = (effect(), () => f());");

    // A script-level `var` is externally observable even when its declaration
    // is nested in a block and visited from that block's scope.
    test_options_source_type(
        "{ var f = function() { f() } }",
        "var f = function() { f() };",
        SourceType::cjs().with_script(true),
        &CompressOptions::smallest(),
    );
}

// A for initializer is an actual declarator removal site, so it can use the
// same local self-reference check without becoming a graph candidate.
#[test]
fn remove_self_recursive_for_init_declarator() {
    test_smallest("for (let f = () => f();;) break;", "for (;;) break;");
}

// ---- Non-candidates: declarator and class cycles ----

// Mutual declarator and class cycles are deliberately kept because only
// function declarations participate in graph reachability. These tests pin
// the currently unsupported shapes.
#[test]
fn keep_recursive_declarator_and_class_cycles() {
    // const arrow cycle.
    test_smallest(
        "const a = () => b(); const b = () => a();",
        "const a = () => b(), b = () => a();",
    );
    // Class cycle with side-effect-free evaluation.
    test_same_smallest(
        "class A {\n\tm() {\n\t\tnew B();\n\t}\n}\nclass B {\n\tm() {\n\t\tnew A();\n\t}\n}",
    );
    // Mixed function / const arrow / class cycle: the non-function members
    // keep the function member live, so nothing is removed.
    test_smallest(
        "function a() { b() } const b = () => { new C() }; class C { m() { a() } }",
        "function a() {\n\tb();\n}\nconst b = () => {\n\tnew C();\n};\nclass C {\n\tm() {\n\t\ta();\n\t}\n}",
    );
}

#[test]
fn keep_recursive_multi_declarator_cycle() {
    // The declarator member of the cycle keeps the function member live, so
    // the cycle survives even after the used sibling declarator is inlined.
    test_smallest(
        "const a = () => b(), keep = 1; function b() { a() } console.log(keep);",
        "const a = () => b();\nfunction b() {\n\ta();\n}\nconsole.log(1);",
    );
}

// This declarator is outside every registered function, so its reference makes
// the target function unconditionally live even though the declaration sits in
// a bare statement slot.
#[test]
fn keep_declarator_cycle_in_bare_statement_slot() {
    test_same_smallest("function a() {\n\tb();\n}\nif (g) var b = a;");
}

// Future extension: mutual declarator cycles need graph participation rather
// than the removal-site-local check used for self-recursive initializers.
#[test]
#[ignore = "TODO: extend recursive reachability to mutual variable declarators"]
fn remove_recursive_unused_mutual_declarator_cycles() {
    test_smallest("const a = () => b(); const b = () => a();", "");
    test_smallest(
        "const a = () => b(), keep = 1; function b() { a() } console.log(keep);",
        "console.log(1);",
    );
}

// Future extension: class evaluation needs a stable removability proof before
// classes can participate in the graph. These are the side-effect-free shapes
// that should become removable once that proof exists.
#[test]
#[ignore = "TODO: extend recursive reachability to class declarations"]
fn remove_recursive_unused_class_cycles() {
    test_smallest("class A { m() { new B() } } class B { m() { new A() } }", "");
    test_smallest("function a() { b() } const b = () => { new C() }; class C { m() { a() } }", "");
    test_smallest("function F() {} class A extends F { m() { new A() } }", "");
    test_smallest("function F() {} class A extends (0 || F) { m() { new A() } }", "");
}

// ---- References from live code ----

#[test]
fn keep_recursive_function_with_live_references() {
    // A read from live code keeps the cycle live.
    test_same_smallest("function f() { f() } console.log(f);");
    // A write from live code also keeps it live (dropping the function would
    // leave `f = null` assigning to a missing binding).
    test_same_smallest("function f() { f() } f = null;");
    // Direct eval in the declaring scope blocks removal.
    test_same_smallest("function o() { function f() { f() } eval('x') } o();");
    // Root direct eval disables publication before the first pass.
    test_same_smallest("eval('x');\nfunction f() {\n\tf();\n}");
}

#[test]
fn keep_recursive_cycle_with_side_effectful_evaluation() {
    // The side-effectful initializer survives, and its reference to `b`
    // keeps the cycle live.
    test_smallest(
        "const a = (console.log(1), () => b()); const b = () => a();",
        "const a = (console.log(1), () => b()), b = () => a();",
    );
    // Side-effectful heritage keeps the class cycle.
    test_same_smallest(
        "class A extends (console.log(1), Object) { m() { new B() } } class B { m() { new A() } }",
    );
    // A PURE static value still keeps the class cycle: `remove_unused_class`
    // extracts every present static value, so removal would not be clean —
    // the extracted `B` would reference a removed cycle member (see
    // `classify_class_removability`).
    test_same_smallest("class A { static x = B; m() { new B() } } class B { m() { new A() } }");
}

// ---- Class heritage classification ----

#[test]
fn remove_unused_class_identifier_heritage_under_assumptions() {
    // ASSUMPTIONS.md excludes TDZ violations and side effects from extending a class.
    test_smallest("var A = class {}; var B = class extends A {}; var C = class extends B {};", "");
    test_smallest("(class extends g() {});", "g();");
}

#[test]
fn keep_class_cycle_with_wrapped_arrow_heritage() {
    // The arrow check sees through a sequence wrapper.
    test_smallest("class C extends (0, () => {}) {}", "class C extends (() => {}) {}");
}

#[test]
fn keep_referenced_classes_with_identifier_heritage() {
    // Ordinary references keep these classes live independently of heritage errors.
    test_same_smallest("class C extends C {}");
    test_same_smallest("g(function() {\n\tclass C extends C {}\n});");
    test_smallest("class C extends (0, C) {}", "class C extends C {}");
    test_same_smallest(
        "class A extends B {\n\tm() {\n\t\tnew A();\n\t}\n}\nclass B {\n\tm() {\n\t\tnew A();\n\t}\n}",
    );
    test_same_smallest("var B = class {};\nclass A extends B {\n\tm() {\n\t\tnew A();\n\t}\n}");
}

#[test]
fn keep_class_cycle_with_hoisted_function_heritage() {
    // The class is not a candidate, and its heritage reference keeps `F` live.
    test_same_smallest("function F() {}\nclass A extends F {\n\tm() {\n\t\tnew A();\n\t}\n}");
}

// Classes remain count-managed. A logical fold (`0 || Y` -> `Y`) may change
// heritage structure mid-pass, but it must not make the surrounding class
// cycle removable. Everything is kept; only the fold itself changes output.
#[test]
fn keep_class_cycle_with_fold_unstable_heritage() {
    // Identifier heritage inside a foldable wrapper.
    test_smallest(
        "class B { m() { new A() } } var Y = class {}; class A extends (0 || Y) { [B]() {} }",
        "class B {\n\tm() {\n\t\tnew A();\n\t}\n}\nvar Y = class {};\nclass A extends Y {\n\t[B]() {}\n}",
    );
    // Arrow inside the same wrapper (`extends` an arrow is a guaranteed
    // TypeError, so surfacing it also flips to `Keep`).
    test_smallest(
        "class B { m() { new A() } } class A extends (0 || (() => {})) { [B]() {} }",
        "class B {\n\tm() {\n\t\tnew A();\n\t}\n}\nclass A extends (() => {}) {\n\t[B]() {}\n}",
    );
}

#[test]
fn keep_class_cycle_with_wrapped_heritage() {
    // The `0 || F` fold still surfaces `F`; the class is kept either way
    // (classes are not candidates), and the heritage reference keeps `F` live.
    test_smallest(
        "function F() {}\nclass A extends (0 || F) {\n\tm() {\n\t\tnew A();\n\t}\n}",
        "function F() {}\nclass A extends F {\n\tm() {\n\t\tnew A();\n\t}\n}",
    );
}

// Exported classes are externally observable, and references in class method
// scopes keep function candidates live because classes are not graph candidates.
#[test]
fn keep_exported_class_cycle() {
    test_same_smallest("export class A { m() { new B() } } class B { m() { new A() } }");
    test_same_smallest("export default class A { m() { new B() } } class B { m() { new A() } }");
}

// Script-root class bindings are cross-script observable, like vars. The
// module-mode keep for this cycle is pinned in
// `keep_recursive_declarator_and_class_cycles`.
#[test]
fn keep_recursive_class_in_script_mode_top_level() {
    let options = CompressOptions::smallest();
    test_same_options_source_type(
        "class A { m() { new B() } } class B { m() { new A() } }",
        SourceType::cjs().with_script(true),
        &options,
    );
}

// ---- Function/var redeclarations ----

// A symbol shared by a graph-eligible function declaration and a count-only
// variable declaration must stay consistent at both removal sites.
#[test]
fn recursive_function_with_var_redeclaration() {
    // The array init has a specialized (residue-leaving) handler, so the
    // variable site is count-only and its references come from live code: the
    // function declaration can be removed while the initializer residue stays.
    test_smallest("function f() { f() } var f = [g()];", "g();");
    test_same_smallest("function f() { f() } var f = [f];");
}

#[test]
fn recursive_function_var_redeclaration_converges_on_count_pass() {
    let options = CompressOptions::smallest();
    test_options_with_iterations("function f() { f() } var f;", "", 2, &options);

    // The first pass removes the function using published graph deadness.
    // Its body reference is pruned only at the following flush, so a capped
    // run conservatively retains the sibling `var` declaration.
    let options = CompressOptions { max_iterations: Some(0), ..CompressOptions::smallest() };
    test_options_once_with_iterations("function f() { f() } var f;", "var f;", 0, &options);
}

// ---- Export observability ----

#[test]
fn module_export_observability_kinds() {
    test_same_smallest("export function f() {\n\tf();\n}");
    test_same_smallest("function f() {\n\tf();\n}\nexport { f };");
    test_same_smallest("export default function f() {\n\tf();\n}");
    // A default identifier contributes an ordinary evaluated-value reference;
    // it does not add stable observability metadata to the local binding.
    test_same_smallest("function f() {\n\tf();\n}\nexport default f;");

    let options = CompressOptions::smallest();
    test_options_source_type(
        "function f() { f() } export type { f };",
        "export type { f };",
        SourceType::ts(),
        &options,
    );
}

// Export observability is stable symbol metadata. It protects a binding even
// when another declaration of the same symbol supplies its runtime value.
#[test]
fn keep_recursive_cycle_with_exported_redeclaration() {
    // `export var f;` carries no reference, but importers observe the
    // binding: removing the initializing redeclaration would export
    // undefined.
    test_same_smallest("export var f; var f = function() { setTimeout(f) };");
}

// `export var f;` carries no ordinary reference. Stable export observability
// must protect a sibling initializer after removal of the dead cycle that held
// its last in-module read.
#[test]
fn keep_exported_var_initializer_when_a_dead_cycle_held_its_only_reference() {
    test_smallest(
        "export var f;\nvar f = function () { return 'F' };\nfunction d1() { console.log(f); return d2() }\nfunction d2() { return d1() }",
        "export var f;\nvar f = function() {\n\treturn 'F';\n};",
    );
}

// Every count-based consumer shares the same export observability predicate.
// Deleting the dead cycle removes the last ordinary read of `f`, but assignments
// and member writes remain observable through the exported binding.
#[test]
fn keep_exported_binding_writes_when_a_dead_cycle_held_its_other_reads() {
    test_smallest(
        "export var f; var f = 0; function d1() { console.log(f); d2() } function d2() { d1() } f = 1;",
        "export var f;\nvar f = 0;\nf = 1;",
    );
    test_smallest(
        "export var f; var f = {}; function d1() { console.log(f); d2() } function d2() { d1() } f.x = 1;",
        "export var f;\nvar f = {};\nf.x = 1;",
    );
}

// Export observability also gates `is_expression_result_unused`
// (`substitute_alternate_syntax`). A dead cycle's removal can discard all
// ordinary reads while importers still observe the binding. Without the
// stable export bit, the empty async/generator IIFE arms
// collapse the initializer to `void 0` — importers would read `undefined`
// instead of a Promise / Generator object. (The pure-arrow arms share the
// gate but their shapes dissolve on pass 1 via `try_take_iife_body`,
// before the count can zero; the async/generator family keeps its shape,
// which is what makes this reachable.)
#[test]
fn keep_exported_iife_init_when_a_dead_cycle_held_its_only_reference() {
    test_smallest(
        "export var f; var f = (async () => {})(); function d1() { f(); return d2() } function d2() { return d1() }",
        "export var f;\nvar f = (async () => {})();",
    );
    test_smallest(
        "export var g; var g = (function* () {})(); function d1() { g; return d2() } function d2() { return d1() }",
        "export var g;\nvar g = (function* () {})();",
    );
}

// Export observability must not leak through an arrow: `export default () =>
// { function f() {} }` declares an ordinary candidate, not an exported
// binding.
#[test]
fn export_observability_ignores_declarations_inside_exported_arrow() {
    test_smallest(
        "export default () => { function nested() {} nested(); }; function dead1() { dead2() } function dead2() { dead1() }",
        "export default () => {};",
    );
}

// ---- CommonJS and script sources ----

#[test]
fn analyze_commonjs_and_script_local_functions() {
    let options = CompressOptions::smallest();
    let cycle = "function c() {\n\td();\n}\nfunction d() {\n\tc();\n}\nconsole.log(\"k\");";
    test_options_source_type(cycle, "console.log(\"k\");", SourceType::cjs(), &options);
    test_options_source_type("{ function f() { f() } }", "", SourceType::cjs(), &options);
    // `g` has references only outside registered functions, so counts own its
    // lifecycle. Once the false branch and then `g` disappear, dropping `g`'s
    // body reference wakes the graph and exposes recursive `f`.
    test_options_source_type(
        "if (false) g(); function g() { f() } function f() { f() }",
        "",
        SourceType::cjs(),
        &options,
    );

    // A strict block binding and bindings local to a function are not visible
    // to later script tags.
    test_options_source_type(
        "\"use strict\"; { function f() { f() } }",
        "\"use strict\";",
        SourceType::script(),
        &options,
    );
    test_options_source_type(
        "function outer() { function c() { d() } function d() { c() } return 1 }",
        "function outer() { return 1 }",
        SourceType::script(),
        &options,
    );
}

#[test]
fn keep_commonjs_references_and_script_global_functions() {
    let options = CompressOptions::smallest();
    // CommonJS export assignments keep `f` through ordinary resolved
    // references; they do not use the Script-root observability rule below.
    test_same_options_source_type(
        "function f() { f() } module.exports = f;",
        SourceType::cjs(),
        &options,
    );
    test_same_options_source_type(
        "function f() { f() } exports.f = f;",
        SourceType::cjs(),
        &options,
    );

    // Script-root declarations are visible to later script tags. A simple
    // sloppy block function is Annex B-hoisted to that observable root binding;
    // duplicate/unhoisted safety is covered by the following test.
    test_same_options_source_type("function f() { f() }", SourceType::script(), &options);
    test_same_options_source_type("{ function f() { f() } }", SourceType::script(), &options);
    test_options_source_type(
        "function f() {} function outer() { function d1() { console.log(f); d2() } function d2() { d1() } return 1 } f.x = 1;",
        "function f() {} function outer() { return 1 } f.x = 1;",
        SourceType::script(),
        &options,
    );
}

#[test]
fn keep_sloppy_duplicate_block_functions() {
    let options = CompressOptions::smallest();
    let source =
        "{ function f() { return 1 } } { function f() { return f } } console.log(typeof f());";
    for source_type in [SourceType::script(), SourceType::cjs()] {
        test_same_options_source_type(source, source_type, &options);
    }
    test_same_options_source_type(
        "{ function f() { return f } } { function f() { return f } } console.log(typeof f());",
        SourceType::ts().with_script(true),
        &options,
    );

    // Strict block functions have no Annex B var alias and remain removable.
    test_options_source_type(
        "'use strict'; { function f() { f() } }",
        "'use strict';",
        SourceType::cjs(),
        &options,
    );
    test_options_source_type(
        "'use strict'; { function f() { f() } }",
        "'use strict';",
        SourceType::ts().with_script(true),
        &options,
    );
}

#[test]
fn keep_sloppy_annex_b_alias_member_write_after_cycle_removed() {
    let options = CompressOptions::smallest();
    let source = "function outer() { { function f() {} } { function f() {} f.x = 1; function d1() { consume(f); d2() } function d2() { d1() } } console.log(f.x); } outer();";
    let expected = "function outer() { { function f() {} } { function f() {} f.x = 1; } console.log(f.x); } outer();";
    for source_type in [SourceType::script(), SourceType::cjs()] {
        test_options_source_type(source, expected, source_type, &options);
    }
}

#[test]
fn keep_script_root_var_in_nested_statement_after_cycle_removed() {
    let options = CompressOptions::smallest();
    let source = "function outer() { function d1() { return x + d2() } function d2() { return d1() } return 1 } outer(); switch (1) { case 1: var x = 42; }";
    // Script globals can be rebound through global-object properties without a
    // resolved write reference, so the call cannot reuse a pure summary.
    let expected = "function outer() { return 1 } switch (outer(), 1) { case 1: var x = 42; }";
    test_options_source_type(source, expected, SourceType::script(), &options);

    // CommonJS top-level vars are wrapper-local, so ordinary counts may remove them.
    test_options_source_type("{ var x = 42; }", "", SourceType::cjs(), &options);
}

// ---- Direct eval ----

// A dropped direct eval must re-enable the analysis: the initial compute
// skips the whole program while the root scope carries `DirectEval`, so only
// the `direct_eval_dropped` recompute trigger lets a later pass remove the cycle.
#[test]
fn remove_recursive_function_after_direct_eval_dropped() {
    test_smallest("if (false) eval('x'); function f() { f() }", "");

    let options = CompressOptions::smallest();
    test_options_source_type(
        "if (false) eval('x'); function f() { f() }",
        "",
        SourceType::cjs(),
        &options,
    );
    test_options_source_type(
        "function outer() { if (false) eval('x'); function f() { f() } return 1 }",
        "function outer() { return 1 }",
        SourceType::script(),
        &options,
    );
}

#[test]
fn keep_non_module_recursive_functions_reachable_by_eval() {
    let options = CompressOptions::smallest();
    test_same_options_source_type("eval('f()'); function f() { f() }", SourceType::cjs(), &options);
    test_same_options_source_type(
        "function outer() { eval('f()'); function f() { f() } }",
        SourceType::script(),
        &options,
    );
}

// ---- Other binding contexts and options ----

#[test]
fn keep_recursive_cycle_in_for_in_head() {
    // No removal site handles for-in/of head declarators, so the head
    // survives; its Annex-B initializer must keep referencing a live `p`.
    let options = CompressOptions::smallest();
    let source_type = SourceType::cjs().with_script(true);
    test_same_options_source_type(
        "function o() { var p = function() { console.log(x) }; for (var x = p in {}); return 1; } g(o());",
        source_type,
        &options,
    );
}

// For-head bindings need no special pin. References in the RHS participate in
// normal scope ownership, while unreachable heads disappear through the
// ordinary removed-reference lifecycle.
#[test]
fn for_head_reachability_uses_ordinary_references() {
    test_same_smallest("function f() {\n\tf();\n}\nfor (var x of [f]);");
    test_smallest(
        "function f() { f() } for (var unused in object);",
        "for (var unused in object);",
    );
    test_smallest(
        "export function f() { return 1; for (const x of arr) g(x); }",
        "export function f() {\n\treturn 1;\n}",
    );
    test_smallest(
        "var f = 1; function d1() { f; d2() } function d2() { d1() } if (false) for (var f of xs) {} export {};",
        "export {};",
    );
    test_smallest("if (false) for (var f of xs) {} f = 1; export {};", "export {};");
}

// References in `using` initializers participate normally in function
// reachability; no separate pin is needed.
#[test]
fn keep_using_declarator_cycle() {
    test_same_smallest(
        "function o() { using u = p; var p = function() { u() }; return 1 } g(o());",
    );
    test_same_smallest(
        "async function o() { await using u = p; var p = function() { u() }; return 1 } g(o());",
    );
    test_same_smallest("function f() {\n\tf();\n}\nusing resource = f;");
}

#[test]
fn keep_using_member_write_observed_by_disposal() {
    let disposer =
        "using resource = { [Symbol.dispose]() { console.log(this.x) } }; resource.x = 1;";
    test_same_options(
        disposer,
        &CompressOptions {
            unused: CompressOptionsUnused::Keep,
            treeshake: TreeShakeOptions {
                property_write_side_effects: false,
                ..TreeShakeOptions::default()
            },
            ..CompressOptions::smallest()
        },
    );
    test_same_smallest(disposer);
    test_same_smallest(
        "await using resource = { [Symbol.asyncDispose]() { console.log(this.x) } }; resource.x = 1;",
    );
    test_smallest(
        "{ using resource = { [Symbol.dispose]() { console.log(this.x) } }; resource.x = 1; function d1() { consume(resource); d2() } function d2() { d1() } }",
        "{ using resource = { [Symbol.dispose]() { console.log(this.x) } }; resource.x = 1; }",
    );
}

#[test]
fn keep_recursive_function_with_unused_keep_option() {
    let options =
        CompressOptions { unused: CompressOptionsUnused::Keep, ..CompressOptions::smallest() };
    test_same_options("function f() { f() }", &options);
    test_same_options("const f = () => f()", &options);
    // The graph is disabled, but export observability still protects the
    // adjacent-declarator single-use substitution path.
    test_same_options("export var f = side(), g = f; use(g);", &options);
    // Non-ESM sources do not need observability metadata when their graph is
    // disabled, but behavior remains identical.
    test_same_options_source_type("function f() { f() }", SourceType::cjs(), &options);
    test_same_options_source_type(
        "function outer() { function f() { f() } }",
        SourceType::script(),
        &options,
    );
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
    test_same_options("import { a } from 'a'; function f() { eval('a'); }", &options);
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
