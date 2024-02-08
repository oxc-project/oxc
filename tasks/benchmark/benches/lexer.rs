use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::lexer::{Kind, Lexer};
use oxc_span::SourceType;
use oxc_tasks_common::{TestFile, TestFiles};

fn bench_lexer(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("lexer");

    // Lexer lacks awareness of JS grammar, so it gets confused by a few things without the parser
    // driving it, notably escapes in regexps and template strings.
    // So simplify the input for it, by removing backslashes and converting template strings to
    // normal string literals.
    let files = TestFiles::complicated()
        .files()
        .iter()
        .map(|file| TestFile {
            url: file.url.clone(),
            file_name: file.file_name.clone(),
            source_text: file.source_text.replace('\\', " ").replace('`', "'"),
        })
        .collect::<Vec<_>>();

    for file in files {
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                b.iter_with_large_drop(|| {
                    // Include the allocator drop time to make time measurement consistent.
                    // Otherwise the allocator will allocate huge memory chunks (by power of two) from the
                    // system allocator, which makes time measurement unequal during long runs.
                    let allocator = Allocator::default();
                    let mut lexer = Lexer::new_for_benchmarks(&allocator, source_text, source_type);
                    while lexer.next_token().kind != Kind::Eof {}
                    allocator
                });
            },
        );
    }
    group.finish();
}

criterion_group!(lexer, bench_lexer);
criterion_main!(lexer);
