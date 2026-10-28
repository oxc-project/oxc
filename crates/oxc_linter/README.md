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

## Architecture

`oxc_linter_core` owns the shared rule traits, contexts, settings, module records,
and diagnostic/fix support. It does not depend on concrete rules or the rule
registry. This lets rule crates depend on core without depending on the linter.

`oxc_linter` owns configuration resolution, rule registration and dispatch,
services, and external-linter integration. Its public API re-exports shared types
from core.

Extracted plugins live in `oxc_linter_rules_<plugin>`. The central registry in
`src/rules.rs` re-exports each rule module from its owning crate. `cargo lintgen`
keeps the rule enum in the linter and emits `RuleRunner` implementations into the
crate that owns each rule. Run `cargo lint-timings` after codegen changes.

Rule crates can use `oxc_linter` with its `tester` feature as a
dev-dependency to run the same end-to-end `Tester` against the central registry.
Use `Tester::test_and_snapshot_in` for snapshots owned by a rule crate.

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
