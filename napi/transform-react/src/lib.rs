#![expect(clippy::needless_pass_by_value)]

#[cfg(all(
    feature = "allocator",
    not(any(
        target_arch = "arm",
        target_os = "android",
        target_os = "freebsd",
        target_os = "windows",
        target_family = "wasm"
    ))
))]
#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

mod options;

use std::path::Path;

use napi::{Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc::{
    CompilerInterface,
    codegen::CodegenReturn,
    diagnostics::{Diagnostics, OxcDiagnostic},
};
use oxc_napi::{OxcError, get_source_type};
use oxc_sourcemap::napi::SourceMap;

pub use crate::options::*;

/// Result returned by the React Compiler transform.
#[derive(Default)]
#[napi(object)]
pub struct TransformResult {
    /// Transformed JavaScript code.
    ///
    /// This is empty when parsing, semantic analysis, option validation, or the
    /// React Compiler reports a fatal error.
    pub code: String,

    /// Source map, populated when `sourcemap` is `true`.
    pub map: Option<SourceMap>,

    /// Parse, semantic, React Compiler, and downstream transform diagnostics.
    pub errors: Vec<OxcError>,
}

#[derive(Default)]
struct Compiler {
    transform_options: oxc::transformer::TransformOptions,
    sourcemap: bool,
    printed: String,
    printed_sourcemap: Option<SourceMap>,
    errors: Diagnostics,
}

impl Compiler {
    fn new(filename: &str, options: Option<TransformOptions>) -> Result<Self, OxcDiagnostic> {
        let sourcemap = options.as_ref().and_then(|options| options.sourcemap).unwrap_or(false);
        let transform_options = options.unwrap_or_default().into_transform_options(filename)?;

        Ok(Self {
            transform_options,
            sourcemap,
            printed: String::new(),
            printed_sourcemap: None,
            errors: Diagnostics::new(),
        })
    }
}

impl CompilerInterface for Compiler {
    fn handle_errors(&mut self, errors: Diagnostics) {
        self.errors.extend(errors);
    }

    fn enable_sourcemap(&self) -> bool {
        self.sourcemap
    }

    fn transform_options(&self) -> Option<&oxc::transformer::TransformOptions> {
        Some(&self.transform_options)
    }

    fn after_codegen(&mut self, ret: CodegenReturn<'_>) {
        self.printed = ret.code;
        self.printed_sourcemap = ret.map.map(SourceMap::from);
    }
}

fn transform_impl(
    filename: &str,
    source_text: &str,
    options: Option<TransformOptions>,
) -> TransformResult {
    let source_type = get_source_type(
        filename,
        options.as_ref().and_then(|options| options.lang.as_deref()),
        options.as_ref().and_then(|options| options.source_type.as_deref()),
    );

    let mut compiler = match Compiler::new(filename, options) {
        Ok(compiler) => compiler,
        Err(error) => {
            return TransformResult {
                errors: OxcError::from_diagnostics(filename, source_text, [error]),
                ..TransformResult::default()
            };
        }
    };

    compiler.compile(source_text, source_type, Path::new(filename));

    TransformResult {
        code: compiler.printed,
        map: compiler.printed_sourcemap,
        errors: OxcError::from_diagnostics(filename, source_text, compiler.errors),
    }
}

/// Compile a JavaScript or TypeScript React module synchronously.
///
/// The React Compiler runs first on the pristine AST. TypeScript and JSX are
/// lowered afterwards, matching the transform pipeline used by `oxc-transform`.
#[napi]
pub fn transform_sync(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> TransformResult {
    transform_impl(&filename, &source_text, options)
}

pub struct TransformTask {
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
}

#[napi]
impl Task for TransformTask {
    type JsValue = TransformResult;
    type Output = TransformResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        Ok(transform_impl(&self.filename, &self.source_text, self.options.take()))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// Compile a JavaScript or TypeScript React module asynchronously.
///
/// This uses a worker-pool thread and can be slower than `transformSync` for a
/// single small module.
#[napi]
pub fn transform(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> AsyncTask<TransformTask> {
    AsyncTask::new(TransformTask { filename, source_text, options })
}
