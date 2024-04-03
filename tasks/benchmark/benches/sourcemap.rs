use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_codegen::{Codegen, CodegenOptions, CodegenReturn};
use oxc_parser::Parser;
use oxc_sourcemap::ConcatSourceMapBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

#[allow(clippy::cast_possible_truncation)]
fn bench_sourcemap(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("sourcemap");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(id, &file.source_text, |b, source_text| {
            let allocator = Allocator::default();
            let program = Parser::new(&allocator, source_text, source_type).parse().program;
            let codegen_options =
                CodegenOptions { enable_source_map: true, ..CodegenOptions::default() };
            b.iter_with_large_drop(|| {
                let CodegenReturn { source_map, source_text } = Codegen::<false>::new(
                    file.file_name.as_str(),
                    source_text,
                    codegen_options.clone(),
                )
                .build(&program);
                let line = source_text.matches('\n').count() as u32;
                if let Some(sourcemap) = source_map {
                    let mut concat_sourcemap_builder = ConcatSourceMapBuilder::default();
                    for i in 0..1 {
                        concat_sourcemap_builder.add_sourcemap(&sourcemap, line * i);
                    }
                    concat_sourcemap_builder.into_sourcemap().to_json_string().unwrap();
                }
            });
        });
    }

    group.finish();
}

criterion_group!(sourcemap, bench_sourcemap);
criterion_main!(sourcemap);
