# Oxc Linter

High-performance ESLint-compatible linter for JavaScript and TypeScript.

## Overview

This crate provides a fast, ESLint-compatible linting engine with comprehensive rule support. It leverages oxc's high-performance AST and semantic analysis to deliver significantly faster linting than traditional tools.

## Key Features

- **ESLint rules**: Native implementations of rules from ESLint and popular plugins
- **High performance**: Shared AST and semantic analysis across rules
- **Comprehensive rules**: Supports most popular ESLint rules and plugins
- **TypeScript support**: Syntax-aware rules; the Oxlint CLI integrates with `oxlint-tsgolint` for type-aware rules
- **Plugin system**: Extensible architecture for custom rules

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

- **Configuration format**: Uses Oxlint configuration; migrate ESLint configuration with the [migration guide](https://oxc.rs/docs/guide/usage/linter/migrate-from-eslint.html)
- **Rule parity**: Implements behavior-compatible versions of popular rules
- **Plugin ecosystem**: The Oxlint CLI supports JavaScript plugins with an ESLint-compatible rule API
- **Migration path**: Easy transition from ESLint to oxlint

The linter is designed to be both a standalone tool and a library component for integration into other development tools.
