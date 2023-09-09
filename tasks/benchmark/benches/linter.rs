use std::rc::Rc;

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_linter::{LintContext, Linter};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_linter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter");
    for file in TestFiles::minimal().files() {
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                let allocator = Allocator::default();
                let source_type = SourceType::default();
                let ret = Parser::new(&allocator, source_text, source_type).parse();
                let program = allocator.alloc(ret.program);
                let semantic_ret = SemanticBuilder::new(source_text, source_type)
                    .with_trivias(ret.trivias)
                    .build(program);
                let linter = Linter::new();
                let semantic = Rc::new(semantic_ret.semantic);
                b.iter_with_large_drop(|| {
                    let ctx = LintContext::new(&Rc::clone(&semantic));
                    linter.run(ctx)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(linter, bench_linter);
criterion_main!(linter);
