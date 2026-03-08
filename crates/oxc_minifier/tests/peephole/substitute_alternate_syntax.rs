use oxc_span::SourceType;

use crate::{
    CompressOptions, CompressOptionsKeepNames, CompressOptionsUnused, default_options, test,
    test_options, test_same, test_same_options, test_same_options_source_type, test_target_same,
};

#[test]
fn test_fold_return_result() {
    test("function f(){return !1;}", "function f(){return !1}");
    test("function f(){return null;}", "function f(){return null}");
    test("function f(){return void 0;}", "function f(){}");
    test("function f(){return void foo();}", "function f(){foo()}");
    test("function f(){return undefined;}", "function f(){}");
    test("function f(){if(a()){return undefined;}}", "function f(){a()}");
    test_same("function a(undefined) { return undefined; }");
    test_same("function f(){return foo()}");

    // `return undefined` has a different semantic in async generator function.
    test("function foo() { return undefined }", "function foo() { }");
    test("function* foo() { return undefined }", "function* foo() { }");
    test("async function foo() { return undefined }", "async function foo() { }");
    test_same("async function* foo() { return void 0 }");
    test_same("class Foo { async * foo() { return void 0 } }");
    test(
        "async function* foo() { function bar () { return void 0 } return bar }",
        "async function* foo() { function bar () {} return bar }",
    );
    test(
        "async function* foo() { let bar = () => { return void 0 }; return bar }",
        "async function* foo() { return () => {} }",
    );
}

#[test]
fn test_undefined() {
    test("let x = undefined", "let x");
    test("const x = undefined", "const x = void 0");
    test("var x = undefined", "var x = void 0");
    test_same("var undefined = 1;function f() {var undefined=2,x;}");
    test_same("function f(undefined) {}");
    test_same("try { foo } catch(undefined) {foo(undefined)}");
    test("for (undefined in {}) {}", "for(undefined in {});");
    test("undefined++;", "undefined++");
    test("undefined += undefined;", "undefined+=void 0");
    // shadowed
    test_same("(function(undefined) { let x = typeof undefined; })()");
    // destructuring throw error side effect
    test_same("var {} = void 0");
    test_same("var [] = void 0");
    // `delete undefined` returns `false`
    // `delete void 0` returns `true`
    test_same("delete undefined");
}

#[test]
fn test_fold_true_false_comparison() {
    test("v = x == true", "v = x == 1");
    test("v = x == false", "v = x == 0");
    test("v = x != true", "v = x != 1");
    test("v = x < true", "v = x < !0");
    test("v = x <= true", "v = x <= !0");
    test("v = x > true", "v = x > !0");
    test("v = x >= true", "v = x >= !0");

    test("v = x instanceof true", "v = x instanceof !0");
    test("v = x + false", "v = x + !1");

    // Order: should perform the nearest.
    test("v = x == x instanceof false", "v = x == x instanceof !1");
    test("v = x in x >> true", "v = x in x >> !0");
    test("v = x == fake(false)", "v = x == fake(!1)");

    // The following should not be folded.
    test("v = x === true", "v = x === !0");
    test("v = x !== false", "v = x !== !1");
}

/// Based on https://github.com/terser/terser/blob/58ba5c163fa1684f2a63c7bc19b7ebcf85b74f73/test/compress/assignment.js
#[test]
fn test_fold_normal_assignment_to_combined_assignment() {
    test("x = x + 3", "x += 3");
    test("x = x - 3", "x -= 3");
    test("x = x / 3", "x /= 3");
    test("x = x * 3", "x *= 3");
    test("x = x >> 3", "x >>= 3");
    test("x = x << 3", "x <<= 3");
    test("x = x >>> 3", "x >>>= 3");
    test("x = x | 3", "x |= 3");
    test("x = x ^ 3", "x ^= 3");
    test("x = x % 3", "x %= 3");
    test("x = x & 3", "x &= 3");
    test("x = x + g()", "x += g()");
    test("x = x - g()", "x -= g()");
    test("x = x / g()", "x /= g()");
    test("x = x * g()", "x *= g()");
    test("x = x >> g()", "x >>= g()");
    test("x = x << g()", "x <<= g()");
    test("x = x >>> g()", "x >>>= g()");
    test("x = x | g()", "x |= g()");
    test("x = x ^ g()", "x ^= g()");
    test("x = x % g()", "x %= g()");
    test("x = x & g()", "x &= g()");

    test_same("x = 3 + x");
    test_same("x = 3 - x");
    test_same("x = 3 / x");
    test_same("x = 3 * x");
    test_same("x = 3 >> x");
    test_same("x = 3 << x");
    test_same("x = 3 >>> x");
    test_same("x = 3 | x");
    test_same("x = 3 ^ x");
    test_same("x = 3 % x");
    test_same("x = 3 & x");
    test_same("x = g() + x");
    test_same("x = g() - x");
    test_same("x = g() / x");
    test_same("x = g() * x");
    test_same("x = g() >> x");
    test_same("x = g() << x");
    test_same("x = g() >>> x");
    test_same("x = g() | x");
    test_same("x = g() ^ x");
    test_same("x = g() % x");
    test_same("x = g() & x");

    test_same("x = (x -= 2) ^ x");

    // GetValue(x) has no sideeffect when x is a resolved identifier
    test("var x; x.y = x.y + 3", "var x; x.y += 3");
    test("var x; x.#y = x.#y + 3", "var x; x.#y += 3");
    test_same("x.y = x.y + 3");
    // this can be compressed if `y` does not have side effect
    test_same("var x; x[y] = x[y] + 3");
    // GetValue(x) has a side effect in this case
    // Example case: `var a = { get b() { console.log('b'); return { get c() { console.log('c') } } } }; a.b.c = a.b.c + 1`
    test_same("var x; x.y.z = x.y.z + 3");
    // This case is not supported, since the minifier does not support with statements
    // test_same("var x; with (z) { x.y || (x.y = 3) }");
}

