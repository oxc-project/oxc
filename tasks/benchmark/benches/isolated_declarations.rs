use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use oxc_tasks_common::TestFile;

fn bench_isolated_declarations(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("isolated-declarations");

    let file = TestFile::new(
        "https://raw.githubusercontent.com/oxc-project/benchmark-files/main/vue-id.ts",
    );

    let id = BenchmarkId::from_parameter(&file.file_name);
    let source_type = SourceType::from_path(&file.file_name).unwrap();

    group.bench_with_input(id, &file.source_text, |b, source_text| {
        b.iter_with_large_drop(|| {
            let allocator = Allocator::default();
            let ParserReturn { program, .. } =
                Parser::new(&allocator, source_text, source_type).parse();
            IsolatedDeclarations::new(
                &allocator,
                IsolatedDeclarationsOptions { strip_internal: true },
            )
            .build(&program);
        });
    });

    group.finish();
}

criterion_group!(transformer, bench_isolated_declarations);
criterion_main!(transformer);
