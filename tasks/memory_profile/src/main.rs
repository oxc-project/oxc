use std::mem::size_of;

use oxc_allocator::{Allocator, TaggedPtr};
use oxc_ast::ast::*;
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;

#[expect(clippy::print_stdout)]
fn main() {
    // Show TaggedPtr size — the building block for compact AST enums
    println!("=== TaggedPtr Sizes ===");
    println!("  TaggedPtr:           {} bytes", size_of::<TaggedPtr>());
    println!("  Option<TaggedPtr>:   {} bytes", size_of::<Option<TaggedPtr>>());
    println!();

    // Current AST enum sizes (16 bytes) — these will shrink to 8 bytes with TaggedPtr
    println!("=== Current AST Enum Sizes (target: 8 bytes each) ===");
    println!("  Expression:          {} bytes -> 8 bytes", size_of::<Expression>());
    println!("  Option<Expression>:  {} bytes -> 8 bytes", size_of::<Option<Expression>>());
    println!("  Statement:           {} bytes -> 8 bytes", size_of::<Statement>());
    println!("  Declaration:         {} bytes -> 8 bytes", size_of::<Declaration>());
    println!("  TSType:              {} bytes -> 8 bytes", size_of::<TSType>());
    println!("  MemberExpression:    {} bytes -> 8 bytes", size_of::<MemberExpression>());
    println!("  ForStatementInit:    {} bytes -> 8 bytes", size_of::<ForStatementInit>());
    println!("  AssignmentTarget:    {} bytes -> 8 bytes", size_of::<AssignmentTarget>());
    println!();

    // Key node sizes that will cascade down
    println!("=== Key Node Sizes (current -> projected with 8-byte enums) ===");
    let binary = size_of::<BinaryExpression>();
    let cond = size_of::<ConditionalExpression>();
    let assign = size_of::<AssignmentExpression>();
    let call = size_of::<CallExpression>();
    let unary = size_of::<UnaryExpression>();
    let ret = size_of::<ReturnStatement>();
    println!("  BinaryExpression:      {:2} bytes -> {:2} bytes (-{:2})", binary, binary - 16, 16);
    println!("  ConditionalExpression: {:2} bytes -> {:2} bytes (-{:2})", cond, cond - 24, 24);
    println!("  AssignmentExpression:  {:2} bytes -> {:2} bytes (-{:2})", assign, assign - 16, 16);
    println!("  CallExpression:        {:2} bytes -> {:2} bytes (- 8)", call, call - 8);
    println!("  UnaryExpression:       {:2} bytes -> {:2} bytes (- 8)", unary, unary - 8);
    println!("  ReturnStatement:       {:2} bytes -> {:2} bytes (- 8)", ret, ret - 8);
    println!();

    // Measure arena memory for real-world files
    println!("=== Arena Memory (parse + semantic) ===");
    let files = TestFiles::complicated();
    let parse_options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };
    let mut allocator = Allocator::default();

    // Warm up
    for file in files.files() {
        allocator.reset();
        let parsed = Parser::new(&allocator, &file.source_text, file.source_type)
            .with_options(parse_options)
            .parse();
        let _ = SemanticBuilder::new().build(&parsed.program);
    }

    for file in files.files() {
        allocator.reset();

        let parsed = Parser::new(&allocator, &file.source_text, file.source_type)
            .with_options(parse_options)
            .parse();
        assert!(parsed.errors.is_empty(), "Parse errors in {}", file.file_name);

        let arena_after_parse = allocator.used_bytes();
        let _semantic = SemanticBuilder::new().build(&parsed.program);
        let arena_after_semantic = allocator.used_bytes();

        println!(
            "  {:40} src: {:>10} bytes  parse: {:>10} bytes  semantic: {:>10} bytes",
            file.file_name,
            file.source_text.len(),
            arena_after_parse,
            arena_after_semantic,
        );
    }
}
