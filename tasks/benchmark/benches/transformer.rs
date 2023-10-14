#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{cell::RefCell, fs, hint::black_box, rc::Rc};

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::project_root;
use oxc_transformer::{TransformOptions, Transformer};

fn bench_transformer(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("transformer");

    let dir = project_root().join("tasks/coverage/typescript/src/compiler");
    let files = ["binder.ts", "scanner.ts", "checker.ts", "parser.ts"];

    for file in files {
        let path = dir.join(file);
        let source_text = fs::read_to_string(&path).unwrap();
        let source_type = SourceType::from_path(file).unwrap();
        let id = BenchmarkId::from_parameter(file);
        group.bench_with_input(id, &source_text, |b, source_text| {
            let allocator = Allocator::default();
            let program = Parser::new(&allocator, source_text, source_type).parse().program;
            let semantic = SemanticBuilder::new(source_text, source_type).build(&program).semantic;
            let (symbols, _scope_tree) = semantic.into_symbol_table_and_scope_tree();
            let symbols = Rc::new(RefCell::new(symbols));
            let program = allocator.alloc(program);
            b.iter(|| {
                let transform_options = TransformOptions::default();
                Transformer::new(&allocator, source_type, &symbols, transform_options)
                    .build(black_box(program));
            });
        });
    }

    group.finish();
}

criterion_group!(transformer, bench_transformer);
criterion_main!(transformer);
