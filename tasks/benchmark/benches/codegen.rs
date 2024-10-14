use std::path::PathBuf;

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_codegen(criterion: &mut Criterion) {
    for file in TestFiles::complicated_one(0).files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        let allocator = Allocator::default();
        let source_text = &file.source_text;
        let ret = Parser::new(&allocator, source_text, source_type).parse();

        let mut group = criterion.benchmark_group("codegen");
        group.bench_with_input(id.clone(), &ret.program, |b, program| {
            b.iter_with_large_drop(|| CodeGenerator::new().build(program).map);
        });
        group.finish();

        let mut group = criterion.benchmark_group("codegen_sourcemap");
        group.bench_with_input(id, &ret.program, |b, program| {
            b.iter_with_large_drop(|| {
                CodeGenerator::new()
                    .with_options(CodegenOptions {
                        source_map_path: Some(PathBuf::from(&file.file_name)),
                        ..CodegenOptions::default()
                    })
                    .build(program)
            });
        });
        group.finish();
    }
}

criterion_group!(codegen, bench_codegen);
criterion_main!(codegen);
