use std::{mem, path::Path};

use napi::{
    Either, Task,
    bindgen_prelude::{AsyncTask, Utf16String},
};
use napi_derive::napi;

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions, CodegenReturn},
    parser::Parser,
    semantic::{SemanticBuilder, SemanticBuilderReturn},
    span::SourceType,
    str::JSStr,
    transformer_plugins::ModuleRunnerTransform,
};
use oxc_napi::OxcError;
use oxc_sourcemap::napi::SourceMap;

#[derive(Clone, Copy, Default)]
#[napi(object)]
pub struct ModuleRunnerTransformOptions {
    /// Enable source map generation.
    ///
    /// When `true`, the `sourceMap` field of transform result objects will be populated.
    ///
    /// @default false
    ///
    /// @see {@link SourceMap}
    pub sourcemap: Option<bool>,
}

#[derive(Default)]
#[napi(object)]
pub struct ModuleRunnerTransformResult {
    /// The transformed code.
    ///
    /// If parsing failed, this will be an empty string.
    pub code: String,

    /// The source map for the transformed code.
    ///
    /// This will be set if {@link TransformOptions#sourcemap} is `true`.
    pub map: Option<SourceMap>,

    // Import sources collected during transformation.
    #[napi(ts_type = "string[]")]
    pub deps: Vec<Either<String, Utf16String>>,

    // Dynamic import sources collected during transformation.
    #[napi(ts_type = "string[]")]
    pub dynamic_deps: Vec<Either<String, Utf16String>>,

    /// Parse and transformation errors.
    ///
    /// Oxc's parser recovers from common syntax errors, meaning that
    /// transformed code may still be available even if there are errors in this
    /// list.
    pub errors: Vec<OxcError>,
}

/// Async task for [`module_runner_transform`].
pub struct ModuleRunnerTransformTask {
    /// File path.
    filename: String,
    /// Source text.
    source_text: String,
    /// Options. `None` if no options provided.
    options: Option<ModuleRunnerTransformOptions>,
}

#[napi]
impl Task for ModuleRunnerTransformTask {
    type JsValue = ModuleRunnerTransformResult;
    type Output = ModuleRunnerTransformResult;

    fn compute(&mut self) -> napi::Result<ModuleRunnerTransformResult> {
        // Move filename and source text out of the task, so they're dropped here on worker thread.
        // `ModuleRunnerTransformOptions` is `Copy` so no need to move it.
        let filename = mem::take(&mut self.filename);
        let source_text = mem::take(&mut self.source_text);
        Ok(module_runner_transform_impl(&filename, &source_text, self.options))
    }

    fn resolve(
        &mut self,
        _: napi::Env,
        result: ModuleRunnerTransformResult,
    ) -> napi::Result<ModuleRunnerTransformResult> {
        Ok(result)
    }
}

/// Transform JavaScript code to a Vite Node runnable module, asynchronously on a separate thread.
///
/// Note: This function can be slower than `moduleRunnerTransformSync`,
/// due to the overhead of running on another thread.
///
/// @deprecated Only works for Vite.
///
/// @param filename - Name of the file being transformed.
/// @param sourceText - Source code
/// @param options - Transformation options. See {@link ModuleRunnerTransformOptions}.
/// @returns A `Promise` that resolves to an object containing the transformed code, source maps,
///   and any errors that occurred during parsing or transformation.
#[napi]
pub fn module_runner_transform(
    filename: String,
    source_text: String,
    options: Option<ModuleRunnerTransformOptions>,
) -> AsyncTask<ModuleRunnerTransformTask> {
    AsyncTask::new(ModuleRunnerTransformTask { filename, source_text, options })
}

/// Transform JavaScript code to a Vite Node runnable module, synchronously on current thread.
///
/// @deprecated Only works for Vite.
///
/// @param filename - Name of the file being transformed.
/// @param sourceText - Source code
/// @param options - Transformation options. See {@link ModuleRunnerTransformOptions}.
/// @returns An object containing the transformed code, source maps,
///   and any errors that occurred during parsing or transformation.
#[napi]
#[allow(clippy::needless_pass_by_value, clippy::allow_attributes)]
pub fn module_runner_transform_sync(
    filename: String,
    source_text: String,
    options: Option<ModuleRunnerTransformOptions>,
) -> ModuleRunnerTransformResult {
    module_runner_transform_impl(&filename, &source_text, options)
}

/// Transform JavaScript code to a Vite Node runnable module.
fn module_runner_transform_impl(
    filename: &str,
    source_text: &str,
    options: Option<ModuleRunnerTransformOptions>,
) -> ModuleRunnerTransformResult {
    let file_path = Path::new(filename);
    let source_type = SourceType::from_path(file_path);
    let source_type = match source_type {
        Ok(s) => s,
        Err(err) => {
            return ModuleRunnerTransformResult {
                code: String::default(),
                map: None,
                deps: vec![],
                dynamic_deps: vec![],
                errors: vec![OxcError::new(err.to_string())],
            };
        }
    };

    let allocator = Allocator::default();
    let mut parser_ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = parser_ret.program;

    let SemanticBuilderReturn { semantic, diagnostics } =
        SemanticBuilder::new_compiler().build(&program);
    parser_ret.diagnostics.extend(diagnostics);

    let scoping = semantic.into_scoping();
    let (deps, dynamic_deps) =
        ModuleRunnerTransform::default().transform(&allocator, &mut program, scoping);

    let CodegenReturn { code, map, .. } = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: options.and_then(|opts| {
                opts.sourcemap.as_ref().and_then(|s| s.then(|| file_path.to_path_buf()))
            }),
            ..Default::default()
        })
        .build(&program);

    let to_napi_string = |value: JSStr<'_>| {
        value.as_str().map_or_else(
            || Either::B(value.encode_utf16().collect::<Vec<_>>().into()),
            |value| Either::A(value.to_owned()),
        )
    };
    ModuleRunnerTransformResult {
        code,
        map: map.map(Into::into),
        deps: deps.into_iter().map(to_napi_string).collect(),
        dynamic_deps: dynamic_deps.into_iter().map(to_napi_string).collect(),
        errors: OxcError::from_diagnostics(filename, source_text, parser_ret.diagnostics),
    }
}
