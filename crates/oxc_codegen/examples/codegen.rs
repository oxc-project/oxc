#![allow(clippy::print_stdout)]
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::{ParseOptions, Parser, ParserReturn};
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_codegen --example codegen`
// or `cargo watch -x "run -p oxc_codegen --example codegen"`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    let twice = args.contains("--twice");
    let minify = args.contains("--minify");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();
    let mut allocator = Allocator::default();

    let printed = {
        let Some(ret) = parse(&allocator, &source_text, source_type) else { return Ok(()) };
        codegen(&ret, minify)
    };
    println!("First time:");
    println!("{printed}");

    if twice {
        // Reset the allocator as we don't need the first AST any more
        allocator.reset();

        let Some(ret) = parse(&allocator, &printed, source_type) else { return Ok(()) };
        println!("Second time:");
        let printed2 = codegen(&ret, minify);
        println!("{printed2}");
        // Check syntax error
        parse(&allocator, &printed2, source_type);
        println!("same = {}", printed == printed2);
    }

    Ok(())
}

fn parse<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> Option<ParserReturn<'a>> {
    let ret = Parser::new(allocator, source_text, source_type)
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse();
    if !ret.errors.is_empty() {
        for error in ret.errors {
            println!("{:?}", error.with_source_code(source_text.to_string()));
        }
        return None;
    }
    Some(ret)
}

fn codegen(ret: &ParserReturn<'_>, minify: bool) -> String {
    CodeGenerator::new()
        .with_options(CodegenOptions { minify, ..CodegenOptions::default() })
        .build(&ret.program)
        .code
}
