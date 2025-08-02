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
//! ```

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_formatter::{FormatOptions, Formatter};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

/// Format a JavaScript or TypeScript file
fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();
    let allocator = Allocator::new();

    // Parse the source code
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions {
            preserve_parens: false,
            allow_v8_intrinsics: true,
            ..ParseOptions::default()
        })
        .parse();

    // Report any parsing errors
    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
        println!("Parsed with Errors.");
    }

    // Format the parsed code
    let options = FormatOptions::default();
    let code = Formatter::new(&allocator, options).build(&ret.program);

    println!("{code}");

    Ok(())
}
