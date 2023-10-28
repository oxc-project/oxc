use std::{cell::RefCell, env, path::Path, rc::Rc};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{
    ReactJsxOptions, ReactJsxRuntime, TransformOptions, TransformTarget, Transformer,
};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_transformer --example transformer`
// or `just watch "run -p oxc_transformer --example transformer"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.tsx".to_string());
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
    println!("{printed}\n");

    let semantic = SemanticBuilder::new(&source_text, source_type).build(&ret.program).semantic;
    let (symbols, scopes) = semantic.into_symbol_table_and_scope_tree();
    let symbols = Rc::new(RefCell::new(symbols));
    let scopes = Rc::new(RefCell::new(scopes));

    let program = allocator.alloc(ret.program);
    let transform_options = TransformOptions {
        target: TransformTarget::ES2015,
        react_jsx: Some(ReactJsxOptions {
            runtime: ReactJsxRuntime::Classic,
            ..ReactJsxOptions::default()
        }),
        ..TransformOptions::default()
    };
    Transformer::new(&allocator, source_type, &symbols, &scopes, transform_options).build(program);
    let printed = Codegen::<false>::new(source_text.len(), codegen_options).build(program);
    println!("Transformed:\n");
    println!("{printed}");
}
