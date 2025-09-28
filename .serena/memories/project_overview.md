# Oxc Project Overview

## What is Oxc?

The Oxidation Compiler (Oxc) is a collection of high-performance JavaScript and TypeScript tools written in Rust. It's a project of VoidZero Inc. and aims to be 2-100x faster than existing JavaScript tools while maintaining 100% compatibility with JavaScript and TypeScript standards.

## Project Goals

- **Performance**: 2-100x faster than existing JavaScript tools
- **Reliability**: 100% compatibility with JavaScript and TypeScript standards
- **Modularity**: Use individual tools or compose them into complete toolchains
- **Developer Experience**: Clear error messages and seamless editor integration

## Core Tools

- **Parser**: JS/TS parser with AST (fastest Rust-based production parser)
- **Linter (oxlint)**: 50-100x faster than ESLint, convention over configuration
- **Formatter**: Prettier-like code formatting (prototype in progress)
- **Transformer**: Babel-like code transformation (TypeScript, React, ES6 transforms complete)
- **Minifier**: JavaScript minification (prototype in progress)
- **Resolver**: Module resolution for bundling and multi-file analysis
- **Isolated Declarations**: TypeScript declaration emit without TypeScript compiler

## Architecture

- Written in Rust for performance
- Uses its own AST and parser
- Built around visitor pattern for traversal
- Uses oxc_allocator for memory management
- Emphasizes performance-critical design avoiding unnecessary allocations

## Who Uses Oxc?

- Rolldown (parsing and transformation)
- Nova engine (parsing)
- Rolldown, swc-node, knip (oxc_resolver for module resolution)
- Companies: Preact, Shopify, ByteDance, Shopee (oxlint for linting)
- Many other projects listed at https://oxc.rs/docs/guide/projects.html

## Project Status

- AST and parser: Production ready
- Resolver: Production ready
- Linter: Ready with 93 default rules (430+ total)
- Transformer: TypeScript, React, ES6 transforms complete
- Formatter: Prototype/work in progress
- Minifier: Prototype in progress

## Key Performance Claims

- Parser: 3x faster than swc, 5x faster than Biome
- Linter: 50-100x faster than ESLint
- Isolated Declarations: 20x faster than TypeScript compiler
- CI runs complete in ~3 minutes despite optimized Rust compilation