#[test]
fn test_fold_subtraction_assignment() {
    test("x -= 1", "--x");
    test("x -= -1", "++x");
    test_same("x -= 2");
    test_same("x += 1"); // The string concatenation may be triggered, so we don't fold this.
    test_same("x += -1");
}

#[test]
fn test_fold_literal_object_constructors() {
    test("x = new Object", "x = ({})");
    test("x = new Object()", "x = ({})");
    test("x = Object()", "x = ({})");

    test_same("x = (function (){function Object(){this.x=4}return new Object();})();");

    test("x = new window.Object", "x = ({})");
    test("x = new window.Object()", "x = ({})");

    // Mustn't fold optional chains
    test("x = window.Object()", "x = ({})");
    test("x = window.Object?.()", "x = Object?.()");

    test(
        "x = (function (){function Object(){this.x=4};return new window.Object;})();",
        "x = (function (){function Object(){this.x=4}return {};})();",
    );
}

#[test]
fn test_fold_literal_array_constructors() {
    test("x = new Array", "x = []");
    test("x = new Array()", "x = []");
    test("x = Array()", "x = []");
    // do not fold optional chains
    test_same("x = Array?.()");

    // One argument
    test("x = new Array(0)", "x = []");
    test("x = new Array(\"a\")", "x = [\"a\"]");
    test("x = new Array(1)", "x = [,]");
    test("x = new Array(6)", "x = [,,,,,,]");
    test("x = new Array(7)", "x = Array(7)");
    test("x = new Array(7n)", "x = [7n]");
    test("x = new Array(y)", "x = Array(y)");
    test("x = new Array(foo())", "x = Array(foo())");
    test("x = Array(0)", "x = []");
    test("x = Array(\"a\")", "x = [\"a\"]");
    test_same("x = Array(7)");
    test_same("x = Array(y)");
    test_same("x = Array(foo())");

    // 1+ arguments
    test("x = new Array(1, 2, 3, 4)", "x = [1, 2, 3, 4]");
    test("x = Array(1, 2, 3, 4)", "x = [1, 2, 3, 4]");
    test("x = new Array('a', 1, 2, 'bc', 3, {}, 'abc')", "x = ['a', 1, 2, 'bc', 3, {}, 'abc']");
    test("x = Array('a', 1, 2, 'bc', 3, {}, 'abc')", "x = ['a', 1, 2, 'bc', 3, {}, 'abc']");
    test("x = new Array(Array(1, '2', 3, '4'))", "x = [[1, '2', 3, '4']]");
    test("x = Array(Array(1, '2', 3, '4'))", "x = [[1, '2', 3, '4']]");
    test(
        "x = new Array(Object(), Array(\"abc\", Object(), Array(Array())))",
        "x = [{}, [\"abc\", {}, [[]]]]",
    );
    test(
        "x = new Array(Object(), Array(\"abc\", Object(), Array(Array())))",
        "x = [{}, [\"abc\", {}, [[]]]]",
    );
}

#[test]
fn test_fold_new_expressions() {
    test("let _ = new Error()", "let _ = /* @__PURE__ */ Error()");
    test("let _ = new Error('a')", "let _ = /* @__PURE__ */ Error('a')");
    test("let _ = new Error('a', { cause: b })", "let _ = Error('a', { cause: b })");
    test_same("var Error; new Error()");
    test("let _ = new EvalError()", "let _ = /* @__PURE__ */ EvalError()");
    test("let _ = new RangeError()", "let _ = /* @__PURE__ */ RangeError()");
    test("let _ = new ReferenceError()", "let _ = /* @__PURE__ */ ReferenceError()");
    test("let _ = new SyntaxError()", "let _ = /* @__PURE__ */ SyntaxError()");
    test("let _ = new TypeError()", "let _ = /* @__PURE__ */ TypeError()");
    test("let _ = new URIError()", "let _ = /* @__PURE__ */ URIError()");
    test("let _ = new AggregateError('a')", "let _ = /* @__PURE__ */ AggregateError('a')");

    test("new Function()", "Function()");
    test("new Function('a', 'b', 'console.log(a, b)')", "Function('a', 'b', 'console.log(a, b)')");
    test_same("var Function; new Function()");

    // RegExp is validated using the regex parser to determine if it's pure.
    // Valid patterns can be removed, invalid patterns must be kept.
    // https://github.com/oxc-project/oxc/issues/18050
    test("new RegExp()", ""); // Valid: empty pattern
    test("new RegExp('a')", ""); // Valid: simple pattern
    test("new RegExp(0)", "RegExp(0)"); // Can't validate non-string literal
    test("new RegExp(null)", "RegExp(null)"); // Can't validate non-string literal
    test("x = new RegExp('a', 'g')", "x = /* @__PURE__ */ RegExp('a', 'g')"); // Valid pattern, marked as pure
    test_same("new RegExp(foo)"); // Can't validate variable
    // RegExp literal is always valid
    test("new RegExp(/foo/)", "");
    // Invalid patterns and flags must not be removed (they throw SyntaxError at runtime)
    test_same("RegExp('[')");
    test_same("RegExp('a', 'xyz')");
}

