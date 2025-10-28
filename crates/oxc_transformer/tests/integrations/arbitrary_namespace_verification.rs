use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use std::path::Path;

#[test]
fn chrome89_should_warn() {
    let source = r#"export { foo as "string-name" };"#;
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    
    let options = TransformOptions::from_target("chrome89").unwrap();
    let ret = Transformer::new(&allocator, Path::new("test.mjs"), &options)
        .build_with_scoping(scoping, &mut program);
    
    assert!(!ret.errors.is_empty(), "Expected warnings for Chrome 89");
}

#[test]
fn chrome90_should_not_warn() {
    let source = r#"export { foo as "string-name" };"#;
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    
    let options = TransformOptions::from_target("chrome90").unwrap();
    let ret = Transformer::new(&allocator, Path::new("test.mjs"), &options)
        .build_with_scoping(scoping, &mut program);
    
    assert!(ret.errors.is_empty(), "Expected no warnings for Chrome 90");
}

#[test]
fn multiple_string_literals_emit_multiple_warnings() {
    let source = r#"import { "a" as a, "b" as b } from "mod";"#;
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    
    let options = TransformOptions::from_target("chrome89").unwrap();
    let ret = Transformer::new(&allocator, Path::new("test.mjs"), &options)
        .build_with_scoping(scoping, &mut program);
    
    assert_eq!(ret.errors.len(), 2, "Expected 2 warnings for 2 string literals");
}

#[test]
fn normal_identifiers_no_warnings() {
    let source = r#"export { foo }; import { bar } from "mod";"#;
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source, source_type).parse();
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    
    let options = TransformOptions::from_target("chrome89").unwrap();
    let ret = Transformer::new(&allocator, Path::new("test.mjs"), &options)
        .build_with_scoping(scoping, &mut program);
    
    assert!(ret.errors.is_empty(), "Expected no warnings for normal identifiers");
}
