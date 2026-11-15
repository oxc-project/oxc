use std::fmt::Write;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_mangler::MangleOptions;
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::{mangle, mangle_script};

#[test]
fn merge_vars() {
    let cases = [
        // `i` and `j` share a name; `n` is live throughout.
        "function f(n) { for (var i = 0; i < n; i++) log(i); for (var j = 0; j < n; j++) log(j); }",
        // A parameter that is dead after its last read is reused.
        "function f(a, b) { var x = a + 1; log(x); var y = b * 2; log(y); var z = y + 3; return z; }",
        // Parameters never share a name with each other.
        "function f(a, b) { log(a); log(b); }",
        // A variable read in the loop stays live across the whole loop.
        "function f(a) { var u = 1; while (a--) { log(u); var w = 2; u = w; } var v = 3; log(v); }",
        "function f(a) { do { var x = g(); log(x); } while (a--); var y = 1; log(y); }",
        "function f(o) { for (var k in o) log(k); for (var v of o) log(v); var x = 1; log(x); }",
        "function f(o) { var k; for (k in o) log(k); var x = 1; log(x); }",
        // A variable read in `catch` is live across the whole `try` block.
        "function f(a) { var r; try { r = g(a); var s = 1; log(s); } catch (e) { log(r); } var q = h(); return q; }",
        "function f(a) { var x = 1; try { g(); } finally { log(x); } var y = 2; log(y); }",
        "function f(n) { var a = 1; for (;;) { try { if (n) break; } finally { log(a); } } var b = 2; log(b); }",
        "function f() { var a = 1; try { return a; } finally { var b = 2; log(b); } }",
        "function f(d) { var a = d + 1; switch (d) { case 1: log(a); case 2: var b = 3; log(b); break; default: log(a); } var c = 2; log(c); }",
        "function f(d) { var a = 1; switch (d) { case g(a): var b = 2; log(b); } }",
        "function f(n) { var x = 0; outer: for (var i = 0; i < n; i++) { for (var j = 0; j < n; j++) { if (j) continue outer; x += j; } } log(x); var k = 1; log(k); }",
        "function f(n) { var x = 1; block: { if (n) break block; log(x); } var y = 2; log(y); }",
        // Optional chains and logical operators may skip a write.
        "function f(o) { var a = o.x; var b = o?.y(a); log(b); var c = 1; log(c); }",
        "function f(o) { var a = 1; o?.[(a = 2)]; log(a); var b = 1; log(b); }",
        "function f(o) { var a = 1; o && (a = 2); log(a); var b = 1; log(b); }",
        "function f(o) { var a = 1; a ||= o; log(a); var b = 1; log(b); }",
        "function f(o) { var a = 1; var b = o ? (a = 2) : 3; log(a, b); var c = 1; log(c); }",
        // Destructuring defaults are evaluated conditionally, in order.
        "function f(o) { var a = 1; var { x = a } = o; log(x); var b = 1; log(b); }",
        "function f(o) { var a = 1, x; [x = a] = o; log(x); var b = 1; log(b); }",
        "function f(o) { var a = 1, b; ({ x: b = a } = o); log(b); var c = 1; log(c); }",
        // A variable read before it is written holds `undefined` until then.
        "function f() { var a = 1; log(a); var b; log(b); }",
        "function f(a) { log(a); var b; if (a) b = 1; log(b); }",
        // Dead writes still occupy their position.
        "function f() { var a = 1; log(2); var b = 3; log(b); a = 4; }",
        // Captured variables keep their own name.
        "function f(a) { var x = 1; log(x); var y = 2; return () => y; }",
        "function f(a) { var x = 1; log(x); var y = 2; return function () { return y; }; }",
        "function f() { var x = 1; log(x); var y = 2; class A { static z = y; } }",
        // `eval` and `with` can see every variable.
        "function f(a) { var x = 1; log(x); var y = 2; eval('y'); }",
        // Variables in nested blocks and arrow functions.
        "function f(a) { log(a); { var x = 1; log(x); } }",
        "(a) => { log(a); var x = 1; return x; }",
        "class A { static { var x = 1; log(x); var y = 2; log(y); } }",
        // Non-simple parameters never merge, but `var`s still can.
        "function f(a = 1) { log(a); var x = 1; log(x); var y = 2; log(y); }",
        "function f({ a }) { log(a); var x = 1; log(x); }",
        // A `var` redeclaring a parameter is the parameter.
        "function f(a) { log(a); var a = 1; log(a); var b = 2; log(b); }",
    ];
    let script_cases = [
        // Sloppy-mode `arguments` aliases the parameters, but not the `var`s.
        "function f(a, b) { log(a); var c = 1; log(arguments[0], c); }",
        "function f(a, b) { 'use strict'; log(a); var c = 1; log(arguments[0], c); }",
        "function f(a, b) { log(a); var c = 1; log(c); return () => arguments; }",
        // Annex B: a block function declaration also assigns the function-scoped `var`.
        "function f() { var g = 1; log(g); { function g() {} } var h = 2; log(h); }",
        "function f(o) { var x = 1; log(x); with (o) { var y = 2; log(y); } }",
        // `var x = 1` in a `catch` clause assigns the catch parameter.
        "function f() { var a = 1; log(a); try { g(); } catch (e) { var e = 2; log(e); } }",
        // The top level is not mangled in scripts by default.
        "var a = 1; log(a); var b = 2; log(b);",
    ];
    let top_level_cases = [
        "var a = 1; log(a); var b = 2; log(b);",
        // Exports are live bindings.
        "var a = 1; log(a); var b = 2; log(b); export { b };",
        "var a = 1; log(a); var b = 2; log(b); export { a };",
    ];

    let mut snapshot = String::new();
    let options = MangleOptions::default();
    for case in cases {
        write!(snapshot, "{case}\n{}\n", mangle(case, &options)).unwrap();
    }
    for case in script_cases {
        write!(snapshot, "{case}\n{}\n", mangle_script(case, &options)).unwrap();
    }
    let top_level = MangleOptions { top_level: Some(true), ..MangleOptions::default() };
    for case in top_level_cases {
        write!(snapshot, "{case}\n{}\n", mangle(case, &top_level)).unwrap();
    }
    let disabled = MangleOptions { merge_vars: false, ..MangleOptions::default() };
    let case =
        "function f(n) { for (var i = 0; i < n; i++) log(i); for (var j = 0; j < n; j++) log(j); }";
    write!(snapshot, "merge_vars: false\n{case}\n{}\n", mangle(case, &disabled)).unwrap();

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("merge_vars", snapshot);
    });
}

