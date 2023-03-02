use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_common::PaddedStringView;
use oxc_parser::Parser;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_parser --example parser`
// or `cargo watch -x "run -p oxc_parser --example parser"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text =
        PaddedStringView::read_from_file(path).unwrap_or_else(|_| panic!("{name} not found"));
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if ret.errors.is_empty() {
        println!("{}", serde_json::to_string_pretty(&ret.program).unwrap());
        println!("Parsed Successfully.");
    } else {
        for error in ret.errors {
            let error = error.with_source_code((*source_text).clone());
            println!("{error:?}");
        }
    }
}
