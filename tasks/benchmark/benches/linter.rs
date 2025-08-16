use std::{path::Path, rc::Rc, sync::Arc};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_linter::{
    ConfigStore, ConfigStoreBuilder, ExternalPluginStore, FixKind, LintOptions, Linter,
    ModuleRecord,
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
        let external_plugin_store = ExternalPluginStore::default();
        let lint_config = ConfigStoreBuilder::all().build(&external_plugin_store).unwrap();
        let linter = Linter::new(
            LintOptions::default(),
            ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store),
            None,
        )
        .with_fix(FixKind::All);
        group.bench_function(id, |b| {
            b.iter(|| {
                linter.run(path, Rc::clone(&semantic), Arc::clone(&module_record), &allocator)
            });
        });
    }
    group.finish();
}

criterion_group!(linter, bench_linter);
criterion_main!(linter);
