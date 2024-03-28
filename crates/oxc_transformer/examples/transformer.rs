use std::{
    env,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{
    ReactJsxOptions, ReactJsxRuntime, ReactJsxRuntimeOption, TransformOptions, TransformTarget,
    Transformer,
};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_transformer --example transformer`
// or `just watch "run -p oxc_transformer --example transformer"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.tsx".to_string());
    let path = Path::new(&name);
    let file_name = path
        .file_name()
        .expect("Expected to have a file name")
        .to_str()
        .expect("File name to be valid UTF-8");
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

    println!("Original:\n");
    println!("{source_text}\n");

    let semantic = SemanticBuilder::new(&source_text, source_type)
        .with_trivias(ret.trivias)
        .build_module_record(PathBuf::new(), &ret.program)
        .build(&ret.program)
        .semantic;

    let program = allocator.alloc(ret.program);
    let transform_options = TransformOptions {
        target: TransformTarget::ES5,
        react_jsx: Some(ReactJsxOptions {
            runtime: Some(ReactJsxRuntimeOption::Valid(ReactJsxRuntime::Classic)),
            development: Some(true),
            ..ReactJsxOptions::default()
        }),
        ..TransformOptions::default()
    };
    Transformer::new(&allocator, source_type, semantic, transform_options, file_name.to_string())
        .build(program)
        .unwrap();

    let printed = Codegen::<false>::new("", &source_text, CodegenOptions::default())
        .build(program)
        .source_text;
    println!("Transformed:\n");
    println!("{printed}");
}
