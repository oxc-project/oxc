use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::Parser;
use oxc_prettier::{Prettier, PrettierOptions};
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_prettier(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("prettier");
    for file in TestFiles::minimal().files() {
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                b.iter(|| {
                    let allocator1 = Allocator::default();
                    let allocator2 = Allocator::default();
                    let ret = Parser::new(&allocator1, source_text, source_type).parse();
                    let _ =
                        Prettier::new(&allocator2, PrettierOptions::default()).build(&ret.program);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(prettier, bench_prettier);
criterion_main!(prettier);
