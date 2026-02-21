mod es_target;
mod jsx_pragmas;
mod targets;

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};

/// # Panics
/// Panics if there are parse errors.
pub fn codegen(source_text: &str, source_type: SourceType) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty());
    Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&ret.program)
        .code
}

pub(crate) fn test(
    source_text: &str,
    options: &TransformOptions,
) -> Result<String, Vec<OxcDiagnostic>> {
    test_with_source_type(source_text, SourceType::default(), options)
}

pub(crate) fn test_with_source_type(
    source_text: &str,
    source_type: SourceType,
    options: &TransformOptions,
) -> Result<String, Vec<OxcDiagnostic>> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let ret = Transformer::new(&allocator, Path::new(""), options)
        .build_with_scoping(scoping, &mut program);
    if !ret.errors.is_empty() {
        return Err(ret.errors);
    }
    let code = Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code;
    Ok(code)
}
