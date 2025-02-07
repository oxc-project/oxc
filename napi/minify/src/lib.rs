#![allow(clippy::needless_pass_by_value)]

mod options;

use std::path::PathBuf;

use napi::Either;
use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::Minifier;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::options::{MinifyOptions, MinifyResult};

/// Minify synchronously.
///
/// # Errors
///
/// * Fails to parse the options.
#[napi]
pub fn minify(
    filename: String,
    source_text: String,
    options: Option<MinifyOptions>,
) -> napi::Result<MinifyResult> {
    let options = options.unwrap_or_default();

    let minifier_options = match oxc_minifier::MinifierOptions::try_from(&options) {
        Ok(options) => options,
        Err(error) => return Err(napi::Error::from_reason(&error)),
    };

    let allocator = Allocator::default();

    let source_type = SourceType::from_path(&filename).unwrap_or_default().with_typescript(true);

    let mut program = Parser::new(&allocator, &source_text, source_type).parse().program;

    let symbol_table = Minifier::new(minifier_options).build(&allocator, &mut program).symbol_table;

    let mut codegen_options = match &options.codegen {
        Some(Either::A(false)) => CodegenOptions { minify: false, ..CodegenOptions::default() },
        None | Some(Either::A(true)) => {
            CodegenOptions { minify: true, ..CodegenOptions::default() }
        }
        Some(Either::B(o)) => CodegenOptions::from(o),
    };

    if options.sourcemap == Some(true) {
        codegen_options.source_map_path = Some(PathBuf::from(filename));
    }

    let ret = Codegen::new()
        .with_options(codegen_options)
        .with_symbol_table(symbol_table)
        .build(&program);

    Ok(MinifyResult { code: ret.code, map: ret.map.map(oxc_sourcemap::napi::SourceMap::from) })
}
