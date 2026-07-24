use cow_utils::CowUtils;
use rustc_hash::FxHashSet;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_compat::EngineTargets;
use oxc_minifier::{CompressOptions, Compressor, TreeShakeOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[track_caller]
fn run(source_text: &str, source_type: SourceType, options: Option<CompressOptions>) -> String {
    run_with_iterations(source_text, source_type, options).0
}

#[track_caller]
fn run_with_iterations(
    source_text: &str,
    source_type: SourceType,
    options: Option<CompressOptions>,
) -> (String, u8) {
    let allocator = Allocator::default();
    let mut ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.diagnostics.is_empty(), "Parser errors: {:?}", ret.diagnostics);
    let program = &mut ret.program;
    let iterations = options
        .map_or(0, |options| Compressor::new(&allocator).dead_code_elimination(program, options));
    (Codegen::new().build(program).code, iterations)
}

#[track_caller]
fn test(source_text: &str, expected: &str) {
    let t = "('production' == 'production')";
    let f = "('production' == 'development')";
    let source_text = source_text.cow_replace("true", t);
    let source_text = source_text.cow_replace("false", f);
    test_with_options_source_type(
        &source_text,
        expected,
        SourceType::default(),
        CompressOptions::dce(),
    );
}

#[track_caller]
fn test_with_iterations(source_text: &str, expected: &str, expected_iterations: u8) {
    let t = "('production' == 'production')";
    let f = "('production' == 'development')";
    let source_text = source_text.cow_replace("true", t);
    let source_text = source_text.cow_replace("false", f);
    let iterations = test_with_options_source_type(
        &source_text,
        expected,
        SourceType::default(),
        CompressOptions::dce(),
    );
    assert_eq!(
        iterations, expected_iterations,
        "\niteration count for source\n{source_text}\nexpect\n{expected_iterations}\ngot\n{iterations}"
    );
}

#[track_caller]
fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[track_caller]
fn test_source_type(source_text: &str, expected: &str, source_type: SourceType) {
    test_with_options_source_type(source_text, expected, source_type, CompressOptions::dce());
}

#[track_caller]
fn test_same_source_type(source_text: &str, source_type: SourceType) {
    test_source_type(source_text, source_text, source_type);
}

#[track_caller]
fn test_with_options(source_text: &str, expected: &str, options: CompressOptions) {
    test_with_options_source_type(source_text, expected, SourceType::default(), options);
}

#[track_caller]
fn options_for_target(target: &str) -> CompressOptions {
    CompressOptions {
        target: EngineTargets::from_target(target).unwrap(),
        ..CompressOptions::dce()
    }
}

#[track_caller]
fn test_with_target(source_text: &str, expected: &str, target: &str) {
    test_with_options(source_text, expected, options_for_target(target));
}

#[track_caller]
fn test_same_with_target(source_text: &str, target: &str) {
    test_with_target(source_text, source_text, target);
}

#[track_caller]
fn test_with_options_source_type(
    source_text: &str,
    expected: &str,
    source_type: SourceType,
    options: CompressOptions,
) -> u8 {
    let (result, iterations) = run_with_iterations(source_text, source_type, Some(options.clone()));
    let expected = run(expected, source_type, None);
    assert_eq!(result, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{result}");

    // Check idempotency.
    let second = run(&result, source_type, Some(options));
    assert_eq!(
        result, second,
        "\nidempotency for source\n{source_text}\ngot\n{result}\nthen\n{second}"
    );
    iterations
}

#[test]
fn dce_if_statement() {
    test("if (true) { foo }", "foo");
    test("if (true) { foo } else { bar }", "foo");
    test("if (false) { foo } else { bar }", "bar");

    test("if (xxx) { foo } else if (false) { bar }", "if (xxx) foo");
    test("if (xxx) { foo } else if (false) { bar } else { baz }", "if (xxx) foo; else baz");
    test("if (xxx) { foo } else if (false) { bar } else if (false) { baz }", "if (xxx) foo");
    test(
        "if (xxx) { foo } else if (false) { bar } else if (false) { baz } else { quaz }",
        "if (xxx) foo; else quaz",
    );
    test(
        "if (xxx) { foo } else if (true) { bar } else if (false) { baz }",
        "if (xxx) foo; else bar",
    );
    test(
        "if (xxx) { foo } else if (false) { bar } else if (true) { baz }",
        "if (xxx) foo; else baz",
    );
    test(
        "if (xxx) { foo } else if (true) { bar } else if (true) { baz }",
        "if (xxx) foo; else bar",
    );
    test(
        "if (xxx) { foo } else if (false) { var a; var b; } else if (false) { var c; var d; } f(a,b,c,d)",
        "if (xxx) foo; else if (0) var a, b; else if (0) var c, d; f(a,b,c,d)",
    );

    test("if (!false) { foo }", "foo");
    test("if (!true) { foo } else { bar }", "bar");

    test("if (!false && xxx) { foo }", "if (xxx) foo");
    test("if (!true && yyy) { foo } else { bar }", "bar");
    test("if (xxx && false) { foo } else { bar }", "if (xxx && false); else bar");

    test("if (true || xxx) { foo }", "foo");
    test("if (false || xxx) { foo }", "if (xxx) foo");
    test("if (xxx || true) { foo } else { bar }", "if (xxx || true) foo");

    test("if ('production' == 'production') { foo } else { bar }", "foo");
    test("if ('development' == 'production') { foo } else { bar }", "bar");

    test("if ('production' === 'production') { foo } else { bar }", "foo");
    test("if ('development' === 'production') { foo } else { bar }", "bar");

    // Shadowed `undefined` as a variable should not be erased.
    // This is a rollup test.
    // <https://github.com/rollup/rollup/blob/master/test/function/samples/allow-undefined-as-parameter/main.js>
    test_same("function foo(undefined) { if (!undefined) throw Error('') } foo()");

    test("function foo() { if (undefined) { bar } } foo()", "");
    test("function foo() { { bar } } foo()", "function foo() { bar } foo()");

    test("if (true) { foo; } if (true) { foo; }", "foo; foo;");
    test(
        "export function baz() { if (true) { foo; return } foo; if (true) { bar; return } bar; }",
        "export function baz() { foo }",
    );

    // nested expression
    test(
        "const a = { fn: function() { if (true) { foo; } } }; bar(a)",
        "bar({ fn: function() { foo; } })",
    );

    // parenthesized
    test("if (!!(false)) { REMOVE; } else { KEEP; }", "KEEP");

    // typeof
    test("if (typeof 1 !== 'number') { REMOVE; }", "");
    test("if (typeof false !== 'boolean') { REMOVE; }", "");
    test("if (typeof 1 === 'string') { REMOVE; }", "");

    // Complicated
    test(
        "if (unknown)
            for (var x = 1; x-- > 0; )
                if (foo++, false) foo++;
                else 'Side effect free code to be dropped';
            else throw new Error();",
        "if (unknown) {
            for (var x = 1; x-- > 0;) if (foo++, false);
           } else throw new Error();",
    );
}

