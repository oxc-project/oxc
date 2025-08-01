#![expect(clippy::print_stdout)]
//! # Simple Linter Example
//!
//! This example demonstrates how to build a basic linter using the Oxc parser and semantic analyzer.
//! It implements simple rules like detecting `debugger` statements and empty destructuring patterns.
//!
//! ## Rules Implemented
//!
//! - **no-debugger**: Detects and reports `debugger` statements
//! - **no-empty-pattern**: Detects empty array and object destructuring patterns
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_linter --example linter [filename]
//!    ```
//!    Or with cargo watch:
//!    ```bash
//!    cargo watch -x "run -p oxc_linter --example linter"
//!    ```

use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{SourceType, Span};

/// Main entry point for the simple linter example
fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);

    // Read and validate the source file
    let source_text = std::fs::read_to_string(path)?;
    println!("Linting file: {name}");
    println!("File size: {} bytes\n", source_text.len());

    // Set up parser and semantic analyzer
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    // Handle parser errors first
    if !ret.errors.is_empty() {
        println!("Parser errors found:");
        print_errors(&source_text, ret.errors);
        return Ok(());
    }

    // Build semantic information
    let semantic_ret = SemanticBuilder::new().build(&ret.program);

    // Collect linting errors
    let mut errors = Vec::new();
    run_linting_rules(&semantic_ret.semantic, &mut errors);

    // Report results
    if errors.is_empty() {
        println!("✅ No linting errors found!");
    } else {
        println!("❌ Linting errors found:");
        print_errors(&source_text, errors);
    }

    Ok(())
}

/// Run all implemented linting rules on the semantic model
fn run_linting_rules(semantic: &oxc_semantic::Semantic, errors: &mut Vec<OxcDiagnostic>) {
    // Traverse all AST nodes and apply rules
    for node in semantic.nodes() {
        match node.kind() {
            // Rule: no-debugger
            AstKind::DebuggerStatement(stmt) => {
                errors.push(create_no_debugger_error(stmt.span));
            }
            // Rule: no-empty-pattern (arrays)
            AstKind::ArrayPattern(array) if array.elements.is_empty() => {
                errors.push(create_no_empty_pattern_error("array", array.span));
            }
            // Rule: no-empty-pattern (objects)
            AstKind::ObjectPattern(object) if object.properties.is_empty() => {
                errors.push(create_no_empty_pattern_error("object", object.span));
            }
            // No other rules implemented
            _ => {}
        }
    }
}

/// Print diagnostic errors with source code context
fn print_errors(source_text: &str, errors: Vec<OxcDiagnostic>) {
    for error in errors {
        let error = error.with_source_code(source_text.to_string());
        println!("{error:?}");
    }
}

/// Create a diagnostic for debugger statements
///
/// Example output:
/// ```
///   ⚠ `debugger` statement is not allowed
///   ╭────
/// 1 │ debugger;
///   · ─────────
///   ╰────
/// ```
fn create_no_debugger_error(debugger_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("`debugger` statement is not allowed").with_label(debugger_span)
}

/// Create a diagnostic for empty destructuring patterns
///
/// Example output:
/// ```
///   ⚠ empty destructuring pattern is not allowed
///   ╭────
/// 1 │ let {} = {};
///   ·     ─┬
///   ·      ╰── Empty object binding pattern
///   ╰────
/// ```
fn create_no_empty_pattern_error(binding_kind: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::error("empty destructuring pattern is not allowed")
        .with_label(span.label(format!("Empty {binding_kind} binding pattern")))
}
