//! # React Compiler Example
//!
//! Runs the Rust port of the [React Compiler] over a file through the oxc
//! frontend (parse + semantic -> convert -> compile -> convert back -> codegen)
//! and prints the memoized output.
//!
//! ## Usage
//!
//! ```bash
//! just example react_compiler MyFile.jsx
//! ```
//!
//! [React Compiler]: https://github.com/react/react/tree/main/compiler

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use oxc_react_compiler::{PluginOptions, transform};

/// Compile a React component with the Rust React Compiler and print the result.
fn main() {
    let name = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run -p oxc_react_compiler --example react_compiler -- <FILE>");
        std::process::exit(1);
    });
    let path = Path::new(&name);
    let source_text =
        std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{name} not found.\n{err}"));
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::tsx());

    println!("Original ({name}):\n");
    println!("{source_text}");

    let allocator = Allocator::default();
    let mut program = Parser::new(&allocator, &source_text, source_type).parse().program;

    let mut result = {
        let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
        transform(&program, &semantic, &allocator, PluginOptions::default())
    };
    if let Some(compiled) = result.program.take() {
        program = compiled;
    }

    if !result.diagnostics.is_empty() {
        println!("Diagnostics:\n");
        for diagnostic in &result.diagnostics {
            println!("{diagnostic:?}");
        }
        println!();
    }

    if result.changed {
        let output = Codegen::new().build(&program).code;
        println!("Compiled:\n");
        println!("{output}");
    } else {
        println!("No changes: no React component or hook found (or compilation bailed out).");
    }
}
