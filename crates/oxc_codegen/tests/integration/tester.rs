use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn test(source_text: &str, expected: &str) {
    let source_type = SourceType::default().with_module(true);
    check(source_text, expected, source_type);
}

fn check(source_text: &str, expected: &str, source_type: SourceType) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let result = CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true })
        .build(&ret.program)
        .source_text;
    assert_eq!(expected, result, "for source {source_text}, expect {expected}, got {result}");
}
