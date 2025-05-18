use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::TestFile;

fn bench_isolated_declarations(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("isolated-declarations");

    let file =
        TestFile::new("https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/vue-id.ts");

    let id = BenchmarkId::from_parameter(&file.file_name);
    let source_text = file.source_text.as_str();
    let source_type = SourceType::from_path(&file.file_name).unwrap();

    let ast_allocator = Allocator::new();
    let program = Parser::new(&ast_allocator, source_text, source_type).parse().program;
    let program = black_box(program);

    // Create `Allocator` outside of `bench_function`, so same allocator is used for
    // both the warmup and measurement phases
    let mut output_allocator = Allocator::new();
    group.bench_function(id, |b| {
        b.iter_with_setup_wrapper(|runner| {
            // Reset allocator at start of each iteration
            output_allocator.reset();

            // Include dropping `IsolatedDeclarations::build`'s return value in benchmark timing.
            // Drop time is part of the cost of using this API.
            runner.run(|| {
                let options = IsolatedDeclarationsOptions { strip_internal: true };
                let ret = IsolatedDeclarations::new(&output_allocator, options).build(&program);
                black_box(ret);
            });
        });
    });

    group.finish();
}

criterion_group!(transformer, bench_isolated_declarations);
criterion_main!(transformer);
