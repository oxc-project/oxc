mod closure;
mod mangler;
mod oxc;
// mod tdewolff;
// mod terser;

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub(crate) fn test_same(source_text: &str, options: CompressOptions) {
    test(source_text, source_text, options);
}

pub(crate) fn test(source_text: &str, expected: &str, options: CompressOptions) {
    let source_type = SourceType::default();
    let result = run(source_text, source_type, Some(options));
    let expected = run(expected, source_type, None);
    assert_eq!(
        result, expected,
        "\nfor source {source_text:?}\nexpect {expected:?}\ngot    {result:?}"
    );
}

fn run(source_text: &str, source_type: SourceType, options: Option<CompressOptions>) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    if let Some(options) = options {
        Compressor::new(&allocator, options).build(program);
    }
    CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true })
        .build(program)
        .source_text
}
