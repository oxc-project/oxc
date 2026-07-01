#![expect(clippy::print_stdout)]
//! # Oxc Type Checker Example
//!
//! Runs the experimental [`oxc_type_checker`] type checker over a file, wiring up the parser
//! and semantic analyzer along the way.
//!
//! ## Usage
//!
//! Create a `test.ts` file and run:
//!
//! ```bash
//! cargo run -p oxc_type_checker --example checker [filename]
//! ```
//!
//! Since the checker is a scaffold that performs no checks yet, a well-formed file simply
//! reports "No type errors found". Add checks in `crates/oxc_type_checker/src/lib.rs` and watch
//! them show up here.

use std::{env, path::Path, sync::Arc};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_type_checker::TypeChecker;

fn main() -> std::io::Result<()> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.ts".to_string());
    let path = Path::new(&name);
    let source_text: Arc<str> = Arc::from(std::fs::read_to_string(path)?);
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());

    // Memory arena in which the parser and semantic model allocate.
    let allocator = Allocator::default();

    // 1. Parse the source text into an AST.
    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
    if !parser_ret.diagnostics.is_empty() {
        println!("Parsing failed:\n");
        print_diagnostics(parser_ret.diagnostics, &source_text);
        return Ok(());
    }
    let program = parser_ret.program;

    // 2. Run semantic analysis to build the symbol table and scope tree.
    let semantic_ret = SemanticBuilder::new().build(&program);
    if !semantic_ret.diagnostics.is_empty() {
        println!("Semantic analysis reported problems:\n");
        print_diagnostics(semantic_ret.diagnostics, &source_text);
    }

    // 3. Type check.
    let checker_ret = TypeChecker::new().check(&program, &semantic_ret.semantic);
    if checker_ret.diagnostics.is_empty() {
        println!("No type errors found in {name}.");
    } else {
        print_diagnostics(checker_ret.diagnostics, &source_text);
    }

    Ok(())
}

fn print_diagnostics(diagnostics: oxc_diagnostics::Diagnostics, source_text: &Arc<str>) {
    for diagnostic in diagnostics.into_vec() {
        let report = diagnostic.with_source_code(Arc::clone(source_text));
        println!("{report:?}");
    }
}
