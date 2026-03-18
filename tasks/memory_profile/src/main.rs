use std::mem::size_of;
use std::time::Instant;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::{ParseOptions, Parser};
use oxc_tasks_common::TestFiles;

#[expect(clippy::print_stdout)]
fn main() {
    println!("=== AST Type Sizes ===");
    println!("  Expression:          {} bytes", size_of::<Expression>());
    println!("  Statement:           {} bytes", size_of::<Statement>());
    println!("  Option<Expression>:  {} bytes", size_of::<Option<Expression>>());
    println!("  Option<Statement>:   {} bytes", size_of::<Option<Statement>>());
    println!("  Declaration:         {} bytes", size_of::<Declaration>());
    println!("  BinaryExpression:    {} bytes", size_of::<BinaryExpression>());
    println!("  ConditionalExpr:     {} bytes", size_of::<ConditionalExpression>());
    println!("  IfStatement:         {} bytes", size_of::<IfStatement>());
    println!("  ReturnStatement:     {} bytes", size_of::<ReturnStatement>());
    println!();

    let files = TestFiles::complicated();
    let opts = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };
    let mut alloc = Allocator::default();

    // Warmup
    for _ in 0..3 {
        for f in files.files() {
            alloc.reset();
            let _ = Parser::new(&alloc, &f.source_text, f.source_type).with_options(opts).parse();
        }
    }

    println!("=== Arena Memory (parse only) ===");
    for f in files.files() {
        alloc.reset();
        let parsed = Parser::new(&alloc, &f.source_text, f.source_type).with_options(opts).parse();
        assert!(parsed.errors.is_empty(), "Parse errors in {}", f.file_name);
        println!(
            "  {:40} src: {:>10} bytes  parse: {:>10} bytes",
            f.file_name, f.source_text.len(), alloc.used_bytes(),
        );
    }
    println!();

    println!("=== Parser Performance ===");
    for f in files.files() {
        let iterations = if f.source_text.len() > 1_000_000 { 20 } else { 100 };
        let start = Instant::now();
        for _ in 0..iterations {
            alloc.reset();
            let _ = Parser::new(&alloc, &f.source_text, f.source_type).with_options(opts).parse();
        }
        let per_iter = start.elapsed() / iterations as u32;
        println!(
            "  {:40} {:>8.2} µs/iter  ({} iters)",
            f.file_name, per_iter.as_nanos() as f64 / 1000.0, iterations
        );
    }
}
