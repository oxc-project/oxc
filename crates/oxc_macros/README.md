# Oxc Macros

Procedural macros for declaring lint rules and other oxc components.

## Overview

This crate provides procedural macros that simplify the declaration and implementation of lint rules and other oxc components. These macros reduce boilerplate code and ensure consistent patterns across the codebase.

## Key Features

- **`declare_oxc_lint!`**: Macro for declaring lint rules with metadata
- **Rule documentation**: Auto-generates documentation for website
- **Category management**: Organize rules into logical categories
- **Boilerplate reduction**: Eliminates repetitive rule declaration code

## Architecture

### Macro System

The macros generate:

- Rule struct definitions
- Metadata for documentation generation
- Registration code for the rule system
- Consistent interfaces across all rules

### Documentation Generation

Rule documentation is automatically extracted and used to build the oxc website documentation, ensuring that rule descriptions stay in sync with implementation.

### Benefits

- **Consistency**: All rules follow the same declaration pattern
- **Documentation**: Automatic documentation generation
- **Type safety**: Compile-time verification of rule metadata
- **Maintainability**: Centralized rule management

This macro system enables rapid development of new lint rules while maintaining high quality and consistency.
