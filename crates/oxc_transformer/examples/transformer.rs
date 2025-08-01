#![expect(clippy::print_stdout)]
//! # Transformer Example
//!
//! This example demonstrates how to use the Oxc transformer to convert modern JavaScript
//! and TypeScript code into compatible versions for different target environments.
//!
//! ## Features
//!
//! - ES6+ to ES5 transformation
//! - TypeScript to JavaScript compilation
//! - Browser compatibility transformations
//! - Custom Babel configuration support
//! - Target environment specification
//!
//! ## Usage
//!
//! 1. Create a test file (e.g., `test.js` or `test.ts`)
//! 2. Run the example:
//!    ```bash
//!    cargo run -p oxc_transformer --example transformer [options] [filename]
//!    ```
//!    Or with just:
//!    ```bash
//!    just example transformer
//!    just watch-example transformer
//!    ```
//!
//! ## Options
//!
//! - `--babel-options <path>`: Path to Babel configuration file
//! - `--targets <query>`: Browserslist query for target environments
//! - `--target <name>`: Predefined target environment name

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{BabelOptions, EnvOptions, HelperLoaderMode, TransformOptions, Transformer};
use pico_args::Arguments;

/// Main entry point for the transformer example
fn main() {
    // Parse command line arguments
    let mut args = Arguments::from_env();
    let babel_options_path: Option<String> =
        args.opt_value_from_str("--babel-options").unwrap_or(None);
    let targets: Option<String> = args.opt_value_from_str("--targets").unwrap_or(None);
    let target: Option<String> = args.opt_value_from_str("--target").unwrap_or(None);
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    // Read and validate the source file
    let path = Path::new(&name);
    let source_text =
        std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{name} not found.\n{err}"));
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();

    // Display transformation settings
    println!("Oxc Transformer Example");
    println!("=======================");
    println!("File: {name}");
    println!("Source type: {source_type:?}");
    println!("Settings:");
    if let Some(ref babel_path) = babel_options_path {
        println!("  - Babel config: {babel_path}");
    }
    if let Some(ref targets_query) = targets {
        println!("  - Targets: {targets_query}");
    }
    if let Some(ref target_name) = target {
        println!("  - Target: {target_name}");
    }
    println!();

    // Parse the source code
    let ret = Parser::new(&allocator, &source_text, source_type).parse();

    // Handle parsing errors
    if !ret.errors.is_empty() {
        println!("❌ Parser Errors:");
        for error in &ret.errors {
            let error = error.clone().with_source_code(source_text.clone());
            println!("{error:?}");
        }
        return;
    }

    println!("Original code:");
    println!("{}", "─".repeat(50));
    println!("{source_text}");
    println!("{}", "─".repeat(50));
    println!();

    let mut program = ret.program;

    // Build semantic information
    let ret = SemanticBuilder::new()
        // Estimate transformer will triple scopes, symbols, references
        .with_excess_capacity(2.0)
        .build(&program);

    // Handle semantic errors
    if !ret.errors.is_empty() {
        println!("❌ Semantic Errors:");
        for error in &ret.errors {
            let error = error.clone().with_source_code(source_text.clone());
            println!("{error:?}");
        }
        return;
    }

    let scoping = ret.semantic.into_scoping();

    // Configure transformation options
    let mut transform_options = configure_transform_options(
        babel_options_path.as_deref(),
        targets.as_deref(),
        target.as_deref(),
    );
    transform_options.helper_loader.mode = HelperLoaderMode::External;

    // Apply transformations
    let ret = Transformer::new(&allocator, path, &transform_options)
        .build_with_scoping(scoping, &mut program);

    // Handle transformation errors
    if !ret.errors.is_empty() {
        println!("❌ Transformer Errors:");
        for error in &ret.errors {
            let error = error.clone().with_source_code(source_text.clone());
            println!("{error:?}");
        }
        return;
    }

    // Generate the final transformed code
    let transformed_code = Codegen::new().build(&program).code;

    println!("✅ Transformation successful!");
    println!("Transformed code:");
    println!("{}", "─".repeat(50));
    println!("{transformed_code}");
    println!("{}", "─".repeat(50));
}

/// Configure transformation options based on command line arguments
fn configure_transform_options(
    babel_options_path: Option<&str>,
    targets: Option<&str>,
    target: Option<&str>,
) -> TransformOptions {
    if let Some(babel_options_path) = babel_options_path {
        let babel_options_path = Path::new(babel_options_path);
        let babel_options = BabelOptions::from_test_path(babel_options_path);
        TransformOptions::try_from(&babel_options).unwrap()
    } else if let Some(query) = targets {
        TransformOptions {
            env: EnvOptions::from_browserslist_query(query).unwrap(),
            ..TransformOptions::default()
        }
    } else if let Some(target_name) = target {
        TransformOptions::from_target(target_name).unwrap()
    } else {
        TransformOptions::enable_all()
    }
}
