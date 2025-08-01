#![expect(clippy::print_stdout)]
//! # Minifier Example
//!
//! This example demonstrates how to use the Oxc minifier to compress JavaScript code.
//! It includes dead code elimination, mangling, and compression features.
//!
//! ## Features
//!
//! - Code compression and optimization
//! - Dead code elimination
//! - Variable and property mangling
//! - Source map generation
//! - Multiple passes for maximum compression
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_minifier --example minifier [options] [filename]
//!    ```
//!    Or with just:
//!    ```bash
//!    just example minifier
//!    ```
//!
//! ## Options
//!
//! - `--mangle`: Enable variable and property name mangling
//! - `--nospace`: Remove unnecessary whitespace
//! - `--twice`: Run minification twice to test idempotency
//! - `--sourcemap`: Generate source maps for debugging

use std::path::{Path, PathBuf};

use base64::{Engine, prelude::BASE64_STANDARD};
use pico_args::Arguments;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn, CommentOptions};
use oxc_mangler::MangleOptions;
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Main entry point for the minifier example
fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let mut args = Arguments::from_env();
    let mangle = args.contains("--mangle");
    let nospace = args.contains("--nospace");
    let twice = args.contains("--twice");
    let sourcemap = args.contains("--sourcemap");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read and validate the source file
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();
    let source_map_path = sourcemap.then(|| path.to_path_buf());

    // Display minification settings
    println!("Oxc Minifier Example");
    println!("===================");
    println!("File: {name}");
    println!("Source type: {source_type:?}");
    println!("Settings:");
    println!("  - Mangle: {mangle}");
    println!("  - Remove spaces: {nospace}");
    println!("  - Source maps: {sourcemap}");
    println!("  - Double pass: {twice}");
    println!();

    println!("Original code ({} bytes):", source_text.len());
    println!("{}", "─".repeat(50));
    println!("{source_text}");
    println!("{}", "─".repeat(50));
    println!();

    // Perform minification
    let mut allocator = Allocator::default();
    let result = minify(&allocator, &source_text, source_type, source_map_path, mangle, nospace);
    let minified_code = result.code;

    // Display results
    println!(
        "Minified code ({} bytes, {:.1}% reduction):",
        minified_code.len(),
        (1.0 - minified_code.len() as f64 / source_text.len() as f64) * 100.0
    );
    println!("{}", "─".repeat(50));
    println!("{minified_code}");
    println!("{}", "─".repeat(50));

    // Display source map link if available
    if let Some(source_map) = result.map {
        let result_json = source_map.to_json_string();
        let hash = BASE64_STANDARD.encode(format!(
            "{}\0{}{}\0{}",
            minified_code.len(),
            minified_code,
            result_json.len(),
            result_json
        ));
        println!();
        println!("Source map visualization:");
        println!("https://evanw.github.io/source-map-visualization/#{hash}");
    }

    // Test idempotency if requested
    if twice {
        println!();
        println!("Testing idempotency (second pass):");
        allocator.reset();
        let second_result = minify(&allocator, &minified_code, source_type, None, mangle, nospace);
        let second_minified = second_result.code;

        println!("Second pass ({} bytes):", second_minified.len());
        println!("{second_minified}");
        println!("Results identical: {}", minified_code == second_minified);
    }

    Ok(())
}

/// Perform minification on the given source code
fn minify(
    allocator: &Allocator,
    source_text: &str,
    source_type: SourceType,
    source_map_path: Option<PathBuf>,
    mangle: bool,
    nospace: bool,
) -> CodegenReturn {
    // Parse the source code
    let ret = Parser::new(allocator, source_text, source_type).parse();
    let mut program = ret.program;

    // Configure minifier options
    let options = MinifierOptions {
        mangle: mangle.then(MangleOptions::default),
        compress: Some(CompressOptions::smallest()),
    };

    // Apply minification transformations
    let ret = Minifier::new(options).build(allocator, &mut program);

    // Generate the final code with appropriate options
    Codegen::new()
        .with_options(CodegenOptions {
            source_map_path,
            minify: nospace,
            comments: CommentOptions::disabled(),
            ..CodegenOptions::default()
        })
        .with_scoping(ret.scoping)
        .build(&program)
}
