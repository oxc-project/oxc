use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_formatter::{FormatOptions, Formatter};
use oxc_parser::{ParseOptions, Parser};
use oxc_tasks_common::TestFiles;

fn bench_formatter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("formatter");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;
        let mut allocator = Allocator::default();
        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                let parse_options = ParseOptions {
                    parse_regular_expression: false,
                    // Enable all syntax features
                    allow_v8_intrinsics: true,
                    allow_return_outside_function: true,
                    // `oxc_formatter` expects this to be false
                    preserve_parens: false,
                };
                let program = Parser::new(&allocator, source_text, source_type)
                    .with_options(parse_options)
                    .parse()
                    .program;
                let format_options = FormatOptions::default();
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
