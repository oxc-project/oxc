#![allow(clippy::print_stdout)]
use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_isolated_declarations::IsolatedDeclarations;
use oxc_parser::Parser;
use oxc_span::SourceType;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_isolated_declarations --example isolated_declarations`
// or `just watch "run -p oxc_isolated_declarations --example isolated_declarations"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.tsx".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).expect("{name} not found");
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();

    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !ret.errors.is_empty() {
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
        return;
    }

    println!("Original:\n");
    println!("{source_text}\n");

    let ret = IsolatedDeclarations::new(&allocator).build(&ret.program);
    let printed = CodeGenerator::new().build(&ret.program).source_text;

    println!("Dts Emit:\n");
    println!("{printed}\n");

    if !ret.errors.is_empty() {
        println!("Transformed dts failed:\n");
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
    }
}
