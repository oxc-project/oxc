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
    semantic::SemanticBuilder,
    span::SourceType,
};
use oxc_napi::{OxcError, SendableSource, get_source_type};
use oxc_relay::Relay;
use oxc_sourcemap::napi::SourceMap;

pub use crate::options::*;

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

/// Get [`SourceType`] for a file, from its filename and `options`.
fn source_type_for(filename: &str, options: Option<&TransformOptions>) -> SourceType {
    get_source_type(
        filename,
        options.and_then(|options| options.lang.as_deref()),
        options.and_then(|options| options.source_type.as_deref()),
    )
}

/// Parse `source`, apply the Relay transform, and return the generated code, source map and errors.
fn transform_impl(
    filename: &str,
    source: Source<'_>,
    options: Option<TransformOptions>,
) -> TransformResult {
    let sourcemap = options.as_ref().and_then(|options| options.sourcemap).unwrap_or(false);

    let relay_options = match options.unwrap_or_default().resolve() {
        Ok(options) => options,
        Err(error) => {
            // Options diagnostic has no labels, so source text is not needed to render it
            return TransformResult {
                errors: OxcError::from_diagnostics(filename, "", [error]),
                ..TransformResult::default()
            };
        }
    };

    let allocator = source.allocator();
    let parser_return = source.parse();
    let mut diagnostics = parser_return.diagnostics;
    let mut program = parser_return.program;
    let source_text = program.source_text;
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
        Relay::new(relay_options, Path::new(filename)).build(allocator, &mut program, scoping);
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

/// Apply the Relay `graphql` tagged template transform synchronously.
///
/// Only `graphql` tags are rewritten; TypeScript and JSX syntax are preserved
/// untouched, so the output composes with any downstream toolchain.
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

/// Apply the Relay `graphql` tagged template transform asynchronously.
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