#[test]
fn dce_while_statement() {
    test_same("while (true);");
    test_same("while (false);");
}

#[test]
fn dce_conditional_expression() {
    test("false ? foo : bar;", "bar");
    test("true ? foo : bar;", "foo");

    test("!true ? foo : bar;", "bar");
    test("!false ? foo : bar;", "foo");

    test("!!false ? foo : bar;", "bar");
    test("!!true ? foo : bar;", "foo");

    test("const foo = true ? A : B", "A");
    test("const foo = false ? A : B", "B");
}

#[test]
fn dce_logical_expression() {
    test("false && bar()", "");
    test("true && bar()", "bar()");

    test("var foo = false && bar(); baz(foo)", "baz(false)");
    test("var foo = true && bar(); baz(foo)", "var foo = bar(); baz(foo)");

    test("foo = false && bar()", "foo = false");
    test("foo = true && bar()", "foo = bar()");

    test(
        "const x = 'keep'; const y = 'remove'; foo(x || y), foo(y && x)",
        "const x = 'keep'; foo(x), foo(x);",
    );
}

#[test]
fn dce_var_hoisting() {
    test(
        "function f() {
          KEEP();
          return () => {
            var x;
          }
          REMOVE;
          function KEEP() { FOO }
          REMOVE;
        } f()",
        "function f() {
          KEEP();
          return () => { }
          function KEEP() { FOO }
        } f()",
    );
    // `KEEP` has an empty body (pure), so `f` is treated as pure too and the call
    // to `f()` is removed; then `f` has no remaining references and is dropped.
    test(
        "function f() {
          KEEP();
          return function g() {
            var x;
          }
          REMOVE;
          function KEEP() {}
          REMOVE;
        } f()",
        "",
    );
}

// Dropping a dead-after-throw statement (`module.exports = x`) removes the
// only reference to `x`. Without recording that as a mutation, the peephole
// loop terminates before `flush_pass_changes` prunes the dropped reference,
// leaving the unused-declarator pass to see a stale reference and keep
// `var x = {}`.
#[test]
fn dead_after_throw_drop_triggers_unused_declarator_removal() {
    test(
        "export function f() {
            var x = {};
            throw new Error('boom');
            module.exports = x;
         }",
        "export function f() { throw new Error('boom'); }",
    );
}

#[test]
fn pure_comment_for_pure_global_constructors() {
    test("var x = new WeakSet; foo(x)", "var x = /* @__PURE__ */ new WeakSet();\nfoo(x)");
    test("var x = new WeakSet(null); foo(x)", "var x = /* @__PURE__ */ new WeakSet(null);\nfoo(x)");
    test(
        "var x = new WeakSet(undefined); foo(x)",
        "var x = /* @__PURE__ */ new WeakSet(void 0);\nfoo(x)",
    );
    test("var x = new WeakSet([]); foo(x)", "var x = /* @__PURE__ */ new WeakSet([]);\nfoo(x)");
}

// `Normalize` sets pure flags before the peephole loop runs, so it misses
// args that the loop later folds/inlines into pure-eligible shapes.
#[test]
fn pure_comment_re_evaluated_after_string_concat_fold() {
    test(
        "var r = new RegExp('foo' + 'bar'); foo(r)",
        "var r = /* @__PURE__ */ new RegExp(\"foobar\");\nfoo(r)",
    );
}

#[test]
fn keep_regexp_feature_detection() {
    // https://github.com/rolldown/rolldown/issues/10279
    test_same(
        r"export function supportsUnicodeRegExp() {
            try {
                new RegExp('\\p{Ll}', 'u');
                return true;
            } catch {
                return false;
            }
        }",
    );

    // Patterns supported by all ECMAScript targets remain removable.
    test("new RegExp('a', 'g')", "");

    test_with_target("new RegExp('a', 'u')", "", "es2015");
    test_same_with_target(r"new RegExp('\\p{Ll}', 'u')", "es2017");
    test_with_target(r"new RegExp('\\p{Ll}', 'u')", "", "es2018");

    // A shadowed `RegExp` may have arbitrary constructor side effects.
    test_same("function f(RegExp) { new RegExp('a', 'g') } export { f }");
}

