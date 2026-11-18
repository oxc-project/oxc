#![expect(clippy::print_stdout)]
//! # Complete Compiler Example
//!
//! This example demonstrates the complete Oxc compilation pipeline including
//! parsing, semantic analysis, transformation, and code generation.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc --example compiler --features="full" [filename]
//! ```

use std::{env, path::Path};

use oxc::{
    Compiler,
    allocator::Allocator,
    parser::{Source, SourceFileError},
};

// Instruction:
// 1. create a `test.js`
// 2. run either
//   * `cargo run -p oxc --example compiler --features="full"`
//   * `just watch 'run -p oxc --example compiler --features="full"'`

/// Run the complete Oxc compilation pipeline on a JavaScript/TypeScript file
fn main() -> Result<(), SourceFileError> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);

    let allocator = Allocator::default();
    let source = Source::from_file(path, &allocator)?;

    // Execute the full compilation pipeline
    match Compiler::default().execute_source(source, path) {
        Ok(printed) => {
            println!("{printed}");
        }
        Err((errors, source_text)) => {
            for error in errors {
                println!("{}", error.render_with_source_code(source_text));
            }
        }
    }

    Ok(())
}
