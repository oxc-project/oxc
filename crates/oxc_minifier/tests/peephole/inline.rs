use oxc_span::SourceType;

use crate::{
    CompressOptions, test_options, test_options_source_type, test_same_options, test_smallest,
};

// https://github.com/oxc-project/oxc/issues/13051
#[test]
fn readonly_var() {
    // Top-level `var` with constant initializer, only read from inside a hoisted
    // function that is called after the declaration. Safe to inline because no
    // statement before the `var` can run code that reads it.
    test_smallest(
        "var used = false; function test() { if (used) return 123; return 321; } log(test());",
        "function test() { return 321; } log(test());",
    );

    // Multiple readonly vars in a row — every preceding statement is itself a
    // safe `var = literal` so each one is inlineable in turn.
    test_smallest(
        "var a = 1; var b = 2; function f() { return a + b; } log(f());",
        "function f() { return 3; } log(f());",
    );
}

#[test]
fn readonly_var_unsafe_preceding_call() {
    // sapphi-red's case: a preceding call could invoke `output` before `foo` is
    // assigned. The read inside `output` would see `undefined`, so inlining
    // `foo` to `true` would change observable behavior — `foo` must stay.
    test_smallest(
        "output(); var foo = true; function output() { if (!foo) log('foo'); }",
        "output(); var foo = !0; function output() { foo || log('foo'); }",
    );
}

#[test]
fn readonly_var_unsafe_preceding_read() {
    // A preceding statement reads the var before its initializer runs;
    // the read must observe `undefined`, not the constant.
    test_smallest("var y = foo; var foo = 1; log(y);", "var y = foo, foo = 1; log(y);");

    // Canonical hoisting trap: the name is used directly before its own `var`
    // declaration, so the read sees the hoisted `undefined`. `console.log(a)`
    // must print `undefined`, never `0`. Doubly guarded — the read is in the
    // same call frame (does not cross a function boundary) and the preceding
    // call ends the declarative prelude.
    test_smallest("console.log(a); var a = 0;", "console.log(a); var a = 0;");
}

#[test]
fn readonly_var_reassigned() {
    // `foo` has a write reference, so even though `var foo = 1;` is at the top,
    // inlining is unsafe.
    test_smallest("var foo = 1; foo = 2; log(foo);", "var foo = 1; foo = 2, log(foo);");
}

#[test]
fn readonly_var_reassigned_cross_function_read() {
    // The read crosses a function boundary (the gap this path targets), but
    // `foo` is also reassigned. The predicate's read-loop ignores writes, so it
    // relies on the downstream `write_references_count` guard in
    // `inline_identifier_reference` to block inlining — substituting `1` would be
    // wrong once `foo = 2` runs before `f()` is called.
    test_smallest(
        "var foo = 1; foo = 2; function f() { return foo; } log(f());",
        "var foo = 1; foo = 2; function f() { return foo; } log(f());",
    );
}

#[test]
fn readonly_var_reader_declared_before_var() {
    // Known limitation: when the reading function is declared *before* the var,
    // the symbol's constant isn't recorded until `exit_variable_declarator` —
    // after `f`'s body (and its `foo` reference) was already visited in source
    // order. The in-pass design can't reach back, so this otherwise-safe case is
    // conservatively left un-inlined. Asserted to make any future improvement a
    // conscious change rather than a silent one.
    test_smallest(
        "function f() { return foo; } var foo = true; log(f());",
        "function f() { return foo; } var foo = !0; log(f());",
    );
}

#[test]
fn readonly_var_with_imports_present() {
    // A circular importer can observe ANY module-private var our exported
    // functions/classes close over, regardless of export status. As long as the
    // module has any static import, skip program-scope inlining outright.
    // Non-exported var captured by an exported function — the cyclic-closure
    // hazard Codex flagged. Must NOT inline.
    test_smallest(
        "import './b.js'; var flag = true; export function check() { return flag; }",
        "import './b.js'; var flag = !0; export function check() { return flag; }",
    );
    // A write-once falsy `var` read only in boolean context (`if (DEBUG)`) folds
    // even with imports present: the cyclic-import hazard is that an observer sees
    // the hoisted `undefined` instead of the init value, but in boolean context
    // `undefined` and the falsy init are indistinguishable (`if (undefined)` ===
    // `if (false)`), and an importer cannot write the binding to make it truthy.
    // So `DEBUG` collapses and `log` becomes a no-op. (boolean_falsy, #14001)
    test_smallest(
        "import './side-effect.js'; var DEBUG = false; function log(x) { if (DEBUG) console.log(x); } log('hi');",
        "import './side-effect.js';",
    );
    // Imports are hoisted, so an import appearing *after* the var in source
    // still triggers the gate — the pre-scan checks the whole body.
    test_smallest(
        "var flag = true; import './b.js'; export function check() { return flag; }",
        "var flag = !0; import './b.js'; export function check() { return flag; }",
    );
}

