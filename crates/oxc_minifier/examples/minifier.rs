use std::{env, path::Path};

use oxc_minifier::{Minifier, MinifierOptions, PrinterOptions};
use oxc_span::SourceType;

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example minifier`
// or `cargo watch -x "run -p oxc_minifier --example minifier"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).expect("{name} not found");
    let source_type = SourceType::from_path(path).unwrap();

    println!("------------------------------");
    let options = MinifierOptions {
        mangle: false,
        print: PrinterOptions { minify_whitespace: false, ..PrinterOptions::default() },
        ..MinifierOptions::default()
    };
    let printed = Minifier::new(&source_text, source_type, options).build();
    println!("{printed}");

    println!("------ Mangle ----------------");
    let options = MinifierOptions {
        print: PrinterOptions { minify_whitespace: false, ..PrinterOptions::default() },
        ..MinifierOptions::default()
    };
    let printed = Minifier::new(&source_text, source_type, options).build();
    println!("{printed}");

    println!("------ Minify Whitespace -----");
    let options = MinifierOptions {
        mangle: false,
        print: PrinterOptions { minify_whitespace: true, ..PrinterOptions::default() },
        ..MinifierOptions::default()
    };
    let printed = Minifier::new(&source_text, source_type, options).build();
    println!("{printed}");
    println!("------------------------------");
}
