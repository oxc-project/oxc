use std::{env, path::Path, rc::Rc, sync::Arc};

use oxc_allocator::Allocator;
use oxc_benchmark::{criterion_group, criterion_main, BenchmarkId, Criterion};
use oxc_linter::{ConfigStoreBuilder, FixKind, LintOptions, Linter, ModuleRecord};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_tasks_common::TestFiles;

fn bench_linter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter");

    // If `FIXTURE` env is set, only run the specified benchmark. This is used for sharding in CI.
    let test_files = TestFiles::complicated();
    let mut test_files = test_files.files().iter().collect::<Vec<_>>();

    match env::var("FIXTURE").map(|n| n.parse::<usize>().unwrap()).ok() {
        Some(0) => test_files = vec![&test_files[0]],
        Some(1) => {
            test_files = vec![&test_files[1], &test_files[2]];
        }
        _ => {}
    }

    for file in test_files {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = file.source_text.as_str();
        let source_type = SourceType::from_path(&file.file_name).unwrap();
        group.bench_function(id, |b| {
            let allocator = Allocator::default();
            let ret = Parser::new(&allocator, source_text, source_type).parse();
            let path = Path::new("");
            let semantic_ret = SemanticBuilder::new()
                .with_build_jsdoc(true)
                .with_scope_tree_child_ids(true)
                .with_cfg(true)
                .build(&ret.program);
            let semantic = semantic_ret.semantic;
            let module_record = Arc::new(ModuleRecord::new(path, &ret.module_record, &semantic));
            let semantic = Rc::new(semantic);
            let lint_config =
                ConfigStoreBuilder::all().build().expect("Failed to build config store");
            let linter = Linter::new(LintOptions::default(), lint_config).with_fix(FixKind::All);
            b.iter(|| linter.run(path, Rc::clone(&semantic), Arc::clone(&module_record)));
        });
    }
    group.finish();
}

criterion_group!(linter, bench_linter);
criterion_main!(linter);
