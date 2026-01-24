use std::path::Path;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_mangler::{MangleOptions, Mangler};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;
use oxc_transformer::{TransformOptions, Transformer};

/// Benchmark the complete compilation pipeline:
/// allocate -> parse -> semantic -> transform -> minify -> mangle -> codegen -> drop
fn bench_pipeline(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("pipeline");

    for file in TestFiles::minimal().files() {
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

                // Create options inside the closure to avoid move issues
                let transform_options = TransformOptions::from_target("esnext").unwrap();
                let compress_options = CompressOptions::smallest();
                let mangle_options = MangleOptions::default();
                let codegen_options = CodegenOptions::default();

                runner.run(|| {
                    // Parse
                    let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
                    let mut program = parser_ret.program;

                    // Semantic
                    let scoping = SemanticBuilder::new()
                        .with_excess_capacity(2.0)
                        .build(&program)
                        .semantic
                        .into_scoping();

                    // Transform
                    let transformer_ret = Transformer::new(
                        &allocator,
                        Path::new(&file.file_name),
                        &transform_options,
                    )
                    .build_with_scoping(scoping, &mut program);
                    let _scoping = transformer_ret.scoping;

                    // Compress (minify) - rebuilds semantic internally
                    Compressor::new(&allocator).build(&mut program, compress_options);

                    // Mangle - rebuilds semantic internally
                    let mangler_ret = Mangler::new().with_options(mangle_options).build(&program);

                    // Codegen
                    Codegen::new()
                        .with_options(codegen_options)
                        .with_scoping(Some(mangler_ret.scoping))
                        .with_private_member_mappings(Some(mangler_ret.class_private_mappings))
                        .build(&program)
                });
            });
        });
    }

    group.finish();
}

criterion_group!(pipeline, bench_pipeline);
criterion_main!(pipeline);
