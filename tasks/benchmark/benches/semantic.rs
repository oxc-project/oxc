mod comment_inputs;

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
    }
    group.finish();
}

fn bench_comment_input(
    criterion: &mut Criterion,
    name: &str,
    source: &str,
    source_type: oxc_span::SourceType,
) {
    let mut group = criterion.benchmark_group(format!("semantic_comments/{name}"));
    let mut allocator = Allocator::default();
    group.bench_function("build", |b| {
        b.iter_with_setup_wrapper(|runner| {
            allocator.reset();
            let parsed = Parser::new(&allocator, source, source_type).parse();
            assert!(parsed.diagnostics.is_empty(), "{name}: {:?}", parsed.diagnostics);
            let program = black_box(parsed.program);
            runner.run(|| black_box(SemanticBuilder::new_compiler().build(&program)).diagnostics);
        });
    });
    group.bench_function("parse_and_build", |b| {
        b.iter(|| {
            let program = Parser::new(&allocator, black_box(source), source_type).parse().program;
            black_box(SemanticBuilder::new_compiler().build(&program));
            allocator.reset();
        });
    });
    group.finish();
}

fn bench_semantic_comments(criterion: &mut Criterion) {
    for (name, source) in comment_inputs::inputs() {
        bench_comment_input(criterion, name, &source, oxc_span::SourceType::mjs());
    }
    for file in TestFiles::minimal().files() {
        bench_comment_input(criterion, &file.file_name, &file.source_text, file.source_type);
    }
}

criterion_group!(semantic, bench_semantic, bench_semantic_comments);
criterion_main!(semantic);
