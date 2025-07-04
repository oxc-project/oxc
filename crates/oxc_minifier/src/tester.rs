use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use crate::{CompressOptions, Compressor};

#[track_caller]
pub fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[track_caller]
pub fn test_same_options(source_text: &str, options: &CompressOptions) {
    test_options(source_text, source_text, options);
}

#[track_caller]
pub fn test(source_text: &str, expected: &str) {
    test_options(source_text, expected, &CompressOptions::smallest());
}

#[track_caller]
pub fn test_options(source_text: &str, expected: &str, options: &CompressOptions) {
    let result = run(source_text, Some(options.clone()));
    let expected = run(expected, None);
    assert_eq!(result, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{result}");
}

#[track_caller]
pub fn run(source_text: &str, options: Option<CompressOptions>) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source_text, source_type)
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse();
    assert!(!ret.panicked, "{source_text}");
    assert!(ret.errors.is_empty(), "{source_text}");
    let mut program = ret.program;
    if let Some(options) = options {
        Compressor::new(&allocator, options).build(&mut program);
    }
    Codegen::new()
        .with_options(CodegenOptions {
            single_quote: true,
            minify: false,
            ..CodegenOptions::default()
        })
        .build(&program)
        .code
}
