//! # TypeScript JSX Parsing Example
//!
//! This example demonstrates parsing TypeScript files with JSX syntax.
//! It shows how to parse a simple React component and validate the parsing results.
//!
//! ## Usage
//!
//! ```bash
//! cargo run -p oxc_parser --example parser_tsx
//! ```

use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

/// Parse a TypeScript JSX file and validate the results
fn main() -> Result<(), String> {
    let source_text = r"
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

    // Memory arena where AST nodes get stored
    let allocator = Allocator::default();
    // Infers TypeScript + JSX + ESM modules
    let source_type = SourceType::from_path("Counter.tsx").unwrap();

    let ParserReturn {
        program,  // AST
        errors,   // Syntax errors
        panicked, // Parser encountered an error it couldn't recover from
        ..
    } = Parser::new(&allocator, source_text, source_type).parse();

    if panicked {
        return Err("Parser panicked".to_string());
    }

    if !errors.is_empty() {
        return Err(format!("Parsing errors: {}", errors.len()));
    }

    assert!(!program.body.is_empty());
    assert_eq!(program.comments.len(), 1);

    Ok(())
}
