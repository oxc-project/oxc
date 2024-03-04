use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_codegen_sourcemap(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("codegen_sourcemap");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(id, &file.source_text, |b, source_text| {
            let allocator = Allocator::default();
            let program = Parser::new(&allocator, source_text, source_type).parse().program;
            let codegen_options = CodegenOptions::default();
            b.iter_with_large_drop(|| {
                let mut codegen = Codegen::<false>::new(source_text.len(), codegen_options);
                codegen.with_sourcemap(source_text, "").build(&program);
                codegen.into_sourcemap();
            });
        });
    }

    group.finish();
}

criterion_group!(codegen_sourcemap, bench_codegen_sourcemap);
criterion_main!(codegen_sourcemap);
