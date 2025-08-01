# Oxc Macros

Procedural macros for declaring lint rules and other oxc components.

## Overview

This crate provides procedural macros that simplify the declaration and implementation of lint rules and other oxc components. These macros reduce boilerplate code and ensure consistent patterns across the codebase.

## Key Features

- **`declare_oxc_lint!`**: Macro for declaring lint rules with metadata
- **Rule documentation**: Auto-generates documentation for website
- **Category management**: Organize rules into logical categories
- **Boilerplate reduction**: Eliminates repetitive rule declaration code

## Usage

### Declaring a Lint Rule

```rust
use oxc_macros::declare_oxc_lint;

declare_oxc_lint!(
    /// # No Unused Variables
    /// 
    /// Disallow unused variables to keep code clean and prevent potential bugs.
    ///
    /// ## Examples
    ///
    /// Bad:
    /// ```javascript
    /// let unusedVar = 42;
    /// ```
    ///
    /// Good:
    /// ```javascript  
    /// let usedVar = 42;
    /// console.log(usedVar);
    /// ```
    NoUnusedVars,
    correctness,  // Rule category
    pending       // Implementation status
);

impl Rule for NoUnusedVars {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Rule implementation
    }
}
```

### Rule Categories

Rules are organized into categories:
- **correctness**: Catch bugs and incorrect code
- **suspicious**: Flag potentially problematic patterns  
- **pedantic**: Enforce best practices and style
- **performance**: Identify performance issues
- **restriction**: Prevent certain language features

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