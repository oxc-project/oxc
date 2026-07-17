#![expect(clippy::print_stdout)]
//! PROTOTYPE: Measurement harness for the pointer-compression prototype.
//!
//! For each input file: parse + semantic, report arena `used_bytes()` after parse and after
//! parse+semantic, and the median wall time of 10 parse iterations and 10 parse+semantic
//! iterations (allocator reset between iterations, mirroring pooled reuse).
//!
//! Run identically on the base commit and the compressed-pointer branch to A/B:
//!
//! ```bash
//! cargo run --release -p oxc_semantic --example bench_compressed -- file1.ts file2.js ...
//! ```

use std::time::Instant;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

const ITERATIONS: usize = 10;

fn median(mut times: Vec<f64>) -> f64 {
    times.sort_by(f64::total_cmp);
    let n = times.len();
    if n % 2 == 0 { (times[n / 2 - 1] + times[n / 2]) / 2.0 } else { times[n / 2] }
}

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    assert!(!files.is_empty(), "usage: bench_compressed <file>...");

    println!(
        "{:<16} {:>12} {:>14} {:>15} {:>12} {:>17}",
        "file", "bytes", "parse_bytes", "semantic_bytes", "parse_ms", "parse+semantic_ms"
    );

    for path in &files {
        let source_text = std::fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("failed to read {path}: {err}"));
        let source_type = SourceType::from_path(path).unwrap();
        let file_name = std::path::Path::new(path).file_name().unwrap().to_string_lossy();

        // --- Memory: fresh allocator, one parse + semantic ---
        let allocator = Allocator::default();
        let (parse_bytes, semantic_bytes) = {
            let ret = Parser::new(&allocator, &source_text, source_type).parse();
            assert!(
                ret.diagnostics.iter().all(|d| !d.severity.is_error()),
                "parse errors in {path}"
            );
            let parse_bytes = allocator.used_bytes();
            let semantic_ret = SemanticBuilder::new().build(&ret.program);
            assert!(semantic_ret.diagnostics.is_empty(), "semantic errors in {path}");
            (parse_bytes, allocator.used_bytes())
        };
        drop(allocator);

        // --- Time: parse only, allocator reset between iterations ---
        let mut allocator = Allocator::default();
        let mut parse_times = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let ret = Parser::new(&allocator, &source_text, source_type).parse();
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(&ret.program);
            drop(ret);
            parse_times.push(elapsed);
            allocator.reset();
        }

        // --- Time: parse + semantic ---
        let mut both_times = Vec::with_capacity(ITERATIONS);
        for _ in 0..ITERATIONS {
            let start = Instant::now();
            let ret = Parser::new(&allocator, &source_text, source_type).parse();
            let semantic_ret = SemanticBuilder::new().build(&ret.program);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            std::hint::black_box(&semantic_ret.semantic);
            drop(semantic_ret);
            drop(ret);
            both_times.push(elapsed);
            allocator.reset();
        }

        println!(
            "{:<16} {:>12} {:>14} {:>15} {:>12.3} {:>17.3}",
            file_name,
            source_text.len(),
            parse_bytes,
            semantic_bytes,
            median(parse_times),
            median(both_times),
        );
    }
}
