use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn minify(filename: String, source_text: String) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(&filename).unwrap_or_default().with_typescript(true);

    let mut program = Parser::new(&allocator, &source_text, source_type).parse().program;

    let mangler =
        Minifier::new(MinifierOptions { mangle: true, compress: CompressOptions::default() })
            .build(&allocator, &mut program)
            .mangler;

    Codegen::new()
        .with_options(CodegenOptions { minify: true, ..CodegenOptions::default() })
        .with_mangler(mangler)
        .build(&program)
        .code
}
