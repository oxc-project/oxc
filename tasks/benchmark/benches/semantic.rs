use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;

fn bench_semantic(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("semantic");
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

                // Create fresh AST for each iteration, as `SemanticBuilder` alters the AST
                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let program = black_box(program);

                runner.run(|| {
                    // We drop `Semantic` inside this closure as drop time is part of cost of using this API.
                    // We return `errors` to be dropped outside of the measured section, as usually
                    // code would have no errors. One of our benchmarks `cal.com.tsx` has a lot of errors,
                    // but that's atypical, so don't want to include it in benchmark time.
                    let ret = SemanticBuilder::new().with_check_syntax_error(true).build(&program);
                    let ret = black_box(ret);
                    ret.errors
                });
            });
        });
    }
    group.finish();
}

/// Mirrors rolldown's repeated scoping rebuilds after transform passes invalidate prior `Scoping`.
/// Compares the previous full semantic rebuild against rebuilding with recycled `Scoping` storage.
fn bench_semantic_rebuild(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("semantic_rebuild");
    for file in TestFiles::minimal().files() {
        let source_text = &file.source_text;
        let source_type = file.source_type;

        let mut allocator = Allocator::default();

        // Compute `Stats` once on a throwaway build so subsequent rebuilds can pass them in via
        // `with_stats`, matching rolldown's preprocessing pipeline.
        let stats = {
            allocator.reset();
            let program = Parser::new(&allocator, source_text, source_type).parse().program;
            SemanticBuilder::new().with_enum_eval(true).build(&program).semantic.stats()
        };

        // One-shot: print used-bytes for one recycled 4-rebuild sequence so the storage-portion
        // signal is visible alongside criterion's wall-time.
        {
            allocator.reset();
            let program = Parser::new(&allocator, source_text, source_type).parse().program;
            let mut stats = stats;
            let mut recycled_scoping = None;
            for _ in 0..4 {
                let mut builder = SemanticBuilder::new().with_enum_eval(true).with_stats(stats);
                if let Some(scoping) = recycled_scoping.take() {
                    builder = builder.with_recycled_scoping(scoping);
                }
                let ret = black_box(builder.build_scoping(&program));
                stats = ret.stats;
                recycled_scoping = Some(ret.scoping);
            }
            eprintln!(
                "semantic_rebuild[{}]: allocator used_bytes after 4x recycled scoping rebuild = {}",
                file.file_name,
                allocator.used_bytes()
            );
        }

        group.bench_function(BenchmarkId::new("full", &file.file_name), |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let program = black_box(program);

                runner.run(|| {
                    let mut last_errors = Vec::new();
                    let mut stats = stats;
                    for _ in 0..4 {
                        let ret = SemanticBuilder::new()
                            .with_enum_eval(true)
                            .with_stats(stats)
                            .build(&program);
                        stats = ret.semantic.stats();
                        last_errors = black_box(ret).errors;
                    }
                    last_errors
                });
            });
        });

        group.bench_function(BenchmarkId::new("build_scoping_recycled", &file.file_name), |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let program = black_box(program);

                runner.run(|| {
                    let mut last_errors = Vec::new();
                    let mut stats = stats;
                    let mut recycled_scoping = None;
                    for _ in 0..4 {
                        let mut builder =
                            SemanticBuilder::new().with_enum_eval(true).with_stats(stats);
                        if let Some(scoping) = recycled_scoping.take() {
                            builder = builder.with_recycled_scoping(scoping);
                        }
                        let ret = black_box(builder.build_scoping(&program));
                        stats = ret.stats;
                        last_errors = ret.errors;
                        recycled_scoping = Some(ret.scoping);
                    }
                    last_errors
                });
            });
        });
    }
    group.finish();
}

criterion_group!(semantic, bench_semantic, bench_semantic_rebuild);
criterion_main!(semantic);
