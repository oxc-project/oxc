#![expect(clippy::needless_pass_by_value)]

use std::{mem, path::Path};

use napi::{Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    diagnostics::Diagnostics,
    parser::Parser,
    semantic::SemanticBuilder,
};
use oxc_napi::{OxcError, get_source_type};
use oxc_relay::Relay;
use oxc_sourcemap::napi::SourceMap;

mod options;
pub use options::*;

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

/// Result returned by the Relay transform.
#[derive(Default)]
#[napi(object)]
pub struct TransformResult {
    /// Transformed code.
    ///
    /// This is empty when parsing, semantic analysis, option validation, or
    /// the Relay transform reports an error.
    pub code: String,

    /// Source map, populated when `sourcemap` is `true`.
    pub map: Option<SourceMap>,

    /// Parse, semantic, option validation, and Relay transform diagnostics.
    pub errors: Vec<OxcError>,
}

/// Async task for [`transform`].
pub struct TransformTask {
    /// File path.
    filename: String,
    /// Source text.
    source_text: String,
    /// Options. `None` if no options provided, or if `compute` has run.
    options: Option<TransformOptions>,
}

#[napi]
impl Task for TransformTask {
    type JsValue = TransformResult;
    type Output = TransformResult;

    fn compute(&mut self) -> napi::Result<TransformResult> {
        // Move all data out of the task, so it's dropped here on worker thread
        let filename = mem::take(&mut self.filename);
        let source_text = mem::take(&mut self.source_text);
        let options = self.options.take();
        Ok(transform_impl(&filename, &source_text, options))
    }

    fn resolve(&mut self, _: napi::Env, result: TransformResult) -> napi::Result<TransformResult> {
        Ok(result)
    }
}

/// Apply the Relay `graphql` tagged template transform, asynchronously on a separate thread.
///
/// Only `graphql` tags are rewritten. TypeScript and JSX syntax are preserved untouched,
/// so the output composes with any downstream toolchain.
///
/// Note: This function can be slower than `transformSync`,
/// due to the overhead of running on another thread.
///
/// @param filename - Name of the file being transformed.
/// @param sourceText - Source code
/// @param options - Transformation options. See {@link TransformOptions}.
/// @returns A `Promise` that resolves to an object containing the transformed code, source map,
///   and any errors that occurred during parsing or transformation.
#[napi]
pub fn transform(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> AsyncTask<TransformTask> {
    AsyncTask::new(TransformTask { filename, source_text, options })
}

/// Apply the Relay `graphql` tagged template transform, synchronously on current thread.
///
/// Only `graphql` tags are rewritten. TypeScript and JSX syntax are preserved untouched,
/// so the output composes with any downstream toolchain.
///
/// @param filename - Name of the file being transformed.
/// @param sourceText - Source code
/// @param options - Transformation options. See {@link TransformOptions}.
/// @returns An object containing the transformed code, source map,
///   and any errors that occurred during parsing or transformation.
#[napi]
pub fn transform_sync(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> TransformResult {
    transform_impl(&filename, &source_text, options)
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
    let sourcemap = options.as_ref().and_then(|options| options.sourcemap).unwrap_or(false);

    let relay_options = match options.unwrap_or_default().resolve() {
        Ok(options) => options,
        Err(error) => {
            // Options error is not an error in the source text, so don't produce a codeframe
            return TransformResult {
                errors: vec![OxcError::from(&error)],
                ..TransformResult::default()
            };
        }
    };

    let allocator = Allocator::default();
    let parser_return = Parser::new(&allocator, source_text, source_type).parse();
    let mut diagnostics = parser_return.diagnostics;
    let mut program = parser_return.program;
    if diagnostics.has_errors() {
        return error_result(filename, source_text, diagnostics);
    }

    let semantic_return = SemanticBuilder::new().build(&program);
    if !semantic_return.diagnostics.is_empty() {
        diagnostics.extend(semantic_return.diagnostics);
        return error_result(filename, source_text, diagnostics);
    }

    let scoping = semantic_return.semantic.into_scoping();
    let relay_return =
        Relay::new(relay_options, Path::new(filename)).build(&allocator, &mut program, scoping);
    let relay_has_errors = relay_return.diagnostics.has_errors();
    diagnostics.extend(relay_return.diagnostics);
    if relay_has_errors {
        return error_result(filename, source_text, diagnostics);
    }

    let codegen_return = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: sourcemap.then(|| Path::new(filename).to_path_buf()),
            ..CodegenOptions::default()
        })
        .build(&program);
    TransformResult {
        code: codegen_return.code,
        map: codegen_return.map.map(SourceMap::from),
        errors: OxcError::from_diagnostics(filename, source_text, diagnostics),
    }
}

fn error_result(filename: &str, source_text: &str, diagnostics: Diagnostics) -> TransformResult {
    TransformResult {
        errors: OxcError::from_diagnostics(filename, source_text, diagnostics),
        ..TransformResult::default()
    }
}
