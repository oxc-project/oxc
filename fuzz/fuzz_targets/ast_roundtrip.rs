#![no_main]

use libfuzzer_sys::fuzz_target;
use oxc_allocator::Allocator;
use oxc_ast::AstBuilder;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::{SourceType, Span};
use oxc_syntax::number::NumberBase;

// Generate a simple AST node from fuzz input
fn generate_simple_program<'a>(ast: AstBuilder<'a>, data: &[u8]) -> oxc_ast::ast::Program<'a> {
    let mut statements = ast.vec();
    
    // Generate simple statements based on input bytes
    for chunk in data.chunks(4) {
        let value = chunk.iter().fold(0u32, |acc, &b| (acc << 8) | u32::from(b));
        
        match value % 3 {
            0 => {
                // Create a simple numeric literal expression statement
                let expr = ast.expression_numeric_literal(
                    Span::default(),
                    (value % 1000) as f64,
                    None,
                    NumberBase::Decimal
                );
                statements.push(ast.statement_expression(Span::default(), expr));
            },
            1 => {
                // Create a string literal expression statement
                let expr = ast.expression_string_literal(
                    Span::default(),
                    ast.atom(&format!("str{}", value % 100)),
                    None
                );
                statements.push(ast.statement_expression(Span::default(), expr));
            },
            _ => {
                // Create a boolean literal expression statement
                let expr = ast.expression_boolean_literal(
                    Span::default(),
                    value % 2 == 0
                );
                statements.push(ast.statement_expression(Span::default(), expr));
            }
        }
    }

    // Create a program with the generated statements
    ast.program(
        Span::default(),
        SourceType::default(),
        "", // source text
        ast.vec(), // comments  
        None, // hashbang
        ast.vec(), // directives
        statements // body
    )
}

fuzz_target!(|data: &[u8]| {
    // Skip empty or very large inputs
    if data.is_empty() || data.len() > 256 {
        return;
    }

    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    
    // Generate AST from fuzz input
    let original_ast = generate_simple_program(ast, data);
    
    // Convert AST to code
    let codegen_options = CodegenOptions::default();
    let generated_code = Codegen::new().with_options(codegen_options).build(&original_ast);
    let code = generated_code.code;
    
    // Parse the generated code back to AST
    let parser_allocator = Allocator::default();
    let source_type = SourceType::default();
    let parser_options = ParseOptions::default();
    
    let ret = Parser::new(&parser_allocator, &code, source_type)
        .with_options(parser_options)
        .parse();
    
    // Basic checks to ensure the round-trip is valid
    if !ret.errors.is_empty() {
        // If there are parse errors, this indicates a bug in codegen
        // or our AST generation, which is valuable to find
        eprintln!("Generated code: {}", code);
        for error in &ret.errors {
            eprintln!("Parse error: {:?}", error);
        }
        panic!("Code generation created unparseable code");
    }
    
    // If we get here, the round-trip succeeded
});
