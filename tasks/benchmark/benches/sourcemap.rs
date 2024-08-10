use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_codegen::{CodeGenerator, CodegenReturn};
use oxc_parser::Parser;
use oxc_sourcemap::ConcatSourceMapBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

#[allow(clippy::cast_possible_truncation)]
fn bench_sourcemap(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("sourcemap");

    for file in TestFiles::complicated_one(1).files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(id, &file.source_text, |b, source_text| {
            let allocator = Allocator::default();
            let ret = Parser::new(&allocator, source_text, source_type).parse();

            let CodegenReturn { source_text: output_txt, .. } = CodeGenerator::new()
                .enable_source_map(file.file_name.as_str(), source_text)
                .build(&ret.program);
            let lines = output_txt.matches('\n').count() as u32;

            b.iter(|| {
                let CodegenReturn { source_map, .. } = CodeGenerator::new()
                    .enable_source_map(file.file_name.as_str(), source_text)
                    .build(&ret.program);
                if let Some(sourcemap) = source_map {
                    let concat_sourcemap_builder = ConcatSourceMapBuilder::from_sourcemaps(&[
                        (&sourcemap, 0),
                        (&sourcemap, lines),
                    ]);
                    concat_sourcemap_builder.into_sourcemap().to_json_string();
                }
            });
        });
    }

    group.finish();
}

criterion_group!(sourcemap, bench_sourcemap);
criterion_main!(sourcemap);
