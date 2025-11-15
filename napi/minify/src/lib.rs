#![expect(clippy::needless_pass_by_value)]

#[cfg(all(
    feature = "allocator",
    not(any(target_arch = "arm", target_os = "freebsd", target_family = "wasm"))
))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

mod options;

use std::path::{Path, PathBuf};

use napi::{Either, Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_diagnostics::OxcDiagnostic;
use oxc_minifier::Minifier;
use oxc_napi::OxcError;
use oxc_parser::Parser;
use oxc_sourcemap::napi::SourceMap;
use oxc_span::SourceType;

pub use crate::options::*;

#[derive(Default)]
#[napi(object)]
pub struct MinifyResult {
    pub code: String,
    pub map: Option<SourceMap>,
    pub errors: Vec<OxcError>,
}

fn minify_impl(filename: &str, source_text: &str, options: Option<MinifyOptions>) -> MinifyResult {
    use oxc_codegen::CodegenOptions;
    let options = options.unwrap_or_default();

    let minifier_options = match oxc_minifier::MinifierOptions::try_from(&options) {
        Ok(options) => options,
        Err(error) => {
            return MinifyResult {
                errors: OxcError::from_diagnostics(
                    filename,
                    source_text,
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
        SourceType::from_path(filename).unwrap_or_default()
    };

    let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = parser_ret.program;

    let scoping = Minifier::new(minifier_options).minify(&allocator, &mut program).scoping;

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
        errors: OxcError::from_diagnostics(filename, source_text, parser_ret.errors),
    }
}

/// Minify synchronously.
#[napi]
pub fn minify_sync(
    filename: String,
    source_text: String,
    options: Option<MinifyOptions>,
) -> MinifyResult {
    minify_impl(&filename, &source_text, options)
}

pub struct MinifyTask {
    filename: String,
    source_text: String,
    options: Option<MinifyOptions>,
}

#[napi]
impl Task for MinifyTask {
    type JsValue = MinifyResult;
    type Output = MinifyResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(minify_impl(&self.filename, &self.source_text, self.options.take()))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// Minify asynchronously.
///
/// Note: This function can be slower than `minifySync` due to the overhead of spawning a thread.
#[napi]
pub fn minify(
    filename: String,
    source_text: String,
    options: Option<MinifyOptions>,
) -> AsyncTask<MinifyTask> {
    AsyncTask::new(MinifyTask { filename, source_text, options })
}
