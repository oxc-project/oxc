#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_parser(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("parser");
    for file in TestFiles::new().files() {
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                b.iter_with_large_drop(|| {
                    // Include the allocator drop time to make time measurement consistent.
                    // Otherwise the allocator will allocate huge memory chunks (by power of two) from the
                    // system allocator, which makes time measurement unequal during long runs.
                    let allocator = Allocator::default();
                    _ = Parser::new(&allocator, source_text, SourceType::default()).parse();
                    allocator
                });
            },
        );
    }
    group.finish();
}

criterion_group!(parser, bench_parser);
criterion_main!(parser);
