//! Linter with plugin

use std::{env, path::Path, rc::Rc, time::Instant};

use oxc_allocator::Allocator;
use oxc_linter::{calculate::Calculate, Linter};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

// Instruction:
// create a `test.js`,
// run `OXC_PLUGIN=./crates/oxc_linter/examples/queries cargo run -p oxc_linter --example plugin`
// or `OXC_PLUGIN=./crates/oxc_linter/examples/queries cargo watch -x "run -p oxc_linter --example plugin"`

fn main() {
    let start = Instant::now();
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{name} not found"));
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    // Handle parser errors
    if !ret.errors.is_empty() {
        print_errors(&source_text, ret.errors);
        return;
    }

    let program = allocator.alloc(ret.program);
    let semantic_ret =
        SemanticBuilder::new(&source_text, source_type).with_trivias(&ret.trivias).build(program);

    // let mut errors: Vec<oxc_diagnostics::Error> = vec![];
    let linter = Linter::new();
    let messages =
        linter.run(&Rc::new(semantic_ret.semantic), path.related_to(Path::new(".")).unwrap());
    let errors = messages.into_iter().map(|m| m.error).collect::<Vec<_>>();

    if !errors.is_empty() {
        print_errors(&source_text, errors);
        println!("elapsed: {:?}", start.elapsed());
        return;
    }

    println!("Success!");
}

fn print_errors(source_text: &str, errors: Vec<oxc_diagnostics::Error>) {
    for error in errors {
        let error = error.with_source_code(source_text.to_string());
        println!("{error:?}");
    }
}
