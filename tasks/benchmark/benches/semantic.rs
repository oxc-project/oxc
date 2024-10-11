use std::path::Path;

use oxc_allocator::Allocator;
use oxc_benchmark::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_semantic(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("semantic");
    for file in TestFiles::complicated().files() {
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                let allocator = Allocator::default();
                let ret = Parser::new(&allocator, source_text, source_type).parse();
                let program = allocator.alloc(black_box(ret.program));
                b.iter_with_large_drop(|| {
                    // We drop `Semantic` inside this closure as drop time is part of cost of using this API.
                    // We return `error`s to be dropped outside of the measured section, as usually
                    // code would have no errors. One of our benchmarks `cal.com.tsx` has a lot of errors,
                    // but that's atypical, so don't want to include it in benchmark time.
                    let ret = SemanticBuilder::new()
                        .with_build_jsdoc(true)
                        .build_module_record(Path::new(""), program)
                        .build(program);
                    let ret = black_box(ret);
                    ret.errors
                });
            },
        );
    }
    group.finish();
}

criterion_group!(semantic, bench_semantic);
criterion_main!(semantic);
