use oxc_allocator::Allocator;
use oxc_codegen::WhitespaceRemover;
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
    WhitespaceRemover::new().build(program).source_text
}

pub(crate) fn test(source_text: &str, expected: &str) {
    let minified = print(source_text, true);
    let expected = print(expected, false);
    assert_eq!(minified, expected, "for source {source_text}");
}

#[test]
fn remove_dead_code() {
    test("if (true) { foo }", "{ foo }");
    test("if (true) { foo } else { bar }", "{ foo }");
    test("if (false) { foo } else { bar }", "{ bar }");

    test("if (!false) { foo }", "{ foo }");
    test("if (!true) { foo } else { bar }", "{ bar }");

    test("if ('production' == 'production') { foo } else { bar }", "{ foo }");
    test("if ('development' == 'production') { foo } else { bar }", "{ bar }");

    test("if ('production' === 'production') { foo } else { bar }", "{ foo }");
    test("if ('development' === 'production') { foo } else { bar }", "{ bar }");

    test("false ? foo : bar;", "bar");
    test("true ? foo : bar;", "foo");

    test("!true ? foo : bar;", "bar");
    test("!false ? foo : bar;", "foo");

    test("!!false ? foo : bar;", "bar");
    test("!!true ? foo : bar;", "foo");

    test("const foo = true ? A : B", "const foo = A");
    test("const foo = false ? A : B", "const foo = B");

    // Shadowed `undefined` as a variable should not be erased.
    test(
        "function foo(undefined) { if (!undefined) { } }",
        "function foo(undefined) { if (!undefined) { } }",
    );
}

// https://github.com/terser/terser/blob/master/test/compress/dead-code.js
#[test]
fn remove_dead_code_from_terser() {
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
        "
          function f() {
            a();
            b();
            x = 10;
            return;
        }",
    );
}
