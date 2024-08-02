use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_minifier::{Minifier, MinifierOptions, RemoveParens};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_minifier(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("minifier");
    for file in TestFiles::minimal().files() {
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                let options = MinifierOptions::default();
                b.iter_with_large_drop(|| {
                    let allocator = Allocator::default();
                    let program = Parser::new(&allocator, source_text, source_type).parse().program;
                    let program = allocator.alloc(program);
                    Minifier::new(options).build(&allocator, program);
                    allocator
                });
            },
        );
    }
    group.finish();
}

fn bench_passes(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("prepass");

    for file in TestFiles::minimal().files() {
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                let allocator = Allocator::default();
                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let program = allocator.alloc(program);
                b.iter(|| RemoveParens::new(&allocator).build(program));
            },
        );
    }

    group.finish();
}

criterion_group!(minifier, bench_minifier, bench_passes);
criterion_main!(minifier);
