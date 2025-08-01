#![expect(clippy::print_stdout)]
//! # Formatter Example
//!
//! This example demonstrates how to use the Oxc formatter to format JavaScript and TypeScript code.
//! It parses source code and then formats it according to configured options.
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_formatter --example formatter [filename]
//!    ```

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_formatter::{FormatOptions, Formatter};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

/// Main entry point for the formatter example
fn main() -> Result<(), String> {
    // Parse command line arguments
    let mut args = Arguments::from_env();
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read and validate the source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();

    println!("Formatting file: {name}");
    println!("Source type: {source_type:?}");
    println!("Original code:");
    println!("{}", "─".repeat(50));
    println!("{source_text}");
    println!("{}", "─".repeat(50));
    println!();

    // Set up parser with appropriate options
    let allocator = Allocator::new();
    let parse_options = ParseOptions {
        preserve_parens: false,
        allow_v8_intrinsics: true,
        ..ParseOptions::default()
    };

    // Parse the source code
    let ret =
        Parser::new(&allocator, &source_text, source_type).with_options(parse_options).parse();

    // Handle parsing errors
    if !ret.errors.is_empty() {
        println!("Parsing errors found:");
        for error in &ret.errors {
            let error = error.clone().with_source_code(source_text.clone());
            println!("{error:?}");
        }
        println!(); // Add spacing for readability
    }

    // Format the parsed AST
    let format_options = FormatOptions::default();
    let formatted_code = Formatter::new(&allocator, format_options).build(&ret.program);

    // Display the formatted result
    println!("Formatted code:");
    println!("{}", "─".repeat(50));
    println!("{formatted_code}");
    println!("{}", "─".repeat(50));

    Ok(())
}
