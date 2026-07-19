#![expect(clippy::print_stdout)]
//! Measurement harness for the tagged-pointer `Expression` prototype.
//!
//! For each benchmark file: parses + runs semantic analysis, prints `allocator.used_bytes()`
//! after parsing and the median wall time of 10 parse(+semantic) iterations.
//!
//! ```bash
//! cargo run --release -p oxc_semantic --example bench_tagged
//! ```

use std::{path::Path, time::Instant};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

const FILES: &[&str] = &[
    "/Users/boshen/github/oxc-project/benchmark-files/checker.ts",
    "/Users/boshen/github/oxc-project/benchmark-files/cal.com.tsx",
    "/Users/boshen/github/oxc-project/benchmark-files/pdf.mjs",
    "/Users/boshen/github/oxc-project/benchmark-files/antd.js",
    "/Users/boshen/github/oxc-project/benchmark-files/binder.ts",
];

const ITERATIONS: usize = 10;

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        f64::midpoint(values[mid - 1], values[mid])
    } else {
        values[mid]
    }
}

fn main() {
    println!("{:<14} {:>14} {:>12} {:>16}", "file", "arena_bytes", "parse_ms", "parse+semantic_ms");
    for file in FILES {
        let path = Path::new(file);
        let source_text = std::fs::read_to_string(path).unwrap();
        let source_type = SourceType::from_path(path).unwrap();

        let mut arena_bytes = 0_usize;
        let mut parse_times = Vec::with_capacity(ITERATIONS);
        let mut total_times = Vec::with_capacity(ITERATIONS);

        for i in 0..ITERATIONS {
            let allocator = Allocator::default();
            let start = Instant::now();
            let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
            let parse_elapsed = start.elapsed().as_secs_f64() * 1000.0;
            assert!(!parser_ret.panicked, "parser panicked in {file}");
            if i == 0 {
                arena_bytes = allocator.used_bytes();
            }
            let _semantic_ret = SemanticBuilder::new().build(&parser_ret.program);
            let total_elapsed = start.elapsed().as_secs_f64() * 1000.0;
            parse_times.push(parse_elapsed);
            total_times.push(total_elapsed);
        }

        let name = path.file_name().unwrap().to_str().unwrap();
        println!(
            "{:<14} {:>14} {:>12.2} {:>16.2}",
            name,
            arena_bytes,
            median(parse_times),
            median(total_times),
        );
    }
}
