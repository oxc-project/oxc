#![expect(clippy::print_stdout)]
//! # AST Visitor Example
//!
//! This example demonstrates how to use the visitor pattern to traverse and analyze
//! an AST. It counts different types of nodes (functions, classes, TypeScript imports).
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_parser --example visitor [filename]
//! ```

use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::ast::{Class, Function, TSImportType};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example visitor`
// or `cargo watch -x "run -p oxc_parser --example visitor"`

/// Demonstrate AST traversal using the visitor pattern
fn main() -> std::io::Result<()> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    // Report any parsing errors
    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
    }

    let program = ret.program;

    // Use visitor to count different AST node types
    let mut ast_pass = CountASTNodes::default();
    ast_pass.visit_program(&program);
    println!("{ast_pass:?}");

    Ok(())
}

/// A visitor that counts different types of AST nodes
#[derive(Debug, Default)]
struct CountASTNodes {
    functions: usize,
    classes: usize,
    ts_import_types: usize,
}

impl<'a> Visit<'a> for CountASTNodes {
    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        self.functions += 1;
        walk::walk_function(self, func, flags);
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.classes += 1;
        walk::walk_class(self, class);
    }

    fn visit_ts_import_type(&mut self, ty: &TSImportType<'a>) {
        self.ts_import_types += 1;
        walk::walk_ts_import_type(self, ty);
    }
}
