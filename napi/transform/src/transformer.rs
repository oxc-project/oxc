use std::{mem, ops::ControlFlow, path::Path};

use napi::{Either, Task, bindgen_prelude::AsyncTask};
use napi_derive::napi;
use rustc_hash::FxHashMap;

use oxc::{
    CompilerInterface,
    codegen::CodegenReturn,
    diagnostics::{Diagnostics, OxcDiagnostic},
    transformer_plugins::{InjectGlobalVariablesConfig, InjectImport, ReplaceGlobalDefinesConfig},
};
use oxc_napi::{OxcError, get_source_type};
use oxc_sourcemap::napi::SourceMap;

use crate::TransformOptions;

#[derive(Default)]
#[napi(object)]
pub struct TransformResult {
    /// The transformed code.
    ///
    /// If parsing failed, this will be an empty string.
    pub code: String,

    /// The source map for the transformed code.
    ///
    /// This will be set if {@link TransformOptions#sourcemap} is `true`.
    pub map: Option<SourceMap>,

    /// The `.d.ts` declaration file for the transformed code. Declarations are
    /// only generated if `declaration` is set to `true` and a TypeScript file
    /// is provided.
    ///
    /// If parsing failed and `declaration` is set, this will be an empty string.
    ///
    /// @see {@link TypeScriptOptions#declaration}
    /// @see [declaration tsconfig option](https://www.typescriptlang.org/tsconfig/#declaration)
    pub declaration: Option<String>,

    /// Declaration source map. Only generated if both
    /// {@link TypeScriptOptions#declaration declaration} and
    /// {@link TransformOptions#sourcemap sourcemap} are set to `true`.
    pub declaration_map: Option<SourceMap>,

    /// Helpers used.
    ///
    /// @internal
    ///
    /// Example:
    ///
    /// ```text
    /// { "_objectSpread": "@oxc-project/runtime/helpers/objectSpread2" }
    /// ```
    #[napi(ts_type = "Record<string, string>")]
    pub helpers_used: FxHashMap<String, String>,