#[test]
fn test_compress_typed_array_constructor() {
    test("new Int8Array(0)", "new Int8Array()");
    test("new Uint8Array(0)", "new Uint8Array()");
    test("new Uint8ClampedArray(0)", "new Uint8ClampedArray()");
    test("new Int16Array(0)", "new Int16Array()");
    test("new Uint16Array(0)", "new Uint16Array()");
    test("new Int32Array(0)", "new Int32Array()");
    test("new Uint32Array(0)", "new Uint32Array()");
    test("new Float32Array(0)", "new Float32Array()");
    test("new Float64Array(0)", "new Float64Array()");
    test("new BigInt64Array(0)", "new BigInt64Array()");
    test("new BigUint64Array(0)", "new BigUint64Array()");

    test_same("var Int8Array; new Int8Array(0)");
    test_same("new Int8Array(1)");
    test_same("new Int8Array(a)");
    test_same("new Int8Array(0, a)");
}

#[test]
fn test_string_array_splitting() {
    const REPEAT: usize = 20;
    let additional_args = ",'1'".repeat(REPEAT);
    let test_with_longer_args =
        |source_text_partial: &str, expected_partial: &str, delimiter: &str| {
            let expected = &format!(
                "var x=/* @__PURE__ */'{expected_partial}{}'.split('{delimiter}')",
                format!("{delimiter}1").repeat(REPEAT)
            );
            test(&format!("var x=[{source_text_partial}{additional_args}]"), expected);
        };
    let test_same_with_longer_args = |source_text_partial: &str| {
        test_same(&format!("var x=[{source_text_partial}{additional_args}]"));
    };

    test_same_with_longer_args("'1','2','3','4'");
    test_same_with_longer_args("'1','2','3','4','5'");
    test_same_with_longer_args("`1${a}`,'2','3','4','5','6'");
    test_with_longer_args("'1','2','3','4','5','6'", "123456", "");
    test_with_longer_args("'1','2','3','4','5','00'", "1.2.3.4.5.00", ".");
    test_with_longer_args("'1','2','3','4','5','6','7'", "1234567", "");
    test_with_longer_args("'1','2','3','4','5','6','00'", "1.2.3.4.5.6.00", ".");
    test_with_longer_args("'.,',',',',',',',',',','", ".,(,(,(,(,(,", "(");
    test_with_longer_args("',,','.',',',',',',',','", ",,(.(,(,(,(,", "(");
    test_with_longer_args("'a,','.',',',',',',',','", "a,(.(,(,(,(,", "(");
    test_with_longer_args("`1`,'2','3','4','5','6'", "123456", "");

    // all possible delimiters used, leave it alone
    test_same_with_longer_args("'.', ',', '(', ')', ' '");

    test_options(
        &format!("var x=['1','2','3','4','5','6'{additional_args}]"),
        "",
        &CompressOptions { unused: CompressOptionsUnused::Remove, ..default_options() },
    );
}

#[test]
fn test_template_string_to_string() {
    test("x = `abcde`", "x = 'abcde'");
    test("x = `ab cd ef`", "x = 'ab cd ef'");
    test_same("x = `hello ${name}`");
    test_same("tag `hello ${name}`");
    test_same("tag `hello`");
    test("x = `hello ${'foo'}`", "x = 'hello foo'");
    test("x = `${2} bananas`", "x = '2 bananas'");
    test("x = `This is ${true}`", "x = 'This is true'");
}

#[test]
#[ignore = "TODO: Function.bind to Function.call optimization not yet implemented"]
fn test_bind_to_call() {
    test("((function(){}).bind())()", "((function(){}))()");
    test("((function(){}).bind(a))()", "((function(){})).call(a)");
    test("((function(){}).bind(a,b))()", "((function(){})).call(a,b)");

    test("((function(){}).bind())(a)", "((function(){}))(a)");
    test("((function(){}).bind(a))(b)", "((function(){})).call(a,b)");
    test("((function(){}).bind(a,b))(c)", "((function(){})).call(a,b,c)");

    // Without using type information we don't know "f" is a function.
    test_same("(f.bind())()");
    test_same("(f.bind(a))()");
    test_same("(f.bind())(a)");
    test_same("(f.bind(a))(b)");
}

