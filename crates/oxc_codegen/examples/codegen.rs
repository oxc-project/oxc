#![expect(clippy::print_stdout)]
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::{ParseOptions, Parser};
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
        let program = parse(&allocator, &source_text, source_type);
        codegen(&program, minify)
    };
    println!("First time:");
    println!("{printed}");

    if twice {
        // Reset the allocator as we don't need the first AST any more
        allocator.reset();

        let program = parse(&allocator, &printed, source_type);
        println!("Second time:");
        let printed2 = codegen(&program, minify);
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
) -> Program<'a> {
    let ret = Parser::new(allocator, source_text, source_type)
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..ParseOptions::default()
        })
        .parse();
    for error in ret.errors {
        println!("{:?}", error.with_source_code(source_text.to_string()));
    }
    ret.program
}

fn codegen(program: &Program<'_>, minify: bool) -> String {
    Codegen::new()
        .with_options(if minify { CodegenOptions::minify() } else { CodegenOptions::default() })
        .build(program)
        .code
}
