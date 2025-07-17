use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

use crate::{CompressOptions, CompressOptionsUnused, Compressor};

pub fn default_options() -> CompressOptions {
    CompressOptions { unused: CompressOptionsUnused::Keep, ..CompressOptions::smallest() }
}

#[track_caller]
pub fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[track_caller]
pub fn test_same_options(source_text: &str, options: &CompressOptions) {
    test_options(source_text, source_text, options);
}

#[track_caller]
pub fn test_same_options_source_type(
    source_text: &str,
    source_type: SourceType,
    options: &CompressOptions,
) {
    test_options_source_type(source_text, source_text, source_type, options);
}

#[track_caller]
pub fn test(source_text: &str, expected: &str) {
    test_options(source_text, expected, &default_options());
}

#[track_caller]
pub fn test_options(source_text: &str, expected: &str, options: &CompressOptions) {
    let source_type = SourceType::mjs();
    test_options_source_type(source_text, expected, source_type, options);
}

#[track_caller]
pub fn test_options_source_type(
    source_text: &str,
    expected: &str,
    source_type: SourceType,
    options: &CompressOptions,
) {
    let result = run(source_text, source_type, Some(options.clone()));
    let expected = run(expected, source_type, None);
    assert_eq!(result, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{result}");
}

#[track_caller]
fn run(source_text: &str, source_type: SourceType, options: Option<CompressOptions>) -> String {
    let allocator = Allocator::default();
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
        Compressor::new(&allocator).build(&mut program, options);
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
