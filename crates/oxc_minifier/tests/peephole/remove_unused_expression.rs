use oxc_ecmascript::side_effects::PropertyReadSideEffects;

use crate::{
    CompressOptions, CompressOptionsUnused, TreeShakeOptions, default_options, test, test_options,
    test_options_source_type, test_same, test_same_options, test_same_options_source_type,
};

#[test]
fn test_remove_unused_expression() {
    test("null", "");
    test("true", "");
    test("false", "");
    test("1", "");
    test("1n", "");
    test(";'s'", "");
    // test("this", "");
    test("/asdf/", "");
    test("(function () {})", "");
    test("(() => {})", "");
    test("import.meta", "");
    test("var x; x", "var x");
    test("x", "x");
    test("void 0", "");
    test("void x", "x");
}

#[test]
fn test_new_constructor_side_effect() {
    test("new WeakSet()", "");
    test("new WeakSet(null)", "");
    test("new WeakSet(void 0)", "");
    test("new WeakSet([])", "");
    test_same("new WeakSet([x])");
    test_same("new WeakSet(x)");
    test_same("throw new WeakSet()");
    test("new WeakMap()", "");
    test("new WeakMap(null)", "");
    test("new WeakMap(void 0)", "");
    test("new WeakMap([])", "");
    test_same("new WeakMap([x])");
    test_same("new WeakMap(x)");
    test("new Date()", "");
    test("new Date('')", "");
    test("new Date(0)", "");
    test("new Date(null)", "");
    test("new Date(true)", "");
    test("new Date(false)", "");
    test("new Date(undefined)", "");
    test_same("new Date(x)");
    test("new Set()", "");
    // test("new Set([a, b, c])", "");
    test("new Set(null)", "");
    test("new Set(undefined)", "");
    test("new Set(void 0)", "");
    test_same("new Set(x)");
    test("new Map()", "");
    test("new Map(null)", "");
    test("new Map(undefined)", "");
    test("new Map(void 0)", "");
    // test_same("new Map([x])");
    test_same("new Map(x)");
    // test("new Map([[a, b], [c, d]])", "");
}

#[test]
fn test_array_literal() {
    test("([])", "");
    test("([1])", "");
    test("([a])", "a");
    test("var a; ([a])", "var a;");
    test("([foo()])", "foo()");
    test("[[foo()]]", "foo()");
    test_same("baz.map((v) => [v])");
}

#[test]
fn test_array_literal_containing_spread() {
    test_same("([...c])");
    test("([4, ...c, a])", "[...c, a]");
    test("var a; ([4, ...c, a])", "var a; [...c]");
    test_same("([foo(), ...c, bar()])");
    test_same("([...a, b, ...c])");
    test("var b; ([...a, b, ...c])", "var b; [...a, ...c]");
    test_same("([...b, ...c])"); // It would also be fine if the spreads were split apart.
}

#[test]
fn test_fold_unary_expression_statement() {
    test("typeof x", "");
    test("typeof x?.y", "x?.y");
    test("typeof x.y", "x.y");
    test("typeof x.y.z()", "x.y.z()");
    test("void x", "x");
    test("void x?.y", "x?.y");
    test("void x.y", "x.y");
    test("void x.y.z()", "x.y.z()");

    test("!x", "x");
    test("!x?.y", "x?.y");
    test("!x.y", "x.y");
    test("!x.y.z()", "x.y.z()");
    test_same("-x.y.z()");

    test_same("delete x");
    test_same("delete x.y");
    test_same("delete x.y.z()");
    test_same("+0n"); // Uncaught TypeError: Cannot convert a BigInt value to a number
    test("-0n", "");
    test("-1n", "");
}

