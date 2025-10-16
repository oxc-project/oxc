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
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    let mut ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = &mut ret.program;
    Compressor::new(&allocator).dead_code_elimination(program, options);
    let result = Codegen::new().build(program).code;
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

    test("function foo() { if (undefined) { bar } } foo()", "function foo() { } foo()");
    test("function foo() { { bar } } foo()", "function foo() { bar } foo()");

    test("if (true) { foo; } if (true) { foo; }", "foo; foo;");
    test("if (true) { foo; return } foo; if (true) { bar; return } bar;", "foo; return");

    // nested expression
    test(
        "const a = { fn: function() { if (true) { foo; } } } bar(a)",
        "const a = { fn: function() { foo; } } bar(a)",
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
        "function f() {
          KEEP();
          return function g() {}
          function KEEP() {}
        } f()",
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
