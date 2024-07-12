use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn check(source_text: &str, expected: &str, source_type: SourceType, options: CodegenOptions) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new().with_options(options).build(&ret.program).source_text;
    assert_eq!(expected, result, "for source {source_text}, expect {expected}, got {result}");
}

// dead_code is getting triggered during test runs for some reason, even though
// these functions are definitely used.

#[allow(dead_code)]
pub fn test(source_text: &str, expected: &str) {
    const OPTIONS: CodegenOptions = CodegenOptions::new().with_single_quotes();
    let source_type = SourceType::default().with_module(true);

    check(source_text, expected, source_type, OPTIONS);
}

#[allow(dead_code)]
pub fn test_ts(source_text: &str, expected: &str, is_typescript_definition: bool) {
    const OPTIONS: CodegenOptions = CodegenOptions::new().with_single_quotes();
    let source_type = SourceType::default()
        .with_typescript(true)
        .with_typescript_definition(is_typescript_definition)
        .with_module(true);
    check(source_text, expected, source_type, OPTIONS);
}

#[allow(dead_code)]
pub fn test_opt(source_text: &str, expected: &str, options: CodegenOptions) {
    let source_type = SourceType::default().with_module(true);
    check(source_text, expected, source_type, options);
}
