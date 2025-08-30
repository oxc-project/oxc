#![expect(clippy::print_stdout)]
//! # Minifier Example
//!
//! This example demonstrates the Oxc minifier with options for compression,
//! mangling, and source map generation.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_minifier --example minifier [filename] [options]
//! ```
//!
//! ## Options
//!
//! - `--mangle`: Enable variable name mangling
//! - `--nospace`: Remove extra whitespace
//! - `--twice`: Test idempotency by running twice
//! - `--sourcemap`: Generate source maps
//! - `--max-iterations <u8>`: Set the maximum number of compress pass iterations

use std::path::{Path, PathBuf};

use pico_args::Arguments;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn, CommentOptions};
use oxc_mangler::MangleOptions;
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_sourcemap::SourcemapVisualizer;
use oxc_span::SourceType;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example minifier` or `just example minifier`

fn main() -> std::io::Result<()> {
    let mut args = Arguments::from_env();

    let mangle = args.contains("--mangle");
    let nospace = args.contains("--nospace");
    let twice = args.contains("--twice");
    let sourcemap = args.contains("--sourcemap");
    let max_iterations = args
        .opt_value_from_str::<&str, u8>("--max-iterations")
        .expect("Invalid number for --max-iterations");
    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path)?;
    let source_type = SourceType::from_path(path).unwrap();
    let source_map_path = sourcemap.then(|| path.to_path_buf());

    let mut allocator = Allocator::default();
    let ret = minify(
        &allocator,
        &source_text,
        source_type,
        source_map_path,
        mangle,
        nospace,
        max_iterations,
    );
    let printed = ret.code;
    println!("{printed}");

    if let Some(map) = ret.map {
        let visualizer = SourcemapVisualizer::new(&printed, &map);
        println!("{}", visualizer.get_url());
        println!("{}", visualizer.get_text());
    }

    if twice {
        allocator.reset();
        let printed2 = minify(&allocator, &printed, source_type, None, mangle, nospace, None).code;
        println!("{printed2}");
        println!("same = {}", printed == printed2);
    }

    Ok(())
}

fn minify(
    allocator: &Allocator,
    source_text: &str,
    source_type: SourceType,
    source_map_path: Option<PathBuf>,
    mangle: bool,
    nospace: bool,
    max_iterations: Option<u8>,
) -> CodegenReturn {
    let ret = Parser::new(allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let options = MinifierOptions {
        mangle: mangle.then(MangleOptions::default),
        compress: Some(CompressOptions { max_iterations, ..CompressOptions::smallest() }),
    };
    let ret = Minifier::new(options).minify(allocator, &mut program);
    Codegen::new()
        .with_options(CodegenOptions {
            source_map_path,
            minify: nospace,
            comments: CommentOptions::disabled(),
            ..CodegenOptions::default()
        })
        .with_scoping(ret.scoping)
        .build(&program)
}