fn minify(source_text: &str, compress: Option<CompressOptions>) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    assert!(ret.diagnostics.is_empty(), "Parser errors: {:?}", ret.diagnostics);
    let mut program = ret.program;
    let options = MinifierOptions {
        mangle: Some(MangleOptions::default()),
        compress,
        ..MinifierOptions::default()
    };
    let ret = Minifier::new(options).minify(&allocator, &mut program);
    Codegen::new().with_scoping(ret.scoping).build(&program).code
}

#[test]
fn remove_redundant_vars() {
    let cases = [
        "export function f(n) { for (var i = 0; i < n; i++) log(i); for (var j = 0; j < n; j++) log(j); }",
        "export function f(n) { for (var i = 0; i < n; i++) log(i); for (var j = 0, k = g(); j < n; j++) log(j, k); }",
        "export function f(a, b) { log(a, b); var x = g(), y = g(); log(x, y); }",
        "export function f(a) { log(a); var b; log(b); }",
        "export function f(a) { log(a); var b, c = g(); b = g(); log(b, c); }",
        "export function f(a) { log(a); var b; b = g(); log(b); }",
        "export function f(o) { for (var k in o) log(k); for (var v of o) log(v); }",
        "export function f(a) { log(a); if (a) var b = g(); else var c; log(b, c); }",
        "export function f(a) { log(a); if (a) var b; else log(a); b = g(); log(b); }",
        "export function f(a) { log(a); while (a) var b; b = g(); log(b); }",
        "export function f(a) { log(a); l: var b; b = g(); log(b); }",
        "export function f(a) { log(a); switch (a) { case 1: var b = g(); log(b); } }",
        "export function f(a) { log(a); var [b] = g(); log(b); }",
        "export class A { static { var a = g(); log(a); var b = g(); log(b); } }",
        "var a = g(); log(a); var b = g(); log(b);",
        "var a = g(); log(a); var b = g(); log(b); export var c = 3;",
        "export const f = g(function (a) { log(a); var b = g(); log(b); });",
        "export const f = () => { var a = g(); log(a); var b = g(); log(b); };",
        "export class A { m() { var a = g(); log(a); var b = g(); log(b); } }",
        "export const o = { m() { var a = g(); log(a); var b = g(); log(b); } };",
        "export class A { p = () => { var a = g(); log(a); var b = g(); log(b); }; }",
        "export function f(a) { log(a); { var b = g(); log(b); } try { var c = g(); log(c); } finally { log(a); } }",
    ];

    let mut snapshot = String::new();
    for case in cases {
        for compress in [None, Some(CompressOptions::smallest())] {
            let label = if compress.is_some() { "compress" } else { "mangle" };
            let output = minify(case, compress.clone());
            if compress.is_none() {
                assert_eq!(minify(&output, None), output, "idempotency for {case}");
            }
            write!(snapshot, "{case}\n[{label}]\n{output}\n").unwrap();
        }
    }

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("remove_redundant_vars", snapshot);
    });
}
