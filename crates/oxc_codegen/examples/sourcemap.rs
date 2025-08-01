#![expect(clippy::print_stdout)]
//! # Source Map Generation Example
//!
//! This example demonstrates how to generate source maps using the Oxc codegen.
//! Source maps are essential for debugging transpiled or minified JavaScript code
//! by mapping the generated code back to the original source.
//!
//! ## Features
//!
//! - Parse JavaScript/TypeScript source code into AST
//! - Generate code with corresponding source map
//! - Create visualization URL for the source map
//! - Handle parsing errors gracefully
//! - Support for various source file types
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_codegen --example sourcemap [filename]
//!    ```
//!
//! ## Output
//!
//! The example generates a URL for viewing the source map visualization
//! at https://evanw.github.io/source-map-visualization/
//!
//! ## What is a Source Map?
//!
//! A source map is a JSON file that maps the generated/compiled code back to
//! the original source code. This is crucial for debugging minified or transpiled
//! code in development tools.

use std::{env, path::Path};

use base64::{Engine, prelude::BASE64_STANDARD};
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Main entry point for the source map generation example
fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);

    println!("Oxc Source Map Generation Example");
    println!("=================================");
    println!("Input file: {name}");

    // Read and validate the source file
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
            eprintln!("❌ Error determining source type: {e}");
            return Ok(());
        }
    };

    println!("Source type: {source_type:?}");
    println!("Source length: {} bytes", source_text.len());
    println!();

    println!("Original code:");
    println!("{}", "─".repeat(50));
    println!("{source_text}");
    println!("{}", "─".repeat(50));
    println!();

    // Parse the source code
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    // Handle parsing errors
    if !ret.errors.is_empty() {
        println!("❌ Parsing failed with {} error(s):", ret.errors.len());
        println!();

        for (i, error) in ret.errors.iter().enumerate() {
            println!("Error #{}: ", i + 1);
            let error = error.clone().with_source_code(source_text.clone());
            println!("{error:?}");
            println!();
        }

        eprintln!("💡 Fix the parsing errors above and try again");
        return Ok(());
    }

    println!("✅ Parsing successful!");
    println!();

    // Generate code with source map
    println!("Generating code with source map...");
    let CodegenReturn { code, map, .. } = Codegen::new()
        .with_options(CodegenOptions {
            source_map_path: Some(path.to_path_buf()),
            ..CodegenOptions::default()
        })
        .build(&ret.program);

    println!("Generated code ({} bytes):", code.len());
    println!("{}", "─".repeat(50));
    println!("{code}");
    println!("{}", "─".repeat(50));
    println!();

    // Generate and display source map information
    if let Some(source_map) = map {
        println!("✅ Source map generated successfully!");

        let source_map_json = source_map.to_json_string();
        println!("Source map size: {} bytes", source_map_json.len());

        // Create visualization URL
        let hash = BASE64_STANDARD.encode(format!(
            "{}\0{}{}\0{}",
            code.len(),
            code,
            source_map_json.len(),
            source_map_json
        ));

        println!();
        println!("🔗 Source Map Visualization:");
        println!("https://evanw.github.io/source-map-visualization/#{hash}");
        println!();
        println!("💡 Click the URL above to visualize the source map mapping");
        println!("   between the original source and generated code");
    } else {
        println!("⚠️  No source map was generated");
        println!("💡 This might be because the code doesn't require any transformations");
    }

    Ok(())
}