#[test]
fn keep_regexp_literal_with_flags() {
    test("new RegExp(/x/)", "");
    test_same("new RegExp(/x/, 'i')");
    test_same("new RegExp(/x/, 'u')");
    test_same_with_target("new RegExp(/x/, '!')", "esnext");
    test_same_with_target("new RegExp(/x/, 'gg')", "esnext");
    test_same_with_target("new RegExp(/x/, 'uv')", "esnext");
    test_same_with_target("new RegExp(/x/, 'i')", "chrome48");
    test_with_target("new RegExp(/x/, 'i')", "", "chrome49");
    test_with_target("new RegExp(/x/, 'i')", "", "es2015");
    test_with_target("new RegExp(/x/, 'u')", "", "es2015");
    test_same_with_target("new RegExp(/x/, 'v')", "es2015");
    test_with_target("new RegExp(/x/, 'v')", "", "es2024");
}

#[test]
fn keep_regexp_for_incomplete_engine_feature_data() {
    test_same_with_target("new RegExp('(?i:a)')", "safari16");
    test_same_with_target("new RegExp('(?<a>x)|(?<a>y)')", "deno1");
    test_with_target("new RegExp('(?i:a)')", "", "chrome125");
    test_with_target("new RegExp('(?<a>x)|(?<a>y)')", "", "chrome126");

    // Browser-only targets contain an implicit `esnext`; every configured engine must still
    // satisfy the feature version, regardless of feature-table iteration order.
    test_same_with_target("new RegExp('a', 'u')", "chrome49");
    test_with_target("new RegExp('a', 'u')", "", "chrome50");
}

#[test]
fn regexp_named_group_targets() {
    test_with_target("new RegExp('(?<a>x)')", "", "es2018");
    test_same_with_target("new RegExp('(?<a>x)|(?<a>y)')", "es2024");
    test_with_target("new RegExp('(?<a>x)|(?<a>y)')", "", "es2025");
}

#[test]
fn keep_regexp_with_lone_surrogates() {
    test_same(r"new RegExp('[\uD801-\uD800]')");
}

#[test]
fn keep_regexp_with_invalid_flags_for_modern_target() {
    test_same_with_target("new RegExp('a', 'gg')", "esnext");
    test_same_with_target("new RegExp('a', 'uv')", "esnext");
}

#[test]
fn pure_comment_re_evaluated_after_variable_inline() {
    test(
        "export function f() { var ab = new ArrayBuffer(1); var dv = new DataView(ab); foo(dv); }",
        "export function f() {\n\tvar dv = /* @__PURE__ */ new DataView(/* @__PURE__ */ new ArrayBuffer(1));\n\tfoo(dv);\n}",
    );
}

#[test]
fn fold_number_nan() {
    test("foo(Number.NaN)", "foo(NaN)");
    test_same("let Number; foo(Number.NaN)");
}

// https://github.com/terser/terser/blob/v5.9.0/test/compress/dead-code.js
#[test]
fn dce_from_terser() {
    test(
        "function f() {
            a();
            b();
            x = 10;
            return;
            if (x) {
                y();
            }
        } f()",
        "function f() {
            a();
            b();
            x = 10;
        } f()",
    );

    test(
        r#"function f() {
            g();
            x = 10;
            throw new Error("foo");
            if (true) {
                y();
                var x;
                function g(){};
                (function(){
                    var q;
                    function y(){};
                })();
            }
        }
        f();
        "#,
        r#"function f() {
            g();
            x = 10;
            throw new Error("foo");
            var x;
        }
        f();
        "#,
    );

    test(
        "if (0) {
            let foo = 6;
            const bar = 12;
            class Baz {};
            var qux;
        }
        console.log(foo, bar, Baz);
        ",
        "console.log(foo, bar, Baz);",
    );
}

#[test]
fn dce_iterations() {
    test(
        "
var a1 = 'a1'
var a2 = 'a2'
var a3 = 'a3'
var a4 = 'a4'
var a5 = 'a5'
var a6 = 'a6'
var a7 = 'a7'
var a8 = 'a8'
var a9 = 'a9'
var a10 = 'a10'
var a11 = 'a11'
var a12 = 'a12'
var a13 = 'a13'
var a14 = 'a14'
var a15 = 'a15'
var a16 = 'a16'
var a17 = 'a17'
var a18 = 'a18'
var a19 = 'a19'
var a20 = 'a20'
var arr = [
  a1,
  a2,
  a3,
  a4,
  a5,
  a6,
  a7,
  a8,
  a9,
  a10,
  a11,
  a12,
  a13,
  a14,
  a15,
  a16,
  a17,
  a18,
  a19,
  a20
]
console.log(arr)
        ",
        "
console.log([
  'a1',
  'a2',
  'a3',
  'a4',
  'a5',
  'a6',
  'a7',
  'a8',
  'a9',
  'a10',
  'a11',
  'a12',
  'a13',
  'a14',
  'a15',
  'a16',
  'a17',
  'a18',
  'a19',
  'a20'
])
        ",
    );
}

#[test]
fn dropped_direct_eval_converges_after_liveness_refresh() {
    test_with_iterations("if (false) eval('x'); function f() { f() }", "", 2);
}

