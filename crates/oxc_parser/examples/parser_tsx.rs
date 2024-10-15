use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

fn main() {
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

    assert!(!panicked);
    assert!(errors.is_empty());
    assert!(!program.body.is_empty());
    assert_eq!(program.comments.len(), 1);
}
