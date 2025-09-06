#![expect(clippy::print_stdout)]
//! # Variable Name Mangling Example
//!
//! This example demonstrates variable name mangling using the Oxc mangler.
//! It shortens variable names to reduce file size.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_minifier --example mangler [filename] [options]
//! ```
//!
//! ## Options
//!
//! - `--keep-names`: Preserve function and class names
//! - `--debug`: Enable debug output
//! - `--twice`: Test idempotency by running twice

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames, Mangler};
use oxc_parser::Parser;
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example mangler`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    let keep_names = args.contains("--keep-names");
    let debug = args.contains("--debug");
    let twice = args.contains("--twice");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();

    let options = MangleOptions {
        top_level: source_type.is_module(),
        keep_names: MangleOptionsKeepNames { function: keep_names, class: keep_names },
        debug,
    };
    let printed = mangler(&source_text, source_type, options);
    println!("{printed}");

    if twice {
        let printed2 = mangler(&printed, source_type, options);
        println!("{printed2}");
        println!("same = {}", printed == printed2);
    }

    Ok(())
}

fn mangler(source_text: &str, source_type: SourceType, options: MangleOptions) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let symbol_table = Mangler::new().with_options(options).build(&ret.program);
    Codegen::new().with_scoping(Some(symbol_table)).build(&ret.program).code
}
