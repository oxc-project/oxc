#![expect(clippy::print_stdout)]
//! # Formatter Example
//!
//! This example demonstrates how to use the Oxc formatter to format JavaScript and TypeScript code.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_formatter --example formatter [filename]
//! cargo run -p oxc_formatter --example formatter -- --no-semi [filename]
//! ```

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_formatter::{BracketSameLine, FormatOptions, Formatter, Semicolons};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

/// Format a JavaScript or TypeScript file
fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();
    let no_semi = args.contains("--no-semi");
    let show_ir = args.contains("--ir");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();
    let allocator = Allocator::new();

    // Parse the source code
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions {
            parse_regular_expression: false,
            // Enable all syntax features
            allow_v8_intrinsics: true,
            allow_return_outside_function: true,
            // `oxc_formatter` expects this to be false
            preserve_parens: false,
        })
        .parse();

    // Report any parsing errors
    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
        println!("Parsed with Errors.");
    }

    // Format the parsed code
    let semicolons = if no_semi { Semicolons::AsNeeded } else { Semicolons::Always };
    let options = FormatOptions {
        bracket_same_line: BracketSameLine::from(true),
        semicolons,
        ..Default::default()
    };

    let formatter = Formatter::new(&allocator, options);
    let formatted = formatter.format(&ret.program);
    if show_ir {
        println!("--- IR ---");
        println!("{}", &formatted.document().to_string());
        println!("--- End IR ---\n");
    }

    println!("--- Formatted Code ---");
    let code = formatted.print().map_err(|e| e.to_string())?.into_code();
    println!("{code}");
    println!("--- End Formatted Code ---");
    Ok(())
}
