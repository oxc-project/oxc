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

use std::path::Path;

use napi as _;
use napi_derive::napi;
use oxc::{
    allocator::Allocator,
    codegen::Codegen,
    diagnostics::{Diagnostics, Severity},
    parser::Parser,
    semantic::SemanticBuilder,
    transformer::{JsxOptions, TransformOptions, Transformer},
};
use oxc_napi::{OxcError, get_source_type};
use oxc_react_compiler_upstream::transform as react_compiler_transform;

/// Result returned by the benchmark-only upstream React Compiler transform.
#[derive(Default)]
#[napi(object)]
pub struct TransformResult {
    pub fatal: bool,
    pub code: String,
    pub errors: Vec<OxcError>,
}

/// Compile a JavaScript or TypeScript React module synchronously.
///
/// React 19 is fixed as the compiler target. JSX is preserved, TypeScript is
/// stripped, and source maps are disabled so benchmark work is identical to
/// the transform-only comparison in `bench-transformer`.
#[napi]
pub fn transform_sync(filename: String, source_text: String) -> TransformResult {
    transform_impl(&filename, &source_text)
}

fn transform_impl(filename: &str, source_text: &str) -> TransformResult {
    let source_type = get_source_type(filename, None, None);
    let allocator = Allocator::default();
    let parser_return = Parser::new(&allocator, source_text, source_type).parse();
    let mut diagnostics = parser_return.diagnostics;
    if diagnostics.has_errors() {
        return error_result(filename, source_text, diagnostics);
    }

    let mut program = parser_return.program;
    let mut react_result = react_compiler_transform(&mut program, &allocator, filename);
    if !react_result.fatal {
        for diagnostic in react_result.diagnostics.iter_mut() {
            if diagnostic.severity == Severity::Error {
                diagnostic.severity = Severity::Warning;
            }
        }
    }
    diagnostics.extend(react_result.diagnostics);
    if react_result.fatal {
        return error_result(filename, source_text, diagnostics);
    }

    let semantic_return = SemanticBuilder::new().with_enum_eval(true).build(&program);
    if !semantic_return.diagnostics.is_empty() {
        diagnostics.extend(semantic_return.diagnostics);
        return error_result(filename, source_text, diagnostics);
    }

    let options = TransformOptions { jsx: JsxOptions::disable(), ..TransformOptions::default() };
    let transformer_return = Transformer::new(&allocator, Path::new(filename), &options)
        .build_with_scoping(semantic_return.semantic.into_scoping(), &mut program);
    let transform_has_errors = transformer_return.diagnostics.has_errors();
    diagnostics.extend(transformer_return.diagnostics);
    if transform_has_errors {
        return error_result(filename, source_text, diagnostics);
    }

    TransformResult {
        fatal: false,
        code: Codegen::new().build(&program).code,
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
