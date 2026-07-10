use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_parser::{Parser, ParserReturn};
use oxc_react_compiler::{PluginOptions, transform};
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;

fn bench_react_compiler(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("react_compiler");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        let options = PluginOptions::default();

        // Create `Allocator` outside of `bench_function`, so the same allocator is
        // used for both the warmup and measurement phases.
        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                // Reset allocator at start of each iteration.
                allocator.reset();

                // Create a fresh AST for each iteration. The compiler builds
                // semantic data and allocates the compiled program in the same arena.
                let ParserReturn { mut program, .. } =
                    Parser::new(&allocator, source_text, source_type).parse();

                runner.run(|| {
                    let mut result = {
                        let semantic =
                            SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
                        transform(&program, &semantic, &allocator, options.clone())
                    };
                    if let Some(compiled) = result.program.take() {
                        program = compiled;
                    }
                    result
                });
            });
        });
    }

    group.finish();
}

criterion_group!(react_compiler, bench_react_compiler);
criterion_main!(react_compiler);