#[test]
fn drop_labels() {
    let mut options = CompressOptions::dce();
    let mut drop_labels = FxHashSet::default();
    drop_labels.insert("PURE".to_string());
    options.drop_labels = drop_labels;

    test_with_options("PURE: { foo(); bar(); }", "", options);
}

#[test]
fn drop_multiple_labels() {
    let mut options = CompressOptions::dce();
    let mut drop_labels = FxHashSet::default();
    drop_labels.insert("PURE".to_string());
    drop_labels.insert("TEST".to_string());
    options.drop_labels = drop_labels;

    test_with_options(
        "PURE: { foo(); } TEST: { bar(); } OTHER: { baz(); }",
        "OTHER: baz();",
        options,
    );
}

#[test]
fn drop_labels_nested() {
    let mut options = CompressOptions::dce();
    let mut drop_labels = FxHashSet::default();
    drop_labels.insert("PURE".to_string());
    options.drop_labels = drop_labels;

    test_with_options("PURE: { PURE: { foo(); } }", "", options);
}

#[test]
fn drop_labels_with_vars() {
    let mut options = CompressOptions::dce();
    let mut drop_labels = FxHashSet::default();
    drop_labels.insert("PURE".to_string());
    options.drop_labels = drop_labels;

    test_with_options("PURE: { var x = 1; foo(x); }", "", options);
}

#[test]
fn keep_use_strict_directives() {
    test_same("'use strict'; export function foo() { 'use strict'; return 1; }");
}

#[test]
fn preserve_legal_comment_when_dce_removes_anchor() {
    // https://github.com/oxc-project/oxc/issues/19750
    // Each test below covers a scope where DCE removes a legal comment's
    // anchor; codegen must re-emit it at the next surviving anchor in scope.
    test_with_options(
        "//! some license\nconst foo = 'value';\nconsole.log(foo);",
        "//! some license\nconsole.log('value');",
        CompressOptions::dce(),
    );
    test_with_options(
        "/*! @license */\nconst foo = 'value';\nconsole.log(foo);",
        "/*! @license */\nconsole.log('value');",
        CompressOptions::dce(),
    );
    test_with_options(
        "/* @preserve */\nconst foo = 'value';\nconsole.log(foo);",
        "/* @preserve */\nconsole.log('value');",
        CompressOptions::dce(),
    );
    // Non-legal comment is dropped with its anchor.
    test_with_options(
        "// regular comment\nconst foo = 'value';\nconsole.log(foo);",
        "console.log('value');",
        CompressOptions::dce(),
    );
    // No DCE → comment stays at its original anchor.
    test_with_options(
        "//! some license\nconst foo = 'value';\nconsole.log(foo);\nconsole.log(foo);",
        "//! some license\nconst foo = 'value';\nconsole.log(foo);\nconsole.log(foo);",
        CompressOptions::dce(),
    );
    test_with_options(
        "//! license\nconst foo = 'unused';\nbar();",
        "//! license\nbar();",
        CompressOptions::dce(),
    );
    // Multiple orphans flush in source order.
    test_with_options(
        "//! A\nconst a = 'x';\n//! B\nconst b = 'y';\nf(a, b);",
        "//! A\n//! B\nf('x', 'y');",
        CompressOptions::dce(),
    );
    // Orphan stays above a surviving sibling's own legal comment.
    test_with_options(
        "//! orphan-A\nconst a = 'unused';\n//! kept-B\nconsole.log('hi');",
        "//! orphan-A\n//! kept-B\nconsole.log('hi');",
        CompressOptions::dce(),
    );
    // All top-level removed → flushed at program scope-end.
    test_with_options(
        "//! license\nconst foo = 'unused';\n",
        "//! license",
        CompressOptions::dce(),
    );
    // Inner anchor survives → emitted at original spot.
    test_with_options(
        "//! top\nfunction f() {\n  //! body\n  return 1;\n}\nconst unused = 'x';\nconsole.log(f());",
        "//! top\nfunction f() {\n\t//! body\n\treturn 1;\n}\nconsole.log(f());",
        CompressOptions::dce(),
    );
    // Inner anchor removed, sibling survives → flushed inside function scope.
    test_with_options(
        "function f() {\n  //! body\n  const innerUnused = 'x';\n  return 1;\n}\nconsole.log(f());",
        "function f() {\n\t//! body\n\treturn 1;\n}\nconsole.log(f());",
        CompressOptions::dce(),
    );
    // No surviving sibling → DCE inlines empty `f()` to `void 0`; orphan
    // re-anchors at the next surviving top-level stmt.
    test_with_options(
        "function f() {\n  //! body\n  const innerUnused = 'x';\n}\nconsole.log(f());",
        "//! body\nconsole.log(void 0);",
        CompressOptions::dce(),
    );
    // BlockStatement scope.
    test_with_options(
        "try {\n  //! caught\n  const ignored = 'x';\n  a();\n  b();\n} catch (e) {}",
        "try {\n\t//! caught\n\ta();\n\tb();\n} catch (e) {}",
        CompressOptions::dce(),
    );
    // SwitchCase scope, multi-stmt consequent.
    test_with_options(
        "switch (x) {\n  case 1:\n    //! case\n    const ignored = 'x';\n    a();\n    b();\n}",
        "switch (x) {\n\tcase 1:\n\t\t//! case\n\t\ta();\n\t\tb();\n}",
        CompressOptions::dce(),
    );
    // SwitchCase with one surviving consequent stmt: the inline `case 1: stmt`
    // path must fall through to multi-line so the flush fires inside the case.
    // Asserted directly because canonical roundtrip uses an inline format that
    // doesn't match our multi-line orphan layout.
    {
        let source = "switch (x) {\n  case 1:\n    //! case\n    const ignored = 'x';\n    survivor();\n}\nafter();";
        let result = run(source, SourceType::default(), Some(CompressOptions::dce()));
        let case_pos = result.find("//! case").expect("comment preserved");
        let close_pos = result.find("\n}").expect("switch close");
        assert!(case_pos < close_pos, "legal comment escaped the case scope: {result}");
    }
}

