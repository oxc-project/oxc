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
use oxc_parser::Parser;
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example dce`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    let nospace = args.contains("--nospace");
    let twice = args.contains("--twice");
    let max_iterations = args
        .opt_value_from_str::<&str, u8>("--max-iterations")
        .expect("Invalid number for --max-iterations");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();

    let mut allocator = Allocator::default();
    let printed = dce(&allocator, &source_text, source_type, nospace, max_iterations);
    println!("{printed}");

    if twice {
        allocator.reset();
        let printed2 = dce(&allocator, &printed, source_type, nospace, None);
        println!("{printed2}");
        println!("same = {}", printed == printed2);
    }

    Ok(())
}

fn dce(
    allocator: &Allocator,
    source_text: &str,
    source_type: SourceType,
    nospace: bool,
    max_iterations: Option<u8>,
) -> String {
    let ret = Parser::new(allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty());
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
