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

use napi::{JsString, Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    diagnostics::Diagnostics,
    parser::Source,
    semantic::{SemanticBuilder, SemanticBuilderReturn},
    span::SourceType,
    transformer::Transformer,
};
use oxc_napi::{OxcError, SendableSource, get_source_type};
use oxc_react_compiler::{CompileResult, compile as react_compiler_compile};
use oxc_sourcemap::napi::SourceMap;

pub use crate::options::*;

/// Result returned by the React Compiler transform.
#[derive(Default)]
#[napi(object)]
pub struct TransformResult {
    /// Whether the transform was aborted without emitting code.
    pub fatal: bool,

    /// Transformed JavaScript code.
    ///
    /// This is empty when parsing, semantic analysis, option validation, or the
    /// React Compiler reports a fatal error.
    pub code: String,

    /// Source map, populated when `sourcemap` is `true`.
    pub map: Option<SourceMap>,

    /// Parse, semantic, downstream transform, and fatal React Compiler diagnostics.
    ///
    /// Recoverable React Compiler diagnostics are included when
    /// `reactCompiler.reportDiagnostics` is `true`.
    pub errors: Vec<OxcError>,
}

/// Get [`SourceType`] for a file, from its filename and `options`.
fn source_type_for(filename: &str, options: Option<&TransformOptions>) -> SourceType {
    get_source_type(
        filename,
        options.and_then(|options| options.lang.as_deref()),
        options.and_then(|options| options.source_type.as_deref()),
    )
}

/// Parse `source`, compile and transform it, and return the generated code, source map and errors.
fn transform_impl(
    filename: &str,
    source: Source<'_>,
    options: Option<TransformOptions>,
) -> TransformResult {
    let sourcemap = options.as_ref().and_then(|options| options.sourcemap).unwrap_or(false);

    let options = options.unwrap_or_default();
    let report_diagnostics = options.report_react_compiler_diagnostics();
    let (react_compiler_options, transform_options) = match options.resolve(filename) {
        Ok(options) => options,
        Err(error) => {
            // Options diagnostic has no labels, so source text is not needed to render it
            return TransformResult {
                fatal: true,
                errors: OxcError::from_diagnostics(filename, "", [error]),
                ..TransformResult::default()
            };
        }
    };

    let allocator = source.allocator();
    let parser_return = source.parse();
    let parser_has_errors = parser_return.diagnostics.has_errors();
    let mut diagnostics = parser_return.diagnostics;
    let mut program = parser_return.program;
    let source_text = program.source_text;
    if parser_has_errors {
        return error_result(filename, source_text, diagnostics);
    }

    let SemanticBuilderReturn { semantic, diagnostics: semantic_diagnostics } =
        SemanticBuilder::new_compiler()
            .with_excess_capacity(2.0)
            .with_enum_eval(true)
            .with_build_nodes(react_compiler_options.is_some())
            .build(&program);
    if !semantic_diagnostics.is_empty() {
        diagnostics.extend(semantic_diagnostics);
        return error_result(filename, source_text, diagnostics);
    }

    let react_output = match react_compiler_options {
        None => None,
        Some(options) => match react_compiler_compile(&program, &semantic, allocator, options) {
            // Include recoverable diagnostics only when requested.
            CompileResult::Success { output, diagnostics: react_diagnostics } => {
                if report_diagnostics {
                    diagnostics.extend(react_diagnostics);
                }
                output
            }
            CompileResult::Fatal { diagnostics: react_diagnostics } => {
                diagnostics.extend(react_diagnostics);
                return error_result(filename, source_text, diagnostics);
            }
        },
    };

    let mut scoping = semantic.into_scoping();
    if let Some(output) = react_output {
        output.transform(&mut program);
        scoping =
            SemanticBuilder::new().with_enum_eval(true).build(&program).semantic.into_scoping();
    }

    let transformer_return = Transformer::new(allocator, Path::new(filename), &transform_options)
        .build_with_scoping(scoping, &mut program);
    let transform_has_errors = transformer_return.diagnostics.has_errors();
    diagnostics.extend(transformer_return.diagnostics);
    if transform_has_errors {
        return error_result(filename, source_text, diagnostics);
    }

    let codegen_return = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: sourcemap.then(|| Path::new(filename).to_path_buf()),
            ..CodegenOptions::default()
        })
        .build(&program);
    TransformResult {
        fatal: false,
        code: codegen_return.code,
        map: codegen_return.map.map(SourceMap::from),
        errors: OxcError::from_diagnostics(filename, source_text, diagnostics),
    }
}

fn error_result(filename: &str, source_text: &str, diagnostics: Diagnostics) -> TransformResult {
    TransformResult {
        fatal: true,
        errors: OxcError::from_diagnostics(filename, source_text, diagnostics),
        ..TransformResult::default()
    }
}

/// Compile a JavaScript or TypeScript React module synchronously.
///
/// The React Compiler runs first on the pristine AST. TypeScript syntax is
/// removed and configured JSX transforms run afterwards.
#[napi]
#[allow(clippy::missing_errors_doc, clippy::allow_attributes)]
pub fn transform_sync(
    filename: String,
    source_text: JsString,
    options: Option<TransformOptions>,
) -> napi::Result<TransformResult> {
    let source_type = source_type_for(&filename, options.as_ref());

    // Copy source text from JS string directly into the arena
    let allocator = Allocator::default();
    let source = Source::from_napi_js_string(&source_text, source_type, &allocator)?;

    Ok(transform_impl(&filename, source, options))
}

pub struct TransformTask {
    filename: String,
    /// Source text. `None` after `compute` has run.
    source: Option<SendableSource>,
    options: Option<TransformOptions>,
}

#[napi]
impl Task for TransformTask {
    type JsValue = TransformResult;
    type Output = TransformResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let source_type = source_type_for(&self.filename, self.options.as_ref());

        // Take the source out of the task, so its `Allocator` is dropped at end of this method.
        // That frees its memory on this thread, not on main thread when the task is dropped.
        let mut sendable_source = self.source.take().expect("`compute` called more than once");
        let source = sendable_source.source(source_type);

        Ok(transform_impl(&self.filename, source, self.options.take()))
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
#[allow(clippy::missing_errors_doc, clippy::allow_attributes)]
pub fn transform(
    filename: String,
    source_text: JsString,
    options: Option<TransformOptions>,
) -> napi::Result<AsyncTask<TransformTask>> {
    // Copy source text from JS string directly into an arena.
    // This has to happen here, on main thread, as JS values cannot be accessed on other threads.
    let source = SendableSource::from_napi_js_string(&source_text)?;

    Ok(AsyncTask::new(TransformTask { filename, source: Some(source), options }))
}
