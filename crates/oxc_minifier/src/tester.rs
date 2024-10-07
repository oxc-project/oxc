use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::TraverseCtx;

use crate::{ast_passes::CompressorPass, ast_passes::RemoveSyntax, CompressOptions};

pub fn test<'a, P: CompressorPass<'a>>(
    allocator: &'a Allocator,
    source_text: &'a str,
    expected: &'a str,
    pass: &mut P,
) {
    let result = run(allocator, source_text, Some(pass));
    let expected = run::<P>(allocator, expected, None);
    assert_eq!(result, expected, "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{result}");
}

fn run<'a, P: CompressorPass<'a>>(
    allocator: &'a Allocator,
    source_text: &'a str,
    pass: Option<&mut P>,
) -> String {
    let source_type = SourceType::mjs();
    let mut program = Parser::new(allocator, source_text, source_type).parse().program;

    if let Some(pass) = pass {
        let (symbols, scopes) =
            SemanticBuilder::new("").build(&program).semantic.into_symbol_table_and_scope_tree();
        let mut ctx = TraverseCtx::new(scopes, symbols, allocator);
        RemoveSyntax::new(CompressOptions::all_false()).build(&mut program, &mut ctx);
        pass.build(&mut program, &mut ctx);
    }

    CodeGenerator::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code
}
