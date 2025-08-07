// Simple performance test to verify optimizations are working

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::time::Instant;

#[test]
fn test_optimization_performance() {
    let allocator = Allocator::default();

    // Test with numbers that benefit from optimizations
    let test_numbers = vec![
        "0.5",
        "0.001",
        "1000",
        "10000",
        "0.0001",
        "123.456e-7",
        "987654321",
        "255",
        "256",
        "512",
        "1024",
        "0x100",
        "0.123456789",
        "9.876543e10",
    ];

    let mut source = String::with_capacity(1000);
    source.push_str("var numbers = [");
    for (i, num) in test_numbers.iter().enumerate() {
        if i > 0 {
            source.push_str(", ");
        }
        source.push_str(num);
    }
    source.push_str("];");

    let parsed = Parser::new(&allocator, &source, SourceType::mjs()).parse();
    assert!(parsed.errors.is_empty());

    // Test minified output
    let options = CodegenOptions { minify: true, ..Default::default() };

    // Warm up
    for _ in 0..10 {
        let _ = Codegen::new().with_options(options.clone()).build(&parsed.program);
    }

    // Time the generation
    let start = Instant::now();
    for _ in 0..100 {
        let _ = Codegen::new().with_options(options.clone()).build(&parsed.program);
    }
    let duration = start.elapsed();

    println!("Generated 100 iterations in {:?}", duration);

    // Just ensure the output is correct
    let result = Codegen::new().with_options(options).build(&parsed.program);
    assert!(result.code.contains("var numbers="));
    assert!(result.code.len() < source.len()); // Should be minified
}

#[test]
fn test_comment_processing_performance() {
    let allocator = Allocator::default();

    // Test comment with multiple lines that needs processing
    let source = r#"
/* This is a legal comment
   with multiple lines
   and different indentations
   that should be processed efficiently */
var x = 1;
"#;

    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.errors.is_empty());

    let options = CodegenOptions::default();

    // Warm up
    for _ in 0..10 {
        let _ = Codegen::new().with_options(options.clone()).build(&parsed.program);
    }

    // Time the generation
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = Codegen::new().with_options(options.clone()).build(&parsed.program);
    }
    let duration = start.elapsed();

    println!("Generated 1000 comment iterations in {:?}", duration);

    // Verify output includes the comment
    let result = Codegen::new().with_options(options).build(&parsed.program);
    assert!(result.code.contains("legal comment"));
}
