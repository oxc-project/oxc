use std::{path::Path, sync::Arc};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_linter::{
    ConfigStore, ConfigStoreBuilder, ContextSubHost, ExternalPluginStore, FixKind, LintOptions,
    Linter, ModuleRecord,
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::TestFiles;

fn bench_linter(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter");

    for file in TestFiles::minimal().files() {
        let id = BenchmarkId::from_parameter(&file.file_name);
        let source_text = &file.source_text;
        let source_type = file.source_type;

        // Create `Allocator` outside of `bench_function`, so same allocator is used for
        // both the warmup and measurement phases
        let mut allocator = Allocator::default();

        group.bench_function(id, |b| {
            b.iter_with_setup_wrapper(|runner| {
                // Reset allocator at start of each iteration
                allocator.reset();

                let parser_ret = Parser::new(&allocator, source_text, source_type).parse();
                let path = Path::new("");
                let semantic_ret = SemanticBuilder::new()
                    .with_scope_tree_child_ids(true)
                    .with_cfg(true)
                    .build(&parser_ret.program);
                let semantic = semantic_ret.semantic;
                let module_record =
                    Arc::new(ModuleRecord::new(path, &parser_ret.module_record, &semantic));
                let mut external_plugin_store = ExternalPluginStore::default();
                let lint_config =
                    ConfigStoreBuilder::all().build(&mut external_plugin_store).unwrap();
                let linter = Linter::new(
                    LintOptions::default(),
                    ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store),
                    None,
                )
                .with_fix(FixKind::All);

                runner.run(|| {
                    linter.run(
                        path,
                        vec![ContextSubHost::new(semantic, Arc::clone(&module_record), 0)],
                        &allocator,
                    )
                });
            });
        });
    }
    group.finish();
}

criterion_group!(linter, bench_linter);
criterion_main!(linter);
