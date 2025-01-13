use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_codegen::{CodeGenerator, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;
use oxc_transformer::{TransformOptions, Transformer};

fn bench_codegen(criterion: &mut Criterion) {
    for file in TestFiles::complicated_one(0).files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        let allocator = Allocator::default();
        let source_text = &file.source_text;

        // Codegen
        let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(parser_ret.errors.is_empty());
        let mut program = parser_ret.program;

        let mut group = criterion.benchmark_group("codegen");
        group.bench_function(id.clone(), |b| {
            b.iter_with_large_drop(|| CodeGenerator::new().build(&program).map);
        });
        group.finish();

        // Codegen sourcemap
        let (symbols, scopes) =
            SemanticBuilder::new().build(&program).semantic.into_symbol_table_and_scope_tree();

        let transform_options = TransformOptions::enable_all();
        let transformer_ret =
            Transformer::new(&allocator, Path::new(&file.file_name), &transform_options)
                .build_with_symbols_and_scopes(symbols, scopes, &mut program);
        assert!(transformer_ret.errors.is_empty());

        let mut group = criterion.benchmark_group("codegen_sourcemap");
        group.bench_function(id, |b| {
            b.iter_with_large_drop(|| {
                CodeGenerator::new()
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
