use std::sync::Arc;

use napi_derive::napi;

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_diagnostics::{Error, NamedSource};
use oxc_isolated_declarations::IsolatedDeclarations;
use oxc_parser::Parser;
use oxc_span::SourceType;

#[napi(object)]
pub struct IsolatedDeclarationsResult {
    pub source_text: String,
    pub errors: Vec<String>,
}

/// TypeScript Isolated Declarations for Standalone DTS Emit
#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn isolated_declaration(filename: String, source_text: String) -> IsolatedDeclarationsResult {
    let source_type = SourceType::from_path(&filename).unwrap_or_default().with_typescript(true);
    let allocator = Allocator::default();
    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
    let transformed_ret = IsolatedDeclarations::new(&allocator).build(&parser_ret.program);
    let printed = CodeGenerator::new().build(&transformed_ret.program).source_text;

    let mut errors = vec![];
    if !parser_ret.errors.is_empty() || !transformed_ret.errors.is_empty() {
        let source = Arc::new(NamedSource::new(filename, source_text.to_string()));
        errors.extend(
            parser_ret
                .errors
                .into_iter()
                .chain(transformed_ret.errors)
                .map(|diagnostic| Error::from(diagnostic).with_source_code(Arc::clone(&source)))
                .map(|error| format!("{error:?}")),
        );
    }

    IsolatedDeclarationsResult { source_text: printed, errors }
}
