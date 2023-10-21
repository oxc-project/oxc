use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

use pico_args::Arguments;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example minifier`
// or `just watch "run -p oxc_minifier --example minifier"`

fn main() {
    let mut args = Arguments::from_env();

    let name = args.subcommand().ok().flatten().unwrap_or_else(|| String::from("test.js"));
    let mangle = args.contains("--mangle");
    let whitespace = args.contains("--whitespace");
    let twice = args.contains("--twice");

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{name} not found"));
    let source_type = SourceType::from_path(path).unwrap();

    let printed = minify(&source_text, source_type, mangle, whitespace);
    println!("{printed}");

    if twice {
        let printed = minify(&printed, source_type, mangle, whitespace);
        println!("{printed}");
    }
}

fn minify(source_text: &str, source_type: SourceType, mangle: bool, whitespace: bool) -> String {
    let allocator = Allocator::default();
    let program = Parser::new(&allocator, source_text, source_type).parse().program;
    let program = allocator.alloc(program);
    let options = MinifierOptions { mangle, ..MinifierOptions::default() };
    Minifier::new(options).build(&allocator, program);
    if whitespace {
        Codegen::<true>::new(source_text.len(), CodegenOptions).build(program)
    } else {
        Codegen::<false>::new(source_text.len(), CodegenOptions).build(program)
    }
}