#[test]
fn test_fold_sequence_expr() {
    test("('foo', 'bar', 'baz')", "");
    test("('foo', 'bar', baz())", "baz()");
    test("('foo', bar(), baz())", "bar(), baz()");
    test("(() => {}, bar(), baz())", "bar(), baz()");
    test("(function k() {}, k(), baz())", "k(), baz()");
    test_same("(0, o.f)();");
    test("var obj = Object((null, 2, 3), 1, 2);", "var obj = Object(3, 1, 2);");
    test_same("(0 instanceof 0, foo)");
    test_same("(0 in 0, foo)");
    test_same(
        "React.useEffect(() => (isMountRef.current = !1, () => { isMountRef.current = !0; }), [])",
    );
}

#[test]
fn test_logical_expression() {
    test("var a; a != null && a.b()", "var a; a?.b()");
    test("var a; a == null || a.b()", "var a; a?.b()");
    test_same("a != null && a.b()"); // a may have a getter
    test_same("a == null || a.b()"); // a may have a getter
    test("var a; null != a && a.b()", "var a; a?.b()");
    test("var a; null == a || a.b()", "var a; a?.b()");

    test("x == null && y", "x ?? y");
    test("x != null || y", "x ?? y");
    test_same("v = x == null && y");
    test_same("v = x != null || y");
    test("a == null && (a = b)", "a ??= b");
    test("a != null || (a = b)", "a ??= b");
    test_same("v = a == null && (a = b)");
    test_same("v = a != null || (a = b)");
    test("void (x == null && y)", "x ?? y");

    // https://github.com/oxc-project/oxc/pull/16802#discussion_r2619369597
    // Don't transform to ??= when base object may be mutated, but ?? is safe
    test("var x = {}; x.y != null || (x = {}, x.y = 3)", "var x = {}; x.y ?? (x = {}, x.y = 3)");
    test("var x = {}; x.y == null && (x = {}, x.y = 3)", "var x = {}; x.y ?? (x = {}, x.y = 3)");
    test(
        "var x = {}; x.y != null || (a, x = {}, x.y = 3)",
        "var x = {}; x.y ?? (a, x = {}, x.y = 3)",
    );
    test(
        "var x = { y: {} }; x.y.z != null || (x.y = {}, x.y.z = 3)",
        "var x = { y: {} }; x.y.z ?? (x.y = {}, x.y.z = 3)",
    );
    // Safe to transform to ??= when base object is not mutated
    test("var x = {}; x.y != null || (foo(), x.y = 3)", "var x = {}; x.y ??= (foo(), 3)");
    test("var x = {}; x.y != null || (new Foo(), x.y = 3)", "var x = {}; x.y ??= (new Foo(), 3)");
    // x is not mutated, only x.y.z is assigned (doesn't affect x)
    test("var x = {}; x.y != null || (x.y.z = {}, x.y = 3)", "var x = {}; x.y ??= (x.y.z = {}, 3)");

    test("typeof x != 'undefined' && x", "");
    test("typeof x == 'undefined' || x", "");
    test("typeof x < 'u' && x", "");
    test("typeof x > 'u' || x", "");
}

// Regression tests for https://github.com/oxc-project/oxc/issues/21457.
//
// When `a == null && (a = b)` is converted to `a ??= b`, the LHS reference
// must be flagged as Read; otherwise unused-removal sees zero read references
// on the next iteration and strips the assignment, dropping the nullish guard.
//
// Each sub-case is a separate test so one regression doesn't mask the others.
#[test]
fn test_nullish_assign_preserves_guard() {
    let options = CompressOptions::smallest();
    test_options(
        "let rafId; export function foo() { if (rafId == null) { rafId = requestAnimationFrame(() => { console.log('callback'); }); } }",
        "let rafId; export function foo() { rafId ??= requestAnimationFrame(() => { console.log('callback'); }); }",
        &options,
    );
    test_options(
        "let rafId; export function foo() { if (rafId != null) {} else { rafId = requestAnimationFrame(() => { console.log('callback'); }); } }",
        "let rafId; export function foo() { rafId ??= requestAnimationFrame(() => { console.log('callback'); }); }",
        &options,
    );
    test_options(
        "let a; export function foo() { a == null && (a = compute()); }",
        "let a; export function foo() { a ??= compute(); }",
        &options,
    );
    test_options(
        "let a; export function foo() { a != null || (a = compute()); }",
        "let a; export function foo() { a ??= compute(); }",
        &options,
    );
    // Member LHS goes through `remove_unused_member_assignment`, not the
    // identifier path, so it was never affected by the reference-flag bug.
    // Still covered here to pin down expected behavior under `smallest()`.
    test_options(
        "export let o = {}; export function foo() { o.y == null && (o.y = compute()); }",
        "export let o = {}; export function foo() { o.y ??= compute(); }",
        &options,
    );
}

