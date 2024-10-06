use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions, CommentOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn test(source_text: &str, expected: &str) {
    let source_type = SourceType::jsx();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new()
        .enable_comment(
            source_text,
            ret.trivias,
            CommentOptions { preserve_annotate_comments: true },
        )
        .build(&ret.program)
        .code;
    assert_eq!(result, expected, "\nfor source: {source_text:?}");
}

pub fn test_without_source(source_text: &str, expected: &str) {
    let source_type = SourceType::jsx();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new().build(&ret.program).code;
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
