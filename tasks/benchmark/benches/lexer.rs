mod lexer_common;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_parser::config::NoTokensLexerConfig;

use lexer_common::{cleaned_test_files, lex_whole_file};

fn bench_lexer(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("lexer");

    for file in cleaned_test_files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = file.source_text.as_str();
        let source_type = file.source_type;
        group.bench_function(id, |b| {
            // Do not include initializing allocator in benchmark.
            // User code would likely reuse the same allocator over and over to parse multiple files,
            // so we do the same here.
            let mut allocator = Allocator::default();
            b.iter(|| {
                lex_whole_file(&allocator, source_text, source_type, NoTokensLexerConfig);
                allocator.reset();
            });
        });
    }
    group.finish();
}

criterion_group!(lexer, bench_lexer);
criterion_main!(lexer);
