#![expect(clippy::print_stdout)]
//! # Dead Code Elimination Example
//!
//! This example demonstrates using only the compression phase of the minifier,
//! specifically focusing on dead code elimination (DCE). This is useful when
//! you want to remove unreachable code without other optimizations.
//!
//! ## Usage
//!
//! ```bash
//! # Basic dead code elimination
//! cargo run --example dce test.js
//!
//! # Remove whitespace as well
//! cargo run --example dce --nospace test.js
//!
//! # Test stability by running twice  
//! cargo run --example dce --twice test.js
//! ```
//!
//! ## What it does
//!
//! - Removes unreachable code after return/throw statements
//! - Eliminates unused variable declarations
//! - Removes impossible conditional branches
//! - Cleans up empty statements and blocks

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
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();

    let mut allocator = Allocator::default();
    let printed = dce(&allocator, &source_text, source_type, nospace);
    println!("{printed}");

    if twice {
        allocator.reset();
        let printed2 = dce(&allocator, &printed, source_type, nospace);
        println!("{printed2}");
        println!("same = {}", printed == printed2);
    }

    Ok(())
}

fn dce(allocator: &Allocator, source_text: &str, source_type: SourceType, nospace: bool) -> String {
    let ret = Parser::new(allocator, source_text, source_type).parse();
    let mut program = ret.program;
    Compressor::new(allocator).dead_code_elimination(&mut program, CompressOptions::dce());
    Codegen::new()
        .with_options(CodegenOptions { minify: nospace, ..CodegenOptions::default() })
        .build(&program)
        .code
}
