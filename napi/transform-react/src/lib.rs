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
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    diagnostics::{Diagnostics, Severity},
    parser::Parser,
    semantic::{SemanticBuilder, SemanticBuilderReturn},
    transformer::Transformer,
};
use oxc_napi::{OxcError, get_source_type};
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

    /// Parse, semantic, React Compiler, and downstream transform diagnostics.
    pub errors: Vec<OxcError>,
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

    let (react_compiler_options, transform_options) =
        match options.unwrap_or_default().resolve(filename) {
            Ok(options) => options,
            Err(error) => {
                return TransformResult {
                    fatal: true,
                    errors: OxcError::from_diagnostics(filename, source_text, [error]),
                    ..TransformResult::default()
                };
            }
        };

    let allocator = Allocator::default();
    let parser_return = Parser::new(&allocator, source_text, source_type).parse();
    let parser_has_errors = parser_return.diagnostics.has_errors();
    let mut diagnostics = parser_return.diagnostics;
    let mut program = parser_return.program;
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

    let (react_output, mut react_diagnostics, react_fatal) = match react_compiler_options {
        None => (None, Diagnostics::new(), false),
        Some(options) => match react_compiler_compile(&program, &semantic, &allocator, options) {
            CompileResult::Success { output, diagnostics } => (output, diagnostics, false),
            CompileResult::Fatal { diagnostics } => (None, diagnostics, true),
        },
    };
    if !react_fatal {
        for diagnostic in react_diagnostics.iter_mut() {
            if diagnostic.severity == Severity::Error {
                diagnostic.severity = Severity::Warning;
            }
        }
    }
    diagnostics.extend(react_diagnostics);
    if react_fatal {
        return error_result(filename, source_text, diagnostics);
    }

    let mut scoping = semantic.into_scoping();
    if let Some(output) = react_output {
        output.transform(&mut program);
        scoping =
            SemanticBuilder::new().with_enum_eval(true).build(&program).semantic.into_scoping();
    }

    let transformer_return = Transformer::new(&allocator, Path::new(filename), &transform_options)
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
