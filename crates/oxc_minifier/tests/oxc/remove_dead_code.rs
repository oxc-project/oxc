use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_minifier::RemoveDeadCode;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn print(source_text: &str, remove_dead_code: bool) -> String {
    let source_type = SourceType::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    if remove_dead_code {
        RemoveDeadCode::new(&allocator).build(program);
    }
    CodeGenerator::new()
        .with_options(CodegenOptions::new().with_single_quotes())
        .build(program)
        .source_text
}

pub(crate) fn test(source_text: &str, expected: &str) {
    let minified = print(source_text, true);
    let expected = print(expected, false);
    assert_eq!(minified, expected, "for source {source_text}");
}

#[test]
fn dce_if_statement() {
    test("if (true) { foo }", "{ foo }");
    test("if (true) { foo } else { bar }", "{ foo }");
    test("if (false) { foo } else { bar }", "{ bar }");

    test("if (!false) { foo }", "{ foo }");
    test("if (!true) { foo } else { bar }", "{ bar }");

    test("if (!false && xxx) { foo }", "if (xxx) { foo; }");
    test("if (!true && yyy) { foo } else { bar }", "{ bar }");

    test("if ('production' == 'production') { foo } else { bar }", "{ foo }");
    test("if ('development' == 'production') { foo } else { bar }", "{ bar }");

    test("if ('production' === 'production') { foo } else { bar }", "{ foo }");
    test("if ('development' === 'production') { foo } else { bar }", "{ bar }");

    // Shadowed `undefined` as a variable should not be erased.
    test(
        "function foo(undefined) { if (!undefined) { } }",
        "function foo(undefined) { if (!undefined) { } }",
    );

    test("if (true) { foo; } if (true) { foo; }", "{ foo; } { foo; }");

    test(
        "
        if (true) { foo; return }
        foo;
        if (true) { bar; return }
        bar;
        ",
        "{foo; return }",
    );
}

#[test]
fn dce_conditional_expression() {
    test("false ? foo : bar;", "bar");
    test("true ? foo : bar;", "foo");

    test("!true ? foo : bar;", "bar");
    test("!false ? foo : bar;", "foo");

    test("!!false ? foo : bar;", "bar");
    test("!!true ? foo : bar;", "foo");

    test("const foo = true ? A : B", "const foo = A");
    test("const foo = false ? A : B", "const foo = B");
}

#[test]
fn dce_logical_expression() {
    test("false && bar()", "false");
    test("true && bar()", "bar()");

    test("const foo = false && bar()", "const foo = false");
    test("const foo = true && bar()", "const foo = bar()");
}

// https://github.com/terser/terser/blob/master/test/compress/dead-code.js
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
        }",
        "function f() {
            a();
            b();
            x = 10;
            return;
        }",
    );

    // NOTE: `if (x)` is changed to `if (true)` because const inlining is not implemented yet.
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
        "
        var qux;
        console.log(foo, bar, Baz);
        ",
    );
}
