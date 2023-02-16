use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_parser::Parser;
use oxc_printer::{Printer, PrinterOptions};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_printer --example simple`
// or `cargo watch -x "run -p oxc_printer --example simple"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let file = std::fs::read(path).expect("{name} not found");
    let allocator = Allocator::default();
    let source_text = String::from_utf8(file).expect("utf8");
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !ret.errors.is_empty() {
        println!("Parse Failed.");
        for error in &ret.errors {
            println!("{error:?}");
        }
        return;
    }

    let printer_options = PrinterOptions::default();
    let printed = Printer::new(source_text.len(), printer_options).build(&ret.program);
    println!("{printed}");
}
