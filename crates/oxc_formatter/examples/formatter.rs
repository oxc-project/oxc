#![expect(clippy::print_stdout)]

use std::{fs, path::Path};

use oxc_allocator::Allocator;
use oxc_formatter::{FormatOptions, Formatter};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use pico_args::Arguments;

fn main() -> Result<(), String> {
    let mut args = Arguments::from_env();
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let path = Path::new(&name);
    let source_text = fs::read_to_string(path).map_err(|_| format!("Missing '{name}'"))?;
    let source_type = SourceType::from_path(path).unwrap();
    let allocator = Allocator::new();
    let ret = Parser::new(&allocator, &source_text, source_type)
        .with_options(ParseOptions {
            preserve_parens: false,
            allow_v8_intrinsics: true,
            ..ParseOptions::default()
        })
        .parse();

    for error in ret.errors {
        let error = error.with_source_code(source_text.clone());
        println!("{error:?}");
        println!("Parsed with Errors.");
    }

    let options = FormatOptions::default();
    let code = Formatter::new(&allocator, options).build(&ret.program);

    println!("{code}");

    Ok(())
}
