#![expect(clippy::print_stdout)]
//! # Define Plugin Example
//!
//! This example demonstrates how to use the Oxc code generator to convert an AST
//! back into JavaScript code. It supports minification and idempotency testing.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_transformer_plugins --example define -- [filename]
//! ```

use std::path::Path;

use pico_args::Arguments;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_parser::{ParseOptions, Source, SourceFileError};
use oxc_semantic::SemanticBuilder;
use oxc_sourcemap::SourcemapVisualizer;
use oxc_transformer_plugins::{ReplaceGlobalDefines, ReplaceGlobalDefinesConfig};

fn main() -> Result<(), SourceFileError> {
    let mut args = Arguments::from_env();

    let defines = [("process.env.NODE_ENV", "development")];

    let sourcemap = args.contains("--sourcemap");

    let name = args.free_from_str().unwrap_or_else(|_| "test.js".to_string());
    let path = Path::new(&name);
    let sourcemap = sourcemap.then_some(path);

    let allocator = Allocator::default();
    let source = Source::read_from_file(path, &allocator)?;

    let mut program = parse(source);
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let config = ReplaceGlobalDefinesConfig::new(&defines).unwrap();
    let _ = ReplaceGlobalDefines::new(&allocator, config).build(scoping, &mut program);
    let printed = codegen(&program, sourcemap);

    println!("{printed}");

    Ok(())
}

/// Parse JavaScript/TypeScript source code into an AST
fn parse(source: Source<'_>) -> Program<'_> {
    let ret = source.parse_with_options(ParseOptions {
        allow_return_outside_function: true,
        ..ParseOptions::default()
    });
    for error in ret.diagnostics {
        println!("{}", error.render_with_source_code(ret.program.source_text));
    }
    ret.program
}

/// Generate JavaScript code from an AST
fn codegen(program: &Program<'_>, source_map_path: Option<&Path>) -> String {
    let options = CodegenOptions {
        source_map_path: source_map_path.map(Path::to_path_buf),
        ..CodegenOptions::default()
    };

    let CodegenReturn { code, map, .. } = Codegen::new().with_options(options).build(program);

    if let Some(map) = map {
        let visualizer = SourcemapVisualizer::new(&code, &map);
        println!("{}", visualizer.get_url());
        println!("{}", visualizer.get_text());
    }

    code
}
