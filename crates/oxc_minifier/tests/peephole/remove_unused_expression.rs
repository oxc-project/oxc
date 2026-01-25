use crate::{
    CompressOptions, TreeShakeOptions, default_options, test, test_options,
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
