//! Benchmark comparing native oxc linting vs ESTree JSON deserialization path.
//!
//! This measures the overhead of the custom parser flow (Phase 3) where:
//! 1. Source is parsed externally to ESTree JSON
//! 2. JSON is deserialized into oxc AST
//! 3. Rust rules are run on the deserialized AST
//!
//! Benchmarks:
//! - `linter_native`: Full native oxc flow (parse + semantic + lint)
//! - `linter_estree`: Custom parser flow (JSON deser + semantic + lint), JSON pre-generated
//! - `linter_estree_full`: Full custom parser simulation (serialize + deserialize + semantic + lint)
//! - `estree_serialize`: Just ESTree JSON serialization
//! - `estree_deserialize`: Just ESTree JSON deserialization (from pre-parsed serde_json::Value)
//!
//! NOTE: Some test files may fail ESTree deserialization due to incomplete FromESTree
//! implementations (e.g., FormalParameters expecting "items" field but ESTree uses flat array).
//! These files are skipped in the benchmarks that require successful deserialization.

use std::{path::Path, sync::Arc};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_linter::{
    ConfigStore, ConfigStoreBuilder, ContextSubHost, ExternalParserStore, ExternalPluginStore,
    FixKind, LintOptions, Linter, ModuleRecord, lint_with_external_ast,
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;

/// Benchmark native oxc parsing + linting (baseline)
fn bench_linter_native(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter_native");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();

                let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
                let path = Path::new("");
                let semantic_ret = SemanticBuilder::new()
                    .with_scope_tree_child_ids(true)
                    .with_cfg(true)
                    .build(&parser_ret.program);
                let semantic = semantic_ret.semantic;
                let module_record =
                    Arc::new(ModuleRecord::new(path, &parser_ret.module_record, &semantic));
                let mut external_plugin_store = ExternalPluginStore::default();
                let lint_config =
                    ConfigStoreBuilder::all().build(&mut external_plugin_store).unwrap();
                let linter = Linter::new(
                    LintOptions::default(),
                    ConfigStore::new(
                        lint_config,
                        FxHashMap::default(),
                        external_plugin_store,
                        ExternalParserStore::default(),
                    ),
                    None,
                )
                .with_fix(FixKind::All);

                runner.run(|| {
                    linter.run(
                        path,
                        vec![ContextSubHost::new(semantic, Arc::clone(&module_record), 0)],
                        &allocator,
                    )
                });
            });
        });
    }
    group.finish();
}

/// Benchmark ESTree JSON deserialization + linting (custom parser path)
fn bench_linter_estree(criterion: &mut Criterion) {
    use oxc_ast::ast::Program;
    use oxc_ast::deserialize::FromESTree;

    let mut group = criterion.benchmark_group("linter_estree");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        // Pre-generate ESTree JSON outside the benchmark loop
        let allocator = Allocator::default();
        let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
        let estree_json = parser_ret.program.to_estree_js_json(true);

        // Test if deserialization works for this file before benchmarking
        let json_value: serde_json::Value = serde_json::from_str(&estree_json).unwrap();
        let test_allocator = Allocator::default();
        let test_result: Result<Program, _> = FromESTree::from_estree(&json_value, &test_allocator);
        if let Err(e) = &test_result {
            // Skip files that fail deserialization
            eprintln!("SKIPPING linter_estree/{}: {}", file.file_name, e);
            continue;
        }

        // Create linter once (it's reused across iterations)
        let mut external_plugin_store = ExternalPluginStore::default();
        let lint_config = ConfigStoreBuilder::all().build(&mut external_plugin_store).unwrap();
        let linter = Linter::new(
            LintOptions::default(),
            ConfigStore::new(
                lint_config,
                FxHashMap::default(),
                external_plugin_store,
                ExternalParserStore::default(),
            ),
            None,
        )
        .with_fix(FixKind::All);

        let path = Path::new("");

        group.bench_function(id, |b| {
            b.iter(|| {
                // This measures: JSON parsing + FromESTree deserialization + semantic build + linting
                lint_with_external_ast(&linter, path, source_text, &estree_json, None, None)
            });
        });
    }
    group.finish();
}

