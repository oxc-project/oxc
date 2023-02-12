use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_parser::Parser;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example simple`
// or `cargo watch -x "run -p oxc_parser --example simple"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let file = std::fs::read(path).expect("{name} not found");
    let allocator = Allocator::default();
    let source = String::from_utf8(file).expect("utf8");
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source, source_type).parse();

    if ret.errors.is_empty() {
        println!("Parsed Successfully.");
    } else {
        println!("Parse Failed.");
        for error in &ret.errors {
            println!("{error:?}");
        }
    }
}