#[expect(clippy::literal_string_with_formatting_args)]
#[test]
fn test_object_literal() {
    test("({})", "");
    test("({a:1})", "");
    test("({a:foo()})", "foo()");
    test("({'a':foo()})", "foo()");
    // Object-spread may trigger getters.
    test_same("({...a})");
    test_same("({...foo()})");
    // Spreading object literals is safe if contents are safe.
    test("({...{}})", "");
    test("({...{a: 1}})", "");
    test("({...{a: foo()}})", "foo()");
    test("({ [{ foo: foo() }]: 0 })", "foo()");
    test("({ foo: { foo: foo() } })", "foo()");

    test("({ [bar()]: foo() })", "bar(), foo()");
    test("({ ...baz, [bar()]: foo() })", "({ ...baz }), bar(), foo()");
}

#[test]
fn test_fold_template_literal() {
    test("`a${b}c${d}e`", "`${b}${d}`");
    test("`stuff ${x} ${1}`", "`${x}`");
    test("`stuff ${1} ${y}`", "`${y}`");
    test("`stuff ${x} ${y}`", "`${x}${y}`");
    test("`stuff ${x ? 1 : 2} ${y}`", "x, `${y}`");
    test("`stuff ${x} ${y ? 1 : 2}`", "`${x}`, y");
    test("`stuff ${x} ${y ? 1 : 2} ${z}`", "`${x}`, y, `${z}`");

    test("`4${c}${+a}`", "`${c}`, +a");
    test("`${+foo}${c}${+bar}`", "+foo, `${c}`, +bar");
    test("`${a}${+b}${c}`", "`${a}`, +b, `${c}`");
}

#[test]
fn test_fold_conditional_expression() {
    test("(1, foo()) ? 1 : 2", "foo()");
    test("foo() ? 1 : 2", "foo()");
    test("foo() ? 1 : bar()", "foo() || bar()");
    test("foo() ? bar() : 2", "foo() && bar()");
    test_same("foo() ? bar() : baz()");

    test("typeof x == 'undefined' ? 0 : x", "");
    test("typeof x != 'undefined' ? x : 0", "");
    test("typeof x > 'u' ? 0 : x", "");
    test("typeof x < 'u' ? x : 0", "");
}

#[test]
fn test_fold_binary_expression() {
    test("var a, b; a === b", "var a, b;");
    test("var a, b; a() === b", "var a, b; a()");
    test("var a, b; a === b()", "var a, b; b()");
    test("var a, b; a() === b()", "var a, b; a(), b()");

    test("var a, b; a !== b", "var a, b;");
    test("var a, b; a == b", "var a, b;");
    test("var a, b; a != b", "var a, b;");
    test("var a, b; a < b", "var a, b;");
    test("var a, b; a > b", "var a, b;");
    test("var a, b; a <= b", "var a, b;");
    test("var a, b; a >= b", "var a, b;");

    test_same("var a, b; a + b");
    test("var a, b; 'a' + b", "var a, b; '' + b");
    test_same("var a, b; a + '' + b");
    test("var a, b, c; 'a' + (b === c)", "var a, b, c;");
    test("var a, b; 'a' + +b", "var a, b; '' + +b"); // can be improved to "var a, b; +b"
    test_same("var a, b; a + ('' + b)");
    test("var a, b, c; a + ('' + (b === c))", "var a, b, c; a + ''");
}

