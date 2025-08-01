#![expect(clippy::print_stdout)]
//! # Oxc Compiler Example
//!
//! This example demonstrates how to use the Oxc compiler to transform JavaScript/TypeScript code.
//! The Oxc compiler provides a complete compilation pipeline including parsing, semantic analysis,
//! transformation, and code generation.
//!
//! ## Features
//!
//! - Parse JavaScript/TypeScript source code
//! - Perform semantic analysis and transformations
//! - Generate optimized output code
//! - Comprehensive error reporting with source context
//! - Support for various JavaScript/TypeScript features
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js` or `test.ts`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc --example compiler --features="full" [filename]
//!    ```
//!    Or with cargo watch:
//!    ```bash
//!    just watch 'run -p oxc --example compiler --features="full"'
//!    ```
//!
//! ## Example Input/Output
//!
//! For a file containing:
//! ```javascript
//! const x = (a) => a + 1;
//! console.log(x(5));
//! ```
//!
//! The compiler will parse, analyze, and potentially transform the code,
//! then output the result.

use std::{env, io, path::Path};

use oxc::{Compiler, span::SourceType};

/// Main entry point for the Oxc compiler example
fn main() -> io::Result<()> {
    // Parse command line arguments - use provided filename or default to test.js
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);

    println!("Oxc Compiler Example");
    println!("====================");
    println!("Input file: {name}");

    // Read the source file
    let source_text = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error reading file '{name}': {e}");
            eprintln!("💡 Make sure the file exists and is readable");
            return Err(e);
        }
    };

    // Determine source type from file extension
    let source_type = match SourceType::from_path(path) {
        Ok(st) => st,
        Err(e) => {
            eprintln!("❌ Error determining source type for '{name}': {e}");
            eprintln!("💡 Supported extensions: .js, .jsx, .ts, .tsx, .mjs, .cjs");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unsupported file type: {e}"),
            ));
        }
    };

    println!("Source type: {source_type:?}");
    println!("Source length: {} bytes", source_text.len());
    println!("═══════════════════════");
    println!();

    println!("Original code:");
    println!("{}", "─".repeat(50));
    println!("{source_text}");
    println!("{}", "─".repeat(50));
    println!();

    println!("Running Oxc compiler...");
    println!();

    // Execute the compiler with default settings
    match Compiler::default().execute(&source_text, source_type, path) {
        Ok(result) => {
            println!("✅ Compilation successful!");
            println!();
            println!("Compiled output ({} bytes):", result.len());
            println!("{}", "─".repeat(50));
            println!("{result}");
            println!("{}", "─".repeat(50));

            // Calculate compression ratio if applicable
            if result.len() != source_text.len() {
                let ratio = (result.len() as f64 / source_text.len() as f64) * 100.0;
                println!();
                println!(
                    "Size change: {} → {} bytes ({:.1}%)",
                    source_text.len(),
                    result.len(),
                    ratio
                );
            }
        }
        Err(errors) => {
            println!("❌ Compilation failed with {} error(s):", errors.len());
            println!();

            // Display each error with source context
            for (i, error) in errors.iter().enumerate() {
                println!("Error #{}: ", i + 1);
                let error_with_source = error.clone().with_source_code(source_text.to_string());
                println!("{error_with_source:?}");
                println!();
            }

            eprintln!("💡 Fix the errors above and try again");
        }
    }

    Ok(())
}
