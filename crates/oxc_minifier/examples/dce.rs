#![expect(clippy::print_stdout)]
//! # Dead Code Elimination Example
//!
//! This example demonstrates dead code elimination (DCE) using the Oxc compressor.
//! It removes unreachable code and unused variables.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_minifier --example dce [filename] [--nospace] [--twice]
//! ```
//!
//! ## Options
//!
//! - `--nospace`: Remove extra whitespace
//! - `--twice`: Test idempotency by running twice
//! - `--max-iterations <u8>`: Set the maximum number of compress pass iterations

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::{Source, SourceFileError};
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example dce`

fn main() -> Result<(), SourceFileError> {
    let mut args = Arguments::from_env();

    let nospace = args.contains("--nospace");
    let twice = args.contains("--twice");
    let max_iterations = args
        .opt_value_from_str::<&str, u8>("--max-iterations")
        .expect("Invalid number for --max-iterations");

    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());
    let path = Path::new(&name);

    let mut allocator = Allocator::default();
    let source = Source::read_from_file(path, &allocator)?;
    let source_type = source.source_type();

    let printed = dce(source, nospace, max_iterations);
    println!("{printed}");

    if twice {
        allocator.reset();
        let source = Source::from_str(&printed, source_type, &allocator);
        let printed2 = dce(source, nospace, None);
        println!("{printed2}");
        println!("same = {}", printed == printed2);
    }

    Ok(())
}

fn dce(source: Source<'_>, nospace: bool, max_iterations: Option<u8>) -> String {
    let allocator = source.allocator();
    let ret = source.parse();
    assert!(ret.diagnostics.is_empty());
    let mut program = ret.program;
    Compressor::new(allocator).dead_code_elimination(
        &mut program,
        CompressOptions { max_iterations, ..CompressOptions::dce() },
    );
    Codegen::new()
        .with_options(CodegenOptions { minify: nospace, ..CodegenOptions::default() })
        .build(&program)
        .code
}
