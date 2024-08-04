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

pub(crate) fn test(source_text: &str, expected: &str) {
    let options = CompressOptions::all_true();
    test_with_options(source_text, expected, options);
}

pub(crate) fn test_same(source_text: &str) {
    test(source_text, source_text);
}

pub(crate) fn test_with_options(source_text: &str, expected: &str, options: CompressOptions) {
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

pub(crate) fn test_snapshot<S>(name: &str, sources: S)
where
    S: IntoIterator<Item = &'static str>,
{
    let source_type = SourceType::default();
    let options = CompressOptions::all_true();
    let snapshot: String = sources
        .into_iter()
        .map(|source| {
            let minified = run(source, source_type, Some(options));
            format!(
                "==================================== SOURCE ====================================
{source}

=================================== MINIFIED ===================================
{minified}

"
            )
        })
        .fold(String::new(), |mut acc, snapshot| {
            acc.push_str(snapshot.as_str());
            acc
        });
    insta::with_settings!({ prepend_module_to_snapshot => false }, {

        insta::assert_snapshot!(name, snapshot);
    });
}
