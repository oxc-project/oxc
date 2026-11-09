use std::{
    collections::BTreeSet,
    hash::{DefaultHasher, Hash, Hasher},
    hint::black_box,
};

use codspeed_criterion_compat::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn validate(name: &str, source: &str, source_type: SourceType) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type)
        .with_options(ParseOptions { parse_regular_expression: true, ..ParseOptions::default() })
        .parse();
    assert!(parsed.diagnostics.is_empty(), "{name}: {:?}", parsed.diagnostics);
    let mut hash = DefaultHasher::new();
    Codegen::new().build(&parsed.program).code.hash(&mut hash);
    format!("{:?}", parsed.program.comments).hash(&mut hash);
    // Runs outside measured scope; detect output, comment kind, span and annotation changes.
    #[expect(clippy::print_stdout)]
    {
        println!(
            "CHECK {name} {} {} {:016x}",
            source.len(),
            parsed.program.comments.len(),
            hash.finish()
        );
    }
}

fn benchmark(c: &mut Criterion) {
    let variant = std::env::var("COMMENT_VARIANT").unwrap_or_else(|_| "default".into());
    let mut group = c.benchmark_group(format!("comments_{variant}"));
    let large = TestFiles::complicated();
    let small = TestFiles::minimal();
    let mut seen = BTreeSet::new();
    for file in large.files().iter().chain(small.files()) {
        if !seen.insert(&file.file_name) {
            continue;
        }
        validate(&file.file_name, &file.source_text, file.source_type);
        let mut allocator = Allocator::default();
        group.bench_function(BenchmarkId::from_parameter(&file.file_name), |b| {
            b.iter(|| {
                black_box(
                    Parser::new(&allocator, &file.source_text, file.source_type)
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

    let mut group = c.benchmark_group(format!("comment_shapes_{variant}"));
    for (name, comment) in [
        ("line_short", format!("//{}\nx;\n", "x".repeat(8))),
        ("line_medium", format!("//{}\nx;\n", "x".repeat(64))),
        ("line_long", format!("//{}\nx;\n", "x".repeat(4096))),
        ("line_unicode", format!("//{}€中😀{}\r\nx;\n", "x".repeat(32), "y".repeat(32))),
        ("line_ls_ps", format!("//{}\u{2028}x;//{}\u{2029}y;\n", "x".repeat(64), "y".repeat(64))),
        ("block_short", format!("/*{}*/x;\n", "x".repeat(8))),
        ("block_long", format!("/*{}*/x;\n", "x".repeat(4096))),
        ("block_first_line", format!("/*{}\n{}*/x;\n", "x".repeat(4096), "y".repeat(64))),
        ("block_after_newline", format!("/*\n{}*/x;\n", "x".repeat(4096))),
        ("block_stars", format!("/*{}*/x;\n", "*x".repeat(128))),
        (
            "block_unicode",
            format!("/*{}€中😀{}\u{2028}{}*/x;\n", "x".repeat(32), "y".repeat(32), "z".repeat(64)),
        ),
    ] {
        let source = comment.repeat(262_144 / comment.len());
        let source_type = SourceType::mjs();
        validate(name, &source, source_type);
        let mut allocator = Allocator::default();
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(Parser::new(&allocator, &source, source_type).parse());
                allocator.reset();
            });
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
