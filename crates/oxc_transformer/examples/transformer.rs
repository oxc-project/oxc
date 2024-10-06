#![allow(clippy::print_stdout)]
use std::{env, path::Path};

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{EnvOptions, Targets, TransformOptions, Transformer};
use pico_args::Arguments;

// Instruction:
// create a `test.tsx`,
// run `cargo run -p oxc_transformer --example transformer`
// or `just watch "run -p oxc_transformer --example transformer"`

fn main() {
    let mut args = Arguments::from_env();
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let targets: Option<String> = args.opt_value_from_str("--targets").unwrap_or(None);

    let path = Path::new(&name);
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
    let trivias = ret.trivias;

    let ret = SemanticBuilder::new(&source_text)
        // Estimate transformer will triple scopes, symbols, references
        .with_excess_capacity(2.0)
        .build(&program);

    if !ret.errors.is_empty() {
        println!("Semantic Errors:");
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
    }

    let (symbols, scopes) = ret.semantic.into_symbol_table_and_scope_tree();

    let transform_options = if let Some(targets) = &targets {
        TransformOptions::from_preset_env(&EnvOptions {
            targets: Targets::from_query(targets),
            ..EnvOptions::default()
        })
        .unwrap()
    } else {
        TransformOptions::enable_all()
    };

    let ret = Transformer::new(&allocator, path, &source_text, trivias.clone(), transform_options)
        .build_with_symbols_and_scopes(symbols, scopes, &mut program);

    if !ret.errors.is_empty() {
        println!("Transformer Errors:");
        for error in ret.errors {
            let error = error.with_source_code(source_text.clone());
            println!("{error:?}");
        }
    }

    let printed = CodeGenerator::new().build(&program).code;
    println!("Transformed:\n");
    println!("{printed}");
}
