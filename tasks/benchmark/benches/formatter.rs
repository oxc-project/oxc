use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_formatter::{
    JsFormatOptions, JsdocOptions, SortImportsOptions, format_program, parse_for_format,
};
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
                // Parse in setup so the benchmark isolates formatting from parsing.
                // `parse_for_format` + `format_program` is the AST-in path for exactly this.
                let program = parse_for_format(&allocator, source_text, source_type).program;
                let format_options = JsFormatOptions {
                    sort_imports: Some(SortImportsOptions::default()),
                    jsdoc: Some(JsdocOptions::default()),
                    ..Default::default()
                };
                runner.run(|| {
                    format_program(&allocator, &program, format_options.clone(), None)
                        .print()
                        .unwrap()
                        .into_code();
                });
            });
        });
    }

    group.finish();
}

criterion_group!(formatter, bench_formatter);
criterion_main!(formatter);
