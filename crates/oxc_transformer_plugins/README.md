# Oxc Transformer Plugins

Specialized transformation plugins for common JavaScript and TypeScript patterns.

## Overview

This crate provides specific transformation plugins that can be used independently or as part of the broader oxc transformation pipeline. These plugins handle common transformation needs like global variable injection and module format conversion.

## Key Features

- **Global variable injection**: Add global variables to modules
- **Module runner transforms**: Transform modules for different execution environments
- **Global defines replacement**: Replace compile-time constants
- **Composable plugins**: Use individually or combine multiple plugins

## Architecture

### Plugin System

Each plugin implements the transformation logic for a specific use case:

- **Focused responsibility**: Each plugin handles one type of transformation
- **Traverse integration**: Uses oxc's traversal system for efficient AST walking
- **Composable**: Multiple plugins can be applied in sequence
- **Context-aware**: Access to semantic information when needed

### Common Use Cases

#### Build-time Optimizations

- **Dead code elimination**: Remove unreachable code paths
- **Constant folding**: Replace compile-time constants
- **Environment variables**: Inject environment-specific values

#### Module System Compatibility

- **Polyfill injection**: Add necessary polyfills for target environments
- **Global shimming**: Provide Node.js globals in browser environments
- **Format conversion**: Transform between module formats

#### Development Tools

- **Debug injection**: Add development-time debugging code
- **Hot module replacement**: Support for HMR systems
- **Testing utilities**: Inject test-specific globals

### Integration Points

These plugins integrate with:

- **oxc_transformer**: Core transformation infrastructure
- **oxc_traverse**: AST traversal and mutation
- **Build tools**: Webpack, Vite, Rollup, and other bundlers
- **CLI tools**: Direct usage in command-line transformations

The plugin architecture enables extensible, composable transformations while maintaining high performance through oxc's efficient traversal system.
