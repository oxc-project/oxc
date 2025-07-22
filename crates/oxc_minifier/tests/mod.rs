#![allow(clippy::literal_string_with_formatting_args)]

mod ecmascript;
mod mangler;
mod peephole;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

#[track_caller]
pub(crate) fn test(source_text: &str, expected: &str, options: CompressOptions) {
    let source_type = SourceType::default();
    let first = run(source_text, source_type, Some(options.clone()));

    let expected = run(expected, source_type, None);
    assert_eq!(first, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{first}");

    let second = run(&first, source_type, Some(options));
    assert_eq!(
        first, second,
        "\nidempotency for source\n{source_text}\ngot\n{first}\nthen\n{second}"
    );
}

pub(crate) fn run(
    source_text: &str,
    source_type: SourceType,
    options: Option<CompressOptions>,
) -> String {
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
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code
}
