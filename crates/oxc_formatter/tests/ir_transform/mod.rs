mod sort_imports;

use oxc_formatter::FormatOptions;

pub fn assert_format(code: &str, options: &FormatOptions, expected: &str) {
    // NOTE: Strip leading single `\n` for better test case readability.
    let code = code.strip_prefix('\n').expect("Test code should start with a newline");
    let expected = expected.strip_prefix('\n').expect("Expected code should start with a newline");

    let actual = format_code(code, options);
    assert_eq!(
        actual, expected,
        r"
💥 First format does not match expected!
============== actual =============
{actual}
============= expected ============
{expected}
============== options ============
{options}
"
    );

    // Check idempotency
    let actual = format_code(&actual, options);
    assert_eq!(
        actual, expected,
        r"
💥 Formatting is not idempotent!
============== actual =============
{actual}
============= expected ============
{expected}
============== options ============
{options}
"
    );
}

fn format_code(code: &str, options: &FormatOptions) -> String {
    use oxc_allocator::Allocator;
    use oxc_formatter::{Formatter, get_parse_options};
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    let allocator = Allocator::new();
    let source_type = SourceType::from_path("dummy.tsx").unwrap();

    let ret = Parser::new(&allocator, code, source_type).with_options(get_parse_options()).parse();

    if let Some(error) = ret.errors.first() {
        panic!("💥 Parser error: {}", error.message);
    }

    Formatter::new(&allocator, options.clone()).build(&ret.program)
}
