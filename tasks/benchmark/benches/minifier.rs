use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_minifier::{CompressOptions, Compressor};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_minifier(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("minifier");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        let source_text = file.source_text.as_str();

        // Create `Allocator` outside of `bench_function`, so same allocator is used for
        // both the warmup and measurement phases
        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                // Reset allocator at start of each iteration
                allocator.reset();

                // Create fresh AST + semantic data for each iteration
                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let program = allocator.alloc(program);
                let (symbols, scopes) = SemanticBuilder::new()
                    .build(program)
                    .semantic
                    .into_symbol_table_and_scope_tree();

                let options = CompressOptions::all_true();

                runner.run(|| {
                    Compressor::new(&allocator, options)
                        .build_with_symbols_and_scopes(symbols, scopes, program);
                });
            });
        });
    }

    group.finish();
}

criterion_group!(minifier, bench_minifier);
criterion_main!(minifier);