#[test]
fn test_rotate_associative_operators() {
    test(
        "function f(a, b, c) { return a || (b || c) }",
        "function f(a, b, c) { return (a || b) || c }",
    );
    test(
        "function f(a, b, c) { return a && (b && c) }",
        "function f(a, b, c) { return (a && b) && c }",
    );
    test(
        "function f(a, b, c) { return a ?? (b ?? c) }",
        "function f(a, b, c) { return (a ?? b) ?? c }",
    );

    test(
        "function f(a, b, c) { return a | (b | c) }",
        "function f(a, b, c) { return (a | b) | c }",
    );
    test(
        "function f(a, b, c) { return a() | (b | c) }",
        "function f(a, b, c) { return (a() | b) | c }",
    );
    test(
        "function f(a, b, c) { return a | (b() | c) }",
        "function f(a, b, c) { return (a | b()) | c }",
    );
    // c() will not be executed when `a | b` throws an error
    test_same("function f(a, b, c) { return a | (b | c()) }");
    test(
        "function f(a, b, c) { return a & (b & c) }",
        "function f(a, b, c) { return (a & b) & c }",
    );
    test(
        "function f(a, b, c) { return a ^ (b ^ c) }",
        "function f(a, b, c) { return (a ^ b) ^ c }",
    );

    // avoid rotation to prevent precision loss
    // also multiplication is not associative due to floating point precision
    // https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-numeric-types-number-multiply
    test_same("function f(a, b, c) { return a + (b + c) }");
    test_same("function f(a, b, c) { return a - (b - c) }");
    test_same("function f(a, b, c) { return a / (b / c) }");
    test_same("function f(a, b, c) { return a % (b % c) }");
    test_same("function f(a, b, c) { return a ** (b ** c) }");

    test("function f(a, b, c) { return a * (b % c) }", "function f(a, b, c) { return b % c * a }");
    test_same("function f(a, b, c) { return a() * (b % c) }"); // a may update b / c
    test_same("function f(a, b, c) { return a * (b() % c) }"); // b may update b / c
    test_same("function f(a, b, c) { return a * (b % c()) }"); // c may update b / c
    test("function f(a, b, c) { return a * (b / c) }", "function f(a, b, c) { return b / c * a }");
    test("function f(a, b, c) { return a * (b * c) }", "function f(a, b, c) { return b * c * a }");

    test_same("function f(a, b, c, d) { return a * b * (c / d) }");
    test_same("function f(a, b, c, d) { return (a + b) * (c % d) }");
    // Don't swap if left has division (already high precedence)
    test_same("function f(a, b, c, d) { return a / b * (c % d) }");
}

#[test]
fn nullish_coalesce() {
    test("a ?? (b ?? c);", "(a ?? b) ?? c");
}

#[test]
fn test_fold_arrow_function_return() {
    test("const foo = () => { return 'baz' }", "const foo = () => 'baz'");
    test("const foo = () => { foo.foo; return 'baz' }", "const foo = () => (foo.foo, 'baz')");
}

#[test]
fn test_fold_is_typeof_equals_undefined_resolved() {
    test("var x; v = typeof x !== 'undefined'", "var x; v = x !== void 0");
    test("var x; v = typeof x != 'undefined'", "var x; v = x !== void 0");
    test("var x; v = 'undefined' !== typeof x", "var x; v = x !== void 0");
    test("var x; v = 'undefined' != typeof x", "var x; v = x !== void 0");

    test("var x; v = typeof x === 'undefined'", "var x; v = x === void 0");
    test("var x; v = typeof x == 'undefined'", "var x; v = x === void 0");
    test("var x; v = 'undefined' === typeof x", "var x; v = x === void 0");
    test("var x; v = 'undefined' == typeof x", "var x; v = x === void 0");

    test(
        "var x; function foo() { v = typeof x !== 'undefined' }",
        "var x; function foo() { v = x !== void 0 }",
    );
    test(
        "v = typeof x !== 'undefined'; function foo() { var x }",
        "v = typeof x < 'u'; function foo() { var x }",
    );
    test("v = typeof x !== 'undefined'; { var x }", "v = x !== void 0; var x;");
    test("v = typeof x !== 'undefined'; { let x }", "v = typeof x < 'u'; { let x }");
    test("v = typeof x !== 'undefined'; var x", "v = x !== void 0; var x");
    // input and output both errors with same TDZ error
    test("v = typeof x !== 'undefined'; let x", "v = x !== void 0; let x");

    test("v = typeof x.y === 'undefined'", "v = x.y === void 0");
    test("v = typeof x.y !== 'undefined'", "v = x.y !== void 0");
    test("v = typeof (x + '') === 'undefined'", "v = x + '' === void 0");
}

/// Port from <https://github.com/evanw/esbuild/blob/v0.24.2/internal/js_parser/js_parser_test.go#L4658>
#[test]
fn test_fold_is_typeof_equals_undefined() {
    test("v = typeof x !== 'undefined'", "v = typeof x < 'u'");
    test("v = typeof x != 'undefined'", "v = typeof x < 'u'");
    test("v = 'undefined' !== typeof x", "v = typeof x < 'u'");
    test("v = 'undefined' != typeof x", "v = typeof x < 'u'");

    test("v = typeof x === 'undefined'", "v = typeof x > 'u'");
    test("v = typeof x == 'undefined'", "v = typeof x > 'u'");
    test("v = 'undefined' === typeof x", "v = typeof x > 'u'");
    test("v = 'undefined' == typeof x", "v = typeof x > 'u'");
}

