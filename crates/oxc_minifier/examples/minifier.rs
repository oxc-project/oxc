use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_ast::SourceType;
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_printer::{Printer, PrinterOptions};
use oxc_semantic::SemanticBuilder;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example minifier`
// or `cargo watch -x "run -p oxc_minifier --example minifier"`

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

    let program = allocator.alloc(ret.program);

    let minifier_options = MinifierOptions::default();
    Minifier::new(&allocator, minifier_options).build(program);

    let semantic_ret = SemanticBuilder::new(&source_text, source_type, &ret.trivias).build(program);

    let printer_options = PrinterOptions { minify_whitespace: true, ..PrinterOptions::default() };
    let printed = Printer::new(source_text.len(), printer_options)
        .with_symbol_table(&semantic_ret.semantic.symbols(), true)
        .build(program);

    println!("{printed}");
}
