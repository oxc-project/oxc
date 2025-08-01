#![expect(clippy::print_stdout)]
//! # Semantic Analysis Example
//!
//! This example demonstrates how to perform semantic analysis on JavaScript and TypeScript code
//! using the Oxc semantic analyzer. It shows how to build symbol tables, resolve references,
//! and check for semantic errors.
//!
//! ## Features
//!
//! - Parse and analyze JavaScript/TypeScript code
//! - Build symbol tables and scope information
//! - Resolve variable references
//! - Display symbols and their references
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js` or `test.ts`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_semantic --example semantic [filename] [--symbols]
//!    ```
//!    Or with just:
//!    ```bash
//!    just watch "run -p oxc_semantic --example semantic"
//!    ```
//!
//! ## Options
//!
//! - `--symbols`: Display detailed symbol information including references

use std::{env, path::Path, sync::Arc};

use itertools::Itertools;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

/// Main entry point for the semantic analysis example
fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let show_symbols = env::args().skip(1).any(|arg| arg == "--symbols");

    // Read and validate the source file
    let path = Path::new(&name);
    let source_text = Arc::new(std::fs::read_to_string(path)?);
    let source_type = SourceType::from_path(path).unwrap();

    println!("Semantic Analysis Example");
    println!("========================");
    println!("File: {name}");
    println!("Source type: {source_type:?}");
    println!("File size: {} bytes", source_text.len());
    println!();

    // Memory arena where Semantic and Parser allocate objects
    let allocator = Allocator::default();

    // Parse the source text into an AST
    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !parser_ret.errors.is_empty() {
        println!("❌ Parsing failed:");
        let error_message = parser_ret
            .errors
            .into_iter()
            .map(|error| format!("{:?}", error.with_source_code(Arc::clone(&source_text))))
            .join("\n");
        println!("{error_message}");
        return Ok(());
    }

    println!("✅ Parsing successful");
    let program = parser_ret.program;

    // Build semantic information
    let semantic = SemanticBuilder::new()
        // Enable additional syntax checks not performed by the parser
        .with_check_syntax_error(true)
        .build(&program);

    // Check for semantic errors
    if !semantic.errors.is_empty() {
        println!("❌ Semantic analysis failed:");
        let error_message = semantic
            .errors
            .into_iter()
            .map(|error| format!("{:?}", error.with_source_code(Arc::clone(&source_text))))
            .join("\n");
        println!("{error_message}");
        return Ok(());
    }

    println!("✅ Semantic analysis successful");
    println!();

    // Display symbol information if requested
    if show_symbols {
        display_symbol_information(&semantic.semantic);
    } else {
        display_summary(&semantic.semantic);
    }

    Ok(())
}

/// Display a summary of the semantic analysis results
fn display_summary(semantic: &oxc_semantic::Semantic) {
    let scoping = semantic.scoping();
    let symbol_count = scoping.symbol_ids().count();
    let reference_count = scoping
        .symbol_ids()
        .map(|symbol_id| scoping.get_resolved_reference_ids(symbol_id).len())
        .sum::<usize>();

    println!("Analysis Summary:");
    println!("- Symbols: {symbol_count}");
    println!("- References: {reference_count}");
    println!();
    println!("💡 Use --symbols flag to see detailed symbol information");
}

/// Display detailed symbol and reference information
fn display_symbol_information(semantic: &oxc_semantic::Semantic) {
    let scoping = semantic.scoping();
    let symbol_ids: Vec<_> = scoping.symbol_ids().collect();

    if symbol_ids.is_empty() {
        println!("No symbols found in the analyzed code.");
        return;
    }

    println!("Symbol Information:");
    println!("{}", "─".repeat(60));

    for symbol_id in symbol_ids {
        let name = scoping.symbol_name(symbol_id);
        let flags = scoping.symbol_flags(symbol_id);
        let reference_ids = scoping.get_resolved_reference_ids(symbol_id);

        println!("Symbol: {name} (ID: {symbol_id:?})");
        println!("  Flags: {flags:?}");
        println!("  References: {}", reference_ids.len());

        for reference_id in reference_ids {
            let reference = scoping.get_reference(*reference_id);
            println!("    Reference ID: {:?}, Flags: {:?}", reference_id, reference.flags());
        }
        println!();
    }
}
