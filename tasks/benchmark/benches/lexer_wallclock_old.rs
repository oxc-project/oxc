//! Wallclock benchmark of the lexer in `oxc_parser`, as the baseline for `lexer_wallclock_new`.
//!
//! See `lexer_wallclock_new.rs` for how the 2 benchmarks fit together.

mod lexer_common;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use oxc_parser::config::NoTokensLexerConfig;

use lexer_common::{cleaned_test_files, lex_whole_file};

fn bench_lexer_wallclock_old(criterion: &mut Criterion) {
    // Group name and benchmark IDs must match `lexer_wallclock_new`, so criterion can compare the two
    let mut group = criterion.benchmark_group("lexer_wallclock");

    for file in cleaned_test_files() {
        let source_text = file.source_text.as_str();
        let source_type = file.source_type;

        group.throughput(Throughput::Bytes(u64::try_from(source_text.len()).unwrap()));
        group.bench_function(BenchmarkId::from_parameter(&file.file_name), |b| {
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

criterion_group!(lexer_wallclock_old, bench_lexer_wallclock_old);
criterion_main!(lexer_wallclock_old);
