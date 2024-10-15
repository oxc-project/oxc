use std::path::Path;

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;
use oxc_transformer::{ArrowFunctionsOptions, TransformOptions, Transformer};

fn bench_transformer(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("transformer");

    for file in TestFiles::complicated().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        let source_text = file.source_text.as_str();

        // Create `Allocator` outside of `bench_function`, so same allocator is used for
        // both the warmup and measurement phases
        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                // Reset allocator at start of each iteration
                allocator.reset();

                // Create fresh AST + semantic data for each iteration
                let ParserReturn { mut program, .. } =
                    Parser::new(&allocator, source_text, source_type).parse();
                let (symbols, scopes) = SemanticBuilder::new()
                    // Estimate transformer will triple scopes, symbols, references
                    .with_excess_capacity(2.0)
                    .build(&program)
                    .semantic
                    .into_symbol_table_and_scope_tree();

                // `enable_all` enables all transforms except arrow functions transform
                // and async-to-generator
                let mut options = TransformOptions::enable_all();
                options.es2015.arrow_function = Some(ArrowFunctionsOptions { spec: true });
                options.es2017.async_to_generator = true;

                runner.run(|| {
                    let ret = Transformer::new(&allocator, Path::new(&file.file_name), options)
                        .build_with_symbols_and_scopes(symbols, scopes, &mut program);

                    // Return the `TransformerReturn`, so it's dropped outside of the measured section.
                    // `TransformerReturn` contains `ScopeTree` and `SymbolTable` which are costly to drop.
                    // That's not central to transformer, so we don't want it included in this measure.
                    ret
                });
            });
        });
    }

    group.finish();
}

criterion_group!(transformer, bench_transformer);
criterion_main!(transformer);
