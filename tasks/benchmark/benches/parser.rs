use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_benchmark::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use oxc_estree_tokens::{EstreeTokenOptions, to_estree_tokens_json};
use oxc_parser::{ParseOptions, Parser, ParserReturn, config::RuntimeParserConfig};
use oxc_tasks_common::TestFiles;

fn bench_parser(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("parser");

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

fn bench_parser_tokens(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("parser_tokens");

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
                // Use `RuntimeParserConfig` (runtime config), same as NAPI parser package will.
                // `bench_parser` uses `NoTokensParserConfig` (implicitly as default).
                // Usually it's inadvisable to use 2 different configs in the same application,
                // but this is just a benchmark, and it's better if we don't entwine this benchmark with `bench_parser`.
                let config = RuntimeParserConfig::new(true);

                Parser::new(&allocator, source_text, source_type)
                    .with_options(ParseOptions {
                        parse_regular_expression: true,
                        ..ParseOptions::default()
                    })
                    .with_config(config)
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
        let source_text = &file.source_text;
        let source_type = file.source_type;
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
                    let span_converter = Utf8ToUtf16::new(program.source_text);
                    span_converter.convert_program(&mut program);
                    span_converter.convert_comments(&mut program.comments);

                    black_box(program.to_estree_ts_json_with_fixes(false));
                    program
                });
            });
        });
    }

    group.finish();
}

fn bench_estree_tokens(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("estree_tokens");

    for file in TestFiles::complicated().files().iter().take(1) {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;
        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                allocator.reset();

                // Use `RuntimeParserConfig` (runtime config), same as NAPI parser package will.
                // `bench_estree` uses `NoTokensParserConfig` (implicitly as default).
                // Usually it's inadvisable to use 2 different configs in the same application,
                // but this is just a benchmark, and it's better if we don't entwine this benchmark with `bench_estree`.
                let config = RuntimeParserConfig::new(true);

                let ret = Parser::new(&allocator, source_text, source_type)
                    .with_options(ParseOptions {
                        parse_regular_expression: true,
                        ..ParseOptions::default()
                    })
                    .with_config(config)
                    .parse();
                let ParserReturn { program, tokens, .. } = ret;

                // Creating span converter is not performed in measured section, as we only want to measure tokens.
                // Span converter needs to be created anyway for serializing AST.
                let span_converter = Utf8ToUtf16::new(program.source_text);

                runner.run(|| {
                    let tokens_json = to_estree_tokens_json(
                        &tokens,
                        &program,
                        program.source_text,
                        &span_converter,
                        EstreeTokenOptions::test262(),
                    );
                    let tokens_json = black_box(tokens_json);
                    // Allocate tokens JSON into arena, same as linter and NAPI parser package do
                    let _tokens_json = allocator.alloc_str(&tokens_json);

                    program
                });
            });
        });
    }

    group.finish();
}

criterion_group!(parser, bench_parser, bench_parser_tokens, bench_estree, bench_estree_tokens);
criterion_main!(parser);
