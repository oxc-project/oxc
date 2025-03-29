#![expect(clippy::print_stdout)]
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
//
// Debug:
// run `cargo run -p oxc_prettier --example prettier -- --debug`
//
// The output will be the Doc AST JSON(= most verbose form) of the Prettier,
// now you can paste and inspect it in their playground.
// https://prettier.io/playground
// Be sure to:
// - change global option `--parser: doc-explorer`
// - enable debug option `show doc`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    let name = args.subcommand().ok().flatten().unwrap_or_else(|| String::from("test.js"));
    let semi = !args.contains("--no-semi");

    let debug = args.contains("--debug");

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions {
            preserve_parens: false,
            allow_v8_intrinsics: true,
            ..ParseOptions::default()
        })
        .parse();
    let mut prettier = Prettier::new(
        &allocator,
        PrettierOptions { semi, trailing_comma: TrailingComma::All, ..PrettierOptions::default() },
    );

    let output =
        if debug { prettier.doc(&ret.program).to_string() } else { prettier.build(&ret.program) };
    println!("{output}");

    Ok(())
}
