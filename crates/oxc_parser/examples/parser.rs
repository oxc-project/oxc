#![expect(clippy::print_stdout)]
//! # Parser Example
//!
//! This example demonstrates how to use the Oxc parser to parse JavaScript and TypeScript files.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_parser --example parser [filename] [--ast] [--estree] [--comments]
//! ```
//!
//! ## Options
//!
//! - `--ast`: Display the parsed AST structure
//! - `--estree`: Display the ESTree representation
//! - `--comments`: Display extracted comments

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example parser`
// or `just watch "cargo run -p oxc_parser --example parser"`

/// Parse and display information about a JavaScript or TypeScript file
fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();

    // Parse command line arguments
    let show_ast = args.contains("--ast");
    let show_estree = args.contains("--estree");
    let show_comments = args.contains("--comments");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();

    // Parse the source code
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions { parse_regular_expression: true, ..ParseOptions::default() })
        .parse();
    let mut program = ret.program;

    // Display comments if requested
    if show_comments {
        println!("Comments:");
        for comment in &program.comments {
            let s = comment.content_span().source_text(&source_text);
            println!("{s}");
        }
    }

    // Display AST if requested
    if show_ast {
        println!("AST:");
        println!("{program:#?}");
    }

    // Display ESTree representation if requested
    if show_estree {
        Utf8ToUtf16::new(&source_text).convert_program(&mut program);
        if source_type.is_javascript() {
            println!("ESTree AST:");
            println!("{}", program.to_pretty_estree_js_json(false, false));
        } else {
            println!("TS-ESTree AST:");
            println!("{}", program.to_pretty_estree_ts_json(false, false));
        }
    }

    // Report parsing results
    if ret.errors.is_empty() {
        println!("Parsed Successfully.");
    } else {
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
        println!("Parsed with Errors.");
    }

    Ok(())
}
