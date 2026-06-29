//! Format a CSS/SCSS/Less file and print the result.
//!
//! ```sh
//! cargo run -p oxc_formatter_css --example css_formatter -- [filename]
//! ```
#![expect(clippy::print_stdout, clippy::print_stderr)]

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_css::{CssFormatOptions, CssVariant, format};

fn main() {
    let mut args = pico_args::Arguments::from_env();
    let name = args.free_from_str().unwrap_or_else(|_| "test.css".to_string());
    let path = Path::new(&name);

    let source_text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {err}", path.display()));

    let variant = match path.extension().and_then(|e| e.to_str()) {
        Some("scss") => CssVariant::Scss,
        Some("less") => CssVariant::Less,
        _ => CssVariant::Css,
    };

    let options = CssFormatOptions { variant, ..CssFormatOptions::default() };

    let allocator = Allocator::new();
    match format(&allocator, &source_text, options) {
        Ok(formatted) => {
            if std::env::var("DUMP_IR").is_ok() {
                for el in formatted.document().elements() {
                    eprintln!("{el:?}");
                }
            }
            match formatted.print() {
                Ok(printed) => print!("{}", printed.into_code()),
                Err(err) => eprintln!("Print error: {err:?}"),
            }
        }
        Err(diagnostic) => eprintln!("Parse error: {diagnostic:?}"),
    }
}
