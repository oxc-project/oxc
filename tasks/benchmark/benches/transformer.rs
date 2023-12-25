#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::hint::black_box;

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;
use oxc_transformer::{TransformOptions, Transformer};

fn bench_transformer(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("transformer");

    for file in TestFiles::complicated().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();

        group.bench_with_input(id, &file.source_text, |b, source_text| {
            // The whole transformation process needs to be benched otherwise it will end up with
            // transforming an already transformed AST.
            b.iter_with_large_drop(|| {
                let allocator = Allocator::default();
                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let semantic =
                    SemanticBuilder::new(source_text, source_type).build(&program).semantic;
                let program = allocator.alloc(program);
                let transform_options = TransformOptions::default();
                Transformer::new(&allocator, source_type, semantic, transform_options)
                    .build(black_box(program))
                    .unwrap();
                allocator
            });
        });
    }

    group.finish();
}

criterion_group!(transformer, bench_transformer);
criterion_main!(transformer);
