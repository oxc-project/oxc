use std::{env, path::Path};

use oxc_ast::SourceType;
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions, PrinterOptions};

// Instruction:
// create a `test.js`,
// run `cargo run -p oxc_minifier --example minifier`
// or `cargo watch -x "run -p oxc_minifier --example minifier"`

fn main() {
    let name = env::args().nth(1).unwrap_or_else(|| "test.js".to_string());
    let path = Path::new(&name);
    let source_text = std::fs::read_to_string(path).expect("{name} not found");
    let source_type = SourceType::from_path(path).unwrap();

    let options = MinifierOptions {
        compress: CompressOptions::default(),
        print: PrinterOptions { minify_whitespace: true, ..PrinterOptions::default() },
    };

    let printed = Minifier::new(&source_text, source_type, options).build();

    println!("{printed}");
}
