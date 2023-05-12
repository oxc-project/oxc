use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast_lower::AstLower;
use oxc_parser::Parser;
use oxc_span::SourceType;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_ast_lower --example ast_lower`
// or `cargo watch -x "run -p oxc_ast_lower --example ast_lower"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{name} not found"));
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    let program = allocator.alloc(ret.program);
    let hir = AstLower::new(&allocator, source_type).build(program);

    println!("{}", serde_json::to_string_pretty(&hir).unwrap());
}