#[test]
fn test_fold_call_expression() {
    test_same("foo()");
    test("/* @__PURE__ */ foo()", "");
    test("/* @__PURE__ */ foo(a)", "a");
    test("/* @__PURE__ */ foo(a, b)", "a, b");
    test("/* @__PURE__ */ foo(...a)", "[...a]");
    test("/* @__PURE__ */ foo(...'a')", "");
    test("/* @__PURE__ */ new Foo()", "");
    test("/* @__PURE__ */ new Foo(a)", "a");
    test("true && /* @__PURE__ */ noEffect()", "");
    test("false || /* @__PURE__ */ noEffect()", "");

    test("var foo = () => 1; foo(), foo()", "var foo = () => 1");
    test_same("var foo = () => { bar() }; foo(), foo()");
    test_same("const a = (x) => x, b = () => a(1);");
}

#[test]
fn test_fold_iife() {
    test_same("var k = () => {}");
    test_same("var k = function () {}");
    test("var a = (() => {})()", "var a = void 0;");
    test("(() => {})()", "");
    test("(() => a())()", "a();");
    test("(() => { a() })()", "a();");
    test("(() => { return a() })()", "a();");
    test_same("(a => {})()");
    test_same("((a = foo()) => {})()");
    test_same("(a => { a() })()");
    test("((...a) => {})()", "");
    test_same("((...a) => { a() })()");
    test("(() => { let b = a; b() })()", "a();");
    test("(() => { let b = a; return b() })()", "a();");
    test("(async () => {})()", "");
    test_same("(async () => { a() })()");
    test("(async () => { let b = a; b() })()", "(async () => { a() })();");
    test("var a = (function() {})()", "var a = void 0;");
    test("a((() => b())());", "a(b())");
    test("a((() => true)());", "a(!0)");
    test("a((() => { return true })());", "a(!0)");

    test_same("var a = (function () { b() })()");
    test_same("var a = (function () { return b() })()");
    test_same("var a = (function () { return this })()");
    test_same("var a = (function () { return arguments })()");
    test_same("var a = (function () { return new.target })()");
    test_same("var a = (function () { return !0 })()");
    test_same("a((function () { return !0 })());");
    test("(function() {})()", "");
    test("(function*() {})()", "");
    test("(async function() {})()", "");
    test_same("(function() { a() })()");
    test_same("(function*() { a() })()");
    test_same("(async function() { a() })()");

    test("(() => x)()", "x;");
    test("(() => { return x })()", "x;");
    test_same("(function () { return x })()");

    test("var a = /* @__PURE__ */ (() => x)()", "var a = x");
    test_same("var a = /* @__PURE__ */ (() => x)(y, z)");
    test("(/* @__PURE__ */ (() => !0)() ? () => x() : () => {})();", "x();");
    test("/* @__PURE__ */ (() => x)()", "");
    test("/* @__PURE__ */ (() => { return x })()", "");
    test("/* @__PURE__ */ (() => x)(y, z)", "y, z;");

    test(
        "function foo(x) { if (x) { return /* @__PURE__ */ (() => 42)() } return x }",
        "function foo(x) { return x && 42 }",
    );
    test(
        "function foo(x) { if (x) { return /* @__PURE__ */ (() => bar())() } return x }",
        "function foo(x) { return x && bar() }",
    );
    test(
        "function foo(x) { if (x) { return /* @__PURE__ */ (() => { return 42 })() } return x }",
        "function foo(x) { return x && 42 }",
    );
    test("/* @__PURE__ */ (() => 42)()", "");
    test("function foo() { /* @__PURE__ */ (() => 42)() }", "function foo() {}");
    test(
        "function foo(x) { if (x) { return (/* @__PURE__ */ (() => 42)(), foo) } return x }",
        "function foo(x) { return x && foo }",
    );
}

