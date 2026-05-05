use cow_utils::CowUtils;
use rustc_hash::FxHashSet;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_minifier::CompressOptions;
use oxc_minifier::Compressor;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[track_caller]
fn run(source_text: &str, source_type: SourceType, options: Option<CompressOptions>) -> String {
    let allocator = Allocator::default();
    let mut ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty(), "Parser errors: {:?}", ret.errors);
    let program = &mut ret.program;
    if let Some(options) = options {
        Compressor::new(&allocator).dead_code_elimination(program, options);
    }
    Codegen::new().build(program).code
}

#[track_caller]
fn test(source_text: &str, expected: &str) {
    let t = "('production' == 'production')";
    let f = "('production' == 'development')";
    let source_text = source_text.cow_replace("true", t);
    let source_text = source_text.cow_replace("false", f);

    let source_type = SourceType::default();
    let result = run(&source_text, source_type, Some(CompressOptions::dce()));
    let expected = run(expected, source_type, None);
    assert_eq!(result, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{result}");
}

#[track_caller]
fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[track_caller]
fn test_with_options(source_text: &str, expected: &str, options: CompressOptions) {
    let source_type = SourceType::default();
    let result = run(source_text, source_type, Some(options));
    let expected = run(expected, source_type, None);
    assert_eq!(result, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{result}");
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
