#![allow(clippy::print_stdout)]
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example minifier` or `just example minifier`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    let name = args.subcommand().ok().flatten().unwrap_or_else(|| String::from("test.js"));
    let mangle = args.contains("--mangle");
    let twice = args.contains("--twice");

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();

    let printed = minify(&source_text, source_type, mangle);
    println!("{printed}");

    if twice {
        let printed = minify(&printed, source_type, mangle);
        println!("{printed}");
    }

    Ok(())
}

fn minify(source_text: &str, source_type: SourceType, mangle: bool) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = allocator.alloc(ret.program);
    let options = MinifierOptions { mangle, compress: CompressOptions::all_true() };
    let ret = Minifier::new(options).build(&allocator, program);
    CodeGenerator::new().with_mangler(ret.mangler).build(program).source_text
}
