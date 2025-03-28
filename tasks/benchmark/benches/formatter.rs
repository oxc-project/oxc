use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_formatter::{Formatter, FormatterOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_formatter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("formatter");
    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = file.source_text.as_str();
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_function(id, |b| {
            b.iter(|| {
                let allocator1 = Allocator::default();
                let allocator2 = Allocator::default();
                let ret = Parser::new(&allocator1, source_text, source_type).parse();
                let _ =
                    Formatter::new(&allocator2, FormatterOptions::default()).build(&ret.program);
            });
        });
    }
    group.finish();
}

criterion_group!(formatter, bench_formatter);
criterion_main!(formatter);
