use oxc_span::SourceType;

use crate::{
    CompressOptions, CompressOptionsKeepNames, default_options, test, test_options,
    test_options_source_type, test_same, test_same_options_source_type, test_smallest,
};

#[test]
fn merge_into_object_literal() {
    test("var o = {}; o.a = 1", "var o = { a: 1 }");
    test("let o = {}; o.a = 1", "let o = { a: 1 }");
    test("const o = {}; o.a = 1", "const o = { a: 1 }");
    test("var x = 1, o = {}; o.a = 1", "var x = 1, o = { a: 1 }");
    test("var o = { a: 1 }; o.b = 2; o.c = 3", "var o = { a: 1, b: 2, c: 3 }");
    test("var o = { a: 1 }; o.b = 2, o.c = 3", "var o = { a: 1, b: 2, c: 3 }");
    test("var o = {}; o['a b'] = 1", "var o = { 'a b': 1 }");
    test("var o = {}; o[0] = 1; o[1.5] = 2", "var o = { 0: 1, 1.5: 2 }");
    test(
        "var o = {}; o.a = [1, 'b'], o.b = { c: null }",
        "var o = { a: [1, 'b'], b: { c: null } }",
    );
    test("var o = {}; o.f = function() {}", "var o = { f: function() {} }");
    test("var o = {}; o.f = () => o", "var o = { f: () => o }");
    test(
        "var o = {}; o.a = { ...{ x: 1, set y(v) {} } }",
        "var o = { a: { ...{ x: 1, set y(v) {} } } }",
    );
    test("var o = { a: 1 }; o.a = 2; o.b = 3; o.a = 4", "var o = { a: 4, b: 3 }");
    // leading assignments of a sequence or a statement head merge too
    test(
        "function f() { var o = { p: 3 }; return o.q = 'foo', o.r = 'bar' }",
        "function f() { var o = { p: 3, q: 'foo' }; return o.r = 'bar' }",
    );
    test(
        "function f() { var o = { p: 3 }; return o.q = 'foo', o.p += '', g(o.q), o.p }",
        "function f() { var o = { p: 3, q: 'foo' }; return o.p += '', g(o.q), o.p }",
    );
    test(
        "function f() { var o = { p: 3 }; for (o.q = 'foo'; g(o.q);); return o.p }",
        "function f() { for (var o = { p: 3, q: 'foo' }; g(o.q);); return o.p }",
    );
    test(
        "function f() { var o = {}; for (var a in o.a = 'PASS', o) return o[a] }",
        "function f() { var o = { a: 'PASS' }; for (var a in o) return o[a] }",
    );
    test(
        "function f() { var x = { a: 1, c: (g('c'), 'C') }; x.b = 2; x[3] = function() { g(x) }, x['a'] = /foo/, x.bar = x; return x }",
        "function f() { var x = { a: /foo/, c: (g('c'), 'C'), b: 2, 3: function() { g(x) } }; return x.bar = x, x }",
    );
}

#[test]
fn existing_key() {
    // the last plain property with the key is replaced in place
    test("var o = { a: 1, b: 2 }; o.a = 3", "var o = { a: 3, b: 2 }");
    test("var o = { a: 1, b: 2, a: 3 }; o.a = 4", "var o = { a: 1, b: 2, a: 4 }");
    test("var o = { 1: true, '1': false, b: 2 }; o['1'] = 0", "var o = { 1: !0, 1: 0, b: 2 }");
    test("var o = { 'a b': 1, c: 2 }; o['a b'] = 3", "var o = { 'a b': 3, c: 2 }");
    test("var o = { 1: 1, b: 2 }; o['1'] = 3", "var o = { 1: 3, b: 2 }");
    test("var o = { 0x10: 1, b: 2 }; o[16] = 3", "var o = { 16: 3, b: 2 }");
    test("var o = { 0: 1, b: 2 }; o[-0] = 3", "var o = { 0: 3, b: 2 }");
    test("var o = { '-1': 1, b: 2 }; o[-1] = 3", "var o = { '-1': 3, b: 2 }");
    test("var o = { '01': 1, b: 2 }; o[1] = 3", "var o = { '01': 1, b: 2, 1: 3 }");
    test("var o = { ...a, b: 1 }; o.b = 2", "var o = { ...a, b: 2 }");
    test("var o = { [a]: 1, b: 2 }; o.b = 3", "var o = { [a]: 1, b: 3 }");
    test("var o = { a: 1, b: g() }; o.a = 2", "var o = { a: 2, b: g() }");
    // a later spread or unknown computed key may define the key again, so append
    test("var o = { a: 1, ...b }; o.a = 2", "var o = { a: 1, ...b, a: 2 }");
    test("var o = { a: 1, [b]: 2 }; o.a = 3", "var o = { a: 1, [b]: 2, a: 3 }");
    // `1e21` is the key `1e+21`, not `1000000000000000000000`
    test(
        "var o = { '1000000000000000000000': 1, b: 2 }; o[1e21] = 3",
        "var o = { '1000000000000000000000': 1, b: 2, 1e21: 3 }",
    );
    // a shorthand whose read has no side effects is replaced too
    test("function f(a) { var o = { a }; o.a = 1; return o }", "function f(a) { return { a: 1 } }");
    test(
        "function f(a) { var o = { a: a, b: 2 }; o.a = 1; return o }",
        "function f(a) { return { a: 1, b: 2 } }",
    );
    test("var o = { Math, b: 2 }; o.Math = 1", "var o = { Math: 1, b: 2 }");
    // the caller drops the old value, so `h` has no references left
    test_smallest(
        "export function f() { function h() {} var o = { h, b: 2 }; o.h = 1; return o }",
        "export function f() { return { h: 1, b: 2 } }",
    );
    // the old value has side effects, or is a method: append
    test("var o = { a: g() }; o.a = 1", "var o = { a: g(), a: 1 }");
    test("var o = { a() {} }; o.a = 1", "var o = { a() {}, a: 1 }");
    test("var o = { a, b: 2 }; o.a = 1", "var o = { a, b: 2, a: 1 }");
}

