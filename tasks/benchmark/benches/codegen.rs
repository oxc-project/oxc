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

        let mut parser_ret = Parser::new(&allocator, source_text, source_type).parse();
        assert!(parser_ret.errors.is_empty());

        let mut group = criterion.benchmark_group("codegen");
        group.bench_with_input(id.clone(), &parser_ret.program, |b, program| {
            b.iter_with_large_drop(|| CodeGenerator::new().build(program).map);
        });
        group.finish();

        let transformed_program = {
            let transform_options = TransformOptions::enable_all();
            let (symbols, scopes) = SemanticBuilder::new()
                // Estimate transformer will triple scopes, symbols, references
                .with_excess_capacity(2.0)
                .build(&parser_ret.program)
                .semantic
                .into_symbol_table_and_scope_tree();

            let transformer_ret =
                Transformer::new(&allocator, Path::new(&file.file_name), &transform_options)
                    .build_with_symbols_and_scopes(symbols, scopes, &mut parser_ret.program);

            assert!(transformer_ret.errors.is_empty());
            parser_ret.program
        };

        let mut group = criterion.benchmark_group("codegen_sourcemap");
        group.bench_with_input(id, &transformed_program, |b, program| {
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
