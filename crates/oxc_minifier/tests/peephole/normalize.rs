use crate::{
    CompressOptions, default_options, test, test_options, test_options_source_type, test_same,
};

#[test]
fn test_while() {
    // Verify while loops are converted to FOR loops.
    test("while(c < b) foo()", "for(; c < b;) foo()");
}

#[test]
fn test_const_to_let() {
    test_same("const x = 1"); // keep top-level (can be replaced with "let" if it's ESM and not exported)
    test("{ const x = 1 }", "{ let x = 1 }");
    test_same("{ const x = 1; x = 2 }"); // keep assign error
    test_same("{ const x = 1; eval('x = 2') }"); // keep assign error
    test("{ const x = 1, y = 2 }", "{ let x = 1, y = 2 }");
    test("{ const { x } = { x: 1 } }", "{ let { x } = { x: 1 } }");
    test("{ const [x] = [1] }", "{ let [x] = [1] }");
    test("{ const [x = 1] = [] }", "{ let [x = 1] = [] }");
    test("for (const x in y);", "for (let x in y);");
    // TypeError: Assignment to constant variable.
    test_same("for (const i = 0; i < 1; i++);");
    test_same("{ const { a, ...b } = foo; b = 123; }");
    test_same("{ const [a, ...b] = foo; b = 123; }");
    test_same("for (const x in [1, 2, 3]) x++");
    test_same("for (const x of [1, 2, 3]) x++");
    test("{ let foo; const bar = undefined; }", "{ let foo, bar; }");
}

#[test]
fn test_void_ident() {
    test("var x; void x", "var x");
    test("void x", "x"); // reference error
}

#[test]
fn parens() {
    test("(((x)))", "x");
    test("(((a + b))) * c", "(a + b) * c");
}

#[test]
fn drop_console() {
    let options = CompressOptions { drop_console: true, ..default_options() };
    test_options("console.log()", "", &options);
    test_options("(() => console.log())()", "", &options);
    test_options(
        "(() => { try { return console.log() } catch {} })()",
        "(() => { try { return } catch {} })()",
        &options,
    );
}

#[test]
fn drop_debugger() {
    let options = CompressOptions { drop_debugger: true, ..default_options() };
    test_options("debugger", "", &options);
}

#[test]
fn fold_number_nan() {
    test("foo(Number.NaN)", "foo(NaN)");
    test_same("var Number; foo(Number.NaN)");
    test_same("let Number; foo((void 0).NaN)");
}