#[test]
fn preserve_at_license_legal_comment_when_dce_removes_anchor() {
    // The omnibus test above pins `//!`, `/*! @license */`, and `/* @preserve */`.
    // Cover the only remaining marker — `/* @license */` standalone (no `!`).
    test_with_options(
        "/* @license */\nconst foo = 'val';\nconsole.log(foo);",
        "/* @license */\nconsole.log('val');",
        CompressOptions::dce(),
    );
}

#[test]
fn preserve_annotation_comments_when_inlining_single_use_variable() {
    // https://github.com/rolldown/rolldown/issues/8248
    test(
        "const bar = 'some-url'; import(/* @vite-ignore */ bar);",
        "import(/* @vite-ignore */ 'some-url');",
    );
    // `test` replaces "true"/"false" literals, so use `test_with_options` for webpackIgnore
    test_with_options(
        "const bar = 'some-url'; import(/* webpackIgnore: true */ bar);",
        "import(/* webpackIgnore: true */ 'some-url');",
        CompressOptions::dce(),
    );
    test_with_options(
        "const bar = 'some-url'; import(/* @vite-ignore */ /* webpackIgnore: true */ bar);",
        "import(/* @vite-ignore */ /* webpackIgnore: true */ 'some-url');",
        CompressOptions::dce(),
    );
}

#[test]
fn remove_pure_function_calls() {
    // https://github.com/rolldown/rolldown/issues/9211
    test("function noop() {} noop()", "");
    test("var foo = () => 1; foo(), foo()", "");
    test("var foo = function() {}; foo()", "");
    test_same("function foo() { bar() } foo()");
}

#[test]
fn preserve_iife_in_dce_mode() {
    // https://github.com/oxc-project/oxc/issues/17480
    // https://github.com/rolldown/rolldown/issues/9437
    //
    // IIFE body extraction is a peephole / strength-reduction rewrite, not
    // DCE. In DCE-only mode (rolldown's per-module preprocess) the IIFE
    // structure is preserved entirely — matching Rollup, esbuild
    // `--tree-shaking=true`, Terser (no compress), and SWC (no minify).
    //
    // The only DCE-relevant IIFE rewrites that still run: dropping
    // pure-annotated IIFEs whose result is unused (handled separately via
    // `is_expression_result_unused` → `void 0`), and replacing fully-empty
    // IIFEs with `void 0`.

    // Pure-annotated IIFEs preserved across all body shapes.
    test_same("export const exec = /* @__PURE__ */ (() => _M.exec)();");
    test_same("export const x = /* @__PURE__ */ (() => foo()?.bar())();");
    test_same("export const x = /* @__PURE__ */ (() => foo`tpl`)();");
    test_same("export const x = /* @__PURE__ */ (() => (a(), b()))();");
    test_same("export const x = /* @__PURE__ */ (() => c ? a() : b())();");
    test_same("export const x = /* @__PURE__ */ (() => a ? b : c)();");
    test_same("export const x = /* @__PURE__ */ (() => foo())();");
    test_same("export const x = /* @__PURE__ */ (() => new Foo())();");
    test_same("export const x = /* @__PURE__ */ (() => foo(bar()))();");
    test_same("export const x = /* @__PURE__ */ (() => new Foo(bar()))();");
    test_same("export const x = /* @__PURE__ */ (() => 42)();");
    test_same("export const x = /* @__PURE__ */ (() => [1, 2, 3])();");
    test_same("export const x = /* @__PURE__ */ (() => ({ a: 1 }))();");

    // Non-pure IIFEs are also preserved — inlining is not DCE.
    test_same("export const x = (() => foo())();");
    test_same("export const x = (() => [1, 2, 3])();");
    test_same("export const x = (() => 42)();");
    test_same("export const x = (() => { foo(); })();");
    test_same("export const x = (() => { return foo(); })();");

    // A non-pure IIFE with a provably side-effect-free body is also preserved
    // in DCE-only mode even when its result is unused. The body analysis that
    // proves it droppable runs only under full minification.
    // https://github.com/oxc-project/oxc/issues/23777
    test_same("(function () { function t() {} return t })();");
    test_same("(function () { return 1 })();");
    // The unused binding is still dropped, but its side-effect-free IIFE
    // initializer survives as a statement (structure preserved).
    test("var u = (function () { return 1 })();", "(function () { return 1 })();");
    // Nested in a sequence — still preserved (routes through the same path).
    test_same("(function () { return 1 })(), foo();");
    // `for`-init declarator removal must preserve the IIFE too.
    test_same("for (var u = (function () { return 1 })(); cond;) bar();");
}

