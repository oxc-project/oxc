use oxc_span::SourceType;

use crate::{
    CompressOptions, test, test_options, test_options_source_type, test_same, test_same_options,
    test_same_options_source_type, test_smallest,
};

// A hoisted function called before a `let`/`const` executes reads the
// binding inside its temporal dead zone and must throw a ReferenceError.
// Folding the read to the recorded constant (and then dropping the now-pure
// call) erases the throw. The constant is only recorded when the declarator
// sits in its body's clean declarative prelude, where nothing can have run.
#[test]
fn lexical_tdz_read_not_folded() {
    // Arithmetic fold over the implicit undefined.
    test_same("g();\nlet a;\nfunction g() {\n\treturn a * 2;\n}");
    // Explicit initializer via the single-read inliner.
    test_same("g();\nlet a = 1;\nfunction g() {\n\treturn a * 2;\n}");
    // Nullish fold.
    test_same("g();\nlet a;\nfunction g() {\n\treturn a ?? 'x';\n}");
}

#[test]
fn lexical_prelude_reads_still_fold() {
    // Declarator first: nothing can run before it, so a function created
    // later cannot observe the TDZ.
    test("let a = 1; function g() { return a * 2 } g();", "let a = 1; function g() { return 2 }");
    test("let a; window.f = () => a ?? 'x';", "let a; window.f = () => 'x';");
    // Function expressions and arrows created after the declarator cannot
    // run inside the TDZ, even past an executable prelude — only hoisted
    // function declarations can.
    test(
        "k(); let a = 1; NOOP(function() { return a * 2 });",
        "k(); let a = 1; NOOP(function() { return 2 });",
    );
    test("k(); let a = 1; NOOP(() => a * 2);", "k(); let a = 1; NOOP(() => 2);");
    // Same inside a block, which has no prelude tracking at all.
    test("{ let a = 1; NOOP(() => a * 2); }", "{ let a = 1; NOOP(() => 2); }");
    // A hoisted function referenced only after the declarator cannot have
    // run before it, even past an executable prelude — the shape of
    // generated code that specializes on top-level flag constants.
    test(
        "k(); let a = 1; function g() { return a * 2 } NOOP(g);",
        "k(); let a = 1; function g() { return 2 } NOOP(g);",
    );
}

// The purity pre-scan walks a function's body from the statement position
// and its side-effect analysis does not model the TDZ, so a read of an outer
// lexical looked pure with the value completely unknown; the never-reset
// cache then let a later pass drop the pre-declaration `g()` call. The
// `if (1) h()` forces that second pass.
#[test]
fn lexical_tdz_read_not_pure_cached() {
    test_smallest(
        "g(); let a = 1; function g() { return a * 2 } if (1) h();",
        "g();\nlet a = 1;\nfunction g() {\n\treturn a * 2;\n}\nh();",
    );
    // Same with the declarator after the function: at scan time the
    // declarator hasn't been visited, so the cache must also bail.
    test_smallest(
        "function g() { return a * 2 } g(); let a = 1; if (1) h();",
        "function g() {\n\treturn a * 2;\n}\ng();\nlet a = 1;\nh();",
    );
    // Classes have the same temporal dead zone; the bare `C;` read must
    // survive too — dropping it would empty `g` into the pure-call cache.
    test_smallest(
        "g(); class C {} function g() { C; } if (1) h();",
        "g();\nclass C {}\nfunction g() {\n\tC;\n}\nh();",
    );
    test_smallest(
        "g(); let a = 1; function g() { a; } if (1) h();",
        "g();\nlet a = 1;\nfunction g() {\n\ta;\n}\nh();",
    );
    // The hazard is scope-relative: a declaration nested in another function
    // body binds its symbol in that body's `Function`-flagged scope, which
    // must not be mistaken for a named function expression's self-binding.
    test_smallest(
        "function outer() { g(); let a = 1; function g() { return a * 2 } } outer();",
        "function outer() {\n\tg();\n\tlet a = 1;\n\tfunction g() {\n\t\treturn a * 2;\n\t}\n}\nouter();",
    );
}

// Script sources: only TOP-LEVEL hoisted functions are global-object
// properties reachable without a local reference. A nested body (a UMD
// factory, an IIFE) is a closed namespace like a module, so it uses the
// precise reference bit instead of the blunt prelude rule.
#[test]
fn script_nested_body_uses_reference_bit() {
    let script = SourceType::cjs().with_script(true);
    let options = CompressOptions::smallest();
    // No hoisted-function reference before the declarator: folds.
    test_options_source_type(
        "function outer() { k(); let a = 1; function g() { return a * 2 } NOOP(g); } outer();",
        "function outer() {\n\tk();\n\tfunction g() {\n\t\treturn 2;\n\t}\n\tNOOP(g);\n}\nouter();",
        script,
        &options,
    );
    // Hoisted call before the declarator: still withheld.
    test_same_options_source_type(
        "function outer() {\n\tg();\n\tlet a = 1;\n\tfunction g() {\n\t\treturn a * 2;\n\t}\n}\nouter();",
        script,
        &options,
    );
    // Top level of the script stays blunt: an executable statement before
    // the declarator withholds even without a local hoisted reference.
    test_same_options_source_type(
        "k();\nlet a = 1;\nfunction g() {\n\treturn a * 2;\n}\nNOOP(g);",
        script,
        &options,
    );
}

// Accepted limitation: a cyclic importer can call an exported function —
// and transitively any hoisted function — before the module body runs, so
// this fold erases a deterministic ReferenceError under an import cycle.
// Guarding it would withhold every flag-constant specialization in any
// module with imports (the generated deserializers, bundler chunks).
#[test]
fn lexical_tdz_import_cycle_accepted_limitation() {
    test(
        "import 'm'; let a = 1; function g() { return a * 2 } NOOP(g);",
        "import 'm'; let a = 1; function g() { return 2 } NOOP(g);",
    );
}

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
