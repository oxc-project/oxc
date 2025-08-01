# Oxc Isolated Declarations

TypeScript isolated declarations transformer for generating `.d.ts` files.

## Overview

This crate implements TypeScript's isolated declarations feature, which generates TypeScript declaration files (`.d.ts`) from source code without requiring full type checking. This enables faster builds and better incremental compilation.

## Key Features

- **Fast declaration generation**: Generate `.d.ts` files without full type checking
- **TypeScript 5.5+ compatibility**: Implements the latest isolated declarations specification
- **Incremental builds**: Enables faster TypeScript compilation workflows
- **Comprehensive support**: Handles classes, functions, interfaces, and complex types

## Usage

```rust
use oxc_allocator::Allocator;
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;

let allocator = Allocator::default();
let source_text = r#"
export interface User {
    name: string;
    age: number;
}

export function createUser(name: string, age: number): User {
    return { name, age };
}
"#;

let source_type = SourceType::from_path("example.ts").unwrap();
let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();

let options = IsolatedDeclarationsOptions::default();
let declarations = IsolatedDeclarations::new(&allocator, options);

match declarations.build(&program) {
    Ok(declaration_program) => {
        // Generate .d.ts content from declaration_program
        println!("Generated declarations successfully");
    }
    Err(errors) => {
        for error in errors {
            eprintln!("Declaration error: {}", error);
        }
    }
}
```

## Architecture

### Isolated Declarations Concept
Isolated declarations allow generating TypeScript declaration files without full type inference by requiring that:
- All exported functions have explicit return types
- All exported variables have explicit types
- Type information is locally available

### Implementation Details
- **AST transformation**: Convert implementation AST to declaration AST
- **Type extraction**: Extract and preserve type information
- **Export analysis**: Determine what needs to be included in declarations
- **Error reporting**: Provide helpful diagnostics for missing type annotations

### Benefits
- **Faster builds**: No full type checking required
- **Incremental compilation**: Each file can be processed independently
- **Parallel processing**: Multiple files can be processed simultaneously
- **Simplified tooling**: Easier to integrate into build systems

This implementation follows the TypeScript compiler's approach while leveraging oxc's performance-oriented architecture.