#[test]
fn drop_optional_chain_on_non_nullish_base() {
    // https://github.com/oxc-project/oxc/issues/21923
    // ObjectExpression at statement start needs to stay parenthesised; codegen
    // already handles that, so the fold is safe in statement position.
    test("({})?.foo;", "({}).foo;");
    // Side effects on the base are preserved when the `?.` is dropped.
    test("export const v = (foo(), {})?.bar", "export const v = (foo(), {}).bar");
    // Nested: the optional is on the inner access, the outer access is
    // non-optional. Both folds (collapse + drop) reach the deepest `?.`.
    test("export const v = null?.foo.bar", "export const v = void 0");
    test("export const v = []?.foo.bar", "export const v = [].foo.bar");
}

#[test]
fn fold_optional_chain_on_undefined_let_binding() {
    // https://github.com/rolldown/rolldown/issues/9281
    // A `let` binding with no writes is statically known to be `undefined`,
    // so optional calls / member accesses on it should fold to `void 0`.
    test("let slot; export function call() { slot?.() }", "export function call() {}");
    test("let slot; export function call() { slot?.foo }", "export function call() {}");
    test("let slot; export function call() { slot?.[foo()] }", "export function call() {}");
    // A binding that is written somewhere is not nullish-known: leave it alone.
    test_same(
        "let slot; export function setSlot(v) { slot = v } export function call() { slot?.() }",
    );
}

#[test]
fn fold_optional_chain_on_null_const_binding() {
    // A `const` initialized to `null` resolves to `ValueType::Null`, so the
    // optional chain folds the same way the `undefined` case does.
    test("const slot = null; export function call() { slot?.() }", "export function call() {}");
    test("const slot = null; export function call() { slot?.foo }", "export function call() {}");
}

#[test]
fn fold_coalesce_on_tracked_non_nullish_binding() {
    // The new value_type lookup also resolves non-nullish constants, so the
    // right-hand side of `??` can be dropped.
    //
    // Two reads + a string the inliner skips (length > 3) prevents
    // `inline_identifier_reference` from short-circuiting the test by
    // substituting the literal value before the coalesce fold runs.
    test(
        "let s = 'hello'; export function a() { return s ?? other() } export function b() { return s ?? other() }",
        "let s = 'hello'; export function a() { return s } export function b() { return s }",
    );
    // BigInt is never inlined, so a single read is enough to exercise the
    // value-type path here.
    test(
        "let n = 5n; export function a() { return n ?? other() } export function b() { return n ?? other() }",
        "let n = 5n; export function a() { return n } export function b() { return n }",
    );
}

// Convergence regression (monitor-oxc, bluebird.js): `try_fold_if` re-extracts
// the dead branch's `var` names via `KeepVar` on every pass and filters the
// synthesized statement through the unused-declarator removal. Dropping `x`
// from that TRANSIENT statement must not record a mutation — when the slot is
// already in canonical KeepVar shape nothing in the live tree changes, and a
// spurious mutation spins the fixed-point loop past its iteration guard.
#[test]
fn test_fold_if_keep_var_filter_converges() {
    test_same("function f() {\n\tif (0) var x, y;\n\ty = 1;\n\treturn y;\n}\nf();");
}

// DCE mode is rolldown's per-module tree-shaking preprocess; the DEFAULT-mode
// drop of write-only property assignments (full minify only) must stay off
// here — `treeshake.property_write_side_effects: false` is rolldown's own knob
// for opting in.
#[test]
fn dce_keeps_write_only_property_assignments() {
    test_same(
        "(function() {\n\tvar r = require(\"react\");\n\tvar o = function(e, t) {\n\t\treturn r.create(e, t);\n\t};\n\to.displayName = \"X\";\n})();",
    );
}

