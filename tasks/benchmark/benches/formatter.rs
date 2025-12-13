use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_formatter::{FormatOptions, Formatter, SortImportsOptions, get_parse_options};
use oxc_parser::Parser;
use oxc_tasks_common::TestFiles;

fn bench_formatter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("formatter");

    for file in TestFiles::formatter().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;
        let mut allocator = Allocator::default();
        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                let program = Parser::new(&allocator, source_text, source_type)
                    .with_options(get_parse_options())
                    .parse()
                    .program;
                let format_options = FormatOptions {
                    experimental_sort_imports: Some(SortImportsOptions::default()),
                    ..Default::default()
                };
                runner.run(|| {
                    Formatter::new(&allocator, format_options).build(&program);
                });
            });
        });
    }

    group.finish();
}

criterion_group!(formatter, bench_formatter);
criterion_main!(formatter);
