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

use std::{env, io, path::Path};

use oxc::{
    Compiler,
    span::{SourceCodeBuf, SourceType},
};

// Instruction:
// 1. create a `test.js`
// 2. run either
//   * `cargo run -p oxc --example compiler --features="full"`
//   * `just watch 'run -p oxc --example compiler --features="full"'`

/// Run the complete Oxc compilation pipeline on a JavaScript/TypeScript file
fn main() -> io::Result<()> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    // Read the file straight into a buffer which ends with the padding the parser reads
    let source = SourceCodeBuf::read(path)?;
    let source_type = SourceType::from_path(path).unwrap();

    // Execute the full compilation pipeline
    match Compiler::default().execute_source(source.as_source_code(), source_type, path) {
        Ok(printed) => {
            println!("{printed}");
        }
        Err(errors) => {
            for error in errors {
                println!("{}", error.render_with_source_code(source.as_str().to_string()));
            }
        }
    }

    Ok(())
}
