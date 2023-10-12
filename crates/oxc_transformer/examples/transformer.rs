use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, TransformReactOptions, TransformTarget, Transformer};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_transformer --example transformer`
// or `just watch "run -p oxc_transformer --example transformer"`

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

    let codegen_options = CodegenOptions;
    let printed = Codegen::<false>::new(source_text.len(), codegen_options).build(&ret.program);
    println!("Original:\n");
    println!("{printed}");

    let program = allocator.alloc(ret.program);
    let _ = SemanticBuilder::new(&source_text, source_type).build(program);
    let transform_options = TransformOptions {
        target: TransformTarget::ES2015,
        react: Some(TransformReactOptions::default()),
    };
    Transformer::new(&allocator, source_type, transform_options).build(program);
    let printed = Codegen::<false>::new(source_text.len(), codegen_options).build(program);
    println!("Transformed:\n");
    println!("{printed}");
}
