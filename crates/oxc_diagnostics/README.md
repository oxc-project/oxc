# Oxc Diagnostics

Error reporting and diagnostic utilities for JavaScript and TypeScript tooling.

## Overview

This crate provides comprehensive error handling and diagnostic reporting capabilities. It implements the [miette] diagnostic trait, making it compatible with other Rust diagnostic tooling while providing specialized features for JavaScript/TypeScript errors.

## Key Features

- **Rich diagnostics**: Detailed error messages with source context
- **Source highlighting**: Syntax-highlighted error locations
- **Multiple error support**: Collect and report multiple errors at once
- **Miette integration**: Compatible with the miette diagnostic ecosystem
- **Severity levels**: Support for errors, warnings, and informational messages

## Usage

### Basic Error Creation

```rust
use oxc_diagnostics::{OxcDiagnostic, Result};

fn parse_something() -> Result<()> {
    // Create an error diagnostic
    Err(OxcDiagnostic::error("Unexpected token")
        .with_label("here")
        .with_help("Try removing this token"))
}
```

### Diagnostic Service

```rust
use oxc_diagnostics::{DiagnosticService, Error};

let mut service = DiagnosticService::default();

// Process errors from various sources
let errors: Vec<Error> = vec![/* ... */];
for error in errors {
    service.receive(error);
}

// Format and display all collected diagnostics
let formatted = service.format_diagnostics();
println!("{}", formatted);
```

### With Source Context

```rust
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

let diagnostic = OxcDiagnostic::error("Syntax error")
    .with_source_code(source_text)
    .with_labels([span.label("unexpected token here")]);
```

## Architecture

### Diagnostic Components

- **Message**: Primary error/warning description
- **Labels**: Highlight specific source locations
- **Help text**: Suggestions for fixing the issue
- **Source context**: Display relevant source code sections

### Error Flow

1. **Creation**: Tools create `OxcDiagnostic` instances for problems
2. **Collection**: `DiagnosticService` aggregates diagnostics
3. **Formatting**: Rich terminal output with colors and context
4. **Reporting**: Display to users in IDE, CLI, or other interfaces

The diagnostic system ensures consistent, high-quality error reporting across all oxc tools.
