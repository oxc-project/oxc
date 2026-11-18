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

#![expect(clippy::print_stdout)]

use std::path::Path;

use pico_args::Arguments;

use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_parser::{ParseOptions, Source, SourceFileError};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example parser`
// or `just watch "cargo run -p oxc_parser --example parser"`

/// Parse and display information about a JavaScript or TypeScript file
fn main() -> Result<(), SourceFileError> {
    // Parse command line arguments
    let mut args = Arguments::from_env();
    let show_ast = args.contains("--ast");
    let show_estree = args.contains("--estree");
    let show_comments = args.contains("--comments");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read source file
    let path = Path::new(&name);
    let allocator = Allocator::default();
    let source = Source::read_from_file(path, &allocator)?;

    // Parse the source code
    let options = ParseOptions {
        parse_regular_expression: true,
        enable_ident_hashes: false,
        ..ParseOptions::default()
    };
    let parser_ret = source.parse_with_options(options);

    let mut program = parser_ret.program;
    let source_text = program.source_text;

    // Display comments if requested
    if show_comments {
        println!("Comments:");
        for comment in &program.comments {
            let s = comment.content_span().source_text(source_text);
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
        Utf8ToUtf16::new(source_text).convert_program(&mut program);
        if program.source_type.is_javascript() {
            println!("ESTree AST:");
        } else {
            println!("TS-ESTree AST:");
        }
        println!("{}", program.to_pretty_estree_json(!program.source_type.is_javascript(), false));
    }

    // Report parsing results
    if parser_ret.diagnostics.is_empty() {
        println!("Parsed Successfully.");
    } else {
        for error in parser_ret.diagnostics {
            println!("{}", error.render_with_source_code(source_text));
        }
        println!("Parsed with Errors.");
    }

    Ok(())
}
