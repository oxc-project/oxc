#![expect(clippy::print_stdout)]
//! # Parser Example
//!
//! This example demonstrates how to use the Oxc parser to parse JavaScript and TypeScript files.
//! It showcases basic parsing functionality along with options for displaying the AST,
//! ESTree representation, and comments.
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js` or `test.ts`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_parser --example parser [filename]
//!    ```
//!    Or with just:
//!    ```bash
//!    just watch "cargo run -p oxc_parser --example parser"
//!    ```
//!
//! ## Options
//!
//! - `--ast`: Display the parsed AST structure
//! - `--estree`: Display the ESTree/TS-ESTree JSON representation
//! - `--comments`: Display extracted comments

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

/// Main entry point for the parser example
fn main() -> Result<(), String> {
    // Parse command line arguments
    let mut args = Arguments::from_env();
    let show_ast = args.contains("--ast");
    let show_estree = args.contains("--estree");
    let show_comments = args.contains("--comments");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read and parse the source file
    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();

    // Create allocator for AST nodes
    let allocator = Allocator::default();

    // Configure parser options
    let parse_options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };

    // Parse the source code
    let ret =
        Parser::new(&allocator, &source_text, source_type).with_options(parse_options).parse();
    let mut program = ret.program;

    // Display comments if requested
    if show_comments {
        display_comments(&program, &source_text);
    }

    // Display AST if requested
    if show_ast {
        display_ast(&program);
    }

    // Display ESTree representation if requested
    if show_estree {
        display_estree(&mut program, &source_text, source_type);
    }

    // Report parsing results
    report_parsing_result(&ret.errors, &source_text);

    Ok(())
}

/// Display extracted comments from the parsed program
fn display_comments(program: &oxc_ast::ast::Program, source_text: &str) {
    println!("Comments:");
    for comment in &program.comments {
        let content = comment.content_span().source_text(source_text);
        println!("{content}");
    }
}

/// Display the AST structure
fn display_ast(program: &oxc_ast::ast::Program) {
    println!("AST:");
    println!("{program:#?}");
}

/// Display ESTree or TS-ESTree representation
fn display_estree(program: &mut oxc_ast::ast::Program, source_text: &str, source_type: SourceType) {
    // Convert UTF-8 positions to UTF-16 for ESTree compatibility
    Utf8ToUtf16::new(source_text).convert_program(program);

    if source_type.is_javascript() {
        println!("ESTree AST:");
        println!("{}", program.to_pretty_estree_js_json(false));
    } else {
        println!("TS-ESTree AST:");
        println!("{}", program.to_pretty_estree_ts_json(false));
    }
}

/// Report the results of parsing (success or errors)
fn report_parsing_result(errors: &[oxc_diagnostics::OxcDiagnostic], source_text: &str) {
    if errors.is_empty() {
        println!("Parsed Successfully.");
    } else {
        println!("Parsed with Errors:");
        for error in errors {
            let error = error.clone().with_source_code(source_text.to_owned());
            println!("{error:?}");
        }
    }
}