#[test]
fn no_side_effects() {
    fn check(source_text: &str) {
        let input = format!("{source_text}; f()");
        test(&input, source_text);

        let input = format!("{source_text}; new f()");
        test(&input, source_text);

        // TODO https://github.com/evanw/esbuild/issues/3511
        // let input = format!("{source_text}; html``");
        // test(&input, source_text);
    }
    check("/* @__NO_SIDE_EFFECTS__ */ function f() {}");
    check("/* @__NO_SIDE_EFFECTS__ */ export function f() {}");
    check("/* @__NO_SIDE_EFFECTS__ */ export default function f() {}");
    check("export default /* @__NO_SIDE_EFFECTS__ */ function f() {}");
    check("const f = /* @__NO_SIDE_EFFECTS__ */ function() {}");
    check("export const f = /* @__NO_SIDE_EFFECTS__ */ function() {}");
    check("/* @__NO_SIDE_EFFECTS__ */ const f = function() {}");
    check("/* @__NO_SIDE_EFFECTS__ */ export const f = function() {}");
    check("const f = /* @__NO_SIDE_EFFECTS__ */ () => {}");
    check("export const f = /* @__NO_SIDE_EFFECTS__ */ () => {}");
    check("/* @__NO_SIDE_EFFECTS__ */ const f = () => {}");
    check("/* @__NO_SIDE_EFFECTS__ */ export const f = () => {}");
}

#[test]
fn treeshake_options_annotations_false() {
    let options = CompressOptions {
        treeshake: TreeShakeOptions { annotations: false, ..TreeShakeOptions::default() },
        ..default_options()
    };
    test_same_options("function test() { bar } /* @__PURE__ */ test()", &options);
    test_same_options("function test() {} /* @__PURE__ */ new test()", &options);

    let options = CompressOptions {
        treeshake: TreeShakeOptions { annotations: true, ..TreeShakeOptions::default() },
        ..default_options()
    };
    test_options("function test() {} /* @__PURE__ */ test()", "function test() {}", &options);
    test_options("function test() {} /* @__PURE__ */ new test()", "function test() {}", &options);
}

#[test]
fn remove_unused_assignment_expression() {
    use oxc_span::SourceType;
    let options = CompressOptions::smallest();
    test_options("var x = 1; x = 2;", "", &options);
    test_options("var x = 1; x = foo();", "foo()", &options);
    test_same_options("var x = 1; x = 2, eval('x')", &options);
    test_same_options("export var foo; foo = 0;", &options);
    test_same_options("var x = 1; x = 2, foo(x)", &options);
    test_same_options("function foo() { return t = x(); } foo();", &options);
    test_options(
        "function foo() { var t; return t = x(); } foo();",
        "function foo() { return x(); } foo();",
        &options,
    );
    test_same_options("function foo(t) { return t = x(); } foo();", &options);

    test_options("let x = 1; x = 2;", "", &options);
    test_options("let x = 1; x = foo();", "foo()", &options);
    test_same_options("export let foo; foo = 0;", &options);
    test_same_options("let x = 1; x = 2, foo(x)", &options);
    test_same_options("function foo() { return t = x(); } foo();", &options);
    test_options(
        "function foo() { let t; return t = x(); } foo();",
        "function foo() { return x() } foo()",
        &options,
    );
    test_same_options("function foo(t) { return t = x(); } foo();", &options);

    // For loops
    test_options("for (let i;;) i = 0", "for (;;);", &options);
    test_options("for (let i;;) foo(i)", "for (;;) foo(void 0)", &options);
    test_same_options("for (let i;;) i = 0, foo(i)", &options);
    test_same_options("for (let i in []) foo(i)", &options);
    test_same_options("for (let element of list) element && (element.foo = bar)", &options);
    test_same_options("for (let key in obj) key && (obj[key] = bar)", &options);

    test_options("var a; ({ a: a } = {})", "var a; ({ a } = {})", &options);
    test_options("var a; b = ({ a: a })", "var a; b = ({ a })", &options);

    test_options("let foo = {}; foo = 1", "", &options);

    test_same_options(
        "let bracketed = !1; for(;;) bracketed = !bracketed, log(bracketed)",
        &options,
    );

    let options = CompressOptions::smallest();
    let source_type = SourceType::cjs().with_script(true);
    test_same_options_source_type("var x = 1; x = 2;", source_type, &options);
    test_same_options_source_type("var x = 1; x = 2, foo(x)", source_type, &options);
    test_options_source_type(
        "function foo() { var x = 1; x = 2, bar() } foo()",
        "function foo() { bar() } foo()",
        source_type,
        &options,
    );
}

