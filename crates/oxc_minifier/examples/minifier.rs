#![allow(clippy::print_stdout)]
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
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
    let nospace = args.contains("--nospace");
    let twice = args.contains("--twice");

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();

    let mut allocator = Allocator::default();
    let printed = minify(&allocator, &source_text, source_type, mangle, nospace);
    println!("{printed}");

    if twice {
        allocator.reset();
        let printed2 = minify(&allocator, &printed, source_type, mangle, nospace);
        println!("{printed2}");
        println!("same = {}", printed == printed2);
    }

    Ok(())
}

fn minify(
    allocator: &Allocator,
    source_text: &str,
    source_type: SourceType,
    mangle: bool,
    nospace: bool,
) -> String {
    let ret = Parser::new(allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let options = MinifierOptions { mangle, compress: CompressOptions::default() };
    let ret = Minifier::new(options).build(allocator, &mut program);
    CodeGenerator::new()
        .with_options(CodegenOptions { minify: nospace, ..CodegenOptions::default() })
        .with_mangler(ret.mangler)
        .build(&program)
        .code
}
