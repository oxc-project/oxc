use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn test(source_text: &str, expected: &str) {
    test_options(source_text, expected, CodegenOptions::default());
}

pub fn test_options(source_text: &str, expected: &str, options: CodegenOptions) {
    test_options_with_source_type(source_text, expected, SourceType::jsx(), options);
}

pub fn test_tsx(source_text: &str, expected: &str) {
    test_options_with_source_type(
        source_text,
        expected,
        SourceType::tsx(),
        CodegenOptions::default(),
    );
}

pub fn test_options_with_source_type(
    source_text: &str,
    expected: &str,
    source_type: SourceType,
    options: CodegenOptions,
) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new().with_options(options).build(&ret.program).code;
    assert_eq!(result, expected, "\nfor source: {source_text:?}");
}

pub fn test_minify(source_text: &str, expected: &str) {
    let source_type = SourceType::jsx();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new()
        .with_options(CodegenOptions { minify: true, ..CodegenOptions::default() })
        .build(&ret.program)
        .code;
    assert_eq!(result, expected, "\nfor minify source: {source_text}");
}

pub fn test_minify_same(source_text: &str) {
    test_minify(source_text, source_text);
}
