use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;

fn test_current_behavior() {
    let allocator = Allocator::default();
    let source_type = SourceType::default();
    
    // Test case: quoted constructor method (the issue)
    let source1 = r#"class C { "constructor"() {} }"#;
    println!("Test 1 - Quoted constructor:");
    println!("Original: {}", source1);
    
    let ret1 = Parser::new(&allocator, source1, source_type)
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse();
    
    let mut program1 = ret1.program;
    Compressor::new(&allocator).build(&mut program1, CompressOptions::smallest());
    
    let result1 = Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program1)
        .code;
    
    println!("Result:   {}", result1);
    
    // Test case: bracket constructor (should stay same)  
    let source2 = r#"class C { ['constructor']() {} }"#;
    println!("\nTest 2 - Bracket constructor:");
    println!("Original: {}", source2);
    
    let ret2 = Parser::new(&allocator, source2, source_type)
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse();
    
    let mut program2 = ret2.program;
    Compressor::new(&allocator).build(&mut program2, CompressOptions::smallest());
    
    let result2 = Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program2)
        .code;
    
    println!("Result:   {}", result2);
    
    // Test case: static quoted constructor (should be converted)
    let source3 = r#"class C { static "constructor"() {} }"#;
    println!("\nTest 3 - Static quoted constructor:");
    println!("Original: {}", source3);
    
    let ret3 = Parser::new(&allocator, source3, source_type)
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse();
    
    let mut program3 = ret3.program;
    Compressor::new(&allocator).build(&mut program3, CompressOptions::smallest());
    
    let result3 = Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program3)
        .code;
    
    println!("Result:   {}", result3);
}

fn main() {
    test_current_behavior();
}