use std::sync::Arc;

use oxc_benchmark::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use oxc_diagnostics::{
    Error, NamedSource, OxcDiagnostic,
    reporter::{Info, batch_infos},
};
use oxc_span::Span;
use oxc_tasks_common::TestFiles;

/// Number of diagnostics reported per file, as a noisy lint run would.
const DIAGNOSTICS: u32 = 100;

/// Build [`DIAGNOSTICS`] diagnostics spread over `source`, so that looking up their line/column
/// has to walk most of the file.
fn build_diagnostics(source: &Arc<NamedSource<String>>, source_len: u32) -> Vec<Error> {
    (0..DIAGNOSTICS)
        .map(|index| {
            let offset = source_len * index / DIAGNOSTICS;
            let end = u32::min(offset + 1, source_len);
            OxcDiagnostic::warn("Example diagnostic")
                .with_label(Span::new(offset, end))
                .with_source_code(Arc::clone(source))
        })
        .collect()
}

/// Benchmark resolving the line/column of a batch of diagnostics.
///
/// Reporters that render diagnostics one at a time (`--format=github`, `--format=junit`, ...) rescan
/// the source for every diagnostic, while `batch_infos` resolves the whole batch by continuing one scan
/// per file. Both paths are measured, so a regression in either one shows up in results.
fn bench_diagnostics(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("diagnostics");

    for file in TestFiles::minimal().files() {
        let Ok(source_len) = u32::try_from(file.source_text.len()) else {
            continue;
        };
        if source_len == 0 {
            continue;
        }

        let source = Arc::new(NamedSource::new(&file.file_name, file.source_text.clone()));
        let errors = build_diagnostics(&source, source_len);

        // Resolve every diagnostic independently, without sharing state between them.
        group.bench_function(
            BenchmarkId::from_parameter(format!("info_new/{}", file.file_name)),
            |b| b.iter(|| black_box(errors.iter().map(Info::new).collect::<Vec<_>>())),
        );

        // Resolve the batch of diagnostics, sharing one line index between them.
        group.bench_function(
            BenchmarkId::from_parameter(format!("batch_infos/{}", file.file_name)),
            |b| {
                b.iter(|| {
                    black_box(batch_infos(&errors).map(|(_, info)| info).collect::<Vec<_>>())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(diagnostics, bench_diagnostics);
criterion_main!(diagnostics);
