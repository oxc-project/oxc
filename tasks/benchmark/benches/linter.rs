#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{path::PathBuf, rc::Rc};

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_linter::{AllowWarnDeny, LintContext, LintOptions, Linter};
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
                    .build_module_record(PathBuf::new(), program)
                    .build(program);
                let lint_options = LintOptions::default()
                    .with_filter(vec![(AllowWarnDeny::Deny, "all".into())])
                    .with_jest_plugin(true)
                    .with_jsx_a11y_plugin(true);
                let linter = Linter::from_options(lint_options);
                let semantic = Rc::new(semantic_ret.semantic);
                b.iter(|| {
                    linter.run(LintContext::new(PathBuf::from("").into_boxed_path(), &semantic))
                });
            },
        );
    }
    group.finish();
}

criterion_group!(linter, bench_linter);
criterion_main!(linter);
