use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_parser(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("parser");
    for file in TestFiles::complicated().files() {
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                // Do not include initializing allocator in benchmark.
                // User code would likely reuse the same allocator over and over to parse multiple files,
                // so we do the same here.
                let mut allocator = Allocator::default();
                b.iter(|| {
                    Parser::new(&allocator, source_text, source_type)
                        .with_options(ParseOptions {
                            parse_regular_expression: true,
                            ..ParseOptions::default()
                        })
                        .parse();
                    allocator.reset();
                });
            },
        );
    }
    group.finish();
}

criterion_group!(parser, bench_parser);
criterion_main!(parser);
