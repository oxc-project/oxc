use std::{env, path::Path, sync::Arc};

use itertools::Itertools;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_semantic --example simple`
// or `just watch "run -p oxc_semantic --example simple"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text =
        Arc::new(std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{name} not found")));
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    let program = allocator.alloc(ret.program);

    let semantic = SemanticBuilder::new(&source_text, source_type)
        .with_check_syntax_error(true)
        .with_trivias(ret.trivias)
        .build(program);

    if !semantic.errors.is_empty() {
        let error_message: String = semantic
            .errors
            .into_iter()
            .map(|error| error.with_source_code(Arc::clone(&source_text)).to_string())
            .join("\n\n");

        panic!("Semantic analysis failed:\n\n{error_message}",);
    }
}
