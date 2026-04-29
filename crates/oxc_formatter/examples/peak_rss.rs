#![expect(clippy::print_stdout)]
//! Minimal binary for peak-RSS measurement: parses + formats a single file then exits.
//! Use with `/usr/bin/time -l` on macOS to get "maximum resident set size".
//!
//! ```bash
//! /usr/bin/time -l target/release/examples/peak_rss target/App.tsx 2>&1 | tail -5
//! ```

use std::{env, fs, hint::black_box, path::Path};

use oxc_allocator::Allocator;
use oxc_formatter::{
    FormatOptions, Formatter, JsdocOptions, SortImportsOptions, get_parse_options,
};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn main() {
    let path = env::args().nth(1).expect("usage: peak_rss <file>");
    let source_text = fs::read_to_string(&path).expect("read source");
    let source_type = SourceType::from_path(Path::new(&path)).unwrap();

    let allocator = Allocator::default();
    let program = Parser::new(&allocator, &source_text, source_type)
        .with_options(get_parse_options())
        .parse()
        .program;
    let format_options = FormatOptions {
        sort_imports: Some(SortImportsOptions::default()),
        jsdoc: Some(JsdocOptions::default()),
        ..Default::default()
    };
    let printed = Formatter::new(&allocator, format_options).build(&program);
    // Black-box the result so the optimiser can't elide work or release the buffer early.
    black_box(printed);
    println!("done");
}