#[test]
fn readonly_var_with_reexports_present() {
    // `export * from` and `export { … } from` are module loaders too — they
    // evaluate foreign modules and create the same cyclic-eval hazard.
    test_smallest(
        "export * from './other.js'; var flag = true; export function check() { return flag; }",
        "export * from './other.js'; var flag = !0; export function check() { return flag; }",
    );
    test_smallest(
        "export { y } from './y.js'; var flag = true; export function check() { return flag; }",
        "export { y } from './y.js'; var flag = !0; export function check() { return flag; }",
    );
}

#[test]
fn readonly_var_through_declarative_exports() {
    // `export function …`, `export var <literal>`, `export { … }`, and
    // `export default function …` are declarative wrappers — no user code runs
    // at module init, so the body's declarative prelude continues through them
    // and a later var stays inlineable.
    test_smallest(
        "export function helper() {} export var A = 1; var b = 2; function f() { return b; } log(f());",
        "export function helper() {} export var A = 1; function f() { return 2; } log(f());",
    );
    test_smallest(
        "export default function helper() {} var b = 2; function f() { return b; } log(f());",
        "export default function helper() {} function f() { return 2; } log(f());",
    );
}

#[test]
fn readonly_var_export_default_expression_breaks_prelude() {
    // `export default <expr>` evaluates the expression at module init, which
    // can call user code. A later var must not inline.
    test_smallest(
        "export default sideEffect(); var b = 2; function f() { return b; } log(f());",
        "export default sideEffect(); var b = 2; function f() { return b; } log(f());",
    );
}

#[test]
fn readonly_var_unsafe_destructuring_default_prelude() {
    // A preceding destructuring var with a default-call evaluates the call
    // before `flag = true`. If `flag` were inlined inside `inner`, the call
    // would observe `true` instead of the required hoisted `undefined`.
    test_smallest(
        "var [x = inner()] = ''; var flag = true; function inner() { return flag; } log(x);",
        "var [x = inner()] = '', flag = !0; function inner() { return flag; } log(x);",
    );
}

#[test]
fn readonly_var_in_function_body() {
    // #2: same declarative-prelude analysis, applied to function bodies. The
    // var sits at the function's body scope and is read from a nested function,
    // which substitute_single_use_symbol can't reach.
    test_smallest(
        "function outer() { var flag = false; function inner() { return flag ? 1 : 2; } return inner(); } log(outer());",
        "function outer() { function inner() { return 2; } return inner(); } log(outer());",
    );
}

#[test]
fn readonly_var_in_function_body_unsafe_preceding_call() {
    // Same hoisting hazard as top-level: an observable call before the var
    // inside the function body could invoke a hoisted inner function that
    // reads `flag` as `undefined`. Skip.
    test_smallest(
        "function outer() { sideEffect(); var flag = true; function inner() { return flag; } return inner(); } log(outer());",
        "function outer() { sideEffect(); var flag = !0; function inner() { return flag; } return inner(); } log(outer());",
    );
}

#[test]
fn readonly_var_script_mode() {
    // Top-level `var` in script mode creates a property on the global object;
    // another script can mutate it between this line and a later function call.
    // Don't inline.
    test_options_source_type(
        "var used = false; function test() { if (used) return 123; return 321; } log(test());",
        "var used = !1; function test() { return used ? 123 : 321; } log(test());",
        SourceType::cjs().with_script(true),
        &CompressOptions::smallest(),
    );
}

#[test]
fn readonly_var_after_type_declaration() {
    // Type-only declarations (`type`, `interface`) are erased and run no code,
    // so they don't end the declarative prelude — a following readonly var
    // stays inlineable.
    test_options_source_type(
        "type T = number; interface I {} var b = 2; function f() { return b; } log(f());",
        "type T = number; interface I {} function f() { return 2; } log(f());",
        SourceType::ts().with_module(true),
        &CompressOptions::smallest(),
    );
}

// A write-once falsy `var` flag read only in boolean context folds even past a
// dirty declarative prelude — the bundled `var hydrating = false` shape read by
// `if (hydrating)` throughout a framework runtime (Svelte/Vue, #14001). The
// value-context constant is withheld for hoisting safety, but `undefined`
// (pre-init) and the falsy init are indistinguishable in boolean context.
#[test]
fn fold_writeonce_falsy_var_in_boolean_context() {
    // Multiple same-frame boolean reads.
    test_smallest("var h = false; if (h) a(); if (h) b()", "");
    // Read inside a function, past a side-effectful prelude (`g()` runs first):
    // the hoisting gate withholds value-context folding; boolean context is sound.
    test_smallest("g(); var h = false; function f() { if (h) a() } f()", "g();");

    // Value context must NOT fold (a pre-init read would observe `undefined`).
    test_smallest(
        "g(); var h = false; function f() { sink(h) } f()",
        "g(); var h = !1; function f() { sink(h); } f();",
    );
    // Reassigned => not write-once => not folded.
    test_smallest("var h = false; h = 1; if (h) a()", "var h = !1; h = 1, h && a();");

    // Script mode: a top-level `var` is a global another script can reassign, so
    // an in-module write count of 0 doesn't prove write-once — not folded.
    test_options_source_type(
        "var h = false; function f() { if (h) a() } f()",
        "var h = !1; function f() { h && a(); } f();",
        SourceType::cjs().with_script(true),
        &CompressOptions::smallest(),
    );
}

