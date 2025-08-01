# Oxc Mangler

Variable name mangling for JavaScript minification.

## Overview

This crate provides variable name mangling capabilities for JavaScript minification. It shortens variable names to reduce bundle size while preserving program semantics and ensuring gzip-friendly output.

## Key Features

- **Intelligent mangling**: Shortens variable names while avoiding conflicts
- **Scope-aware**: Respects JavaScript scoping rules
- **Gzip optimization**: Generates names that compress well
- **Configurable**: Options to preserve specific names or patterns
- **Base54 encoding**: Efficient character usage for maximum compression

## Architecture

### Mangling Strategy

1. **Symbol Analysis**: Identify all variables and their scopes
2. **Frequency Analysis**: Count usage to optimize for common variables
3. **Name Generation**: Generate short names using base54 encoding
4. **Conflict Resolution**: Ensure no naming conflicts across scopes
5. **Integration**: Update symbol table with new names

### Base54 Encoding

Uses a character set optimized for JavaScript identifiers:

- First character: `a-zA-Z_$` (54 options)
- Subsequent characters: `a-zA-Z0-9_$` (64 options)
- Generates shortest possible names: `a`, `b`, ..., `aa`, `ab`, etc.

### Gzip Optimization

The mangling algorithm considers:

- **Character frequency**: Prefer characters that compress well
- **Repetition patterns**: Generate names that create gzip-friendly patterns
- **Context awareness**: Consider surrounding code when choosing names

### Integration with Minifier

The mangler works as part of the broader minification pipeline:

1. **Parse**: Build AST from source code
2. **Analyze**: Perform semantic analysis
3. **Mangle**: Shorten variable names
4. **Transform**: Apply other minification transforms
5. **Generate**: Output minified code

This approach ensures maximum size reduction while maintaining correctness.