#[test]
fn pure_constructors() {
    test("new AggregateError", "AggregateError()");
    test("new ArrayBuffer", "");
    test("new Boolean", "");
    test("new DataView", "new DataView()");
    test("new Date", "");
    test("new Error", "");
    test("new EvalError", "");
    test("new Map", "");
    test("new Number", "");
    test("new Object", "");
    test("new RangeError", "");
    test("new ReferenceError", "");
    // RegExp with no arguments is valid (returns /(?:)/) and can be removed
    test("new RegExp", "");
    test("new Set", "");
    test("new String", "");
    test("new SyntaxError", "");
    test("new TypeError", "");
    test("new URIError", "");
    test("new WeakMap", "");
    test("new WeakSet", "");

    test("new AggregateError(null)", "AggregateError(null)");
    test("new ArrayBuffer(null)", "");
    test("new Boolean(null)", "");
    test_same("new DataView(null)");
    test("new Date(null)", "");
    test("new Error(null)", "");
    test("new EvalError(null)", "");
    test("new Map(null)", "");
    test("new Number(null)", "");
    test("new Object(null)", "");
    test("new RangeError(null)", "");
    test("new ReferenceError(null)", "");
    // null is not a string literal, can't statically validate
    test("new RegExp(null)", "RegExp(null)");
    test("new Set(null)", "");
    test("new String(null)", "");
    test("new SyntaxError(null)", "");
    test("new TypeError(null)", "");
    test("new URIError(null)", "");
    test("new WeakMap(null)", "");
    test("new WeakSet(null)", "");

    test("new AggregateError(undefined)", "AggregateError(void 0)");
    test("new ArrayBuffer(undefined)", "");
    test("new Boolean(undefined)", "");
    test_same("new DataView(void 0)");
    test("new Date(undefined)", "");
    test("new Error(undefined)", "");
    test("new EvalError(undefined)", "");
    test("new Map(undefined)", "");
    test("new Number(undefined)", "");
    test("new Object(undefined)", "");
    test("new RangeError(undefined)", "");
    test("new ReferenceError(undefined)", "");
    // undefined is not a string literal, can't statically validate
    test("new RegExp(undefined)", "RegExp(void 0)");
    test("new Set(undefined)", "");
    test("new String(undefined)", "");
    test("new SyntaxError(undefined)", "");
    test("new TypeError(undefined)", "");
    test("new URIError(undefined)", "");
    test("new WeakMap(undefined)", "");
    test("new WeakSet(undefined)", "");

    test("new AggregateError(0)", "AggregateError(0)");
    test("new ArrayBuffer(0)", "");
    test("new Boolean(0)", "");
    test_same("new DataView(0)");
    test("new Date(0)", "");
    test("new Error(0)", "");
    test("new EvalError(0)", "");
    test_same("new Map(0)");
    test("new Number(0)", "");
    test("new Object(0)", "");
    test("new RangeError(0)", "");
    test("new ReferenceError(0)", "");
    // 0 is not a string literal, can't statically validate
    test("new RegExp(0)", "RegExp(0)");
    test_same("new Set(0)");
    test("new String(0)", "");
    test("new SyntaxError(0)", "");
    test("new TypeError(0)", "");
    test("new URIError(0)", "");
    test_same("new WeakMap(0)");
    test_same("new WeakSet(0)");

    test("new AggregateError(10n)", "AggregateError(10n)");
    test_same("new ArrayBuffer(10n)");
    test("new Boolean(10n)", "");
    test_same("new DataView(10n)");
    test_same("new Date(10n)");
    test("new Error(10n)", "");
    test("new EvalError(10n)", "");
    test_same("new Map(10n)");
    test("new Number(10n)", "");
    test("new Object(10n)", "");
    test("new RangeError(10n)", "");
    test("new ReferenceError(10n)", "");
    // 10n is not a string literal, can't statically validate
    test("new RegExp(10n)", "RegExp(10n)");
    test_same("new Set(10n)");
    test("new String(10n)", "");
    test("new SyntaxError(10n)", "");
    test("new TypeError(10n)", "");
    test("new URIError(10n)", "");
    test_same("new WeakMap(10n)");
    test_same("new WeakSet(10n)");

    test("new AggregateError('')", "");
    test("new ArrayBuffer('')", "");
    test("new Boolean('')", "");
    test_same("new DataView('')");
    test("new Date('')", "");
    test("new Error('')", "");
    test("new EvalError('')", "");
    test("new Map('')", "");
    test("new Number('')", "");
    test("new Object('')", "");
    test("new RangeError('')", "");
    test("new ReferenceError('')", "");
    // Empty string is a valid pattern (matches everything)
    test("new RegExp('')", "");
    test("new Set('')", "");
    test("new String('')", "");
    test("new SyntaxError('')", "");
    test("new TypeError('')", "");
    test("new URIError('')", "");
    test("new WeakMap('')", "");
    test("new WeakSet('')", "");

    test("new AggregateError(!0)", "AggregateError(!0)");
    test("new ArrayBuffer(!0)", "");
    test("new Boolean(!0)", "");
    test_same("new DataView(!0)");
    test("new Date(!0)", "");
    test("new Error(!0)", "");
    test("new EvalError(!0)", "");
    test_same("new Map(!0)");
    test("new Number(!0)", "");
    test("new Object(!0)", "");
    test("new RangeError(!0)", "");
    test("new ReferenceError(!0)", "");
    // !0 is not a string literal, can't statically validate
    test("new RegExp(!0)", "RegExp(!0)");
    test_same("new Set(!0)");
    test("new String(!0)", "");
    test("new SyntaxError(!0)", "");
    test("new TypeError(!0)", "");
    test("new URIError(!0)", "");
    test_same("new WeakMap(!0)");
    test_same("new WeakSet(!0)");

    test("new AggregateError([])", "");
    test("new ArrayBuffer([])", "");
    test("new Boolean([])", "");
    test_same("new DataView([])");
    test("new Date([])", "");
    test("new Error([])", "");
    test("new EvalError([])", "");
    test("new Map([])", "");
    test("new Number([])", "");
    test("new Object([])", "");
    test("new RangeError([])", "");
    test("new ReferenceError([])", "");
    // Array arguments are object type, so conversion doesn't happen
    test_same("new RegExp([])");
    test("new Set([])", "");
    test("new String([])", "");
    test("new SyntaxError([])", "");
    test("new TypeError([])", "");
    test("new URIError([])", "");
    test("new WeakMap([])", "");
    test("new WeakSet([])", "");

    test("new AggregateError(a)", "AggregateError(a)");
    test_same("new ArrayBuffer(a)");
    test_same("new Boolean(a)");
    test_same("new DataView(a)");
    test_same("new Date(a)");
    test("new Error(a)", "Error(a)");
    test("new EvalError(a)", "EvalError(a)");
    test_same("new Map(a)");
    test_same("new Number(a)");
    test_same("new Object(a)");
    test("new RangeError(a)", "RangeError(a)");
    test("new ReferenceError(a)", "ReferenceError(a)");
    test_same("new RegExp(a)");
    test_same("new Set(a)");
    test_same("new String(a)");
    test("new SyntaxError(a)", "SyntaxError(a)");
    test("new TypeError(a)", "TypeError(a)");
    test("new URIError(a)", "URIError(a)");
    test_same("new WeakMap(a)");
    test_same("new WeakSet(a)");
}

#[test]
fn remove_unused_use_strict_directive() {
    use oxc_span::SourceType;
    let options = default_options();
    let source_type = SourceType::cjs();
    test_options_source_type(
        "'use strict'; function _() { 'use strict' }",
        "'use strict'; function _() {  }",
        source_type,
        &options,
    );
    test_options_source_type(
        "function _() { 'use strict'; function __() { 'use strict' } }",
        "function _() { 'use strict'; function __() { } }",
        source_type,
        &options,
    );
    test("'use strict'; function _() { 'use strict' }", "function _() {}");
    test("'use strict';", "");
}
