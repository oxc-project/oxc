use std::{env, path::PathBuf, rc::Rc};

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_linter::{AllowWarnDeny, LintContext, LintCtx, LintOptions, Linter};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_linter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter");

    // If `FIXTURE` env is set, only run the specified benchmark. This is used for sharding in CI.
    let test_files = if let Ok(fixture_index) = env::var("FIXTURE") {
        let fixture_index = fixture_index.parse::<usize>().unwrap();
        TestFiles::complicated_one(fixture_index)
    } else {
        TestFiles::complicated()
    };

    for file in test_files.files() {
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_with_input(
            BenchmarkId::from_parameter(&file.file_name),
            &file.source_text,
            |b, source_text| {
                let allocator = Allocator::default();
                let ret = Parser::new(&allocator, source_text, source_type).parse();
                let program = allocator.alloc(ret.program);
                let semantic_ret = SemanticBuilder::new(source_text, source_type)
                    .with_trivias(ret.trivias)
                    .with_cfg(true)
                    .build_module_record(PathBuf::new(), program)
                    .build(program);
                let filter = vec![
                    (AllowWarnDeny::Deny, "all".into()),
                    (AllowWarnDeny::Deny, "nursery".into()),
                ];
                let lint_options = LintOptions::default()
                    .with_filter(filter)
                    .with_import_plugin(true)
                    .with_jsdoc_plugin(true)
                    .with_jest_plugin(true)
                    .with_jsx_a11y_plugin(true)
                    .with_nextjs_plugin(true)
                    .with_react_perf_plugin(true);
                let linter = Linter::from_options(lint_options).unwrap();
                let semantic = Rc::new(semantic_ret.semantic);
                b.iter(|| {
                    linter.run(LintContext::new(
                        PathBuf::from("").into_boxed_path(),
                        Rc::clone(&semantic),
                    ))
                });
            },
        );
    }
    group.finish();
}

criterion_group!(linter, bench_linter);
criterion_main!(linter);