#[test]
fn test_fold_is_object_and_not_null() {
    test(
        "var foo; v = typeof foo === 'object' && foo !== null",
        "var foo; v = typeof foo == 'object' && !!foo",
    );
    test(
        "var foo; v = typeof foo == 'object' && foo !== null",
        "var foo; v = typeof foo == 'object' && !!foo",
    );
    test(
        "var foo; v = typeof foo === 'object' && foo != null",
        "var foo; v = typeof foo == 'object' && !!foo",
    );
    test(
        "var foo; v = typeof foo == 'object' && foo != null",
        "var foo; v = typeof foo == 'object' && !!foo",
    );
    test(
        "var foo; v = typeof foo !== 'object' || foo === null",
        "var foo; v = typeof foo != 'object' || !foo",
    );
    test(
        "var foo; v = typeof foo != 'object' || foo === null",
        "var foo; v = typeof foo != 'object' || !foo",
    );
    test(
        "var foo; v = typeof foo !== 'object' || foo == null",
        "var foo; v = typeof foo != 'object' || !foo",
    );
    test(
        "var foo; v = typeof foo != 'object' || foo == null",
        "var foo; v = typeof foo != 'object' || !foo",
    );
    test(
        "var foo, bar; v = typeof foo === 'object' && foo !== null && bar !== 1",
        "var foo, bar; v = typeof foo == 'object' && !!foo && bar !== 1",
    );
    test(
        "var foo, bar; v = bar !== 1 && typeof foo === 'object' && foo !== null",
        "var foo, bar; v = bar !== 1 && typeof foo == 'object' && !!foo",
    );
    test(
        "var foo, bar; v = typeof foo === 'object' && foo !== null || bar !== 1",
        "var foo, bar; v = typeof foo == 'object' && !!foo || bar !== 1",
    );
    test(
        "var foo, bar; v = bar !== 1 || typeof foo === 'object' && foo !== null",
        "var foo, bar; v = bar !== 1 || typeof foo == 'object' && !!foo",
    );
    test(
        "var foo, bar; v = (typeof foo !== 'object' || foo === null) && bar !== 1",
        "var foo, bar; v = (typeof foo != 'object' || !foo) && bar !== 1",
    );
    test(
        "var foo, bar; v = bar !== 1 && (typeof foo !== 'object' || foo === null)",
        "var foo, bar; v = bar !== 1 && (typeof foo != 'object' || !foo)",
    );
    test_same("var foo, bar; v = bar !== 1 && typeof foo != 'object' || foo === null");
    test_same("var foo, bar; v = typeof foo != 'object' || foo === null && bar !== 1");
    test_same("var foo; v = typeof foo.a == 'object' && foo.a !== null"); // cannot be folded because accessing foo.a might have a side effect
    test_same("v = foo !== null && typeof foo == 'object'"); // cannot be folded because accessing foo might have a side effect
    test_same("v = typeof foo == 'object' && foo !== null"); // cannot be folded because accessing foo might have a side effect
    test_same("var foo, bar; v = typeof foo == 'object' && bar !== null");
    test_same("var foo; v = typeof foo == 'string' && foo !== null");
}

#[test]
fn test_swap_binary_expressions() {
    test_same("v = a === 0");
    test("v = 0 === a", "v = a === 0");
    test_same("v = a === '0'");
    test("v = '0' === a", "v = a === '0'");
    test("v = a === `0`", "v = a === '0'");
    test("v = `0` === a", "v = a === '0'");
    test_same("v = a === void 0");
    test("v = void 0 === a", "v = a === void 0");

    test_same("v = a !== 0");
    test("v = 0 !== a", "v = a !== 0");
    test_same("v = a == 0");
    test("v = 0 == a", "v = a == 0");
    test_same("v = a != 0");
    test("v = 0 != a", "v = a != 0");
}

#[test]
fn test_remove_unary_plus() {
    test("v = 1 - +foo", "v = 1 - foo");
    test("v = +foo - 1", "v = foo - 1");
    test_same("v = 1n - +foo");
    test_same("v = +foo - 1n");
    test_same("v = +foo - bar");
    test_same("v = foo - +bar");
    test_same("v = 1 + +foo"); // cannot compress into `1 + foo` because `foo` can be a string

    test("v = +d / 1000", "v = d / 1000");
    test("v = 1000 * +d", "v = 1000 * d");
    test("v = +d * 1000", "v = d * 1000");
    test("v = 2 - +this._x.call(null, node.data)", "v = 2 - this._x.call(null, node.data)");

    test("v = 5 | +b", "v = 5 | b");
    test("v = +b | 5", "v = b | 5");
    test("v = 7 & +c", "v = 7 & c");
    test("v = 3 ^ +d", "v = 3 ^ d");
    // Don't remove - unsafe for BigInt operations
    test_same("v = a - +b");
    test_same("v = +a - b");
    test_same("v = a | +b");
    test_same("v = +a | b");
}

#[test]
fn test_fold_loose_equals_undefined() {
    test_same("v = foo != null");
    test("v = foo != undefined", "v = foo != null");
    test("v = foo != void 0", "v = foo != null");
    test("v = undefined != foo", "v = foo != null");
    test("v = void 0 != foo", "v = foo != null");
}

