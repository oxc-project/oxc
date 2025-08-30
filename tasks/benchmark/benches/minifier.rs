use std::path::Path;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_mangler::Mangler;
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;
use oxc_transformer::{TransformOptions, Transformer};

fn bench_minifier(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("minifier");

    for file in TestFiles::minimal().files().iter().skip(1) {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        // Create `Allocator` outside of `bench_function`, so same allocator is used for
        // both the warmup and measurement phases
        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                // Reset allocator at start of each iteration
                allocator.reset();

                // Create fresh AST + semantic data for each iteration
                let mut program = Parser::new(&allocator, source_text, source_type).parse().program;
                let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();

                // Minifier only works on esnext.
                let transform_options = TransformOptions::from_target("esnext").unwrap();
                let transformer_ret =
                    Transformer::new(&allocator, Path::new(&file.file_name), &transform_options)
                        .build_with_scoping(scoping, &mut program);
                assert!(transformer_ret.errors.is_empty());
                let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();

                let options = CompressOptions::smallest();
                runner.run(|| {
                    Compressor::new(&allocator).build_with_scoping(&mut program, scoping, options);
                });
            });
        });
    }

    group.finish();
}

fn bench_mangler(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("mangler");
    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        let source_text = file.source_text.as_str();
        let mut allocator = Allocator::default();
        let mut temp_allocator = Allocator::default();
        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                temp_allocator.reset();
                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let mut semantic =
                    SemanticBuilder::new().with_scope_tree_child_ids(true).build(&program).semantic;
                runner.run(|| {
                    Mangler::new_with_temp_allocator(&temp_allocator)
                        .build_with_semantic(&mut semantic, &program);
                });
            });
        });
    }
    group.finish();
}

criterion_group!(minifier, bench_minifier, bench_mangler);
criterion_main!(minifier);
