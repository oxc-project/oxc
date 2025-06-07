use std::path::Path;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::{TestFile, TestFiles};
use oxc_transformer::{EnvOptions, TransformOptions, Transformer};

fn bench_transformer(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("transformer");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        // Create `Allocator` outside of `bench_function`, so same allocator is used for
        // both the warmup and measurement phases
        let mut allocator = Allocator::default();

        let mut transform_options = TransformOptions::enable_all();
        // Even the plugins are unfinished, we still want to enable all of them
        // to track the performance changes during the development.
        transform_options.env = EnvOptions::enable_all(/* include_unfinished_plugins */ true);

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                // Reset allocator at start of each iteration
                allocator.reset();

                // Create fresh AST + semantic data for each iteration
                let ParserReturn { mut program, .. } =
                    Parser::new(&allocator, source_text, source_type).parse();
                let scoping = SemanticBuilder::new()
                    // Estimate transformer will triple scopes, symbols, references
                    .with_excess_capacity(2.0)
                    .build(&program)
                    .semantic
                    .into_scoping();

                runner.run(|| {
                    // Return the `TransformerReturn`, so it's dropped outside of the measured section.
                    // `TransformerReturn` contains `ScopeTree` and `SymbolTable` which are costly to drop.
                    // That's not central to transformer, so we don't want it included in this measure.
                    Transformer::new(&allocator, Path::new(&file.file_name), &transform_options)
                        .build_with_scoping(scoping, &mut program)
                });
            });
        });
    }

    group.finish();
}

fn bench_isolated_declarations(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("isolated-declarations");

    let file =
        TestFile::new("https://cdn.jsdelivr.net/gh/oxc-project/benchmark-files@main/vue-id.ts");

    let id = BenchmarkId::from_parameter(&file.file_name);
    let source_text = &file.source_text;
    let source_type = file.source_type;

    let ast_allocator = Allocator::new();
    let program = Parser::new(&ast_allocator, source_text, source_type).parse().program;

    // Create `Allocator` outside of `bench_function`, so same allocator is used for
    // both the warmup and measurement phases
    let mut output_allocator = Allocator::new();
    group.bench_function(id, |b| {
        b.iter_with_setup_wrapper(|runner| {
            // Reset allocator at start of each iteration
            output_allocator.reset();

            // Include dropping `IsolatedDeclarations::build`'s return value in benchmark timing.
            // Drop time is part of the cost of using this API.
            let options = IsolatedDeclarationsOptions { strip_internal: true };
            runner.run(|| IsolatedDeclarations::new(&output_allocator, options).build(&program));
        });
    });

    group.finish();
}

criterion_group!(transformer, bench_transformer, bench_isolated_declarations);
criterion_main!(transformer);
