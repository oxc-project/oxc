//! # TypeScript JSX Parser Example
//!
//! This example demonstrates parsing TypeScript JSX (TSX) code using the Oxc parser.
//! It shows how to handle React components with TypeScript type annotations.
//!
//! ## Usage
//!
//! Run the example:
//! ```bash
//! cargo run -p oxc_parser --example parser_tsx
//! ```

use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

/// Example TypeScript JSX code representing a React counter component
const SAMPLE_TSX_CODE: &str = r"
import React from 'react';

/**
 * A simple counter component
 */
export const Counter: React.FC = () => {
    const [count, setCount] = React.useState(0);

    return (
        <div>
            <p>Count: {count}</p>
            <button onClick={() => setCount(count + 1)}>Increment</button>
            <button onClick={() => setCount(count - 1)}>Decrement</button>
        </div>
    )
}";

fn main() {
    println!("Parsing TypeScript JSX Example");
    println!("==============================\n");

    // Memory arena where AST nodes get stored
    let allocator = Allocator::default();

    // Infers TypeScript + JSX + ESM modules from the filename
    let source_type = SourceType::from_path("Counter.tsx").unwrap();

    println!("Source type: {:?}", source_type);
    println!("Sample code:\n{}\n", SAMPLE_TSX_CODE);

    // Parse the TypeScript JSX code
    let ParserReturn {
        program,  // AST
        errors,   // Syntax errors
        panicked, // Parser encountered an error it couldn't recover from
        ..
    } = Parser::new(&allocator, SAMPLE_TSX_CODE, source_type).parse();

    // Validate parsing results
    validate_parsing_results(panicked, &errors, &program);

    // Display parsing statistics
    display_parsing_statistics(&program);

    println!("✅ TypeScript JSX parsing completed successfully!");
}

/// Validate that parsing completed without critical errors
fn validate_parsing_results(
    panicked: bool,
    errors: &[oxc_diagnostics::OxcDiagnostic],
    program: &oxc_ast::ast::Program,
) {
    assert!(!panicked, "Parser should not panic on valid TSX code");
    assert!(errors.is_empty(), "No syntax errors should be present in the sample code");
    assert!(!program.body.is_empty(), "Program body should contain statements");
}

/// Display statistics about the parsed program
fn display_parsing_statistics(program: &oxc_ast::ast::Program) {
    println!("Parsing Statistics:");
    println!("- Statements: {}", program.body.len());
    println!("- Comments: {}", program.comments.len());
    println!("- Hashbang: {:?}", program.hashbang);
}
