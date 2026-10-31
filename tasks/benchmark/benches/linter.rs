use std::{ffi::OsStr, fs, path::Path, sync::Arc, sync::mpsc};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, criterion_group, criterion_main};
use oxc_linter::{
    ConfigStore, ConfigStoreBuilder, ContextSubHost, ContextSubHostOptions, ExternalPluginStore,
    FixKind, LintOptions, Linter, ModuleRecord, OsFileSystem, SuppressionManager, TsGoLintState,
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
                let semantic_ret = SemanticBuilder::new_linter().build(&parser_ret.program);
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
                        vec![ContextSubHost::new(
                            semantic,
                            Arc::clone(&module_record),
                            0,
                            ContextSubHostOptions::default(),
                        )],
                        &allocator,
                    )
                });
            });
        });
    }
    group.finish();
}

/// oxlint's side of `--type-aware`: `TsGoLintState::lint` on one file, with `src/bin/fake_tsgolint.rs` standing in
/// for `tsgolint`. The stand-in reports a fixed number of diagnostics without type checking, so this measures what
/// oxlint does per diagnostic it receives. The diagnostics stay alive until the iteration ends, as they do until the
/// reporter prints them.
fn bench_linter_type_aware(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter_type_aware");

    let files = TestFiles::minimal();
    let file = files.files().iter().find(|file| file.file_name == "App.tsx").expect("App.tsx");

    // oxlint looks for `node_modules/.bin/tsgolint` from its working directory up.
    let dir = std::env::temp_dir().join("oxc_benchmark_linter_type_aware");
    let bin_dir = dir.join("node_modules").join(".bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let tsgolint = bin_dir.join(if cfg!(windows) { "tsgolint.exe" } else { "tsgolint" });
    fs::copy(env!("CARGO_BIN_EXE_fake_tsgolint"), &tsgolint).unwrap();
    let path = dir.join(&file.file_name);
    fs::write(&path, &file.source_text).unwrap();
    let paths: Vec<Arc<OsStr>> = vec![Arc::from(path.as_os_str())];

    let mut external_plugin_store = ExternalPluginStore::default();
    let lint_config = ConfigStoreBuilder::all().build(&mut external_plugin_store).unwrap();
    let config_store = ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store);
    let diff_manager =
        SuppressionManager::load(&dir, "oxlint-suppressions.json", false, false).build_diff();

    group.bench_function(BenchmarkId::from_parameter(&file.file_name), |b| {
        b.iter_with_setup_wrapper(|runner| {
            let state = TsGoLintState::new(&dir, config_store.clone(), FixKind::None);
            let (tx, rx) = mpsc::channel();
            let diagnostics = runner.run(|| {
                state.lint(&paths, Arc::default(), tx, &OsFileSystem, &diff_manager, None).unwrap();
                rx.iter().flatten().collect::<Vec<_>>()
            });
            assert_eq!(diagnostics.len(), 200, "the stand-in tsgolint reports 200 diagnostics");
        });
    });
    group.finish();
}

criterion_group!(linter, bench_linter, bench_linter_type_aware);
criterion_main!(linter);
