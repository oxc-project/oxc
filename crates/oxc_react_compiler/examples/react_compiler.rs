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
use oxc_parser::{Source, SourceFileError};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

use oxc_react_compiler::{CompileResult, PluginOptions, compile};

/// Compile a React component with the Rust React Compiler and print the result.
fn main() -> Result<(), SourceFileError> {
    let name = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: cargo run -p oxc_react_compiler --example react_compiler -- <FILE>");
        std::process::exit(1);
    });
    let path = Path::new(&name);
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::tsx());

    let allocator = Allocator::default();
    let source = Source::read_from_file_with_source_type(path, source_type, &allocator)?;

    let mut program = source.parse().program;

    println!("Original ({name}):\n");
    println!("{}", program.source_text);

    let result = {
        let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
        compile(&program, &semantic, &allocator, PluginOptions::default())
    };
    let (changed, diagnostics, fatal) = match result {
        CompileResult::Success { output, diagnostics } => {
            let changed = output.is_some();
            if let Some(output) = output {
                output.transform(&mut program);
            }
            (changed, diagnostics, false)
        }
        CompileResult::Fatal { diagnostics } => (false, diagnostics, true),
    };

    if !diagnostics.is_empty() {
        println!("Diagnostics:\n");
        for diagnostic in &diagnostics {
            println!("{diagnostic:?}");
        }
        println!();
    }

    if fatal {
        println!("Compilation aborted.");
    } else if changed {
        let code = Codegen::new().build(&program).code;
        println!("Compiled:\n");
        println!("{code}");
    } else {
        println!("No changes: no React component or hook found (or compilation bailed out).");
    }

    Ok(())
}
