// Tests to verify that the optimizations maintain correct behavior

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

#[test]
fn test_minified_number_optimization() {
    let allocator = Allocator::default();

    // Test various number formats to ensure optimization maintains correctness
    let test_cases = vec![
        ("0.5", ".5"),        // Leading zero is removed
        ("1000", "1e3"),      // Scientific notation for shorter representation
        ("0.0001", "1e-4"),
        ("10000", "1e4"),
        ("0.123456789", ".123456789"),
        ("123456789", "123456789"),
        ("1.5e10", "15e9"),
    ];

    for (input, expected) in test_cases {
        let source = format!("var x = {input};");
        let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
        assert!(parsed.errors.is_empty(), "Parse error for {input}");

        let options = CodegenOptions { minify: true, ..Default::default() };
        let result = Codegen::new().with_options(options).build(&parsed.program);
        
        // Extract the number from the generated code
        let generated = result.code;
        let expected_full = format!("var x={expected};");
        assert_eq!(generated.trim(), expected_full, "Mismatch for input {input}");
    }
}

#[test]
fn test_comment_optimization() {
    let allocator = Allocator::default();
    
    // Test comment formatting with line terminators
    let source = r#"
/* Legal comment
   with multiple lines
   and indentation */
var x = 1;
"#;
    
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.errors.is_empty());

    let options = CodegenOptions::default();
    let result = Codegen::new().with_options(options).build(&parsed.program);
    
    // Should contain properly formatted comment
    assert!(result.code.contains("Legal comment"));
    assert!(result.code.contains("with multiple lines"));
}

#[test]
fn test_hex_number_optimization() {
    let allocator = Allocator::default();

    // Test cases where hex might be shorter than decimal (current behavior check)
    let test_cases = vec![
        ("255", "255"),   // Check current behavior
        ("256", "256"),   // Check current behavior  
        ("15", "15"),     // decimal is shorter
        ("16", "16"),     // Check current behavior
    ];

    for (input, expected) in test_cases {
        let source = format!("var x = {input};");
        let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
        assert!(parsed.errors.is_empty(), "Parse error for {input}");

        let options = CodegenOptions { minify: true, ..Default::default() };
        let result = Codegen::new().with_options(options).build(&parsed.program);
        
        let generated = result.code;
        let expected_full = format!("var x={expected};");
        assert_eq!(generated.trim(), expected_full, "Mismatch for input {input}");
    }
}

#[test]
fn test_scientific_notation_optimization() {
    let allocator = Allocator::default();

    // Test scientific notation optimizations
    let test_cases = vec![
        ("0.00001", "1e-5"),
        ("0.000123", "123e-6"),
        ("1230000", "123e4"),
        ("1.23e10", "123e8"),
    ];

    for (input, expected) in test_cases {
        let source = format!("var x = {input};");
        let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
        assert!(parsed.errors.is_empty(), "Parse error for {input}");

        let options = CodegenOptions { minify: true, ..Default::default() };
        let result = Codegen::new().with_options(options).build(&parsed.program);
        
        let generated = result.code;
        let expected_full = format!("var x={expected};");
        assert_eq!(generated.trim(), expected_full, "Mismatch for input {input}");
    }
}