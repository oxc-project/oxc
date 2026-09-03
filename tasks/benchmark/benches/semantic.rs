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
                    // real-world code has no errors, so allocating and dropping them is atypical and
                    // we don't want to include it in benchmark time.
                    let ret = SemanticBuilder::new_compiler().build(&program);
                    let ret = black_box(ret);
                    ret.diagnostics
                });
            });
        });

        let comments_id = BenchmarkId::new("comments", &file.file_name);
        group.bench_function(comments_id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();

                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let program = black_box(program);

                runner.run(|| {
                    let ret = SemanticBuilder::new_compiler()
                        .with_comment_attachments(true)
                        .build(&program);
                    let ret = black_box(ret);
                    // Drop `Semantic` inside the measured section, matching the
                    // baseline. Attachments and diagnostics normally outlive
                    // semantic construction, so drop them outside it.
                    (ret.comment_attachments, ret.diagnostics)
                });
            });
        });
    }
    group.finish();
}

criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
