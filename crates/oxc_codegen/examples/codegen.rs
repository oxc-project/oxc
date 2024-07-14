#![allow(clippy::print_stdout)]
use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::{CodeGenerator, WhitespaceRemover};
use oxc_parser::Parser;
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
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !ret.errors.is_empty() {
        for error in ret.errors {
            let error = error.with_source_code(source_text.to_string());
            println!("{error:?}");
        }
        return Ok(());
    }

    println!("Original:");
    println!("{source_text}");

    println!("First time:");
    let printed = CodeGenerator::new().build(&ret.program).source_text;
    println!("{printed}");

    if twice {
        println!("Second time:");
        let ret = Parser::new(&allocator, &printed, source_type).parse();
        if !ret.errors.is_empty() {
            for error in ret.errors {
                let error = error.with_source_code(source_text.to_string());
                println!("{error:?}");
            }
            return Ok(());
        }
        let printed = CodeGenerator::new().build(&ret.program).source_text;
        println!("{printed}");
    }

    if minify {
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &source_text, source_type).parse();
        let minified = WhitespaceRemover::new().build(&ret.program).source_text;
        println!("Minified:");
        println!("{minified}");
    }

    Ok(())
}
