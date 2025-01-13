#![allow(clippy::print_stdout)]
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_prettier::{Prettier, PrettierOptions, TrailingComma};
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_prettier --example prettier`
// or `just example prettier`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    let name = args.subcommand().ok().flatten().unwrap_or_else(|| String::from("test.js"));
    let semi = !args.contains("--no-semi");

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions { preserve_parens: false, ..ParseOptions::default() })
        .parse();
    let output = Prettier::new(
        &allocator,
        PrettierOptions { semi, trailing_comma: TrailingComma::All, ..PrettierOptions::default() },
    )
    .build(&ret.program);
    println!("{output}");

    Ok(())
}