#[test]
fn remove_unused_class_expression() {
    let options = CompressOptions::smallest();
    // extends
    test_options("(class {})", "", &options);
    test_options("(class extends Foo {})", "Foo", &options);

    // static block
    test_options("(class { static {} })", "", &options);
    test_same_options("(class { static { foo } })", &options);

    // method
    test_options("(class { foo() {} })", "", &options);
    test_options("(class { [foo]() {} })", "foo", &options);
    test_options("(class { static foo() {} })", "", &options);
    test_options("(class { static [foo]() {} })", "foo", &options);
    test_options("(class { [1]() {} })", "", &options);
    test_options("(class { static [1]() {} })", "", &options);

    // property
    test_options("(class { foo })", "", &options);
    test_options("(class { foo = bar })", "", &options);
    test_options("(class { foo = 1 })", "", &options);
    // TODO: would be nice if this is removed but the one with `this` is kept.
    test_same_options("(class { static foo = bar })", &options);
    test_same_options("(class { static foo = this.bar = {} })", &options);
    test_options("(class { static foo = 1 })", "", &options);
    test_options("(class { [foo] = bar })", "foo", &options);
    test_options("(class { [foo] = 1 })", "foo", &options);
    test_same_options("(class { static [foo] = bar })", &options);
    test_options("(class { static [foo] = 1 })", "foo", &options);

    // accessor
    test_options("(class { accessor foo = 1 })", "", &options);
    test_options("(class { accessor [foo] = 1 })", "foo", &options);

    // order
    test_options("(class extends A { [B] = C; [D]() {} })", "A, B, D", &options);

    // decorators
    test_same_options("(class { @dec foo() {} })", &options);
    test_same_options("(@dec class {})", &options);

    // TypeError
    test_same_options("(class extends (() => {}) {})", &options);
}

