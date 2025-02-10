use oxc_allocator::Allocator;
use oxc_ast::utf8_to_utf16::Utf8ToUtf16;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_parser(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("parser");
    for file in TestFiles::complicated().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = file.source_text.as_str();
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_function(id, |b| {
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
        });
    }
    group.finish();
}

fn bench_estree(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("estree");
    for file in TestFiles::complicated().files().iter().take(1) {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = file.source_text.as_str();
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        let mut allocator = Allocator::default();
        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();
                let mut program = Parser::new(&allocator, source_text, source_type)
                    .with_options(ParseOptions {
                        parse_regular_expression: true,
                        ..ParseOptions::default()
                    })
                    .parse()
                    .program;
                runner.run(|| {
                    Utf8ToUtf16::new().convert(&mut program);
                    program.to_json();
                    program
                });
            });
        });
    }
    group.finish();
}

criterion_group!(parser, bench_parser, bench_estree);
criterion_main!(parser);
