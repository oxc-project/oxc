#![expect(clippy::print_stdout)]
//! # AST Visitor Example
//!
//! This example demonstrates how to traverse and analyze an AST using the visitor pattern.
//! It counts different types of AST nodes (functions, classes, TypeScript import types)
//! to show how to implement custom AST analysis.
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js` or `test.ts`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_parser --example visitor [filename]
//!    ```
//!    Or with cargo watch:
//!    ```bash
//!    cargo watch -x "run -p oxc_parser --example visitor"
//!    ```

use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::ast::{Class, Function, TSImportType};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;

/// Main entry point for the AST visitor example
fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);

    // Read and validate the source file
    let source_text = std::fs::read_to_string(path)?;
    println!("Analyzing file: {}", name);
    println!("File size: {} bytes\n", source_text.len());

    // Set up parser
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    // Handle parsing errors
    if !ret.errors.is_empty() {
        println!("Parsing errors found:");
        for error in &ret.errors {
            let error = error.clone().with_source_code(source_text.clone());
            println!("{error:?}");
        }
        println!(); // Add spacing
    }

    let program = ret.program;

    // Create and run our custom AST visitor
    let mut ast_analyzer = ASTNodeCounter::default();
    ast_analyzer.visit_program(&program);

    // Display analysis results
    println!("AST Analysis Results:");
    println!("{:#?}", ast_analyzer);

    Ok(())
}

/// A visitor that counts different types of AST nodes
///
/// This demonstrates how to implement the `Visit` trait to perform
/// custom analysis on the parsed AST.
#[derive(Debug, Default)]
struct ASTNodeCounter {
    /// Number of function declarations and expressions found
    functions: usize,
    /// Number of class declarations found
    classes: usize,
    /// Number of TypeScript import type declarations found
    ts_import_types: usize,
}

impl<'a> Visit<'a> for ASTNodeCounter {
    /// Visit function declarations and expressions
    ///
    /// This method is called for every function node in the AST.
    /// The `flags` parameter provides information about the scope context.
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.functions += 1;

        // Continue traversing the function's children
        walk::walk_function(self, func, flags);
    }

    /// Visit class declarations
    ///
    /// This method is called for every class node in the AST.
    fn visit_class(&mut self, class: &Class<'a>) {
        self.classes += 1;

        // Continue traversing the class's children
        walk::walk_class(self, class);
    }

    /// Visit TypeScript import type declarations
    ///
    /// This method is called for TypeScript `import type` statements.
    fn visit_ts_import_type(&mut self, ty: &TSImportType<'a>) {
        self.ts_import_types += 1;

        // Continue traversing the import type's children
        walk::walk_ts_import_type(self, ty);
    }
}
