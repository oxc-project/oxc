#![allow(clippy::print_stdout)]
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_mangler::{MangleOptions, Mangler};
use oxc_parser::Parser;
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example mangler`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    let name = args.subcommand().ok().flatten().unwrap_or_else(|| String::from("test.js"));
    let debug = args.contains("--debug");
    let twice = args.contains("--twice");

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();

    let printed = mangler(&source_text, source_type, debug);
    println!("{printed}");

    if twice {
        let printed2 = mangler(&printed, source_type, debug);
        println!("{printed2}");
        println!("same = {}", printed == printed2);
    }

    Ok(())
}

fn mangler(source_text: &str, source_type: SourceType, debug: bool) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mangler = Mangler::new().with_options(MangleOptions { debug }).build(&ret.program);
    CodeGenerator::new().with_mangler(Some(mangler)).build(&ret.program).code
}