#[test]
fn test_property_write_side_effects() {
    let options = CompressOptions {
        unused: CompressOptionsUnused::Remove,
        treeshake: TreeShakeOptions {
            property_write_side_effects: false,
            property_read_side_effects: PropertyReadSideEffects::None,
            ..TreeShakeOptions::default()
        },
        ..CompressOptions::smallest()
    };

    // Issue #14207: drop function declarations with property assignments
    test_options("function A() {} A.from = () => {};", "", &options);

    // Function declaration + multiple property assignments
    test_options("function A() {} A.foo = 1; A.bar = 2;", "", &options);

    // Class declaration + property assignment
    test_options("class A {} A.foo = 1;", "", &options);

    // Property write is kept when variable is read elsewhere (statement fusion merges them)
    test_options(
        "function A() {} A.foo = 1; console.log(A);",
        "function A() {} A.foo = 1, console.log(A);",
        &options,
    );

    // Should keep if the assignment RHS has side effects
    test_same_options("function A() {} A.foo = sideEffect();", &options);

    // Property assignment on global (not local binding) should be kept
    test_same_options("globalObj.foo = 1;", &options);

    // Object literal + property assignment (fresh value, safe to drop)
    test_options("const B = {}; B.foo = 1;", "", &options);

    // Arrow function + property assignment (fresh value, safe to drop)
    test_options("const C = () => {}; C.foo = 1;", "", &options);

    // Function expression + property assignment (fresh value, safe to drop)
    test_options("const D = function() {}; D.foo = 1;", "", &options);

    // Variable initialized from another binding (not fresh, could alias)
    test_same_options("const b = a; b.foo = 1;", &options);

    // Alias where nothing is exported: inlining resolves alias, then everything drops
    test_options("const a = {}; const b = a; b.foo = 1;", "", &options);

    // Alias where target is exported: must preserve the property write
    test_options(
        "const a = {}; const b = a; b.add = 1; export { a };",
        "const a = {}, b = a; b.add = 1; export { a };",
        &options,
    );
    test_options(
        "const a = {}; const b = a; b.add = 1; export { b };",
        "const b = {}; b.add = 1; export { b };",
        &options,
    );
    test_options(
        "const a = {}; const b = a; a.add = 1; export { b };",
        "const a = {}, b = a; a.add = 1; export { b };",
        &options,
    );

    // Chained member expression: b.a.add = 1 must be preserved
    // because b.a could alias exported a
    test_options(
        "const a = {}; const b = { a }; b.a.add = 1; export { a };",
        "const a = {}, b = { a }; b.a.add = 1; export { a };",
        &options,
    );

    // Exported function: property write must be preserved (observable by importers)
    test_same_options("export function A() {} A.foo = 1;", &options);

    // Classes with static setters should NOT be dropped — setters trigger side effects
    test_same_options("class A { static set foo(v) { console.log(v); } } A.foo = 1;", &options);

    // Object literals with setters should NOT be dropped
    test_same_options("const obj = { set foo(v) { console.log(v); } }; obj.foo = 1;", &options);

    // Class expression with static setter should NOT be dropped
    test_same_options(
        "const A = class { static set foo(v) { console.log(v); } }; A.foo = 1;",
        &options,
    );

    // Static accessor auto-generates setter — must NOT be dropped
    test_same_options("class A { static accessor foo = 0; } A.foo = 1;", &options);

    // Any static property with a value prevents removal (matches SWC behavior)
    test_same_options("class A { static b = 0; } A.b = 1;", &options);

    // Static property whose value contains a setter — must NOT be dropped
    test_same_options(
        "class A { static b = { set x(v) { console.log(v); } }; } A.b = 1;",
        &options,
    );

    // Object literal with nested setter in property value
    test_same_options(
        "const obj = { bar: { set x(v) { console.log(v); } } }; obj.bar = 1;",
        &options,
    );

    // Deeply nested setter in property value (depth 2+)
    test_same_options(
        "const obj = { bar: { baz: { set x(v) { console.log(v); } } } }; obj.bar = 1;",
        &options,
    );

    // Inherited static setter via extends — B.foo triggers A's static setter
    // We can't statically detect inherited setters, but B extends A means
    // B has a read reference to A, so A is preserved. B itself is fresh
    // (no own static setters), but the extends clause is a side effect.
    test_same_options(
        "class A { static set foo(v) { console.log(v); } } class B extends A {} B.foo = 1;",
        &options,
    );

    // Object.defineProperty installs setter dynamically — foo has a read reference
    // in the first argument, so foo is not considered unused
    test_options(
        "const foo = () => {}; Object.defineProperty(foo, 'bar', { set: (v) => { console.log(v); } }); foo.bar = 1;",
        "const foo = () => {}; Object.defineProperty(foo, 'bar', { set: (v) => { console.log(v); } }), foo.bar = 1;",
        &options,
    );
    test_options(
        "const foo = []; Object.defineProperty(foo, 'bar', { set: (v) => { console.log(v); } }); foo.bar = 1;",
        "const foo = []; Object.defineProperty(foo, 'bar', { set: (v) => { console.log(v); } }), foo.bar = 1;",
        &options,
    );

    // Non-static setters are fine — property writes on the class itself won't trigger them
    test_options("class A { set foo(v) { console.log(v); } } A.bar = 1;", "", &options);

    // Static getter (not setter) is fine to drop
    test_options("class A { static get foo() { return 1; } } A.bar = 1;", "", &options);

    // __proto__ assignment can install setters that make subsequent property writes
    // side-effectful. When both __proto__ write and property write exist, preserve all.
    test_options(
        "const a = {}; a.__proto__ = { set a(v) { console.log('setter'); } }; a.a = 1;",
        "const a = {}; a.__proto__ = { set a(v) { console.log('setter'); } }, a.a = 1;",
        &options,
    );
    test_options(
        "class A {} A.__proto__ = { set a(v) { console.log('setter'); } }; A.a = 1;",
        "class A {} A.__proto__ = { set a(v) { console.log('setter'); } }, A.a = 1;",
        &options,
    );
    // __proto__ write alone (no subsequent property write) — safe to drop,
    // the setter is installed but never triggered.
    test_options(
        "const a = {}; a.__proto__ = { set a(v) { console.log('setter'); } };",
        "",
        &options,
    );
    // Property write alone (no __proto__) — safe to drop, no setter exists.
    test_options("const a = {}; a.a = 1;", "", &options);

    // Computed member expression could be `"__proto__"`, must be treated as potential proto write
    test_options(
        "const a = {}; a[b] = { set a(v) { console.log('setter'); } }; a.a = 1;",
        "const a = {}; a[b] = { set a(v) { console.log('setter'); } }, a.a = 1;",
        &options,
    );

    // `__proto__` in object literal initializer installs setters via prototype chain
    test_same_options(
        "const a = { __proto__: { set a(v) { console.log('setter'); } } }; a.a = 1;",
        &options,
    );
    test_options(
        "class A {} A.__proto__ = { set a(v) { console.log('setter'); } }; A.a = 1;",
        "class A {} A.__proto__ = { set a(v) { console.log('setter'); } }, A.a = 1;",
        &options,
    );

    // TODO: `__proto__` assignment inside a hoisted function — setter is installed when f() is called.
    // This case is not handled yet because the `__proto__` write inside the function body
    // is encountered after `obj.a = 1` during traversal. Uncomment when two-pass tracking
    // is implemented.
    // test_same_options(
    //     "const obj = {}; f(); obj.a = 1; function f() { obj.__proto__ = { set a(v) { console.log('hello'); } }; }",
    //     &options,
    // );

    // Default options (property_write_side_effects: true) should NOT drop these
    let default_opts =
        CompressOptions { unused: CompressOptionsUnused::Remove, ..CompressOptions::smallest() };
    test_same_options("function A() {} A.from = () => {};", &default_opts);
}

