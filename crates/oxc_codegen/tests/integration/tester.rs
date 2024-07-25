use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions, WhitespaceRemover};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn test(source_text: &str, expected: &str) {
    let source_type = SourceType::default().with_module(true);
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true })
        .build(&ret.program)
        .source_text;
    check(source_text, &result, expected);
}

pub fn test_minify(source_text: &str, expected: &str) {
    let source_type = SourceType::default().with_module(true);
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = WhitespaceRemover::new()
        .with_options(CodegenOptions { single_quote: true })
        .build(&ret.program)
        .source_text;
    check(source_text, &result, expected);
}

fn check(source_text: &str, result: &str, expected: &str) {
    assert_eq!(result, expected, "for source {source_text}, expect {expected}, got {result}");
}
