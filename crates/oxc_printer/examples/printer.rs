use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_parser::Parser;
use oxc_printer::{Printer, PrinterOptions};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_printer --example printer`
// or `cargo watch -x "run -p oxc_printer --example printer"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
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

    let printer_options = PrinterOptions::default();
    let printed = Printer::new(source_text.len(), printer_options).build(&ret.program);
    println!("{printed}");
}
