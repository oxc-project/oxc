#![expect(clippy::print_stdout)]
//! # Code Generation Example
//!
//! This example demonstrates how to use the Oxc code generator to convert an AST
//! back into JavaScript code. It supports minification and idempotency testing.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_codegen --example codegen [filename] [--minify] [--twice]
//! ```
//!
//! ## Options
//!
//! - `--minify`: Generate minified output
//! - `--twice`: Test idempotency by parsing and generating twice
//! - `--sourcemap`: Print the generated source map (requires the `sourcemap` feature)

use std::path::Path;

use pico_args::Arguments;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_parser::{ParseOptions, Source, SourceFileError};
use oxc_sourcemap::SourcemapVisualizer;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_codegen --example codegen`
// or `cargo watch -x "run -p oxc_codegen --example codegen"`

/// Demonstrate code generation with optional minification and idempotency testing
fn main() -> Result<(), SourceFileError> {
    let mut args = Arguments::from_env();

    let twice = args.contains("--twice");
    let minify = args.contains("--minify");
    let sourcemap = args.contains("--sourcemap");

    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());
    let path = Path::new(&name);
    let sourcemap = sourcemap.then_some(path);

    let mut allocator = Allocator::default();
    let source = Source::read_from_file(path, &allocator)?;
    let source_type = source.source_type();

    // First round: parse and generate
    let printed = {
        let program = parse(source);
        codegen(&program, minify, sourcemap)
    };
    println!("First time:");
    println!("{printed}");

    // Optional second round to test idempotency
    if twice {
        // Reset the allocator as we don't need the first AST any more
        allocator.reset();

        let program = parse(Source::from_str(&printed, source_type, &allocator));
        println!("Second time:");
        let printed2 = codegen(&program, minify, None);
        println!("{printed2}");
        // Check syntax error
        parse(Source::from_str(&printed2, source_type, &allocator));
        println!("same = {}", printed == printed2);
    }

    Ok(())
}

/// Parse JavaScript/TypeScript source code into an AST
fn parse(source: Source<'_>) -> Program<'_> {
    let ret = source.parse_with_options(ParseOptions {
        allow_return_outside_function: true,
        enable_ident_hashes: false,
        ..ParseOptions::default()
    });
    let program = ret.program;
    for error in ret.diagnostics {
        println!("{}", error.render_with_source_code(program.source_text));
    }
    program
}

/// Generate JavaScript code from an AST
fn codegen(program: &Program<'_>, minify: bool, source_map_path: Option<&Path>) -> String {
    let mut options = if minify { CodegenOptions::minify() } else { CodegenOptions::default() };
    options.source_map_path = source_map_path.map(Path::to_path_buf);

    let CodegenReturn { code, map, .. } = Codegen::new().with_options(options).build(program);

    if let Some(map) = map {
        let visualizer = SourcemapVisualizer::new(&code, &map);
        println!("{}", visualizer.get_url());
        println!("{}", visualizer.get_text());
    }

    code
}
