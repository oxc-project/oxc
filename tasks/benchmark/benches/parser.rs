use std::fmt::Write;

use oxc_allocator::Allocator;
use oxc_ast_visit::utf8_to_utf16::Utf8ToUtf16;
use oxc_benchmark::{
    BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use oxc_estree_tokens::{ESTreeTokenOptionsJS, to_estree_tokens_json};
use oxc_parser::{
    ParseOptions, Parser, ParserReturn,
    config::{NoTokensLexerConfig, RuntimeParserConfig},
    lexer::{Kind, Lexer},
};
use oxc_span::SourceType;
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

type ParserCases = Vec<(&'static str, String)>;

fn string_parser_cases() -> (ParserCases, ParserCases) {
    const STATEMENTS: usize = 64;
    let quoted =
        |quote: char, value: &str| format!("void {quote}{value}{quote};\n").repeat(STATEMENTS);
    let mut js = Vec::new();
    // These are complete parser workloads, with fixed statement counts and varying string bytes.
    for (quote, lengths) in [
        (
            '\'',
            [
                ("single_ascii_0", 0),
                ("single_ascii_1", 1),
                ("single_ascii_15", 15),
                ("single_ascii_16", 16),
                ("single_ascii_17", 17),
                ("single_ascii_64", 64),
                ("single_ascii_1024", 1024),
            ],
        ),
        (
            '"',
            [
                ("double_ascii_0", 0),
                ("double_ascii_1", 1),
                ("double_ascii_15", 15),
                ("double_ascii_16", 16),
                ("double_ascii_17", 17),
                ("double_ascii_64", 64),
                ("double_ascii_1024", 1024),
            ],
        ),
    ] {
        js.extend(lengths.map(|(name, len)| (name, quoted(quote, &"a".repeat(len)))));
    }
    for (single, double, value) in [
        ("single_unicode", "double_unicode", "é中😀".repeat(16)),
        ("single_escaped_newline", "double_escaped_newline", r"before\nafter".to_owned()),
        ("single_line_continuation", "double_line_continuation", "before\\\nafter".to_owned()),
        ("single_surrogate_pair", "double_surrogate_pair", r"\uD83D\uDE00".to_owned()),
        ("single_lone_surrogate", "double_lone_surrogate", r"\uD800".to_owned()),
    ] {
        js.push((single, quoted('\'', &value)));
        js.push((double, quoted('"', &value)));
    }
    for (quote, property, constructor, import) in [
        ('\'', "single_property_name", "single_constructor_name", "single_import_name"),
        ('"', "double_property_name", "double_constructor_name", "double_import_name"),
    ] {
        js.push((property, format!("({{{quote}property{quote}:0}});\n").repeat(STATEMENTS)));
        let constructors = build_source(STATEMENTS, |s, i| {
            writeln!(s, "class C{i}{{{quote}constructor{quote}(){{}}}}")
        });
        js.push((constructor, constructors));
        let imports = build_source(STATEMENTS, |s, i| {
            writeln!(s, "import {{{quote}name{quote} as value{i}}} from {quote}source{quote};")
        });
        js.push((import, imports));
    }
    let ts = [("single_literal_type", '\''), ("double_literal_type", '"')]
        .map(|(name, quote)| {
            let source =
                build_source(STATEMENTS, |s, i| writeln!(s, "type T{i}={quote}value{quote};"));
            (name, source)
        })
        .into_iter()
        .collect();
    (js, ts)
}

fn bench_parser_strings(criterion: &mut Criterion) {
    let (js, ts) = string_parser_cases();
    bench_parser_sources(criterion, "parser_strings", SourceType::mjs(), js);
    bench_parser_sources(criterion, "parser_strings/ts", SourceType::ts().with_module(true), ts);
}

fn bench_get_js_string(criterion: &mut Criterion) {
    let (js, ts) = string_parser_cases();
    bench_get_js_string_sources(criterion, "parser_get_js_string", SourceType::mjs(), js);
    bench_get_js_string_sources(
        criterion,
        "parser_get_js_string/ts",
        SourceType::ts().with_module(true),
        ts,
    );
}

fn bench_get_js_string_sources(
    criterion: &mut Criterion,
    name: &str,
    source_type: SourceType,
    cases: impl IntoIterator<Item = (&'static str, String)>,
) {
    let mut group = criterion.benchmark_group(name);

    for (name, source_text) in cases {
        // Keep scanning, escape decoding, and all allocation outside the timed queries.
        let allocator = Allocator::default();
        let mut lexer =
            Lexer::new_for_benchmarks(&allocator, &source_text, source_type, NoTokensLexerConfig);
        let mut tokens = Vec::new();
        let mut token = lexer.first_token();
        while token.kind() != Kind::Eof {
            if token.kind() == Kind::Str {
                tokens.push(token);
            }
            token = lexer.next_token_for_benchmarks();
        }
        assert!(lexer.errors().is_empty());
        // Every source has 64 statements; imports contain two queried strings per statement.
        let expected_count = if name.ends_with("_import_name") { 128 } else { 64 };
        assert_eq!(tokens.len(), expected_count);
        group.throughput(Throughput::Elements(tokens.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| {
                for &token in &tokens {
                    black_box(lexer.get_js_string_for_benchmarks(black_box(token)));
                }
            });
        });
    }

    group.finish();
}

fn build_source(
    count: usize,
    mut write: impl FnMut(&mut String, usize) -> std::fmt::Result,
) -> String {
    let mut source = String::new();
    for i in 0..count {
        write(&mut source, i).unwrap();
    }
    source
}

fn bench_parser_sources(
    criterion: &mut Criterion,
    name: &str,
    source_type: SourceType,
    cases: impl IntoIterator<Item = (&'static str, String)>,
) {
    let mut group = criterion.benchmark_group(name);

    for (name, source_text) in cases {
        group.throughput(Throughput::Bytes(source_text.len() as u64));
        group.bench_function(name, |b| {
            let mut allocator = Allocator::default();
            assert!(
                Parser::new(&allocator, &source_text, source_type).parse().diagnostics.is_empty()
            );
            allocator.reset();

            b.iter(|| {
                black_box(
                    Parser::new(&allocator, &source_text, source_type)
                        .with_options(ParseOptions {
                            parse_regular_expression: true,
                            ..ParseOptions::default()
                        })
                        .parse(),
                );
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
                    span_converter.convert_program_and_comments(&mut program);

                    black_box(program.to_estree_json_with_fixes(true, false));
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
                        ESTreeTokenOptionsJS,
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

criterion_group!(
    parser,
    bench_parser,
    bench_parser_strings,
    bench_get_js_string,
    bench_estree,
    bench_estree_tokens
);
criterion_main!(parser);
