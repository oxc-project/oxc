use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use oxc_prettier::{Prettier, PrettierOptions};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_prettier --example prettier`
// or `just example prettier`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{name} not found"));
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();
    let output = Prettier::new(&allocator, PrettierOptions).build(&ret.program);
    println!("{output}");
}
