#![expect(clippy::print_stdout)]
//! # Variable Name Mangling Example
//!
//! This example demonstrates Oxc's variable name mangling capabilities,
//! which shortens variable and function names to reduce code size while
//! preserving program semantics. Mangling is a key minification technique
//! used in production JavaScript bundles.
//!
//! ## Features
//!
//! - Parse JavaScript/TypeScript source code
//! - Apply variable name mangling optimizations
//! - Preserve or mangle function and class names based on options
//! - Support top-level mangling for modules
//! - Debug mode to see mangling statistics
//! - Test idempotency by running mangling multiple times
//!
//! ## Name Mangling
//!
//! The mangler shortens identifiers by:
//! - Replacing long variable names with short ones (a, b, c, ...)
//! - Respecting scope boundaries to avoid conflicts
//! - Preserving global identifiers that must remain accessible
//! - Optionally preserving function and class names for debugging
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_minifier --example mangler [options] [filename]
//!    ```
//!
//! ## Options
//!
//! - `--keep-names`: Preserve function and class names (useful for debugging)
//! - `--debug`: Show detailed mangling statistics and information
//! - `--twice`: Run mangling twice to test for idempotency
//!
//! ## Example Input/Output
//!
//! For code containing:
//! ```javascript
//! function calculateSum(firstNumber, secondNumber) {
//!   const temporaryResult = firstNumber + secondNumber;
//!   return temporaryResult;
//! }
//! ```
//!
//! After mangling (with names preserved):
//! ```javascript
//! function calculateSum(a, b) {
//!   const c = a + b;
//!   return c;
//! }
//! ```

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames, Mangler};
use oxc_parser::Parser;
use oxc_span::SourceType;
use pico_args::Arguments;

/// Main entry point for the variable name mangling example
fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let mut args = Arguments::from_env();
    let keep_names = args.contains("--keep-names");
    let debug = args.contains("--debug");
    let twice = args.contains("--twice");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    println!("Oxc Variable Name Mangling Example");
    println!("==================================");
    println!("Input file: {name}");
    println!("Options:");
    println!("  • Keep function/class names: {keep_names}");
    println!("  • Debug mode: {debug}");
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

    if source_type.is_module() {
        println!("ℹ️  Module detected - top-level mangling will be enabled");
    }

    println!();

    println!("Original code:");
    println!("{}", "─".repeat(60));
    println!("{source_text}");
    println!("{}", "─".repeat(60));
    println!();

    // Configure mangling options
    let options = MangleOptions {
        top_level: source_type.is_module(),
        keep_names: MangleOptionsKeepNames { function: keep_names, class: keep_names },
        debug,
    };

    // First mangling pass
    let first_pass_result = perform_mangling(&source_text, source_type, options);

    println!("✅ First mangling pass completed!");
    println!();
    println!("After name mangling ({} bytes):", first_pass_result.len());
    println!("{}", "─".repeat(60));
    println!("{first_pass_result}");
    println!("{}", "─".repeat(60));

    // Calculate size reduction
    let original_size = source_text.len();
    let mangled_size = first_pass_result.len();
    let reduction = original_size as f64 - mangled_size as f64;
    let reduction_percent = (reduction / original_size as f64) * 100.0;

    println!();
    println!("📊 Mangling Results:");
    println!("   Original: {original_size} bytes");
    println!("   Mangled: {mangled_size} bytes");
    if reduction > 0.0 {
        println!("   ✅ Reduced by {} bytes ({:.1}%)", reduction as usize, reduction_percent);
    } else if reduction < 0.0 {
        println!(
            "   ⚠️  Increased by {} bytes ({:.1}%)",
            (-reduction) as usize,
            -reduction_percent
        );
        println!("      (This can happen with very short variable names)");
    } else {
        println!("   ➖ No size change");
    }

    // Second pass for idempotency testing
    if twice {
        println!();
        println!("🔄 Running second mangling pass for idempotency test...");

        let second_pass_result = perform_mangling(&first_pass_result, source_type, options);

        println!();
        println!("After second mangling pass ({} bytes):", second_pass_result.len());
        println!("{}", "─".repeat(60));
        println!("{second_pass_result}");
        println!("{}", "─".repeat(60));

        // Check idempotency
        let is_identical = first_pass_result == second_pass_result;
        println!();
        if is_identical {
            println!("✅ Idempotency test passed!");
            println!("   Both passes produced identical results, indicating");
            println!("   the mangling algorithm is stable and deterministic.");
        } else {
            println!("⚠️  Idempotency test failed!");
            println!("   The second pass produced different results.");
            println!("   This might indicate an issue with the mangling algorithm");
            println!("   or non-deterministic behavior.");

            let second_size = second_pass_result.len();
            if second_size != mangled_size {
                let diff = second_size as i32 - mangled_size as i32;
                println!("   Size difference: {diff} bytes");
            }
        }
    }

    println!();
    println!("💡 Variable name mangling:");
    println!("   • Shortens variable names (longVariableName → a)");
    println!("   • Respects scope boundaries to avoid conflicts");
    println!("   • Preserves global identifiers and API contracts");
    println!("   • Can optionally preserve function/class names for debugging");

    if debug {
        println!();
        println!("🔍 Debug mode was enabled - check console for detailed statistics");
    }

    Ok(())
}

/// Perform variable name mangling on the given source code
fn perform_mangling(source_text: &str, source_type: SourceType, options: MangleOptions) -> String {
    let allocator = Allocator::default();

    // Parse the source code
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    // Handle parsing errors by displaying them but continuing
    if !ret.errors.is_empty() {
        println!("⚠️  Parsing warnings/errors ({}):", ret.errors.len());
        for (i, error) in ret.errors.iter().enumerate() {
            println!("   {}: {:?}", i + 1, error.clone().with_source_code(source_text.to_string()));
        }
        println!();
    }

    // Create mangler and build symbol table with mangled names
    let symbol_table = Mangler::new().with_options(options).build(&ret.program);

    // Generate code with mangled variable names
    Codegen::new().with_scoping(Some(symbol_table)).build(&ret.program).code
}
