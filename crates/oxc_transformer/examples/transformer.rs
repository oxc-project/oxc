#![allow(clippy::print_stdout)]
use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_transformer::{
    ArrowFunctionsOptions, ES2015Options, ES2016Options, ReactOptions, TransformOptions,
    Transformer, TypeScriptOptions,
};

// Instruction:
// create a `test.tsx`,
// run `cargo run -p oxc_transformer --example transformer`
// or `just watch "run -p oxc_transformer --example transformer"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.tsx".to_string());
    let path = Path::new(&name);
    dbg!(&path);
    let source_text = std::fs::read_to_string(path).expect("{name} not found");
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();

    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    if !ret.errors.is_empty() {
        println!("Parser Errors:");
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
    }

    println!("Original:\n");
    println!("{source_text}\n");

    let mut program = ret.program;
    let transform_options = TransformOptions {
        typescript: TypeScriptOptions::default(),
        es2015: ES2015Options { arrow_function: Some(ArrowFunctionsOptions::default()) },
        es2016: ES2016Options { exponentiation_operator: true },
        react: ReactOptions {
            jsx_plugin: true,
            jsx_self_plugin: true,
            jsx_source_plugin: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let _ = Transformer::new(
        &allocator,
        path,
        source_type,
        &source_text,
        ret.trivias.clone(),
        transform_options,
    )
    .build(&mut program);

    let printed = CodeGenerator::new().build(&program).source_text;
    println!("Transformed:\n");
    println!("{printed}");
}
