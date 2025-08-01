# Oxc Semantic

Comprehensive semantic analysis for JavaScript and TypeScript programs.

## Overview

This crate performs semantic analysis on JavaScript and TypeScript ASTs, building symbol tables, scope trees, and control flow graphs. It provides the foundation for advanced static analysis, linting, and transformation tools.

## Key Features

- **Symbol resolution**: Build complete symbol tables with binding information
- **Scope analysis**: Construct scope trees following ECMAScript scoping rules
- **Reference tracking**: Track all variable references and their relationships
- **Control flow**: Optional control flow graph construction
- **JSDoc parsing**: Extract and parse JSDoc comments
- **Module analysis**: Analyze import/export relationships

## Usage

### Basic Semantic Analysis

```rust
use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::{SemanticBuilder, SemanticBuilderReturn};
use oxc_span::SourceType;

let allocator = Allocator::default();
let source_text = r#"
function greet(name: string): string {
    const message = `Hello, ${name}!`;
    return message;
}
"#;

let source_type = SourceType::from_path("example.ts").unwrap();
let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();

let SemanticBuilderReturn { semantic, errors } = SemanticBuilder::new()
    .with_check_syntax_error(true)
    .with_build_jsdoc(true)
    .with_cfg(true)
    .build(&program);

// Access semantic information
let symbols = semantic.symbols();
let scopes = semantic.scopes();
let references = semantic.symbols().references;
```

### Symbol Analysis

```rust
// Iterate through all symbols
for symbol_id in semantic.symbols().symbol_ids() {
    let symbol = semantic.symbols().get_symbol(symbol_id);
    println!("Symbol: {} at {:?}", symbol.name, symbol.span);
    
    // Get all references to this symbol
    for reference in semantic.symbols().get_references(symbol_id) {
        println!("  Referenced at: {:?}", reference.span());
    }
}
```

### Scope Tree Navigation

```rust
// Walk the scope tree
let root_scope = semantic.scopes().root_scope_id();
for scope_id in semantic.scopes().descendants(root_scope) {
    let scope = semantic.scopes().get_scope(scope_id);
    println!("Scope: {:?}", scope.flags());
    
    // Get bindings in this scope
    for (name, symbol_id) in scope.bindings() {
        println!("  Binding: {} -> {:?}", name, symbol_id);
    }
}
```

## Architecture

### Semantic Analysis Pipeline

1. **AST Traversal**: Visit all nodes to collect declarations
2. **Scope Building**: Construct scope tree following language rules
3. **Symbol Resolution**: Create symbol table with binding information
4. **Reference Analysis**: Track all identifier references
5. **Control Flow**: Optionally build control flow graphs
6. **JSDoc Processing**: Parse and attach documentation

### Key Data Structures

#### Symbol Table

- **Symbols**: All declared identifiers (variables, functions, classes, etc.)
- **References**: All uses of identifiers
- **Bindings**: Association between names and symbols in scopes

#### Scope Tree

- **Scope hierarchy**: Nested scopes following language semantics
- **Binding resolution**: How identifiers resolve to declarations
- **Closure analysis**: Capture and usage patterns

#### Control Flow Graph

- **Basic blocks**: Sequences of statements with single entry/exit
- **Flow edges**: Conditional and unconditional control flow
- **Exception handling**: Try/catch/finally flow modeling

### Applications

- **Linting**: Detect unused variables, undefined references, etc.
- **Transformation**: Safe variable renaming and scope analysis
- **Analysis**: Dead code detection, dependency analysis
- **IDE features**: Go-to-definition, find references, refactoring

The semantic analyzer provides the deep program understanding needed for sophisticated JavaScript and TypeScript tooling.