#[test]
fn r#const() {
    let options = CompressOptions::smallest();
    test_options("const foo = 1; log(foo)", "log(1)", &options);
    test_options("export const foo = 1; log(foo)", "export const foo = 1; log(1)", &options);

    test_options("let foo = 1; log(foo)", "log(1)", &options);
    test_options("export let foo = 1; log(foo)", "export let foo = 1; log(1)", &options);
}

// https://github.com/oxc-project/oxc/issues/20282
// Dead code guarded by a condition that depends on a read-only `const` is
// eliminated even when the `const` is referenced more than once. The value is
// resolved through `SymbolValue` constant tracking during constant evaluation,
// not single-use inlining, so the old refcount==1 restriction no longer blocks
// it.
#[test]
fn dead_code_depending_on_const() {
    // Exact reproduction from the issue: `ENABLE_PKG` is always `false`, so the
    // guarded call and both now-unused declarations are removed.
    test_smallest(
        "const MODE = 'production';
         const ENABLE_PKG = MODE === 'foo' || MODE === 'bar';
         if (ENABLE_PKG) { longFunction() }",
        "",
    );

    // Commenter's variant: `MODE` is read twice (in `ENABLE_PKG`'s initializer
    // and in the `if` test), yet the dead branch still folds away.
    test_smallest(
        "const MODE = 'production';
         const ENABLE_PKG = MODE === 'foo';
         if (MODE !== 'production') { longFunction() }",
        "",
    );

    // Negative case: a reassigned binding is not a constant, so the guard must
    // be preserved (no flow-sensitive last-write analysis here).
    test_smallest(
        "let MODE = 'production'; MODE = 'dev'; if (MODE !== 'production') { longFunction() }",
        "let MODE = 'production'; MODE = 'dev', MODE !== 'production' && longFunction();",
    );

    // Negative case: a non-constant initializer leaves the guard intact (the
    // value is inlined, but the call is not eliminated).
    test_smallest(
        "const MODE = globalThis.mode; if (MODE !== 'production') { longFunction() }",
        "globalThis.mode !== 'production' && longFunction();",
    );
}

// https://github.com/rolldown/rolldown/issues/10174
// A never-assigned binding with no initializer reads as `undefined`, but the
// textual inline prints `void 0` — longer than a mangled identifier read plus
// its share of a declaration, and with no initializer there is nothing whose
// removal pays for it. Keep the read; constant-driven folds still see the
// value through `SymbolValue` tracking.
#[test]
fn keep_value_context_read_of_uninitialized_binding() {
    let options = CompressOptions::smallest();
    // The exact rolldown#10174 shape: a value-context assignment read.
    test_options(
        "let undefinedVar; export let value; export function reset() { value = undefinedVar; }",
        "let undefinedVar; export let value; export function reset() { value = undefinedVar; }",
        &options,
    );
    // Multi-read value context.
    test_options(
        "let u; export function f() { g(u), g(u); }",
        "let u; export function f() { g(u), g(u); }",
        &options,
    );

    // Near-misses: folds that consume the value must keep working without the
    // textual inline — evaluation resolves the read through `SymbolValue`.
    test_options("let u; export function f() { return u; }", "export function f() {}", &options);
    test_options("let u; export function f() { if (u) g(); }", "export function f() {}", &options);
    test_options(
        "let u; export function f() { return u === void 0; }",
        "export function f() { return !0; }",
        &options,
    );
    // Near-miss: an explicit `undefined` initializer still inlines — the inline
    // eliminates the `= void 0` initializer text along with the declaration.
    test_options(
        "const foo = undefined; export function f() { g(foo); }",
        "export function f() { g(void 0); }",
        &options,
    );
}

#[test]
fn small_value() {
    let options = CompressOptions::smallest();
    test_options("const foo = 999; log(foo), log(foo)", "log(999), log(999)", &options);
    test_options("const foo = -99; log(foo), log(foo)", "log(-99), log(-99)", &options);
    test_same_options("const foo = 1000; log(foo), log(foo)", &options);
    test_same_options("const foo = -100; log(foo), log(foo)", &options);

    test_same_options("const foo = 0n; log(foo), log(foo)", &options);

    test_options("const foo = 'aaa'; log(foo), log(foo)", "log('aaa'), log('aaa')", &options);
    test_same_options("const foo = 'aaaa'; log(foo), log(foo)", &options);

    test_options("const foo = true; log(foo), log(foo)", "log(!0), log(!0)", &options);
    test_options("const foo = false; log(foo), log(foo)", "log(!1), log(!1)", &options);
    test_options("const foo = undefined; log(foo), log(foo)", "log(void 0), log(void 0)", &options);
    test_options("const foo = null; log(foo), log(foo)", "log(null), log(null)", &options);

    test_options(
        r#"
        const o = 'o';
        const d = 'd';
        const boolean = false;
        var frag = `<p autocapitalize="${`w${o}r${d}s`}" contenteditable="${boolean}"/>`;
        console.log(frag);
        "#,
        r#"console.log('<p autocapitalize="words" contenteditable="false"/>');"#,
        &options,
    );
}
