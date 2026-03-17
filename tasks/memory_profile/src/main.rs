use std::mem::size_of;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;

#[expect(clippy::print_stdout)]
fn main() {
    // Print AST enum sizes — this is what TaggedPtr integration will shrink
    println!("=== AST Enum Sizes ===");
    println!("  Expression:          {} bytes", size_of::<Expression>());
    println!("  Option<Expression>:  {} bytes", size_of::<Option<Expression>>());
    println!("  Statement:           {} bytes", size_of::<Statement>());
    println!("  Option<Statement>:   {} bytes", size_of::<Option<Statement>>());
    println!("  Declaration:         {} bytes", size_of::<Declaration>());
    println!("  TSType:              {} bytes", size_of::<TSType>());
    println!("  Option<TSType>:      {} bytes", size_of::<Option<TSType>>());
    println!("  MemberExpression:    {} bytes", size_of::<MemberExpression>());
    println!("  BindingPattern:      {} bytes", size_of::<BindingPattern>());
    println!("  ForStatementInit:    {} bytes", size_of::<ForStatementInit>());
    println!("  AssignmentTarget:    {} bytes", size_of::<AssignmentTarget>());
    println!();

    // Print key AST node sizes that contain inline Expression/Statement fields
    println!("=== Key AST Node Sizes (affected by enum shrinkage) ===");
    println!("  BinaryExpression:      {} bytes", size_of::<BinaryExpression>());
    println!("  ConditionalExpression: {} bytes", size_of::<ConditionalExpression>());
    println!("  AssignmentExpression:  {} bytes", size_of::<AssignmentExpression>());
    println!("  CallExpression:        {} bytes", size_of::<CallExpression>());
    println!("  UnaryExpression:       {} bytes", size_of::<UnaryExpression>());
    println!("  BindingPattern:        {} bytes", size_of::<BindingPattern>());
    println!("  VariableDeclarator:    {} bytes", size_of::<VariableDeclarator>());
    println!("  ReturnStatement:       {} bytes", size_of::<ReturnStatement>());
    println!();

    // Measure arena memory for real-world files
    println!("=== Arena Memory (parse + semantic) ===");
    let files = TestFiles::complicated();
    let parse_options = ParseOptions { parse_regular_expression: true, ..ParseOptions::default() };
    let mut allocator = Allocator::default();

    // Warm up so the bump allocator pre-allocates enough space
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
