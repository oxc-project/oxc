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
