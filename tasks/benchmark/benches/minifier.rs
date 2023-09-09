use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_minifier::{Minifier, MinifierOptions};
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_minifier(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("minifier");
    for file in TestFiles::minimal().files() {
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                let source_type = SourceType::from_path(&file.file_name).unwrap();
                let options = MinifierOptions::default();
                b.iter_with_large_drop(|| Minifier::new(source_text, source_type, options).build());
            },
        );
    }
    group.finish();
}

criterion_group!(minifier, bench_minifier);
criterion_main!(minifier);
