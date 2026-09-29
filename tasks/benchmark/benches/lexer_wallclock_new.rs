//! Wallclock benchmark of `oxc_lexer` against the lexer in `oxc_parser`, head-to-head.
//!
//! Not run on CodSpeed. `oxc_lexer` is designed to exploit out-of-order execution and branch prediction,
//! which CodSpeed's CPU simulation does not model. So these benchmarks measure wallclock time instead.
//!
//! `oxc_lexer` compiles to its SIMD core or its scalar fallback depending on target features,
//! which apply to the whole build. So the 2 lexers are benchmarked in separate builds:
//!
//! 1. `lexer_wallclock_old` benchmarks `oxc_parser`'s lexer, built without the SIMD target features,
//!    and saves the results as criterion baseline `lexer_old`.
//! 2. This benchmark benchmarks `oxc_lexer` (either implementation), and compares it against that baseline.
//!    The "change" which criterion reports is `oxc_lexer`'s time relative to `oxc_parser`'s.
//!
//! When saving the baseline, criterion also compares `oxc_parser` against its previous run, if there is one.
//! So the "change" reported for `oxc_parser` is relative to its previous run.
//!
//! Both lexers lex the same cleaned source text, so the contest is fair.
//! See `cleaned_test_files` for why the source text needs cleaning.
//!
//! Run with `just bench-lexer` (scalar fallback) or `just bench-lexer-simd` (SIMD core).
//! CI runs `just bench-lexer-simd` when `oxc_lexer` changes, and prints the results in the job log.

mod lexer_common;

use oxc_benchmark::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use oxc_lexer::{LexOptions, PAD};

use lexer_common::cleaned_test_files;

#[expect(clippy::print_stdout)]
fn bench_lexer_wallclock_new(criterion: &mut Criterion) {
    // Report which implementation of `oxc_lexer` was built.
    // CI checks this line to make sure the build it expected is the build it got.
    let implementation = if oxc_lexer::IS_SIMD { "SIMD" } else { "scalar" };
    println!("Lexer implementation: {implementation}");

    // Group name and benchmark IDs must match `lexer_wallclock_old`, so criterion can compare the two
    let mut group = criterion.benchmark_group("lexer_wallclock");

    for file in cleaned_test_files() {
        let source_text = file.source_text.as_str();
        let source_type = file.source_type;
        let len = u32::try_from(source_text.len()).unwrap();

        // `oxc_lexer` requires source text to be followed by `PAD` zeroed bytes.
        // The caller will be required to supply source text with padding, so don't include it in benchmark.
        let mut padded = Vec::with_capacity(source_text.len() + PAD);
        padded.extend_from_slice(source_text.as_bytes());
        padded.resize(source_text.len() + PAD, 0);

        let options = LexOptions {
            source_type_module: source_type.is_module(),
            jsx: source_type.is_jsx(),
            ts: source_type.is_typescript(),
            ..LexOptions::default()
        };

        // Lex once outside the benchmark, to check `oxc_lexer` completes without errors,
        // and to get an `Arena` to reuse.
        // Reusing it is the equivalent of `oxc_parser`'s benchmark reusing its allocator.
        let (result, mut arena) = oxc_lexer::lex_utf8(&padded, len, options);
        assert!(result.diagnostics().is_empty());

        group.throughput(Throughput::Bytes(u64::from(len)));
        group.bench_function(BenchmarkId::from_parameter(&file.file_name), |b| {
            b.iter(|| oxc_lexer::lex_utf8_arena(&padded, len, options, &mut arena));
        });
    }

    group.finish();
}

criterion_group!(lexer_wallclock_new, bench_lexer_wallclock_new);
criterion_main!(lexer_wallclock_new);
