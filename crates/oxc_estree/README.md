# Oxc ESTree

ESTree compatibility layer for serialization and interoperability.

## Overview

This crate provides compatibility with the ESTree AST specification, primarily for serialization purposes. It enables oxc AST nodes to be serialized to and from JSON in ESTree format, facilitating interoperability with other JavaScript tools.

## Key Features

- **ESTree compatibility**: Convert oxc AST to/from standard ESTree format
- **Serialization support**: JSON serialization/deserialization via serde
- **Tool interoperability**: Enable integration with ESTree-based tools
- **Optional feature**: Only enabled when `serialize` feature is active

## Usage

## Architecture

### ESTree Specification

ESTree is a community standard for representing JavaScript AST nodes. This crate ensures oxc's AST can be represented in this standard format while maintaining compatibility with the broader JavaScript tooling ecosystem.

### Design Principles

- **Compatibility**: Full compatibility with ESTree specification
- **Optional overhead**: Only included when serialization is needed
- **Type safety**: Maintains Rust's type safety during conversion
- **Performance**: Efficient serialization with minimal overhead

### Use Cases

- **IDE integration**: Language servers communicating via JSON
- **Tool interoperability**: Working with Babel, ESLint, and other ESTree tools
- **Data exchange**: Transferring AST data between different systems
- **Debugging**: Human-readable AST representation

When the `serialize` feature is disabled, this crate provides only a placeholder trait to support derive macros without overhead.
