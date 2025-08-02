#![expect(clippy::needless_pass_by_value)]

#[cfg(all(
    feature = "allocator",
    not(any(target_arch = "arm", target_os = "freebsd", target_family = "wasm"))
))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

mod options;

use std::path::{Path, PathBuf};

use napi::Either;
use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::OxcDiagnostic;
use oxc_minifier::Minifier;
use oxc_napi::OxcError;
use oxc_parser::Parser;
use oxc_sourcemap::napi::SourceMap;
use oxc_span::SourceType;

pub use crate::options::MinifyOptions;

#[derive(Default)]
#[napi(object)]
pub struct MinifyResult {
    pub code: String,
    pub map: Option<SourceMap>,
    pub errors: Vec<OxcError>,
}

/// Minify synchronously.
#[napi]
pub fn minify(
    filename: String,
    source_text: String,
    options: Option<MinifyOptions>,
) -> MinifyResult {
    let options = options.unwrap_or_default();

    let minifier_options = match oxc_minifier::MinifierOptions::try_from(&options) {
        Ok(options) => options,
        Err(error) => {
            return MinifyResult {
                errors: OxcError::from_diagnostics(
                    &filename,
                    &source_text,
                    vec![OxcDiagnostic::error(error)],
                ),
                ..MinifyResult::default()
            };
        }
    };

    let allocator = Allocator::default();

    let source_type = if options.module == Some(true) {
        SourceType::mjs()
    } else if Path::new(&filename).extension().is_some_and(|ext| ext == "js") {
        SourceType::cjs()
    } else {
        SourceType::from_path(&filename).unwrap_or_default()
    };

    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
    let mut program = parser_ret.program;

    let scoping = Minifier::new(minifier_options).build(&allocator, &mut program).scoping;

    let mut codegen_options = match &options.codegen {
        // Need to remove all comments.
        Some(Either::A(false)) => CodegenOptions { minify: false, ..CodegenOptions::minify() },
        None | Some(Either::A(true)) => CodegenOptions::minify(),
        Some(Either::B(o)) => CodegenOptions::from(o),
    };

    if options.sourcemap == Some(true) {
        codegen_options.source_map_path = Some(PathBuf::from(&filename));
    }

    let ret = Codegen::new().with_options(codegen_options).with_scoping(scoping).build(&program);

    MinifyResult {
        code: ret.code,
        map: ret.map.map(oxc_sourcemap::napi::SourceMap::from),
        errors: OxcError::from_diagnostics(&filename, &source_text, parser_ret.errors),
    }
}