#[test]
fn bail() {
    // not a plain `=` write to a static key of the declared binding
    test_same("var o = {}; o.a += 1");
    test_same("var o = {}; o.a ||= 1");
    test_same("var o = {}; p.a = 1");
    test_same("var o = {}; o.a.b = 1");
    test_same("var o = {}; o[a] = 1");
    test_same("var o = {}; o[null] = 1");
    test_same("var o = { inf: 1 }; o[1e999] = 2");
    // a regex key is `String(/a/ig)`, which is "/a/gi", not its source text
    test_same("var o = { '/a/ig': 1, b: 2 }; o[/a/ig] = 2");
    // a lone surrogate is stored as `\u{FFFD}` and its code in hex
    test_same("var o = { '\u{FFFD}d800': 1, b: 2 }; o['\\uD800'] = 3");
    // `__proto__` sets the prototype
    test_same("var o = {}; o.__proto__ = null");
    test("var o = {}; o['__proto__'] = null", "var o = {}; o.__proto__ = null");
    // an accessor or a prototype setter would run on assignment
    test_same("var o = { get a() {} }; o.a = 1");
    test_same("var o = { set a(v) {} }; o.a = 1");
    test_same("var o = { get b() {} }; o.a = 1");
    test_same("var o = { __proto__: p }; o.a = 1");
    test_same("var o = { ['__proto__']: p }; o.a = 1");
    // the value may throw or read `o`
    test_same("var o = {}; o.a = b");
    test_same("var o = {}; o.a = b()");
    test_same("var o = {}; o.a = o");
    test_same("var o = {}; o.a = [o]");
    test_same("var o = {}; o.a = `${o}`");
    test_same("var o = {}; o.a = 1n / 0n");
    test_same("var o = {}; o.a = { ...{ get x() { return o } } }");
    // not the last declarator, or not an object literal
    test_same("var o = { a: 'PASS' }, a = o.a; o.a = 'FAIL'");
    test_same("var o = [], p = {}; o.a = 1");
    test_same("var o = []; o.a = 1");
    test_same("var o = g(); o.a = 1");
    test_same("var [o] = [{}]; o.a = 1");
    test_same("using o = {}; o.a = 1");
    test_same("await using o = {}; o.a = 1");
    // not right after the declaration
    test("var o = {}; g(); o.a = 1", "var o = {}; g(), o.a = 1");
    // a Script's top-level bindings are global (`var name = {}` in browsers)
    test_same_options_source_type("var o = {}; o.a = 1", SourceType::script(), &default_options());
    test_same_options_source_type("let o = {}; o.a = 1", SourceType::script(), &default_options());
    test_options_source_type(
        "(function() { var o = {}; o.a = 1; g(o) })()",
        "(function() { g({ a: 1 }) })()",
        SourceType::script(),
        &default_options(),
    );
}

#[test]
fn keep_names() {
    // `keep_names` does not keep a function anonymous, see `CompressOptionsKeepNames`
    let options =
        CompressOptions { keep_names: CompressOptionsKeepNames::all_true(), ..default_options() };
    test_options("var o = {}; o.f = function() {}", "var o = { f: function() {} }", &options);
    test_options("var o = {}; o.f = function g() {}", "var o = { f: function g() {} }", &options);
}