// A statement that never completes normally (a kept block ending in a jump,
// an if/else or try/catch where every branch jumps) makes everything after it
// in the same statement list unreachable. The kept-block shape is what
// `define`-driven branch folding leaves behind: `if (true) { ... return fn; }`
// folds to a block that is pinned by its lexical declaration.
#[test]
fn dce_remove_unreachable_after_terminating_statement() {
    // https://github.com/rolldown/rolldown/issues/10184
    test(
        "export function f() {\n\tif (true) {\n\t\tconst fn = () => 1;\n\t\tfn.stop = fn;\n\t\treturn fn;\n\t}\n\tconst fn = () => 2;\n\tfn.stop = fn;\n\treturn fn;\n}",
        "export function f() {\n\t{\n\t\tconst fn = () => 1;\n\t\tfn.stop = fn;\n\t\treturn fn;\n\t}\n}",
    );
    // Both branches of an if/else terminate.
    test(
        "export function f(c) { if (c) { return 1; } else { return 2; } foo(); }",
        "export function f(c) { if (c) return 1; else return 2; }",
    );
    // Both blocks of a try/catch terminate.
    test(
        "export function f() { try { return g(); } catch { return h(); } i(); }",
        "export function f() { try { return g(); } catch { return h(); } }",
    );
    // A `var` in the unreachable tail hoists; unreferenced, the re-emitted
    // declaration is then dropped as unused.
    test(
        "export function f() {\n\tif (true) {\n\t\tconst a = 1;\n\t\tuse(a);\n\t\treturn a;\n\t}\n\tvar x = g();\n}",
        "export function f() {\n\t{\n\t\tconst a = 1;\n\t\tuse(a);\n\t\treturn a;\n\t}\n}",
    );
    // A `var` in the unreachable tail referenced from live code keeps its
    // binding (only the unreachable initializer goes).
    test(
        "export function f() {\n\tuse(() => x);\n\tif (true) {\n\t\tconst a = 1;\n\t\tuse(a);\n\t\treturn a;\n\t}\n\tvar x = g();\n}",
        "export function f() {\n\tuse(() => x);\n\t{\n\t\tconst a = 1;\n\t\tuse(a);\n\t\treturn a;\n\t}\n\tvar x;\n}",
    );
    // Function declarations in the unreachable tail hoist and stay.
    test_same(
        "export function f() {\n\t{\n\t\tconst a = g();\n\t\tuse(a);\n\t\treturn a;\n\t}\n\tfunction g() {\n\t\treturn 2;\n\t}\n}",
    );
    // Negative: the block can complete normally, so the tail stays.
    test_same(
        "export function f(c) {\n\t{\n\t\tlet a = g();\n\t\tif (c) return a;\n\t}\n\treturn foo();\n}",
    );
    // Hoisting survivors trailing the jump inside the block — a kept
    // `function` declaration or a `var` stub re-emitted by `KeepVar` — don't
    // hide that the block terminates.
    test(
        "export function f() {\n\t{\n\t\tconst a = g();\n\t\ta.x = a;\n\t\treturn a;\n\t\tfunction g() {\n\t\t\treturn {};\n\t\t}\n\t}\n\ttail();\n}",
        "export function f() {\n\t{\n\t\tconst a = g();\n\t\ta.x = a;\n\t\treturn a;\n\t\tfunction g() {\n\t\t\treturn {};\n\t\t}\n\t}\n}",
    );
    test(
        "export function f() {\n\tuse(() => x);\n\t{\n\t\tlet a = g();\n\t\tuse(a);\n\t\treturn a;\n\t\tvar x = h();\n\t}\n\ttail();\n}",
        "export function f() {\n\tuse(() => x);\n\t{\n\t\tlet a = g();\n\t\tuse(a);\n\t\treturn a;\n\t\tvar x;\n\t}\n}",
    );
    // Negative: skipping the hoisting survivors must land on a statement
    // that really terminates — `if` without `else` doesn't.
    test_same(
        "export function f(c) {\n\t{\n\t\tif (c) return g();\n\t\tfunction g() {\n\t\t\treturn 1;\n\t\t}\n\t}\n\treturn tail();\n}",
    );
}

// #13105: dead recursive/cyclic function declarations must also drop in
// dce-only mode (rolldown's per-module treeshake preprocess). Self-recursive
// function-valued declarators use a local removal-site check; mutual
// declarator and class cycles are kept because graph candidacy is function
// declarations only.
#[test]
fn dce_recursive_unused_functions() {
    test("function f() { f() }", "");
    test("function c() { d() } function d() { c() }", "");
    test("var f = function() { f() }", "");
    test("const f = () => f()", "");
    // Cycle whose only external reference sits in dead code: needs the mid-loop
    // recompute trigger (pass 2), not just the initial compute.
    test("if (false) c(); function c() { d() } function d() { c() }", "");
    // Declarator and class cycles are kept (functions-only candidacy).
    test_same("const a = () => b();\nconst b = () => a();");
    test_same("class A {\n\tm() {\n\t\tnew B();\n\t}\n}\nclass B {\n\tm() {\n\t\tnew A();\n\t}\n}");
    // Live references keep the cycle live.
    test_same("function f() {\n\tf();\n}\nconsole.log(f);");
    test_same("export function f() {\n\tf();\n}");
    // Removing a dead cycle can zero an exported sibling redeclaration's
    // ordinary read count; stable export observability protects its writes.
    test(
        "export var f; var f = 0; function d1() { console.log(f); d2() } function d2() { d1() } f = 1;",
        "export var f;\nvar f = 0;\nf = 1;",
    );
    // For-head bindings need no special lifecycle state.
    test(
        "var f = 1; function d1() { f; d2() } function d2() { d1() } if (false) for (var f of xs) {} export {};",
        "export {};",
    );
}

#[test]
fn dce_remove_unused_class_identifier_heritage_under_assumptions() {
    // ASSUMPTIONS.md excludes TDZ violations and side effects from extending a class.
    test(
        "export var Base = class {}; export var Keep = class extends Base {}; var REMOVE = class extends Base {};",
        "export var Base = class {}; export var Keep = class extends Base {};",
    );
}

#[test]
#[ignore = "TODO: extend recursive reachability to mutual declarators and classes"]
fn dce_recursive_unused_mutual_declarators_and_classes() {
    test("const a = () => b(); const b = () => a();", "");
    test("class A { m() { new B() } } class B { m() { new A() } }", "");
}

#[test]
fn dce_recursive_unused_functions_in_commonjs_and_script() {
    test_source_type("var f = function() { f() }", "", SourceType::cjs());
    test_source_type(
        "function c() { d() } function d() { c() } console.log('k');",
        "console.log('k');",
        SourceType::cjs(),
    );
    test_source_type("{ function f() { f() } }", "", SourceType::cjs());
    test_source_type(
        "if (false) g(); function g() { f() } function f() { f() }",
        "",
        SourceType::cjs(),
    );
    test_source_type(
        "function outer() { function c() { d() } function d() { c() } return 1 }",
        "function outer() { return 1 }",
        SourceType::script(),
    );
    test_source_type(
        "function outer() { const f = () => f(); return 1 }",
        "function outer() { return 1 }",
        SourceType::script(),
    );

    test_same_source_type("function f() { f() }", SourceType::script());
    test_same_source_type("var f = () => f()", SourceType::script());
    test_same_source_type("{ function f() { f() } }", SourceType::script());
    test_same_source_type("function f() { f() } module.exports = f;", SourceType::cjs());
}