#[test]
fn test_update_expression_respects_property_read_side_effects() {
    // `obj.prop++` performs an implicit read, so it's side-effectful when
    // `property_read_side_effects` is `All` — even if writes are free.
    let options = CompressOptions {
        unused: CompressOptionsUnused::Remove,
        treeshake: TreeShakeOptions {
            property_write_side_effects: false,
            property_read_side_effects: PropertyReadSideEffects::All,
            ..TreeShakeOptions::default()
        },
        ..CompressOptions::smallest()
    };

    test_options(
        "import { counter } from './c'; counter.value++; console.log(counter);",
        "import { counter } from './c'; counter.value++, console.log(counter);",
        &options,
    );
    test_options(
        "import { counter } from './c'; ++counter.count; console.log(counter);",
        "import { counter } from './c'; ++counter.count, console.log(counter);",
        &options,
    );
    test_options(
        "import { counter } from './c'; counter['another']--; console.log(counter);",
        "import { counter } from './c'; counter.another--, console.log(counter);",
        &options,
    );

    // Static block runs on class evaluation.
    test_options(
        "import { counter } from './c'; (class { static { ++counter.count; } }); console.log(counter);",
        "import { counter } from './c'; (class { static { ++counter.count; } }), console.log(counter);",
        &options,
    );

    // Computed key runs on class evaluation; class body is unused, so only the key's side effect is extracted.
    test_options(
        "import { counter } from './c'; class A { [counter.another++] = 123; } console.log(counter);",
        "import { counter } from './c'; counter.another++, console.log(counter);",
        &options,
    );
}
