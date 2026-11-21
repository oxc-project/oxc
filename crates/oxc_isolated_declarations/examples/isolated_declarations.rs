#![expect(clippy::print_stdout)]
//! # Isolated Declarations Example
//!
//! This example demonstrates how to generate TypeScript declaration files (.d.ts)
//! from TypeScript source code using isolated declarations.
//!
//! ## Usage
//!
//! Create a `test.ts` file and run:
//! ```bash
//! cargo run -p oxc_isolated_declarations --example isolated_declarations [filename]
//! ```

use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::{Source, SourceFileError};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_isolated_declarations --example isolated_declarations`
// or `just example isolated_declarations`

/// Generate TypeScript declaration files from TypeScript source
fn main() -> Result<(), SourceFileError> {
    let name = env::args().nth(1).unwrap_or_else(|| "test.ts".to_string());
    let path = Path::new(&name);

    let allocator = Allocator::default();
    let source = Source::from_file(path, &allocator)?;

    let ret = source.parse();
    let program = ret.program;
    let source_text = program.source_text;

    if !ret.diagnostics.is_empty() {
        for error in ret.diagnostics {
            println!("{}", error.render_with_source_code(source_text));
        }
        return Ok(());
    }

    println!("Original:\n");
    println!("{source_text}\n");

    let id_ret =
        IsolatedDeclarations::new(&allocator, IsolatedDeclarationsOptions { strip_internal: true })
            .build(&program);
    let printed = Codegen::new()
        .with_options(CodegenOptions {
            comments: CommentOptions { jsdoc: true, ..CommentOptions::disabled() },
            ..CodegenOptions::default()
        })
        .build(&id_ret.program)
        .code;

    println!("Dts Emit:\n");
    println!("{printed}\n");

    if !id_ret.diagnostics.is_empty() {
        println!("Transformed dts failed:\n");
        for error in id_ret.diagnostics {
            println!("{}", error.render_with_source_code(source_text));
        }
    }

    Ok(())
}
