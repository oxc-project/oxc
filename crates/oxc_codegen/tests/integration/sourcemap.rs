use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, Statement};
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

use crate::tester::default_options;

/// Upstream may have modified the AST to include incorrect spans.
/// e.g. <https://github.com/rolldown/rolldown/blob/v1.0.0-beta.19/crates/rolldown/src/utils/ecma_visitors/mod.rs>
#[test]
fn incorrect_ast() {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let source_text = "foo\nvar bar = '测试'";
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    let mut program = ret.program;
    program.span = Span::new(0, 0);
    if let Statement::ExpressionStatement(s) = &mut program.body[0] {
        s.span = Span::new(17, 17);
        if let Expression::Identifier(ident) = &mut s.expression {
            ident.span = Span::new(17, 17);
        }
    }

    let ret = Codegen::new().with_options(default_options()).build(&program);
    assert!(ret.map.is_some(), "sourcemap exists");
}

/// Test that array elisions (empty array elements) get proper source mapping
#[test]
fn array_elision_source_mapping() {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let source_text = "[1, , 3]";
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    let mut options = default_options();
    options.source_map_path = Some("test.js".into());
    
    let result = Codegen::new().with_options(options).build(&ret.program);
    
    assert!(result.map.is_some(), "sourcemap should be generated");
    // The output may be formatted differently, but should contain the array elements
    assert!(result.code.contains("1"));
    assert!(result.code.contains("3"));
    
    let sourcemap = result.map.unwrap();
    
    // Verify that we have source mappings - there should be at least one for the array
    // and potentially one for the elision
    let tokens: Vec<_> = sourcemap.get_tokens().collect();
    assert!(!tokens.is_empty(), "should have source mapping tokens");
}

/// Test that JSX empty expressions get proper source mapping
#[test]
fn jsx_empty_expression_source_mapping() {
    let allocator = Allocator::default();
    let source_type = SourceType::jsx();
    let source_text = "<div>{}</div>";
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    let mut options = default_options();
    options.source_map_path = Some("test.jsx".into());
    
    let result = Codegen::new().with_options(options).build(&ret.program);
    
    assert!(result.map.is_some(), "sourcemap should be generated");
    // The output should contain the JSX structure
    assert!(result.code.contains("div"));
    
    let sourcemap = result.map.unwrap();
    
    // Verify that we have source mappings
    let tokens: Vec<_> = sourcemap.get_tokens().collect();
    assert!(!tokens.is_empty(), "should have source mapping tokens");
}