/// Benchmark full custom parser simulation: serialize + deserialize + lint
/// This represents the complete overhead of the custom parser path.
fn bench_linter_estree_full(criterion: &mut Criterion) {
    use oxc_ast::ast::Program;
    use oxc_ast::deserialize::FromESTree;

    let mut group = criterion.benchmark_group("linter_estree_full");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        // Test if deserialization works for this file before benchmarking
        let test_allocator = Allocator::default();
        let parser_ret = Parser::new(&test_allocator, source_text, source_type).parse();
        let estree_json = parser_ret.program.to_estree_js_json(true);
        let json_value: serde_json::Value = serde_json::from_str(&estree_json).unwrap();
        let test_allocator2 = Allocator::default();
        let test_result: Result<Program, _> =
            FromESTree::from_estree(&json_value, &test_allocator2);
        if let Err(e) = &test_result {
            // Skip files that fail deserialization
            eprintln!("SKIPPING linter_estree_full/{}: {}", file.file_name, e);
            continue;
        }

        // Create linter once (it's reused across iterations)
        let mut external_plugin_store = ExternalPluginStore::default();
        let lint_config = ConfigStoreBuilder::all().build(&mut external_plugin_store).unwrap();
        let linter = Linter::new(
            LintOptions::default(),
            ConfigStore::new(
                lint_config,
                FxHashMap::default(),
                external_plugin_store,
                ExternalParserStore::default(),
            ),
            None,
        )
        .with_fix(FixKind::All);

        let path = Path::new("");
        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();

                // Parse with oxc (simulating external parser parsing)
                let parser_ret = Parser::new(&allocator, source_text, source_type).parse();

                runner.run(|| {
                    // Serialize to ESTree JSON (simulating JS parser output)
                    let estree_json = parser_ret.program.to_estree_js_json(true);

                    // Deserialize and lint (the actual custom parser path)
                    lint_with_external_ast(&linter, path, source_text, &estree_json, None, None)
                });
            });
        });
    }
    group.finish();
}

/// Benchmark just ESTree JSON serialization
fn bench_estree_serialize_only(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("estree_serialize");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();

                let parser_ret = Parser::new(&allocator, source_text, source_type).parse();

                runner.run(|| {
                    // Just measure serialization
                    std::hint::black_box(parser_ret.program.to_estree_js_json(true))
                });
            });
        });
    }
    group.finish();
}

/// Benchmark just ESTree JSON deserialization (without linting)
fn bench_estree_deserialize_only(criterion: &mut Criterion) {
    use oxc_ast::ast::Program;
    use oxc_ast::deserialize::FromESTree;

    let mut group = criterion.benchmark_group("estree_deserialize");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        // Pre-generate ESTree JSON
        let allocator = Allocator::default();
        let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
        let estree_json = parser_ret.program.to_estree_js_json(true);

        // Pre-parse JSON to serde_json::Value (to isolate FromESTree performance)
        let json_value: serde_json::Value = serde_json::from_str(&estree_json).unwrap();

        // Test if deserialization works for this file before benchmarking
        let test_allocator = Allocator::default();
        let test_result: Result<Program, _> = FromESTree::from_estree(&json_value, &test_allocator);
        if let Err(e) = &test_result {
            // Skip files that fail deserialization (e.g., due to FormalParameters issue)
            // The linter_estree and linter_estree_full benchmarks still work because
            // lint_with_external_ast handles these internally
            eprintln!("SKIPPING estree_deserialize/{}: {}", file.file_name, e);
            continue;
        }

        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();

                runner.run(|| {
                    let _program: Result<Program, _> =
                        FromESTree::from_estree(&json_value, &allocator);
                });
            });
        });
    }
    group.finish();
}

criterion_group!(
    linter_estree,
    bench_linter_native,
    bench_linter_estree,
    bench_linter_estree_full,
    bench_estree_serialize_only,
    bench_estree_deserialize_only
);
criterion_main!(linter_estree);
