#![expect(clippy::print_stdout)]
//! Measures formatter arena memory usage by loading each bench fixture, parsing it
//! into a fresh `Allocator`, recording `used_bytes`, formatting, and recording
//! `used_bytes` again. The delta is the formatter's own arena footprint.
//!
//! Run on this branch and on `main` to compare.
//!
//! ```bash
//! cargo run --release -p oxc_formatter --example mem_usage --features=oxc_tasks_common
//! ```

use oxc_allocator::Allocator;
use oxc_formatter::{
    FormatOptions, Formatter, JsdocOptions, SortImportsOptions, get_parse_options,
};
use oxc_parser::Parser;
use oxc_tasks_common::TestFiles;

fn main() {
    let mut allocator = Allocator::default();
    println!(
        "{:<40}  {:>12}  {:>12}  {:>12}  {:>8}",
        "file", "ast bytes", "fmt bytes", "total bytes", "fmt/ast"
    );
    println!("{}", "-".repeat(94));

    for file in TestFiles::formatter().files() {
        allocator.reset();

        let program = Parser::new(&allocator, &file.source_text, file.source_type)
            .with_options(get_parse_options())
            .parse()
            .program;
        let ast_bytes = allocator.used_bytes();

        let format_options = FormatOptions {
            sort_imports: Some(SortImportsOptions::default()),
            jsdoc: Some(JsdocOptions::default()),
            ..Default::default()
        };
        Formatter::new(&allocator, format_options).build(&program);

        let total_bytes = allocator.used_bytes();
        let fmt_bytes = total_bytes - ast_bytes;
        #[expect(clippy::cast_precision_loss)]
        let ratio = (fmt_bytes as f64) / (ast_bytes as f64);

        println!(
            "{:<40}  {:>12}  {:>12}  {:>12}  {:>7.2}x",
            file.file_name, ast_bytes, fmt_bytes, total_bytes, ratio
        );
    }
}
