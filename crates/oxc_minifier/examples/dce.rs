#![expect(clippy::print_stdout)]
//! # Dead Code Elimination (DCE) Example
//!
//! This example demonstrates Oxc's dead code elimination capabilities,
//! which remove unreachable or unused code from JavaScript programs.
//! DCE is an important optimization technique that reduces bundle size
//! and improves runtime performance.
//!
//! ## Features
//!
//! - Parse JavaScript/TypeScript source code
//! - Apply dead code elimination optimizations
//! - Generate optimized output with optional minification
//! - Test idempotency by running DCE multiple times
//! - Compare original vs optimized code size
//!
//! ## Dead Code Elimination Types
//!
//! The DCE pass removes:
//! - Unreachable code after return/throw statements
//! - Unused variable declarations
//! - Unused function declarations
//! - Conditional branches that are always false
//! - Empty blocks and statements
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_minifier --example dce [options] [filename]
//!    ```
//!
//! ## Options
//!
//! - `--nospace`: Generate minified output without unnecessary whitespace
//! - `--twice`: Run DCE twice to test for idempotency
//!
//! ## Example Input/Output
//!
//! For code containing:
//! ```javascript
//! function unused() { return 42; }
//! function main() {
//!   if (false) {
//!     console.log("unreachable");
//!   }
//!   return true;
//! }
//! ```
//!
//! DCE will remove the unused function and unreachable code.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::Parser;
use oxc_span::SourceType;
use pico_args::Arguments;

/// Main entry point for the dead code elimination example
fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let mut args = Arguments::from_env();
    let nospace = args.contains("--nospace");
    let twice = args.contains("--twice");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    println!("Oxc Dead Code Elimination Example");
    println!("=================================");
    println!("Input file: {name}");
    println!("Options:");
    println!("  • Minify (no spaces): {nospace}");
    println!("  • Run twice (idempotency test): {twice}");
    println!();

    // Read and validate the source file
    let path = Path::new(&name);
    let source_text = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{name}': {e}");
            eprintln!("💡 Make sure the file exists and is readable");
            return Err(e);
        }
    };

    let source_type = match SourceType::from_path(path) {
        Ok(st) => st,
        Err(e) => {
            eprintln!("❌ Error determining source type: {e}");
            return Ok(());
        }
    };

    println!("Source type: {source_type:?}");
    println!("Original size: {} bytes", source_text.len());
    println!();

    println!("Original code:");
    println!("{}", "─".repeat(60));
    println!("{source_text}");
    println!("{}", "─".repeat(60));
    println!();

    // First DCE pass
    let mut allocator = Allocator::default();
    let first_pass_result = perform_dce(&allocator, &source_text, source_type, nospace);

    println!("✅ First DCE pass completed!");
    println!();
    println!("After dead code elimination ({} bytes):", first_pass_result.len());
    println!("{}", "─".repeat(60));
    println!("{first_pass_result}");
    println!("{}", "─".repeat(60));

    // Calculate size reduction
    let original_size = source_text.len();
    let optimized_size = first_pass_result.len();
    let reduction = original_size as f64 - optimized_size as f64;
    let reduction_percent = (reduction / original_size as f64) * 100.0;

    println!();
    println!("📊 Optimization Results:");
    println!("   Original: {original_size} bytes");
    println!("   Optimized: {optimized_size} bytes");
    if reduction > 0.0 {
        println!("   ✅ Reduced by {} bytes ({:.1}%)", reduction as usize, reduction_percent);
    } else if reduction < 0.0 {
        println!(
            "   ⚠️  Increased by {} bytes ({:.1}%)",
            (-reduction) as usize,
            -reduction_percent
        );
    } else {
        println!("   ➖ No size change");
    }

    // Second pass for idempotency testing
    if twice {
        println!();
        println!("🔄 Running second DCE pass for idempotency test...");

        // Reset allocator for second pass
        allocator.reset();
        let second_pass_result = perform_dce(&allocator, &first_pass_result, source_type, nospace);

        println!();
        println!("After second DCE pass ({} bytes):", second_pass_result.len());
        println!("{}", "─".repeat(60));
        println!("{second_pass_result}");
        println!("{}", "─".repeat(60));

        // Check idempotency
        let is_identical = first_pass_result == second_pass_result;
        println!();
        if is_identical {
            println!("✅ Idempotency test passed!");
            println!("   Both passes produced identical results, indicating");
            println!("   the DCE algorithm is stable and complete.");
        } else {
            println!("⚠️  Idempotency test failed!");
            println!("   The second pass produced different results.");
            println!("   This might indicate an issue with the DCE algorithm");
            println!("   or that further optimizations were possible.");

            let second_size = second_pass_result.len();
            if second_size != optimized_size {
                let diff = second_size as i32 - optimized_size as i32;
                println!("   Size difference: {diff} bytes");
            }
        }
    }

    println!();
    println!("💡 Dead Code Elimination removes:");
    println!("   • Unreachable code after return/throw statements");
    println!("   • Unused variable and function declarations");
    println!("   • Always-false conditional branches");
    println!("   • Empty blocks and statements");

    Ok(())
}

/// Perform dead code elimination on the given source code
fn perform_dce(
    allocator: &Allocator,
    source_text: &str,
    source_type: SourceType,
    minify: bool,
) -> String {
    // Parse the source code
    let ret = Parser::new(allocator, source_text, source_type).parse();

    // Handle parsing errors by displaying them but continuing
    if !ret.errors.is_empty() {
        println!("⚠️  Parsing warnings/errors ({}):", ret.errors.len());
        for (i, error) in ret.errors.iter().enumerate() {
            println!("   {}: {:?}", i + 1, error.clone().with_source_code(source_text.to_string()));
        }
        println!();
    }

    let mut program = ret.program;

    // Apply dead code elimination
    let compressor = Compressor::new(allocator);
    compressor.dead_code_elimination(&mut program, CompressOptions::dce());

    // Generate the optimized code
    let codegen_options = CodegenOptions { minify, ..CodegenOptions::default() };

    Codegen::new().with_options(codegen_options).build(&program).code
}