#[test]
fn test_property_key() {
    // Object Property
    test(
        "v = { '0': _, 'a': _, [1]: _, ['1']: _, ['b']: _, ['c.c']: _, '1.1': _, '😊': _, 'd.d': _ }",
        "v = {  0: _,   a: _,    1: _,     1: _,     b: _,   'c.c': _, '1.1': _, '😊': _, 'd.d': _ }",
    );
    // AssignmentTargetPropertyProperty
    test(
        "({ '0': _, 'a': _, [1]: _, ['1']: _, ['b']: _, ['c.c']: _, '1.1': _, '😊': _, 'd.d': _ } = {})",
        "({  0: _,   a: _,    1: _,   1: _,     b: _,   'c.c': _, '1.1': _, '😊': _, 'd.d': _ } = {})",
    );
    // Binding Property
    test(
        "var { '0': _, 'a': _, [1]: _, ['1']: _, ['b']: _, ['c.c']: _, '1.1': _, '😊': _, 'd.d': _ } = {}",
        "var {  0: _,   a: _,    1: _,   1: _,     b: _,   'c.c': _, '1.1': _, '😊': _, 'd.d': _ } = {}",
    );
    // Method Definition
    test(
        "class F { '0'(){}; 'a'(){}; [1](){}; ['1'](){}; ['b'](){}; ['c.c'](){}; '1.1'(){}; '😊'(){}; 'd.d'(){} }",
        "class F {  0(){};   a(){};    1(){};    1(){};     b(){};   'c.c'(){}; '1.1'(){}; '😊'(){}; 'd.d'(){} }",
    );
    // Property Definition
    test(
        "class F { '0' = _; 'a' = _; [1] = _; ['1'] = _; ['b'] = _; ['c.c'] = _; '1.1' = _; '😊' = _; 'd.d' = _ }",
        "class F {  0 = _;   a = _;    1 = _;    1 = _;     b = _;   'c.c' = _; '1.1' = _; '😊' = _; 'd.d' = _ }",
    );
    // Accessor Property
    test(
        "class F { accessor '0' = _; accessor 'a' = _; accessor [1] = _; accessor ['1'] = _; accessor ['b'] = _; accessor ['c.c'] = _; accessor '1.1' = _; accessor '😊' = _; accessor 'd.d' = _ }",
        "class F { accessor  0 = _;  accessor  a = _;    accessor 1 = _;accessor     1 = _; accessor     b = _; accessor   'c.c' = _; accessor '1.1' = _; accessor '😊' = _; accessor 'd.d' = _ }",
    );

    test("class C { ['-1']() {} }", "class C { '-1'() {} }");

    // <https://tc39.es/ecma262/2024/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-propertydefinitionevaluation>
    test_same("v = ({ ['__proto__']: 0 })"); // { __proto__: 0 } will have `isProtoSetter = true`
    test("v = ({ ['__proto__']() {} })", "v = ({ __proto__() {} })");
    test("({ ['__proto__']: _ } = {})", "({ __proto__: _ } = {})");
    test("class C { ['__proto__'] = 0 }", "class C { __proto__ = 0 }");
    test("class C { ['__proto__']() {} }", "class C { __proto__() {} }");
    test("class C { accessor ['__proto__'] = 0 }", "class C { accessor __proto__ = 0 }");
    test("class C { static ['__proto__'] = 0 }", "class C { static __proto__ = 0 }");
    test(
        "class C { static accessor ['__proto__'] = 0 }",
        "class C { static accessor __proto__ = 0 }",
    );

    // Patch KATAKANA MIDDLE DOT and HALFWIDTH KATAKANA MIDDLE DOT
    // <https://github.com/oxc-project/unicode-id-start/pull/3>
    test_same("x = { 'x・': 0 };");
    test_same("x = { 'x･': 0 };");
    test_same("x = y['x・'];");
    test_same("x = y['x･'];");

    // <https://tc39.es/ecma262/2024/multipage/ecmascript-language-functions-and-classes.html#sec-static-semantics-classelementkind>
    // <https://tc39.es/ecma262/2024/multipage/ecmascript-language-functions-and-classes.html#sec-class-definitions-static-semantics-early-errors>
    // <https://arai-a.github.io/ecma262-compare/?pr=2417&id=sec-class-definitions-static-semantics-early-errors>
    test_same("class C { static ['prototype']() {} }"); // class C { static prototype() {} } is an early error
    test_same("class C { static ['prototype'] = 0 }"); // class C { prototype = 0 } is an early error
    test_same("class C { static accessor ['prototype'] = 0 }"); // class C { accessor prototype = 0 } is an early error
    test("class C { ['prototype']() {} }", "class C { prototype() {} }");
    test("class C { 'prototype'() {} }", "class C { prototype() {} }");
    test("class C { ['prototype'] = 0 }", "class C { prototype = 0 }");
    test("class C { 'prototype' = 0 }", "class C { prototype = 0 }");
    test("class C { accessor ['prototype'] = 0 }", "class C { accessor prototype = 0 }");
    test_same("class C { ['constructor'] = 0 }"); // class C { constructor = 0 } is an early error
    test_same("class C { accessor ['constructor'] = 0 }"); // class C { accessor constructor = 0 } is an early error
    test_same("class C { static ['constructor'] = 0 }"); // class C { static constructor = 0 } is an early error
    test_same("class C { static accessor ['constructor'] = 0 }"); // class C { static accessor constructor = 0 } is an early error
    test_same("class C { ['constructor']() {} }"); // computed `constructor` is not treated as a constructor
    test("class C { 'constructor'() {} }", "class C { constructor() {} }");
    test_same("class C { *['constructor']() {} }"); // class C { *constructor() {} } is an early error
    test_same("class C { async ['constructor']() {} }"); // class C { async constructor() {} } is an early error
    test_same("class C { async *['constructor']() {} }"); // class C { async *constructor() {} } is an early error
    test_same("class C { get ['constructor']() {} }"); // class C { get constructor() {} } is an early error
    test_same("class C { set ['constructor'](v) {} }"); // class C { set constructor(v) {} } is an early error
    test("class C { static ['constructor']() {} }", "class C { static constructor() {} }");
    test("class C { static 'constructor'() {} }", "class C { static constructor() {} }");
    test_same("class C { ['#constructor'] = 0 }"); // class C { #constructor = 0 } is an early error
    test_same("class C { accessor ['#constructor'] = 0 }"); // class C { accessor #constructor = 0 } is an early error
    test_same("class C { ['#constructor']() {} }"); // class C { #constructor() {} } is an early error
    test_same("class C { static ['#constructor'] = 0 }"); // class C { static #constructor = 0 } is an early error
    test_same("class C { static accessor ['#constructor'] = 0 }"); // class C { static accessor #constructor = 0 } is an early error
    test_same("class C { static ['#constructor']() {} }"); // class C { static #constructor() {} } is an early error
}

