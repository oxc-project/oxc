use anyhow::{anyhow, Result};
use serde_json::{json, Value};

/// Get the content for a resource URI
pub async fn get_resource_content(uri: &str) -> Result<Value> {
    match uri {
        "oxc://linter/rules" => get_linter_rules().await,
        "oxc://project/info" => get_project_info().await,
        "oxc://ast/schema" => get_ast_schema().await,
        _ => Err(anyhow!("Unknown resource URI: {}", uri)),
    }
}

/// Get information about available linter rules
async fn get_linter_rules() -> Result<Value> {
    // In a real implementation, this would dynamically fetch rules from oxc_linter
    // For now, we'll provide a representative sample
    let rules_info = json!({
        "rules": {
            "no-unused-vars": {
                "description": "Disallow unused variables",
                "category": "Variables",
                "recommended": true,
                "fixable": false
            },
            "no-console": {
                "description": "Disallow use of console",
                "category": "Best Practices",
                "recommended": false,
                "fixable": false
            },
            "prefer-const": {
                "description": "Require const declarations for variables that are never reassigned after declared",
                "category": "ECMAScript 6",
                "recommended": true,
                "fixable": true
            },
            "no-debugger": {
                "description": "Disallow use of debugger",
                "category": "Possible Errors",
                "recommended": true,
                "fixable": false
            },
            "eqeqeq": {
                "description": "Require the use of === and !==",
                "category": "Best Practices",
                "recommended": true,
                "fixable": false
            }
        },
        "categories": [
            "Possible Errors",
            "Best Practices",
            "Variables",
            "ECMAScript 6",
            "Stylistic Issues"
        ],
        "total_rules": 430,
        "recommended_rules": 93
    });

    Ok(json!({
        "uri": "oxc://linter/rules",
        "mimeType": "application/json",
        "text": serde_json::to_string_pretty(&rules_info)?
    }))
}

/// Get information about the Oxc project
async fn get_project_info() -> Result<Value> {
    let project_info = r#"# Oxc Project Information

## Overview
Oxc (The Oxidation Compiler) is a collection of high-performance tools for JavaScript and TypeScript written in Rust.

## Core Components

### Parser
- **oxc_parser**: High-performance JavaScript/TypeScript parser
- **oxc_ast**: Abstract Syntax Tree definitions and utilities
- Fastest Rust-based parser for JavaScript/TypeScript
- Full support for modern JavaScript and TypeScript syntax

### Linter (oxlint)
- **oxc_linter**: Fast ESLint-compatible linter
- 50-100x faster than ESLint
- 430+ rules available, 93 enabled by default
- Multi-threaded processing

### Other Tools
- **oxc_formatter**: Code formatting (work in progress)
- **oxc_transformer**: Code transformation and compilation
- **oxc_minifier**: Code minification for production
- **oxc_semantic**: Semantic analysis and symbol resolution

## Performance
- Parser: ~3x faster than swc, ~5x faster than Biome
- Linter: 50-100x faster than ESLint
- Memory efficient with arena allocation
- Multi-threaded processing

## Architecture
- Memory arena allocation using bumpalo
- Zero-copy string handling with CompactString
- Multi-threaded parallel processing
- Minimal heap allocations

## Project Structure
- `crates/`: Core Rust crates
- `apps/`: Application binaries (oxlint)
- `napi/`: Node.js bindings
- `tasks/`: Development tools and automation
- `editors/`: Editor integrations

## Testing
- Comprehensive test infrastructure
- Test262 conformance suite
- Babel and TypeScript compatibility tests
- Fuzzing and property-based testing
- Snapshot testing for diagnostics
"#;

    Ok(json!({
        "uri": "oxc://project/info",
        "mimeType": "text/markdown",
        "text": project_info
    }))
}

/// Get AST schema information
async fn get_ast_schema() -> Result<Value> {
    let ast_schema = json!({
        "description": "Oxc Abstract Syntax Tree Schema",
        "differences_from_estree": {
            "specific_types": "Uses specific types like BindingIdentifier, IdentifierReference instead of generic Identifier",
            "memory_layout": "Optimized for arena allocation",
            "performance": "Designed for high-performance parsing and traversal"
        },
        "core_types": {
            "Program": "Root node of the AST",
            "Statement": "Base type for all statements",
            "Expression": "Base type for all expressions",
            "Declaration": "Base type for all declarations",
            "Pattern": "Base type for destructuring patterns",
            "Identifier": {
                "BindingIdentifier": "Identifier in binding position",
                "IdentifierReference": "Identifier reference",
                "IdentifierName": "Property name identifier"
            }
        },
        "typescript_support": {
            "type_annotations": "Full TypeScript type annotation support",
            "decorators": "Decorator syntax support", 
            "namespaces": "TypeScript namespace support",
            "ambient_declarations": "Ambient declaration support"
        },
        "jsx_support": {
            "jsx_elements": "JSX element parsing",
            "jsx_fragments": "JSX fragment support",
            "jsx_expressions": "JSX expression containers"
        }
    });

    Ok(json!({
        "uri": "oxc://ast/schema",
        "mimeType": "application/json",
        "text": serde_json::to_string_pretty(&ast_schema)?
    }))
}