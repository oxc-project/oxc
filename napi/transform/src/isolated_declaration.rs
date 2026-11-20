use std::path::Path;

use napi::{JsString, Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions, CommentOptions},
    isolated_declarations::IsolatedDeclarations,
    parser::Source,
    span::SourceType,
};
use oxc_napi::{OxcError, SendableSource};
use oxc_sourcemap::napi::SourceMap;

#[napi(object)]
pub struct IsolatedDeclarationsResult {
    pub code: String,
    pub map: Option<SourceMap>,
    pub errors: Vec<OxcError>,
}

#[napi(object)]
#[derive(Debug, Default, Clone, Copy)]
pub struct IsolatedDeclarationsOptions {
    /// Do not emit declarations for code that has an @internal annotation in its JSDoc comment.
    /// This is an internal compiler option; use at your own risk, because the compiler does not check that the result is valid.
    ///
    /// Default: `false`
    ///
    /// See <https://www.typescriptlang.org/tsconfig/#stripInternal>
    pub strip_internal: Option<bool>,

    pub sourcemap: Option<bool>,
}

impl From<IsolatedDeclarationsOptions> for oxc::isolated_declarations::IsolatedDeclarationsOptions {
    fn from(options: IsolatedDeclarationsOptions) -> Self {
        Self { strip_internal: options.strip_internal.unwrap_or_default() }
    }
}

/// Get [`SourceType`] for a file, from its filename. Source is always parsed as TypeScript.
fn source_type_for(filename: &str) -> SourceType {
    SourceType::from_path(Path::new(filename)).unwrap_or_default().with_typescript(true)
}

/// Parse `source`, and generate a declaration file for it.
fn isolated_declaration_impl(
    filename: &str,
    source: Source<'_>,
    options: Option<IsolatedDeclarationsOptions>,
) -> IsolatedDeclarationsResult {
    let source_path = Path::new(filename);
    let allocator = source.allocator();
    let options = options.unwrap_or_default();

    let ret = source.parse();
    let program = ret.program;

    let transformed_ret = IsolatedDeclarations::new(
        allocator,
        oxc::isolated_declarations::IsolatedDeclarationsOptions {
            strip_internal: options.strip_internal.unwrap_or(false),
        },
    )
    .build(&program);

    let source_map_path = match options.sourcemap {
        Some(true) => Some(source_path.to_path_buf()),
        _ => None,
    };
    let codegen_ret = Codegen::new()
        .with_options(CodegenOptions {
            comments: CommentOptions { jsdoc: true, ..CommentOptions::disabled() },
            source_map_path,
            ..CodegenOptions::default()
        })
        .build(&transformed_ret.program);

    let diagnostics =
        ret.diagnostics.into_iter().chain(transformed_ret.diagnostics).collect::<Vec<_>>();
    let errors = OxcError::from_diagnostics(filename, program.source_text, diagnostics);

    IsolatedDeclarationsResult {
        code: codegen_ret.code,
        map: codegen_ret.map.map(SourceMap::from),
        errors,
    }
}

/// TypeScript Isolated Declarations for Standalone DTS Emit
#[allow(clippy::needless_pass_by_value, clippy::missing_errors_doc, clippy::allow_attributes)]
#[napi]
pub fn isolated_declaration_sync(
    filename: String,
    source_text: JsString,
    options: Option<IsolatedDeclarationsOptions>,
) -> napi::Result<IsolatedDeclarationsResult> {
    let source_type = source_type_for(&filename);

    // Copy source text from JS string directly into the arena
    let allocator = Allocator::default();
    let source = Source::from_napi_js_string(&source_text, source_type, &allocator)?;

    Ok(isolated_declaration_impl(&filename, source, options))
}

pub struct IsolatedDeclarationTask {
    filename: String,
    /// Source text. `None` after `compute` has run.
    source: Option<SendableSource>,
    options: Option<IsolatedDeclarationsOptions>,
}

#[napi]
impl Task for IsolatedDeclarationTask {
    type JsValue = IsolatedDeclarationsResult;
    type Output = IsolatedDeclarationsResult;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let source_type = source_type_for(&self.filename);

        // Take the source out of the task, so its `Allocator` is dropped at end of this method.
        // That frees its memory on this thread, not on main thread when the task is dropped.
        let mut sendable_source = self.source.take().expect("`compute` called more than once");
        let source = sendable_source.source(source_type);

        Ok(isolated_declaration_impl(&self.filename, source, self.options.take()))
    }

    fn resolve(&mut self, _: napi::Env, result: Self::Output) -> napi::Result<Self::JsValue> {
        Ok(result)
    }
}

/// TypeScript Isolated Declarations for Standalone DTS Emit (async)
///
/// Note: This function can be slower than `isolatedDeclarationSync` due to the overhead of spawning a thread.
#[allow(clippy::missing_errors_doc, clippy::allow_attributes)]
#[napi]
pub fn isolated_declaration(
    filename: String,
    source_text: JsString,
    options: Option<IsolatedDeclarationsOptions>,
) -> napi::Result<AsyncTask<IsolatedDeclarationTask>> {
    // Copy source text from JS string directly into an arena.
    // This has to happen here, on main thread, as JS values cannot be accessed on other threads.
    let source = SendableSource::from_napi_js_string(&source_text)?;

    Ok(AsyncTask::new(IsolatedDeclarationTask { filename, source: Some(source), options }))
}
