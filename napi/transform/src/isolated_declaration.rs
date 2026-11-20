use std::{mem, path::Path};

use napi::{Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;

use oxc::{
    codegen::{Codegen, CodegenOptions, CommentOptions},
    isolated_declarations::IsolatedDeclarations,
    span::SourceType,
};
use oxc_napi::{NapiSource, OxcError};
use oxc_sourcemap::napi::SourceMap;

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

#[napi(object)]
pub struct IsolatedDeclarationsResult {
    pub code: String,
    pub map: Option<SourceMap>,
    pub errors: Vec<OxcError>,
}

/// Async task for [`isolated_declaration`].
pub struct IsolatedDeclarationTask {
    /// File path.
    filename: String,
    /// Source text. `None` if `compute` has run.
    source: Option<NapiSource>,
    /// Options. `None` if no options provided.
    options: Option<IsolatedDeclarationsOptions>,
}

#[napi]
impl Task for IsolatedDeclarationTask {
    type JsValue = IsolatedDeclarationsResult;
    type Output = IsolatedDeclarationsResult;

    fn compute(&mut self) -> napi::Result<IsolatedDeclarationsResult> {
        // Move filename and source text out of the task, so they're dropped here on worker thread.
        // `IsolatedDeclarationsOptions` is `Copy` so no need to move it.
        let filename = mem::take(&mut self.filename);
        let mut napi_source = self.source.take().expect("`compute` called more than once");
        Ok(isolated_declaration_impl(&filename, &mut napi_source, self.options))
    }

    fn resolve(
        &mut self,
        _: napi::Env,
        result: IsolatedDeclarationsResult,
    ) -> napi::Result<IsolatedDeclarationsResult> {
        Ok(result)
    }
}

/// TypeScript Isolated Declarations for Standalone DTS Emit, asynchronously on a separate thread.
///
/// Note: This function can be slower than `isolatedDeclarationSync`,
/// due to the overhead of running on another thread.
///
/// @param filename - Name of the file being transformed.
/// @param sourceText - Source code
/// @param options - Isolated declarations options. See {@link IsolatedDeclarationsOptions}.
/// @returns A `Promise` that resolves to an object containing the declaration code, source map,
///   and any errors that occurred during parsing or transformation.
#[napi]
pub fn isolated_declaration(
    filename: String,
    #[napi(ts_arg_type = "string")] source_text: NapiSource,
    options: Option<IsolatedDeclarationsOptions>,
) -> AsyncTask<IsolatedDeclarationTask> {
    AsyncTask::new(IsolatedDeclarationTask { filename, source: Some(source_text), options })
}

/// TypeScript Isolated Declarations for Standalone DTS Emit, synchronously on current thread.
///
/// @param filename - Name of the file being transformed.
/// @param sourceText - Source code
/// @param options - Isolated declarations options. See {@link IsolatedDeclarationsOptions}.
/// @returns An object containing the declaration code, source map,
///   and any errors that occurred during parsing or transformation.
#[napi]
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
pub fn isolated_declaration_sync(
    filename: String,
    #[napi(ts_arg_type = "string")] mut source_text: NapiSource,
    options: Option<IsolatedDeclarationsOptions>,
) -> IsolatedDeclarationsResult {
    isolated_declaration_impl(&filename, &mut source_text, options)
}

/// Parse the source in `napi_source`, and generate a declaration file for it.
fn isolated_declaration_impl(
    filename: &str,
    napi_source: &mut NapiSource,
    options: Option<IsolatedDeclarationsOptions>,
) -> IsolatedDeclarationsResult {
    let source_path = Path::new(filename);
    let source_type = SourceType::from_path(source_path).unwrap_or_default().with_typescript(true);
    let source = napi_source.source(source_type);
    let allocator = source.allocator();
    let options = options.unwrap_or_default();

    let ret = source.parse();
    let source_text = ret.program.source_text;

    let transformed_ret = IsolatedDeclarations::new(
        allocator,
        oxc::isolated_declarations::IsolatedDeclarationsOptions {
            strip_internal: options.strip_internal.unwrap_or(false),
        },
    )
    .build(&ret.program);

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
    let errors = OxcError::from_diagnostics(filename, source_text, diagnostics);

    IsolatedDeclarationsResult {
        code: codegen_ret.code,
        map: codegen_ret.map.map(SourceMap::from),
        errors,
    }
}
