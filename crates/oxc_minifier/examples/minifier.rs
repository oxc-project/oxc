use std::path::Path;

use oxc_minifier::{Minifier, MinifierOptions};
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
// or `cargo watch -x "run -p oxc_minifier --example minifier"`

fn main() {
    let mut args = Arguments::from_env();

    let name = args.subcommand().ok().flatten().unwrap_or_else(|| String::from("test.js"));
    let mangle = args.contains("--mangle");
    let twice = args.contains("--twice");

    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).unwrap_or_else(|_| panic!("{name} not found"));
    let source_type = SourceType::from_path(path).unwrap();

    let options = MinifierOptions { mangle, ..MinifierOptions::default() };
    let printed = Minifier::new(&source_text, source_type, options).build();
    println!("{printed}");

    if twice {
        let options = MinifierOptions { mangle, ..MinifierOptions::default() };
        let printed = Minifier::new(&printed, source_type, options).build();
        println!("{printed}");
    }
}
