use oxc_ecmascript::side_effects::PropertyReadSideEffects;
use oxc_span::SourceType;

use crate::{
    CompressOptions, CompressOptionsUnused, TreeShakeOptions, default_options, test, test_options,
    test_options_source_type, test_same, test_same_options, test_same_options_source_type,
    test_same_smallest, test_smallest,
};

#[test]
fn test_remove_unused_expression() {
    test("null", "");
    test("true", "");
    test("false", "");
    test("1", "");
    test("1n", "");
    test(";'s'", "");
    test("this", "");
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
fn test_remove_unused_optional_chain_keeps_base_side_effects() {
    test("let log = []; (log.push('base'), null)?.x;", "[].push('base')");
}

#[test]
fn test_remove_unused_this() {
    // In a derived class constructor, `this` before `super()` throws a ReferenceError,
    // so it must be kept (https://github.com/oxc-project/oxc/issues/21364).
    test(
        "export class Foo extends Bar { constructor() { this; super(); } }",
        "export class Foo extends Bar { constructor() { this, super(); } }",
    );
    // The `this` inside an arrow captures the enclosing derived constructor's `this`.
    test(
        "export class Foo extends Bar { constructor() { (() => { this; })(); super(); } }",
        "export class Foo extends Bar { constructor() { this, super(); } }",
    );

    // Non-derived constructors always have `this` initialized — safe to drop.
    test("export class Foo { constructor() { this; } }", "export class Foo { constructor() {} }");
    // Derived constructor, but `this` is after `super()` — safe to drop.
    test(
        "export class Foo extends Bar { constructor() { super(); this; } }",
        "export class Foo extends Bar { constructor() { super(); } }",
    );

    // Non-adjacent `super()` and `this` — `this` is dropped because `super()`
    // was called unconditionally in a preceding statement.
    test(
        "export class Foo extends Bar { constructor() { super(); foo(); this; } }",
        "export class Foo extends Bar { constructor() { super(), foo(); } }",
    );
    // Conditional `super()` — `this` must be kept.
    test(
        "export class Foo extends Bar { constructor() { if (x) { super(); } this; } }",
        "export class Foo extends Bar { constructor() { x && super(), this; } }",
    );
    test(
        "export class Foo extends Bar { constructor() { x ? super() : foo(); this; } }",
        "export class Foo extends Bar { constructor() { x ? super() : foo(), this; } }",
    );
    // `super()` in closure — `this` must be kept (it's before the `super()` call).
    test(
        "export class Foo extends Bar { constructor() { const s = () => super(); this; s(); } }",
        "export class Foo extends Bar { constructor() { this, super(); } }",
    );

    // A regular function inside a derived constructor has its own `this` — safe to drop.
    test(
        "export class Foo extends Bar { constructor() { (function() { this; })(); super(); } }",
        "export class Foo extends Bar { constructor() { super(); } }",
    );

    // Nested class constructor inside a derived constructor — the inner `this`
    // belongs to the inner constructor, not the outer one.
    test(
        "export class A extends B { constructor() { class C { constructor() { this; } } super(); } }",
        "export class A extends B { constructor() { class C { constructor() {} } super(); } }",
    );

    // In all other positions `this` is always initialized and can be dropped.
    test("{ this; }", "");
    test("export class Foo { foo() { this; } }", "export class Foo { foo() {} }");
    test("export class Foo { static foo() { this; } }", "export class Foo { static foo() {} }");
    test("export class Foo { static { this; } }", "export class Foo {}");
    test("export function foo() { this; }", "export function foo() {}");
    test("export class Foo { get bar() { this; } }", "export class Foo { get bar() {} }");
    test("export class Foo { set bar(v) { this; } }", "export class Foo { set bar(v) {} }");
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
    test("new Set([1, 2, 3])", "");
    // Element side effects are preserved when the pure construction is dropped.
    test("new Set([foo(), bar()])", "foo(), bar();");
    test("new Map([[foo(), bar()]])", "foo(), bar();");
    // A string is a valid iterable of values for `Set`, but `Map`/`WeakSet`/`WeakMap`
    // require `[k, v]` entries / object keys, so a non-empty string argument throws and
    // is kept. An empty string yields no entries and stays pure for all of them.
    test(r#"new Set("ab")"#, "");
    test(r#"new Map("")"#, "");
    test(r#"new WeakSet("")"#, "");
    test(r#"new WeakMap("")"#, "");
    test_same(r#"new Map("ab")"#);
    test_same(r#"new WeakSet("ab")"#);
    test_same(r#"new WeakMap("ab")"#);
    test("new Set(null)", "");
    test("new Set(undefined)", "");
    test("new Set(void 0)", "");
    test_same("new Set(x)");
    test("new Map()", "");
    test("new Map([[1, 2], [3, 4]])", "");
    test("new Map(null)", "");
    test("new Map(undefined)", "");
    test("new Map(void 0)", "");
    test_same("new Map(x)");
    // Map entries must be array literals, otherwise they are not iterable and throw.
    test_same("new Map([x])");
    test_same("new Map([1, 2])");
    // WeakSet/WeakMap keys must be objects, so array-literal args throw.
    test_same("new WeakSet([1])");
    test_same("new WeakMap([[1, 2]])");
    // Typed arrays allocate a zeroed buffer with no user code for a numeric-literal
    // length: a valid length is pure, and a too-large length is a max-length
    // RangeError the minifier is allowed to drop (see docs/ASSUMPTIONS.md).
    test("new Int8Array()", "");
    test("new Uint8Array()", "");
    test("new Int8Array(8)", "");
    test("new Int8Array(1024)", "");
    // Kept: `-1` throws a negative-length RangeError, `0n` throws a TypeError, an
    // object arg can run user code, and a shadowed `Int8Array` is not the builtin.
    test_same("new Int8Array(-1)");
    test_same("new Int8Array(0n)");
    test_same("new Int8Array(x)");
    test_same("new Int8Array([1, 2])");
    test_same("var Int8Array; new Int8Array()");
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

// Leak regression: `remove_unused_template_literal` drains the template's
// elements; an element that `remove_unused_expression` reports removable was
// silently discarded without a `drop_expression` walk, leaking its refs.
// Two computed keys keep `p` multi-use so single-use inlining can't paper
// over the leak; the stale reads then block unused-declaration removal.
#[test]
fn test_template_literal_drop_walks_removed_element_refs() {
    let options = CompressOptions::smallest();
    test_options(
        "function f() { let p = 'metric'; let t = { [`${p}_x`]: 0, [`${p}_y`]: 0 }; void t; return 1; } g(f());",
        "function f() { return 1; } g(f());",
        &options,
    );
}

// Regression: when `remove_unused_array_expr` elides a side-effect-free
// `SpreadElement`, the argument subtree must be walked through
// `drop_expression` so identifier references inside don't leak across
// passes (#22736).
//
// The spread argument is an array literal with two holes, so
// `try_flatten_array_expression_elements` (gated at < 2 holes) does not
// flatten it and the spread reaches the elision branch with `p`'s
// references still inside; `p` is multi-use so single-use inlining can't
// paper over the leak. Without the drop walk the stale reads keep `let p`
// alive and panic the under-prune debug guard.
#[test]
fn test_array_spread_drop_walks_argument_refs() {
    test("([...[function(){}]])", "");
    test("([4, ...[function(){}], a])", "a");
    let options = CompressOptions::smallest();
    test_options(
        "function f() { let p = 'metric'; [...[p, , , p]]; return 1; } g(f());",
        "function f() { return 1; } g(f());",
        &options,
    );
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
    test("(a => {})()", "");
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
        "function foo(x) { return x && /* @__PURE__ */ bar() }",
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

    // Empty-body IIFE called with arguments: drop the wrapper, args still
    // evaluate for side effects.
    test("(() => {})(a);", "a;");
    test("((x, y) => {})(a, b);", "a, b;");
    test("((x) => {})(a, b);", "a, b;");
    test("(function(x) {})(a);", "a;");
    test("var u = (() => {})(a)", "var u = (a, void 0)");
    // Rest binding to an identifier is safe (collected array unobserved).
    test("((x, ...r) => {})(a, b)", "a, b;");
    // Spread arg kept as `[...a]` to preserve iterator-protocol invocation.
    test("(() => {})(...a)", "[...a];");
    // All-pure args → `void 0` directly (no single-element sequence).
    test("(() => {})(1, 2);", "");

    // Negative cases — wrapper must NOT drop.
    test_same("(([x]) => {})(a)");
    test_same("(({z}) => {})(a)");
    test_same("((x = side()) => {})(a)");
    test_same("((...{x}) => {})(a)");
    test_same("(async () => {})(a)");
    test_same("(function*() {})(a)");
    test_same("(() => { foo() })(a)");
    // Directive-only body: in module source the redundant `'use strict'` is
    // stripped upstream, then the empty-body path drops the wrapper.
    test("(function() { 'use strict' })(a)", "a;");
}

#[test]
fn test_remove_side_effect_free_iife() {
    // https://github.com/oxc-project/oxc/issues/23777
    // Calling a function/arrow literal in place runs its body once; when the
    // body (and the args + params) are side-effect-free, the whole call is too,
    // so a discarded result drops entirely — even with a non-trivial body.
    test("(function () { function test() {} return test })()", "");
    test("(function () { return 1 })()", "");
    test("(function () { var x = 1; return x })()", "");
    test("(function () { let a = 1, b = 2; return a + b })()", "");
    test("(function () { return new.target })()", "");
    test("(function () { if (1) { return 2 } else { return 3 } })()", "");
    test("(function foo() { return foo })()", "");
    // Args that are themselves side-effect-free drop with the call.
    test("(function (a, b) { return a })(1, 2)", "");
    test("(function (...rest) { return rest })()", "");
    // The exact issue reproduction: an unused `var` initialized by the IIFE.
    let remove = CompressOptions { unused: CompressOptionsUnused::Remove, ..default_options() };
    test_options("var unused = (function () { function test() {} return test })()", "", &remove);

    // Negative cases — the call has real side effects and must be kept.
    test_same("(function () { sideEffect() })()"); // global call in body
    test_same("(function () { return sideEffect() })()"); // global call in return
    test_same("(function () { globalRead })()"); // global read can throw ReferenceError
    test_same("(function () { throw 1 })()"); // throws
    test_same("(function () { for (;;) sideEffect() })()"); // loop body
    test_same("(function (a) { return a })(sideEffect())"); // side-effect-bearing argument
    test_same("(function (a = sideEffect()) { })()"); // param default runs user code
    test_same("(function ({ x }) { })(obj)"); // destructuring param reads properties
    test_same("(function* () { return 1 })()"); // generator: kept conservatively
    test_same("(async function () { return 1 })()"); // async: kept conservatively
    // `this` / `arguments` reads are kept conservatively: the shared
    // side-effect analysis treats them as potentially effectful.
    test_same("(function () { return this })()");
    test_same("(function () { return arguments })()");
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
    // `i` reads as the implicit `undefined`, but `void 0` prints longer than a
    // mangled identifier read, so the read (and thus the decl) stays (rolldown#10174).
    test_same_options("for (let i;;) foo(i)", &options);
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

    // Assignment RHS with side effects: the RHS is hoisted, the write dropped
    // (same as the default path — see
    // `test_drop_write_only_property_assignments_by_default`).
    test_options("function A() {} A.foo = sideEffect();", "sideEffect();", &options);

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

    // A static getter (get-only) makes a write to its key throw in strict mode,
    // so a class with any static accessor is no longer treated as a fresh value —
    // its writes are kept (conservative for an unrelated key like `bar`, sound).
    test_same_options("class A { static get foo() { return 1; } } A.bar = 1;", &options);

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
    // Literal non-string keys can't coerce to `"__proto__"` but are kept
    // conservatively — see `MemberWriteEffect::MayMutatePrototype`.
    test_options(
        "const a = {}; a[null] = 1; a.a = 1;",
        "const a = {}; a[null] = 1, a.a = 1;",
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

    // `__proto__` assignment inside a hoisted function — traversal reaches the
    // function body only AFTER `obj.a = 1`, so the old per-pass tracking never saw
    // it in time and wrongly dropped `obj.a = 1`. `MayMutatePrototype` is
    // recorded by `Normalize` before the fixed-point loop, so it is caught
    // regardless of order. `f` has an observable side effect (`g()`), so it is
    // not tree-shaken:
    // the setter really is installed when `f()` runs, and the sibling `obj.a = 1`
    // that would trigger it must survive.
    test_options(
        "const obj = {}; f(); obj.a = 1; function f() { g(); obj.__proto__ = { set a(v) { console.log('hello'); } }; }",
        "const obj = {};\nf(), obj.a = 1;\nfunction f() {\n\tg(), obj.__proto__ = { set a(v) {\n\t\tconsole.log('hello');\n\t} };\n}\n",
        &options,
    );

    // Destructuring proto write (`[o.__proto__] = [x]`) is now caught by the
    // Normalize scan (the old per-pass tracking only saw `=` assignments), so the
    // sibling `o.a = 1` is kept.
    test_options(
        "var o = {}; [o.__proto__] = [x]; o.a = 1;",
        "var o = {};\n[o.__proto__] = [x], o.a = 1;\n",
        &options,
    );

    // Default options (property_write_side_effects: true) also drop these now —
    // see `test_drop_write_only_property_assignments_by_default`.
    let default_opts =
        CompressOptions { unused: CompressOptionsUnused::Remove, ..CompressOptions::smallest() };
    test_options("function A() {} A.from = () => {};", "", &default_opts);
}

#[test]
fn test_drop_write_only_property_assignments_by_default() {
    // Under DEFAULT options (`property_write_side_effects: true` untouched —
    // that stays rolldown's tree-shaking knob), full-minify mode drops property
    // assignments whose base is a provably-unused, fresh, non-escaping local
    // binding (terser parity). See `docs/ASSUMPTIONS.md`.

    // Headline: one `displayName` write must not keep an entire module alive.
    // (The `(function(){...})()` wrapper itself survives only because plain-
    // function IIFE inlining is a separate, pre-existing limitation —
    // `substitute_iife_call` unwraps arrow bodies only.)
    test_smallest(
        "(function() { var r = require('react'); var o = function(e, t) { return r.create(e, t); }; o.displayName = 'X'; })();",
        "(function() { require('react'); })();",
    );
    test_smallest(
        "(() => { var r = require('react'); var o = function(e, t) { return r.create(e, t); }; o.displayName = 'X'; })();",
        "require('react');",
    );

    // Pure RHS: the whole statement drops, for every fresh-value init shape.
    test_smallest("var o = {}; o.x = 1;", "");
    test_smallest("var o = []; o.x = 1;", "");
    test_smallest("var o = () => {}; o.x = 1;", "");
    test_smallest("var o = function() {}; o.x = 1;", "");
    test_smallest("var o = class {}; o.x = 1;", "");
    test_smallest("function o() {} o.x = 1;", "");

    // Impure RHS: the RHS is hoisted in place, the write dropped.
    test_smallest("var o = {}; o.x = impure();", "impure();");

    // Value position: a plain assignment's value IS the RHS value.
    test_smallest("var o = {}; use(o.x = impure());", "use(impure());");

    // Computed keys: literal string/number keys are safe...
    test_smallest("var o = {}; o['x'] = 1;", "");
    test_smallest("var o = {}; o[0] = 1;", "");
    // ...but expression keys could evaluate to `"__proto__"` (installing
    // setters) or have their own effects, and `__proto__` itself never drops.
    test_same_smallest("var o = {}; o[k()] = 1;");
    test_same_smallest("var o = {}; o[b] = 1;");
    test_same_smallest("var o = {}; o.__proto__ = x;");
    test_smallest("var o = {}; o['__proto__'] = x;", "var o = {}; o.__proto__ = x;");

    // Escapes: any non-member-write use of the binding blocks the drop.
    test_smallest("var o = {}; o.x = 1; use(o);", "var o = {}; o.x = 1, use(o);");
    test_same_smallest("var o = {}; o.x = o;");
    test_same_smallest("var o = {}; o.x = () => o;");
    test_same_smallest("export var o = {}; o.x = 1;");

    // Read-modify interference (hazard): compound/logical/update ops READ the
    // property, so sibling plain writes must survive — dropping
    // `o.x = { valueOf: f }` would delete the observable `f()` call from
    // `+=`'s coercion.
    test_smallest(
        "var o = {}; o.x = { valueOf: f }; o.x += 1;",
        "var o = {}; o.x = { valueOf: f }, o.x += 1;",
    );
    test_same_smallest("var o = {}; o.x += 1;");
    test_smallest("var o = {}; o.y ||= 2; o.x = 1;", "var o = {}; o.y ||= 2, o.x = 1;");
    test_smallest("var o = {}; o.x++; o.y = 1;", "var o = {}; o.x++, o.y = 1;");
    // Formed-compound staleness: the loop rewrites `o.y = o.y + 1` to
    // `o.y += 1` (dropping the plain read that blocked the counts predicate);
    // the hazard set must still block the sibling plain write.
    test_smallest("var o = {}; o.x = evil; o.y = o.y + 1;", "var o = {}; o.x = evil, o.y += 1;");

    // Chained-write base (hazard): dropping `a.b = {}` while `a.b.c = 1`
    // survives would throw at runtime.
    test_smallest("var a = {}; a.b = {}; a.b.c = 1;", "var a = {}; a.b = {}, a.b.c = 1;");

    // `__proto__` write in a hoisted function runs before the property write —
    // the hazard scan is execution-order independent because `Normalize` seeds
    // the facts before the fixed-point loop.
    test_smallest(
        "const obj = {}; f(); obj.a = 1; function f() { obj.__proto__ = { set a(v) { console.log(v); } }; }",
        "const obj = {}; f(), obj.a = 1; function f() { obj.__proto__ = { set a(v) { console.log(v); } }; }",
    );

    // Single-level `delete` neither reads the property nor triggers setters:
    // the plain write drops, the delete stays.
    test_smallest("var o = {}; o.x = 1; delete o.x;", "var o = {}; delete o.x;");
    // Chained delete reads the intermediate object — everything stays.
    test_smallest("var a = {}; a.b = {}; delete a.b.c;", "var a = {}; a.b = {}, delete a.b.c;");

    // Setter observation via the object itself: not a fresh value.
    test_same_smallest("class A { static set foo(v) { console.log(v); } } A.foo = 1;");
    test_same_smallest("var o = { set x(v) { console.log(v) } }; o.x = 1;");

    // Gates.
    // Script-mode top-level vars are global state.
    test_same_options_source_type(
        "var o = {}; o.x = 1;",
        SourceType::cjs().with_script(true),
        &CompressOptions::smallest(),
    );
    // Direct eval can observe anything.
    test_smallest(
        "export function f() { var o = {}; o.x = 1; eval(''); }",
        "export function f() { var o = {}; o.x = 1, eval(''); }",
    );
    // --- Kind-aware key denylist ---
    // A write that would throw a strict-mode `TypeError`, or observably coerce,
    // is NOT dead even though the base binding is otherwise unused.

    // Function objects (fn-expr / arrow / fn-decl): `name` and `length` are
    // non-writable own props; `caller` / `arguments` are the %ThrowTypeError%
    // poison. Class objects (class-expr / class-decl) share those. All throw.
    for (form, sep) in [
        ("var o = function() {}", ";"),
        ("var o = () => {}", ";"),
        ("function o() {}", ""),
        ("var o = class {}", ";"),
        ("class o {}", ""),
    ] {
        for key in ["caller", "arguments", "name", "length"] {
            test_same_smallest(&format!("{form}{sep} o.{key} = 1;"));
        }
    }

    // A class's `prototype` is non-writable (throws), so it stays. A plain
    // function's `prototype` IS writable, so that write still drops.
    test_same_smallest("var o = class {}; o.prototype = 1;");
    test_same_smallest("class o {} o.prototype = 1;");
    test_smallest("var f = function() {}; f.prototype = {};", "");
    test_smallest("function f() {} f.prototype = {};", "");

    // Array `length`: `a.length = -1` throws a `RangeError`; `a.length = {...}`
    // runs a `valueOf` coercion. Both kept; a computed string key counts too.
    test_same_smallest("var a = []; a.length = -1;");
    test_same_smallest("var a = []; a.length = { valueOf() { g(); } };");
    test_smallest("var a = []; a['length'] = 1;", "var a = []; a.length = 1;");
    // A numeric index write is an ordinary array write — still drops.
    test_smallest("var a = []; a[0] = 1;", "");

    // `name` / `length` on an object literal are ordinary writable props: the
    // denylist is kind-scoped, so Object-kind bases still drop.
    test_smallest("var o = {}; o.length = 5;", "");
    test_smallest("var o = {}; o.name = 'n';", "");

    // Instance-private write (`o.#x = 1`) is a brand check that throws unless
    // `o` is an instance of the declaring class — a fresh literal never is.
    test_same_smallest("export class C { #x; m() { var o = {}; o.#x = 1; } }");

    // A class with a static block, a static getter, or a decorator is not a
    // fresh value (arbitrary code runs / a write to that key throws), so a
    // later write to it survives.
    test_same_smallest("class o { static { g(); } } o.x = 1;");
    test_same_smallest("class o { static get foo() { return 1; } } o.foo = 1;");
    test_same_smallest("@dec class o {} o.x = 1;");

    // unused: Keep (default_options()) / KeepAssign keep the write.
    test_same("function A() {} A.from = () => {};");
    let keep_assign = CompressOptions {
        unused: CompressOptionsUnused::KeepAssign,
        ..CompressOptions::smallest()
    };
    test_same_options("function A() {} A.from = () => {};", &keep_assign);
}

#[test]
fn test_summary_invalidation_preserves_member_write_hazard() {
    // Both initializers keep the dense fresh-value kind intact. Clearing the
    // function summary by removing its shared metadata entry would lose the
    // `||=` hazard and incorrectly drop `foo.x = 1`.
    test_smallest(
        "var foo = function() {}; var foo = function() {}; foo.x = 1; foo.x ||= send();",
        "var foo = function() {}, foo = function() {}; foo.x = 1, foo.x ||= send();",
    );
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