#[test]
fn fold_function_spread_args() {
    test_same("f(...a)");
    test_same("f(...a, ...b)");
    test_same("f(...a, b, ...c)");
    test_same("new F(...a)");

    test("f(...[])", "f()");
    test("f(...[1])", "f(1)");
    test("f(...[1, 2])", "f(1, 2)");
    test("f(...[1,,,3])", "f(1, void 0, void 0, 3)");
    test("f(a, ...[])", "f(a)");
    test("new F(...[])", "new F()");
    test("new F(...[1])", "new F(1)");
}

#[test]
fn test_fold_boolean_constructor() {
    test("var a = Boolean(true)", "var a = !0");
    // Don't fold the existence check to preserve behavior
    test("var a = Boolean?.(true)", "var a = Boolean?.(!0)");

    test("var a = Boolean(false)", "var a = !1");
    // Don't fold the existence check to preserve behavior
    test("var a = Boolean?.(false)", "var a = Boolean?.(!1)");

    test("var a = Boolean(1)", "var a = !0");
    // Don't fold the existence check to preserve behavior
    test_same("var a = Boolean?.(1)");

    test("var a = Boolean(x)", "var a = !!x");
    // Don't fold the existence check to preserve behavior
    test_same("var a = Boolean?.(x)");

    test("var a = Boolean({})", "var a = !0");
    // Don't fold the existence check to preserve behavior
    test_same("var a = Boolean?.({})");

    test("var a = Boolean()", "var a = !1;");
    test_same("var a = Boolean(!0, !1);");
}

#[test]
fn test_fold_string_constructor() {
    test("x = String()", "x = ''");
    test("var a = String(23)", "var a = '23'");
    // Don't fold the existence check to preserve behavior
    test_same("var a = String?.(23)");

    test("var a = String('hello')", "var a = 'hello'");
    test("var a = String(true)", "var a = 'true'");
    test("var a = String(!0)", "var a = 'true'");
    // Don't fold the existence check to preserve behavior
    test_same("var a = String?.('hello')");

    test_same("var s = Symbol(), a = String(s);");

    test_same("var a = String('hello', bar());");
    test_same("var a = String({valueOf: function() { return 1; }});");
}

#[test]
fn test_fold_number_constructor() {
    test("x = Number()", "x = 0");
    test("x = Number(true)", "x = 1");
    test("x = Number(false)", "x = 0");
    test("x = Number('foo')", "x = NaN");
}

#[test]
fn test_fold_big_int_constructor() {
    test("var x = BigInt(1n)", "var x = 1n");
    test_same("BigInt()");
    test("BigInt(1)", "");
}

#[test]
fn optional_catch_binding() {
    test("try { foo } catch(e) {}", "try { foo } catch {}");
    test("try { foo } catch(e) {foo}", "try { foo } catch {foo}");
    test_same("try { foo } catch(e) { bar(e) }");
    test_same("try { foo } catch([e]) {}");
    test_same("try { foo } catch({e}) {}");
    test_same("try { foo } catch(e) { var e = baz; bar(e) }");
    test("try { foo } catch(e) { var e = 2 }", "try { foo } catch { var e = 2 }");
    test_same("try { foo } catch(e) { var e = 2 } bar(e)");

    // FIXME catch(a) has no references but it cannot be removed.
    // test_same(
    // r#"var a = "PASS";
    // try {
    // throw "FAIL1";
    // } catch (a) {
    // var a = "FAIL2";
    // }
    // console.log(a);"#,
    // );

    test_target_same("try { foo } catch(e) {}", "chrome65");
}

#[test]
fn test_remove_name_from_expressions() {
    test("var a = function f() {}", "var a = function () {}");
    test_same("var a = function f() { return f; }");

    test("var a = class C {}", "var a = class {}");
    test_same("var a = class C { foo() { return C } }");

    let options = CompressOptions {
        keep_names: CompressOptionsKeepNames::function_only(),
        ..default_options()
    };
    test_same_options("var a = function f() {}", &options);

    let options =
        CompressOptions { keep_names: CompressOptionsKeepNames::class_only(), ..default_options() };
    test_same_options("var a = class C {}", &options);
}

