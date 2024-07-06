use oxc_allocator::Allocator;
use oxc_codegen::WhitespaceRemover;
use oxc_minifier::RemoveDeadCode;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn minify(source_text: &str) -> String {
    let source_type = SourceType::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    RemoveDeadCode::new(&allocator).build(program);
    WhitespaceRemover::new().build(program).source_text
}

pub(crate) fn test(source_text: &str, expected: &str) {
    let minified = minify(source_text);
    assert_eq!(minified, expected, "for source {source_text}");
}

#[test]
fn remove_dead_code() {
    test("if (true) { foo }", "{foo}");
    test("if (true) { foo } else { bar }", "{foo}");
    test("if (false) { foo } else { bar }", "{bar}");

    test("if (!false) { foo }", "{foo}");
    test("if (!true) { foo } else { bar }", "{bar}");

    test("if ('production' == 'production') { foo } else { bar }", "{foo}");
    test("if ('development' == 'production') { foo } else { bar }", "{bar}");

    test("if ('production' === 'production') { foo } else { bar }", "{foo}");
    test("if ('development' === 'production') { foo } else { bar }", "{bar}");

    test("false ? foo : bar;", "bar");
    test("true ? foo : bar;", "foo");

    test("!true ? foo : bar;", "bar");
    test("!false ? foo : bar;", "foo");

    test("!!false ? foo : bar;", "bar");
    test("!!true ? foo : bar;", "foo");

    // Shadowed `undefined` as a variable should not be erased.
    test(
        "function foo(undefined) { if (!undefined) { } }",
        "function foo(undefined){if(!undefined){}}",
    );
}
