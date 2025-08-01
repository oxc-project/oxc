# Oxc Linter

High-performance ESLint-compatible linter for JavaScript and TypeScript.

## Overview

This crate provides a fast, ESLint-compatible linting engine with comprehensive rule support. It leverages oxc's high-performance AST and semantic analysis to deliver significantly faster linting than traditional tools.

## Key Features

- **ESLint compatibility**: Drop-in replacement for most ESLint workflows
- **High performance**: 50-100x faster than ESLint on large codebases
- **Comprehensive rules**: Supports most popular ESLint rules and plugins
- **TypeScript support**: Full TypeScript linting with type-aware rules
- **Plugin system**: Extensible architecture for custom rules

## Usage

### Basic Linting

```rust
use oxc_allocator::Allocator;
use oxc_linter::{Linter, LintOptions};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

let allocator = Allocator::default();
let source_text = "const x = 1; console.log(x);";
let source_type = SourceType::from_path("example.js").unwrap();

// Parse and build semantic info
let ParserReturn { program, .. } = Parser::new(&allocator, source_text, source_type).parse();
let semantic = SemanticBuilder::new().build(&program);

// Run linter
let options = LintOptions::default();
let linter = Linter::from_options(options);
let diagnostics = linter.run(&program, &semantic);

// Process diagnostics
for diagnostic in diagnostics {
    println!("{}", diagnostic);
}
```

### Configuration

```rust
use oxc_linter::{LintOptions, RuleCategory};

let options = LintOptions {
    categories: RuleCategory::all(),
    fix: true,  // Enable auto-fixing
    ..Default::default()
};
```

## Architecture

### Rule System
- **Rule categories**: Correctness, Suspicious, Pedantic, Style, etc.
- **Configurable severity**: Error, Warning, or Off for each rule
- **Auto-fixing**: Many rules can automatically fix issues
- **Plugin support**: Load external rule collections

### Performance Design
- **Single AST pass**: Most rules run in a single traversal
- **Efficient analysis**: Leverages oxc's semantic analysis
- **Parallel execution**: Process multiple files concurrently
- **Memory efficiency**: Uses arena allocation for optimal performance

### ESLint Compatibility
- **Configuration format**: Supports ESLint config files
- **Rule parity**: Implements behavior-compatible versions of popular rules
- **Plugin ecosystem**: Compatible with many ESLint plugins
- **Migration path**: Easy transition from ESLint to oxlint

The linter is designed to be both a standalone tool and a library component for integration into other development tools.