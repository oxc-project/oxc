use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_codegen::{CodeGenerator, CodegenReturn};
use oxc_parser::Parser;
use oxc_sourcemap::ConcatSourceMapBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

#[allow(clippy::cast_possible_truncation)]
fn bench_sourcemap(criterion: &mut Criterion) {
    for file in TestFiles::complicated_one(1).files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &file.source_text, source_type).parse();
        let CodegenReturn { source_map, source_text } = CodeGenerator::new()
            .enable_source_map(file.file_name.as_str(), &file.source_text)
            .build(&ret.program);

        let mut group = criterion.benchmark_group("concat-sourcemap-to-json-string");
        group.bench_with_input(id.clone(), &source_map, |b, source_map| {
            b.iter(|| {
                let line = source_text.matches('\n').count() as u32;
                if let Some(sourcemap) = source_map {
                    let mut concat_sourcemap_builder = ConcatSourceMapBuilder::default();
                    for i in 0..1 {
                        concat_sourcemap_builder.add_sourcemap(sourcemap, line * i);
                    }
                    concat_sourcemap_builder.into_sourcemap().to_json_string().unwrap();
                }
            });
        });
        group.finish();

        let mut group = criterion.benchmark_group("sourcemap-to-json-string");
        group.bench_with_input(id, &source_map, |b, source_map| {
            b.iter(|| {
                source_map.as_ref().map(|sourcemap| sourcemap.to_json_string().unwrap());
            });
        });
        group.finish();
    }
}

criterion_group!(sourcemap, bench_sourcemap);
criterion_main!(sourcemap);
