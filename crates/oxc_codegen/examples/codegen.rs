#![expect(clippy::print_stdout)]
//! # Code Generation Example
//!
//! This example demonstrates how to use the Oxc codegen to generate JavaScript code from an AST.
//! It shows the parse-codegen round-trip process and tests for idempotency.
//!
//! ## Features
//!
//! - Parse JavaScript/TypeScript code into AST
//! - Generate code from AST back to string
//! - Test idempotency (parsing generated code should produce the same result)
//! - Support for minified output
//! - Syntax error detection and reporting
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_codegen --example codegen [options] [filename]
//!    ```
//!    Or with cargo watch:
//!    ```bash
//!    cargo watch -x "run -p oxc_codegen --example codegen"
//!    ```
//!
//! ## Options
//!
//! - `--twice`: Run codegen twice to test idempotency
//! - `--minify`: Generate minified output

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

/// Main entry point for the code generation example
fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let mut args = Arguments::from_env();
    let twice = args.contains("--twice");
    let minify = args.contains("--minify");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read and validate the source file
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();
    let mut allocator = Allocator::default();

    println!("Oxc Code Generation Example");
    println!("===========================");
    println!("File: {name}");
    println!("Source type: {source_type:?}");
    println!("Settings:");
    println!("  - Minify: {minify}");
    println!("  - Test idempotency: {twice}");
    println!();

    println!("Original code ({} bytes):", source_text.len());
    println!("{}", "─".repeat(50));
    println!("{source_text}");
    println!("{}", "─".repeat(50));
    println!();

    // First pass: parse and generate code
    let first_pass_result = {
        let program = parse(&allocator, &source_text, source_type);
        codegen(&program, minify)
    };

    println!("First pass result ({} bytes):", first_pass_result.len());
    println!("{}", "─".repeat(50));
    println!("{first_pass_result}");
    println!("{}", "─".repeat(50));

    // Test idempotency if requested
    if twice {
        println!();
        println!("Testing idempotency (second pass):");

        // Reset the allocator as we don't need the first AST anymore
        allocator.reset();

        let program = parse(&allocator, &first_pass_result, source_type);
        let second_pass_result = codegen(&program, minify);

        println!("Second pass result ({} bytes):", second_pass_result.len());
        println!("{}", "─".repeat(50));
        println!("{second_pass_result}");
        println!("{}", "─".repeat(50));

        // Verify parsing the generated code doesn't produce syntax errors
        allocator.reset();
        let _validation_program = parse(&allocator, &second_pass_result, source_type);

        // Check if results are identical
        let identical = first_pass_result == second_pass_result;
        println!();
        if identical {
            println!("✅ Idempotency test passed - both passes produced identical results");
        } else {
            println!("❌ Idempotency test failed - passes produced different results");
        }
    }

    Ok(())
}

/// Parse source code into an AST, handling any parsing errors
fn parse<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> Program<'a> {
    // Configure parser options for maximum compatibility
    let ret = Parser::new(allocator, source_text, source_type)
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse();

    // Report any parsing errors
    if !ret.errors.is_empty() {
        println!("⚠️  Parsing errors found:");
        for error in &ret.errors {
            println!("{:?}", error.clone().with_source_code(source_text.to_string()));
        }
    }

    ret.program
}

/// Generate code from an AST with the specified options
fn codegen(program: &Program<'_>, minify: bool) -> String {
    let options = if minify { CodegenOptions::minify() } else { CodegenOptions::default() };

    Codegen::new().with_options(options).build(program).code
}
