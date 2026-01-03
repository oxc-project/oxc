use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use oxc_span::SourceType;
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

    for file in TestFiles::linter().files() {
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

#[derive(Debug)]
struct RepoTestFile {
    path: PathBuf,
    source_type: SourceType,
    source_text: String,
}

impl RepoTestFile {
    fn new(path: PathBuf) -> Self {
        let source_type = SourceType::from_path(&path).expect("invalid source type");
        let source_text = std::fs::read_to_string(&path).expect("Failed to read source file");
        Self { path, source_type, source_text }
    }
}

fn bench_linter_real_world(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter_real_world");

    let calcom_path = std::env::var("CALCOM_PATH").expect("CALCOM_PATH env var not set");

    // TODO: Enable walker to lint whole repository
    let walker = ignore::WalkBuilder::new(&calcom_path)
        .ignore(false)
        .git_global(false)
        .git_ignore(true)
        .follow_links(true)
        .hidden(false)
        .require_git(false)
        .build_parallel();

    // Read source files once, outside of the benchmark
    let test_files = Arc::new(Mutex::new(Vec::new()));
    walker.run(|| {
        let test_files = Arc::clone(&test_files);
        Box::new(move |result| {
            if let Ok(entry) = result {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    let extension = extension.to_string_lossy();
                    if ["js", "jsx", "ts", "tsx"].contains(&extension.as_ref()) {
                        test_files.lock().unwrap().push(RepoTestFile::new(path.to_path_buf()));
                    }
                }
            }
            ignore::WalkState::Continue
        })
    });
    let test_files = Arc::try_unwrap(test_files).unwrap().into_inner().unwrap();

    // Build the linter config once, outside of the benchmark
    let mut external_plugin_store = ExternalPluginStore::default();
    let lint_config = ConfigStoreBuilder::all().build(&mut external_plugin_store).unwrap();
    let linter = Linter::new(
        LintOptions::default(),
        ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store),
        None,
    )
    .with_fix(FixKind::None);

    // Create allocator outside of bench_function so same allocator is used for
    // both the warmup and measurement phases
    let mut allocator = Allocator::default();

    // Group all of the files together as a single benchmark for cal.com
    group.bench_function("calcom_repo", |b| {
        b.iter_with_setup_wrapper(|runner| {
            // Reset allocator at start of each iteration to reclaim memory
            allocator.reset();

            // Step 1: Parse all files first
            let parser_results: Vec<_> = test_files
                .iter()
                .map(|test_file| {
                    Parser::new(&allocator, &test_file.source_text, test_file.source_type).parse()
                })
                .collect();

            // Step 2: Build semantic analysis for each file
            let lint_inputs: Vec<_> = test_files
                .iter()
                .zip(parser_results.iter())
                .map(|(test_file, parser_ret)| {
                    let semantic_ret = SemanticBuilder::new()
                        .with_scope_tree_child_ids(true)
                        .with_cfg(true)
                        .build(&parser_ret.program);
                    let semantic = semantic_ret.semantic;
                    let module_record = Arc::new(ModuleRecord::new(
                        &test_file.path,
                        &parser_ret.module_record,
                        &semantic,
                    ));
                    (&test_file.path, semantic, module_record)
                })
                .collect();

            runner.run(|| {
                for (path, semantic, module_record) in lint_inputs {
                    linter.run(
                        path,
                        vec![ContextSubHost::new(semantic, Arc::clone(&module_record), 0)],
                        &allocator,
                    );
                }
            });
        });
    });
}

criterion_group!(linter, bench_linter, bench_linter_real_world);
criterion_main!(linter);
