use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

// Instruction:
// 1. create a `test.js`
// 2. run `cargo run -p oxc_codegen --example codegen` or `just example codegen`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.ts".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{name} not found"));
    let source_type = SourceType::from_path(path).unwrap();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !ret.errors.is_empty() {
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
        return;
    }

    let codegen_options = CodegenOptions;
    let printed = Codegen::<false>::new(source_text.len(), codegen_options).build(&ret.program);
    println!("{printed}");

    let ret = Parser::new(&allocator, &printed, source_type).parse();
    let printed = Codegen::<false>::new(source_text.len(), codegen_options).build(&ret.program);
    println!("{printed}");
}
