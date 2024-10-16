#![allow(clippy::print_stdout)]
use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use pico_args::Arguments;

// Instruction:
// 1. create a `test.js`
// 2. run `cargo run -p oxc_codegen --example codegen` or `just example codegen`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let twice = args.contains("--twice");
    let minify = args.contains("--minify");

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
        let printed = codegen(&ret, minify);
        println!("{printed}");
        // Check syntax error
        parse(&allocator, &printed, source_type);
    }

    Ok(())
}

fn parse<'a>(
    allocator: &'a Allocator,
    source_text: &'a str,
    source_type: SourceType,
) -> Option<ParserReturn<'a>> {
    let ret = Parser::new(allocator, source_text, source_type).parse();
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
