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
//! cargo run -p oxc_formatter --example formatter -- --diff [filename]
//! ```

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_formatter::{
    BracketSameLine, FormatOptions, Formatter, LineWidth, Semicolons, get_parse_options,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::print_diff_in_terminal;
use pico_args::Arguments;

/// Format a JavaScript or TypeScript file
fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();
    let no_semi = args.contains("--no-semi");
    let show_ir = args.contains("--ir");
    // Show diff between original and formatted code
    let show_diff = args.contains("--diff");
    let print_width = args.opt_value_from_str::<&'static str, u16>("--print-width").unwrap_or(None);
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();

    // Format the parsed code
    let semicolons = if no_semi { Semicolons::AsNeeded } else { Semicolons::Always };
    let line_width = match print_width {
        Some(width) => LineWidth::try_from(width).unwrap(),
        None => LineWidth::try_from(80).unwrap(),
    };
    let options = FormatOptions {
        bracket_same_line: BracketSameLine::from(true),
        semicolons,
        line_width,
        ..Default::default()
    };

    let allocator = Allocator::new();

    // Parse the source code
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(get_parse_options())
        .parse();

    // Report any parsing errors
    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
        println!("Parsed with Errors.");
    }

    let formatted = Formatter::new(&allocator, options).format(&ret.program);

    if show_ir {
        println!("--- IR ---");
        println!("{}", &formatted.document().to_string());
        println!("--- End IR ---\n");
    }

    let formatted_code = formatted.print().unwrap().into_code();

    if show_diff {
        // First diff: compare formatted output to original input
        if source_text == formatted_code {
            println!("{formatted_code}");
        } else {
            print_diff_in_terminal(&source_text, &formatted_code);
        }
    } else {
        println!("--- Formatted Code ---");
        println!("{formatted_code}");
        println!("--- End Formatted Code ---");
    }

    Ok(())
}