    /// Parse and transformation errors.
    ///
    /// Oxc's parser recovers from common syntax errors, meaning that
    /// transformed code may still be available even if there are errors in this
    /// list.
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

/// Transpile a JavaScript or TypeScript into a target ECMAScript version,
/// asynchronously on a separate thread.
///
/// Note: This function can be slower than `transformSync`,
/// due to the overhead of running on another thread.
///
/// @param filename - Name of the file being transformed.
///   If this is a relative path, consider setting the {@link TransformOptions#cwd} option.
/// @param sourceText - Source code
/// @param options - Transformation options. See {@link TransformOptions}.
/// @returns A `Promise` that resolves to an object containing the transformed code, source maps,
///   and any errors that occurred during parsing or transformation.
#[napi]
pub fn transform(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> AsyncTask<TransformTask> {
    AsyncTask::new(TransformTask { filename, source_text, options })
}

/// Transpile a JavaScript or TypeScript into a target ECMAScript version,
/// synchronously on current thread.
///
/// @param filename - Name of the file being transformed.
///   If this is a relative path, consider setting the {@link TransformOptions#cwd} option.
/// @param sourceText - Source code
/// @param options - Transformation options. See {@link TransformOptions}.
/// @returns An object containing the transformed code, source maps,
///   and any errors that occurred during parsing or transformation.
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
#[napi]
pub fn transform_sync(
    filename: String,
    source_text: String,
    options: Option<TransformOptions>,
) -> TransformResult {
    transform_impl(&filename, &source_text, options)
}

/// Parse `source_text`, transform it, and return the generated code, source map,
/// declaration and errors.
fn transform_impl(
    filename: &str,
    source_text: &str,
    options: Option<TransformOptions>,
) -> TransformResult {
    let source_path = Path::new(filename);

    let source_type = get_source_type(
        filename,
        options.as_ref().and_then(|options| options.lang.as_deref()),
        options.as_ref().and_then(|options| options.source_type.as_deref()),
    );

    let mut compiler = match Compiler::new(options) {
        Ok(compiler) => compiler,
        Err(errors) => {
            // Options errors are not errors in the source text. If they have labels
            // (e.g. a syntax error in a `define` value), the labels are offsets into
            // the option's value, so don't produce a codeframe.
            // They're also unrelated to this specific file, so don't include file path.
            let errors = errors.iter().map(OxcError::from).collect();
            return TransformResult { errors, ..Default::default() };
        }
    };

    compiler.compile(source_text, source_type, source_path);

    TransformResult {
        code: compiler.printed,
        map: compiler.printed_sourcemap,
        declaration: compiler.declaration,
        declaration_map: compiler.declaration_map,
        helpers_used: compiler.helpers_used,
        errors: OxcError::from_diagnostics(filename, source_text, compiler.errors),
    }
}

#[derive(Default)]
struct Compiler {
    transform_options: oxc::transformer::TransformOptions,
    isolated_declaration_options: Option<oxc::isolated_declarations::IsolatedDeclarationsOptions>,

    sourcemap: bool,

    printed: String,
    printed_sourcemap: Option<SourceMap>,

    declaration: Option<String>,
    declaration_map: Option<SourceMap>,

    define: Option<ReplaceGlobalDefinesConfig>,
    inject: Option<InjectGlobalVariablesConfig>,

    helpers_used: FxHashMap<String, String>,
    errors: Diagnostics,
}

impl Compiler {
    fn new(options: Option<TransformOptions>) -> Result<Self, Vec<OxcDiagnostic>> {
        let mut options = options;

        let isolated_declaration_options = options
            .as_ref()
            .and_then(|o| o.typescript.as_ref())
            .and_then(|o| o.declaration)
            .map(oxc::isolated_declarations::IsolatedDeclarationsOptions::from);

        let sourcemap = options.as_ref().and_then(|o| o.sourcemap).unwrap_or_default();

        let define = options
            .as_mut()
            .and_then(|options| options.define.take())
            .map(|map| {
                let define = map.into_iter().collect::<Vec<_>>();
                ReplaceGlobalDefinesConfig::new(&define)
            })
            .transpose()?;

        let inject = options
            .as_mut()
            .and_then(|options| options.inject.take())
            .map(|map| {
                map.into_iter()
                    .map(|(local, value)| match value {
                        Either::A(source) => Ok(InjectImport::default_specifier(&source, &local)),
                        Either::B(v) => {
                            if v.len() != 2 {
                                return Err(vec![OxcDiagnostic::error(
                                    "Inject plugin did not receive a tuple [string, string].",
                                )]);
                            }
                            let source = &v[0];
                            Ok(if v[1] == "*" {
                                InjectImport::namespace_specifier(source, &local)
                            } else {
                                InjectImport::named_specifier(source, Some(&v[1]), &local)
                            })
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .map(InjectGlobalVariablesConfig::new);

        let transform_options = match options {
            Some(options) => oxc::transformer::TransformOptions::try_from(options)
                .map_err(|err| vec![OxcDiagnostic::error(err)])?,
            None => oxc::transformer::TransformOptions::default(),
        };

        Ok(Self {
            transform_options,
            isolated_declaration_options,
            sourcemap,
            printed: String::default(),
            printed_sourcemap: None,
            declaration: None,
            declaration_map: None,
            define,
            inject,
            helpers_used: FxHashMap::default(),
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

    fn isolated_declaration_options(
        &self,
    ) -> Option<oxc::isolated_declarations::IsolatedDeclarationsOptions> {
        self.isolated_declaration_options
    }

    fn define_options(&self) -> Option<ReplaceGlobalDefinesConfig> {
        self.define.clone()
    }

    fn inject_options(&self) -> Option<InjectGlobalVariablesConfig> {
        self.inject.clone()
    }

    fn after_codegen(&mut self, ret: CodegenReturn<'_>) {
        self.printed = ret.code;
        self.printed_sourcemap = ret.map.map(SourceMap::from);
    }

    fn after_isolated_declarations(&mut self, ret: CodegenReturn<'_>) {
        self.declaration.replace(ret.code);
        self.declaration_map = ret.map.map(SourceMap::from);
    }

    #[expect(deprecated)]
    fn after_transform(
        &mut self,
        _program: &mut oxc::ast::ast::Program<'_>,
        transformer_return: &mut oxc::transformer::TransformerReturn,
    ) -> ControlFlow<()> {
        self.helpers_used = transformer_return
            .helpers_used
            .drain()
            .map(|(helper, source)| (helper.name().to_string(), source))
            .collect();
        ControlFlow::Continue(())
    }
}
