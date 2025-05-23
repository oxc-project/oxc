use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;
use oxc_transformer::{TransformOptions, Transformer};

fn bench_codegen(criterion: &mut Criterion) {
    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;
        let allocator = Allocator::default();

        let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(parser_ret.errors.is_empty());
        let mut program = parser_ret.program;

        let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();

        let transform_options = TransformOptions::enable_all();
        let transformer_ret =
            Transformer::new(&allocator, Path::new(&file.file_name), &transform_options)
                .build_with_scoping(scoping, &mut program);
        assert!(transformer_ret.errors.is_empty());

        let mut group = criterion.benchmark_group("codegen");
        group.bench_function(id, |b| {
            b.iter_with_large_drop(|| {
                Codegen::new()
                    .with_options(CodegenOptions {
                        source_map_path: Some(PathBuf::from(&file.file_name)),
                        ..CodegenOptions::default()
                    })
                    .build(&program)
            });
        });
        group.finish();
    }
}

criterion_group!(codegen, bench_codegen);
criterion_main!(codegen);
