mod sort_imports;

use oxc_formatter::{FormatOptions, Oxfmtrc};

pub fn assert_format(code: &str, config_json: &str, expected: &str) {
    // NOTE: Strip leading single `\n` for better test case readability.
    let code = code.strip_prefix('\n').expect("Test code should start with a newline");
    let expected = expected.strip_prefix('\n').expect("Expected code should start with a newline");

    let config: Oxfmtrc = serde_json::from_str(config_json).expect("Invalid JSON config");
    let (options, _) = config.into_options().expect("Failed to convert config to FormatOptions");

    let actual = format_code(code, &options);
    assert_eq!(
        actual, expected,
        r"
💥 First format does not match expected!
============== input ==============
{code}
============== actual =============
{actual}
============= expected ============
{expected}
============== config =============
{config_json}
"
    );

    // Check idempotency
    let actual2 = format_code(&actual, &options);
    assert_eq!(
        actual2, expected,
        r"
💥 Formatting is not idempotent!
============== input ==============
{actual}
============== actual =============
{actual2}
============= expected ============
{expected}
============== config =============
{config_json}
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