#[test]
fn test_compress_destructuring_assignment_target() {
    test_same("var {y} = x");
    test_same("var {y, z} = x");
    test_same("var {y: z, z: y} = x");
    test("var {y: y} = x", "var {y} = x");
    test("var {y: z, 'z': y} = x", "var {y: z, z: y} = x");
    test("var {y: y, 'z': z} = x", "var {y, z} = x");
}

#[test]
fn test_object_callee_indirect_call() {
    test("Object(f)(1,2)", "f(1, 2)");
    test("(Object(g))(a)", "g(a)");
    test("Object(a.b)(x)", "(0, a.b)(x)");
    test_same("Object?.(f)(1)");
    test_same("function Object(x){return x} Object(f)(1)");
    test_same("Object(...a)(1)");
}

#[test]
fn test_rewrite_arguments_copy_loop() {
    test(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r) }",
        "function _() { var r = [...arguments]; console.log(r) }",
    );
    test(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) { r[a] = arguments[a]; } console.log(r) }",
        "function _() { var r = [...arguments]; console.log(r) }",
    );
    test(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) { r[a] = arguments[a] } console.log(r) }",
        "function _() { var r = [...arguments]; console.log(r) }",
    );
    test(
        "function _() { for (var e = arguments.length, r = new Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r) }",
        "function _() { var r = [...arguments]; console.log(r) }",
    );
    test(
        "function _() { for (var e = arguments.length, r = Array(e > 1 ? e - 1 : 0), a = 1; a < e; a++) r[a - 1] = arguments[a]; console.log(r) }",
        "function _() { var r = [...arguments].slice(1); console.log(r) }",
    );
    test(
        "function _() { for (var e = arguments.length, r = Array(e > 2 ? e - 2 : 0), a = 2; a < e; a++) r[a - 2] = arguments[a]; console.log(r) }",
        "function _() { var r = [...arguments].slice(2); console.log(r) }",
    );
    test(
        "function _() { for (var e = arguments.length, r = [], a = 0; a < e; a++) r[a] = arguments[a]; console.log(r) }",
        "function _() { var r = [...arguments]; console.log(r) }",
    );
    test(
        "function _() { for (var r = [], a = 0; a < arguments.length; a++) r[a] = arguments[a]; console.log(r) }",
        "function _() { var r = [...arguments]; console.log(r) }",
    );
    test(
        "function _() { for (var r = [], a = 1; a < arguments.length; a++) r[a - 1] = arguments[a]; console.log(r) }",
        "function _() { var r = [...arguments].slice(1); console.log(r) }",
    );
    test(
        "function _() { for (var r = [], a = 2; a < arguments.length; a++) r[a - 2] = arguments[a]; console.log(r) }",
        "function _() { var r = [...arguments].slice(2); console.log(r) }",
    );
    test(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; }",
        "function _() {}",
    );
    test(
        "function _() { for (var e = arguments.length, r = Array(e > 1 ? e - 1 : 0), a = 1; a < e; a++) r[a - 1] = arguments[a] }",
        "function _() {}",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) console.log(r[a]); }",
    );
    test(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) { r[a] = arguments[a]; console.log(r); } }",
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) (r[a] = arguments[a], console.log(r)) }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] += arguments[a]; }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a + 1] = arguments[a]; }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a - 0.5] = arguments[a]; }",
    );
    test(
        "function _() { var arguments; for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; }",
        "function _() { for (var arguments, e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = foo[a]; }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; e--) r[a] = arguments[a]; }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; r++) r[a] = arguments[a]; }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < r; r++) r[a] = arguments[a]; }",
    );
    test(
        "function _() { var arguments; for (var r = [], a = 0; a < arguments.length; a++) r[a] = arguments[a]; }",
        "function _() { for (var arguments, r = [], a = 0; a < arguments.length; a++) r[a] = arguments[a]; }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e > 1 ? e - 2 : 0), a = 2; a < e; a++) r[a - 2] = arguments[a]; }",
    );

    test_same_options_source_type(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r) }",
        SourceType::cjs(),
        &default_options(),
    );

    test_same(
        "for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r)",
    );
    test_same(
        "const _ = () => { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r) }",
    );
    test_same(
        "{ let _; for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r) }",
    );
    test(
        "function _() { { let _; for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r) } }",
        "function _() { { let _; var r = [...arguments]; console.log(r) } }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r, e) }",
    );
    test_same(
        "function _() { for (var e = arguments.length, r = Array(e), a = 0; a < e; a++) r[a] = arguments[a]; console.log(r, a) }",
    );
}

#[test]
fn test_flatten_nested_chain_expression() {
    test("(a.b)?.c", "a.b?.c");

    test("(a?.b)?.c", "a?.b?.c");
    test("(a?.b?.c)?.d", "a?.b?.c?.d");
    test("(((a?.b)?.c)?.d)?.e", "a?.b?.c?.d?.e");
    test("(a?.b)?.()", "a?.b?.()");
    test("(a?.b)?.(arg)", "a?.b?.(arg)");
    test("(a?.b)?.[0]", "a?.b?.[0]");
    test("(a?.b)?.[key]", "a?.b?.[key]");
    test("(a?.#b)?.c", "a?.#b?.c");
    test_same("a.b?.c");
    test_same("a?.b?.c");
    test_same("(a?.b).c");
}
