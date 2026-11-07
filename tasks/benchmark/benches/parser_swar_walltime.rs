use codspeed_criterion_compat::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_allocator::Allocator;
use oxc_benchmark as _;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_parser(criterion: &mut Criterion) {
    let variant = std::env::var("PARSER_SWAR_VARIANT").unwrap_or_else(|_| "default".into());
    let mut group = criterion.benchmark_group(format!("parser_{variant}"));

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

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

fn bench_string_scanning(criterion: &mut Criterion) {
    let variant = std::env::var("PARSER_SWAR_VARIANT").unwrap_or_else(|_| "default".into());
    let mut group = criterion.benchmark_group(format!("parser_string_scanning_{variant}"));
    for (name, literal) in [
        ("short_strings", format!("\"{}\"", "a".repeat(8))),
        ("medium_strings", format!("\"{}\"", "a".repeat(64))),
        ("long_strings", format!("\"{}\"", "a".repeat(4096))),
        ("unicode_strings", format!("\"{}中😀\"", "a".repeat(256))),
        ("escaped_strings", format!("\"{}\\n{}\"", "a".repeat(128), "b".repeat(128))),
        ("short_templates", format!("`{}`", "a".repeat(8))),
        ("long_templates", format!("`{}`", "a".repeat(4096))),
        ("template_substitutions", format!("`{}${{value}}{}`", "a".repeat(128), "b".repeat(128))),
    ] {
        let source = format!("[{}];", format!("{literal},").repeat(262_144 / (literal.len() + 1)));
        let source_type = SourceType::mjs();
        let mut allocator = Allocator::default();
        let ret = Parser::new(&allocator, &source, source_type).parse();
        assert!(ret.diagnostics.is_empty());
        allocator.reset();
        group.bench_function(name, |b| {
            b.iter(|| {
                Parser::new(&allocator, &source, source_type).parse();
                allocator.reset();
            });
        });
    }
    group.finish();
}

criterion_group!(parser_swar_walltime, bench_parser, bench_string_scanning);
criterion_main!(parser_swar_walltime);