#[test]
fn dce_keeps_sloppy_duplicate_block_functions() {
    let source =
        "{ function f() { return 1 } } { function f() { return f } } console.log(typeof f());";
    for source_type in [SourceType::script(), SourceType::cjs()] {
        test_same_source_type(source, source_type);
    }
    test_same_source_type(
        "{ function f() { return f } } { function f() { return f } } console.log(typeof f());",
        SourceType::ts().with_script(true),
    );

    // Strict block functions have no Annex B var alias and remain removable.
    test_source_type("'use strict'; { function f() { f() } }", "'use strict';", SourceType::cjs());
    test_source_type(
        "'use strict'; { function f() { f() } }",
        "'use strict';",
        SourceType::ts().with_script(true),
    );
}

#[test]
fn dce_keeps_script_root_var_in_nested_statement_after_cycle_removed() {
    let source = "function outer() { function d1() { return x + d2() } function d2() { return d1() } return 1 } outer(); switch (1) { case 1: var x = 42; }";
    // Script globals can be rebound through global-object properties without a
    // resolved write reference, so the call cannot reuse a pure summary.
    let expected = "function outer() { return 1 } outer(); switch (1) { case 1: var x = 42; }";
    test_source_type(source, expected, SourceType::script());

    // CommonJS top-level vars are wrapper-local, so ordinary counts may remove them.
    test_source_type("{ var x = 42; }", "", SourceType::cjs());
}

#[test]
fn dce_keeps_implicitly_observable_bindings() {
    let options = CompressOptions {
        treeshake: TreeShakeOptions {
            property_write_side_effects: false,
            ..TreeShakeOptions::default()
        },
        ..CompressOptions::dce()
    };

    let annex_source = "function outer() { { function f() {} } { function f() {} f.x = 1; function d1() { consume(f); d2() } function d2() { d1() } } console.log(f.x); } outer();";
    let annex_expected = "function outer() { { function f() {} } { function f() {} f.x = 1; } console.log(f.x); } outer();";
    for source_type in [SourceType::script(), SourceType::cjs()] {
        test_with_options_source_type(annex_source, annex_expected, source_type, options.clone());
    }

    test_with_options(
        "{ using resource = { [Symbol.dispose]() { console.log(this.x) } }; resource.x = 1; function d1() { consume(resource); d2() } function d2() { d1() } }",
        "{ using resource = { [Symbol.dispose]() { console.log(this.x) } }; resource.x = 1; }",
        options,
    );
}

// https://github.com/oxc-project/oxc/issues/23866
#[test]
fn dce_drops_dead_trailing_function_arguments() {
    test(
        "const foo = async (assets) => ({}); export default await foo({ bar: 'baz' })",
        "const foo = async (assets) => ({}); export default await foo()",
    );

    // Dropping the argument also removes a nested dynamic import that would
    // otherwise keep an unnecessary chunk alive in Rolldown.
    test(
        "const foo = async (assets) => ({}); export default await foo({ image: () => import('./image.js') })",
        "const foo = async (assets) => ({}); export default await foo()",
    );
    test(
        "export const foo = async (unused) => bar(); foo({ image: () => import('./image.js') })",
        "export const foo = async (unused) => bar(); foo()",
    );
    test(
        "export default async function foo(unused) { bar() } foo({ image: () => import('./image.js') })",
        "export default async function foo(unused) { bar() } foo()",
    );
    test(
        "foo({ image: () => import('./image.js') }); async function foo(unused) { bar() }",
        "foo(); async function foo(unused) { bar() }",
    );
    test(
        "const foo = (unused) => { bar() }; foo(1); foo(2)",
        "const foo = (unused) => { bar() }; foo(); foo()",
    );
    test(
        "const foo = (a) => { bar(a) }; foo(1, 2, 3); foo(4)",
        "const foo = (a) => { bar(a) }; foo(1); foo(4)",
    );
    test(
        "const foo = function* (unused) { yield bar() }; foo(1).next(); foo(2).next()",
        "const foo = function* (unused) { yield bar() }; foo().next(); foo().next()",
    );
    test(
        "foo(1, 2); function foo(a, unused) { inner(a, unused) } function inner(a, ignored) { bar(a) }",
        "foo(1); function foo(a, unused) { inner(a) } function inner(a, ignored) { bar(a) }",
    );
}

#[test]
fn dce_keeps_observable_trailing_function_arguments() {
    test_same("const foo = (unused) => bar(); foo(sideEffect())");
    test_same("let foo = (unused) => bar(); foo(1); foo = replacement");
    test_same("const foo = (unused) => eval(\"unused\"); consume(foo(1))");
    test_same("const foo = (a, b) => bar(b); foo(1, 2)");
    test_same("const foo = (unused = sideEffect()) => bar(); foo(1)");
    test_same("const foo = ({ unused }) => bar(); foo(value)");
    test_same("const foo = (...args) => bar(args); foo(1)");
    test_same("const foo = function(unused) { return arguments.length }; consume(foo(1))");
    test_same("const foo = (unused) => bar(); foo(...values)");
    test_same(
        "class Base {} class Derived extends Base { constructor() { const foo = (unused) => bar(); foo(this); super() } } consume(new Derived())",
    );
    test_same_source_type(
        "function outer(object) { const foo = () => {}; with (object) foo(1) } outer(source)",
        SourceType::cjs().with_script(true),
    );
}
