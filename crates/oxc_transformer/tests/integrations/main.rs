mod es_target;
mod plugins;
mod targets;

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};

pub fn codegen(source_text: &str, source_type: SourceType) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&ret.program)
        .code
}

pub(crate) fn test(
    source_text: &str,
    options: &TransformOptions,
) -> Result<String, Vec<OxcDiagnostic>> {
    let source_type = SourceType::default();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let (symbols, scopes) =
        SemanticBuilder::new().build(&program).semantic.into_symbol_table_and_scope_tree();
    let ret = Transformer::new(&allocator, Path::new(""), options).build_with_symbols_and_scopes(
        symbols,
        scopes,
        &mut program,
    );
    if !ret.errors.is_empty() {
        return Err(ret.errors);
    }
    let code = CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code;
    Ok(code)
}
