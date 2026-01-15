#![expect(clippy::print_stdout)]
//! # Semantic Analysis Example
//!
//! This example demonstrates how to use Oxc's semantic analyzer to perform symbol analysis
//! and scope resolution on JavaScript and TypeScript code.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_semantic --example semantic [filename] [--symbols] [--symbol-references]
//! ```
//!
//! ## Options
//!
//! - `--symbols`: Display symbol table and reference information
//! - `--symbol-references`: Display detailed reference information for each symbol

use std::{env, path::Path, sync::Arc};

use itertools::Itertools;

use oxc_allocator::Allocator;
use oxc_diagnostics::{GraphicalReportHandler, OxcDiagnostic};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_semantic --example semantic`
// or `just watch "run -p oxc_semantic --example semantic"`

/// Perform semantic analysis on a JavaScript or TypeScript file
fn main() -> std::io::Result<()> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let show_symbols = env::args().skip(1).any(|arg| arg == "--symbols");
    let show_symbol_references = env::args().skip(1).any(|arg| arg == "--symbol-references");
    let path = Path::new(&name);
    let source_text = Arc::new(std::fs::read_to_string(path)?);
    let source_type = SourceType::from_path(path).unwrap();
    // Memory arena where Semantic and Parser allocate objects
    let allocator = Allocator::default();

    // Parse the source text into an AST
    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
    if !parser_ret.errors.is_empty() {
        let error_message: String = parser_ret
            .errors
            .into_iter()
            .map(|error| format!("{:?}", error.with_source_code(Arc::clone(&source_text))))
            .join("\n");
        println!("Parsing failed:\n\n{error_message}",);
        return Ok(());
    }

    let program = parser_ret.program;

    // Build semantic model with syntax error checking
    let semantic = SemanticBuilder::new()
        // Enable additional syntax checks not performed by the parser
        .with_check_syntax_error(true)
        .build(&program);

    // Report semantic analysis errors
    if !semantic.errors.is_empty() {
        let error_message: String = semantic
            .errors
            .into_iter()
            .map(|error| format!("{:?}", error.with_source_code(Arc::clone(&source_text))))
            .join("\n");
        println!("Semantic analysis failed:\n\n{error_message}",);
    }

    // Display symbol information if requested
    if show_symbols {
        let scoping = semantic.semantic.scoping();
        for symbol_id in scoping.symbol_ids() {
            let name = scoping.symbol_name(symbol_id);
            let flags = scoping.symbol_flags(symbol_id);
            println!("Symbol: {name}, ID: {symbol_id:?}, Flags: {flags:?}");
            for reference_id in scoping.get_resolved_reference_ids(symbol_id) {
                let reference = scoping.get_reference(*reference_id);
                println!("  Reference ID: {reference_id:?}, Flags: {:?}", reference.flags());
            }
        }
    }

    if show_symbol_references {
        let reporter = GraphicalReportHandler::new();

        for sym in semantic.semantic.scoping().symbol_ids() {
            let symbol_name = semantic.semantic.scoping().symbol_name(sym);
            let declaration_node_id = semantic.semantic.scoping().symbol_declaration(sym);
            let declaration_span =
                semantic.semantic.nodes().get_node(declaration_node_id).kind().span();

            let reference_spans = semantic.semantic.symbol_references(sym).map(|reference| {
                (
                    semantic.semantic.nodes().get_node(reference.node_id()).kind().span(),
                    reference.flags(),
                )
            });

            let has_zero_references = semantic.semantic.symbol_references(sym).next().is_none();

            let mut info = OxcDiagnostic::warn(format!("References for symbol `{symbol_name}`"))
                .with_label(declaration_span.primary_label("declared here"))
                .and_labels(
                    reference_spans
                        .map(|(span, flags)| span.label(format!("referenced here: ({flags:?})"))),
                );

            if has_zero_references {
                info = info.with_note("This symbol has no references.");
            }

            let info = info.with_source_code(Arc::clone(&source_text));

            let mut s = String::new();
            reporter.render_report(&mut s, info.as_ref()).unwrap();
            println!("{s}");
        }
    }

    Ok(())
}
