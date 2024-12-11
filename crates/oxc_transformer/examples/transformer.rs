#![allow(clippy::print_stdout)]
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{BabelOptions, EnvOptions, HelperLoaderMode, TransformOptions, Transformer};
use pico_args::Arguments;

// Instruction:
// create a `test.js`,
// run `just example transformer` or `just watch-example transformer`

fn main() {
    let mut args = Arguments::from_env();
    let babel_options_path: Option<String> =
        args.opt_value_from_str("--babel-options").unwrap_or(None);
    let targets: Option<String> = args.opt_value_from_str("--targets").unwrap_or(None);
    let target: Option<String> = args.opt_value_from_str("--target").unwrap_or(None);
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let path = Path::new(&name);
    let source_text =
        std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{name} not found.\n{err}"));
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

    let ret = SemanticBuilder::new()
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

    let mut transform_options = if let Some(babel_options_path) = babel_options_path {
        let babel_options_path = Path::new(&babel_options_path);
        let babel_options = BabelOptions::from_test_path(babel_options_path);
        TransformOptions::try_from(&babel_options).unwrap()
    } else if let Some(query) = &targets {
        TransformOptions {
            env: EnvOptions::from_browserslist_query(query).unwrap(),
            ..TransformOptions::default()
        }
    } else if let Some(target) = &target {
        TransformOptions::from_target(target).unwrap()
    } else {
        TransformOptions::enable_all()
    };

    transform_options.helper_loader.mode = HelperLoaderMode::External;

    let ret = Transformer::new(&allocator, path, &transform_options).build_with_symbols_and_scopes(
        symbols,
        scopes,
        &mut program,
    );

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
