use std::{path::Path, sync::Arc};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_benchmark::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use oxc_linter::{
    AllowWarnDeny, ConfigStore, ConfigStoreBuilder, ContextSubHost, ContextSubHostOptions,
    ExternalPluginStore, FixKind, LintFilter, LintOptions, Linter, ModuleRecord,
    create_unused_directives_diagnostics,
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
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
                let semantic_ret = SemanticBuilder::new().with_cfg(true).build(&parser_ret.program);
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

fn bench_unused_disable_directives(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter_unused_disable_directives");
    let id = BenchmarkId::from_parameter("used-next-line-no-console");
    let source_text = "// eslint-disable-next-line no-console\nconsole.log('x');\n".repeat(20_000);
    let source_type = SourceType::from_path(Path::new("fixture.js")).unwrap();
    let path = Path::new("fixture.js");
    let mut allocator = Allocator::default();

    let mut external_plugin_store = ExternalPluginStore::default();
    let lint_config = ConfigStoreBuilder::empty()
        .with_filter(&LintFilter::new(AllowWarnDeny::Warn, "no-console").unwrap())
        .build(&mut external_plugin_store)
        .unwrap();
    let linter = Linter::new(
        LintOptions::default(),
        ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store),
        None,
    );

    group.bench_function(id, |b| {
        b.iter_with_setup_wrapper(|runner| {
            allocator.reset();

            let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
            let semantic_ret = SemanticBuilder::new().with_cfg(true).build(&parser_ret.program);
            let semantic = semantic_ret.semantic;
            let module_record =
                Arc::new(ModuleRecord::new(path, &parser_ret.module_record, &semantic));

            runner.run(|| {
                let context_sub_hosts =
                    vec![ContextSubHost::new(
                        semantic,
                        Arc::clone(&module_record),
                        0,
                        ContextSubHostOptions::default(),
                    )];
                let (messages, directives) =
                    linter.run_with_disable_directives(path, context_sub_hosts, &allocator, None);
                let diagnostics = create_unused_directives_diagnostics(
                    directives
                        .as_ref()
                        .expect("disable directives should always be available for JS input"),
                    AllowWarnDeny::Warn,
                );

                black_box((messages, diagnostics));
            });
        });
    });

    group.finish();
}

fn bench_unused_disable_directives_many_files(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("linter_unused_disable_directives_many_files");
    let id = BenchmarkId::from_parameter("1000-files-all-used");
    let source_text = "// eslint-disable-next-line no-console\nconsole.log('x');\n".repeat(20);
    let source_type = SourceType::from_path(Path::new("fixture.js")).unwrap();
    let path = Path::new("fixture.js");
    let mut allocator = Allocator::default();

    let mut external_plugin_store = ExternalPluginStore::default();
    let lint_config = ConfigStoreBuilder::empty()
        .with_filter(&LintFilter::new(AllowWarnDeny::Warn, "no-console").unwrap())
        .build(&mut external_plugin_store)
        .unwrap();
    let linter = Linter::new(
        LintOptions::default(),
        ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store),
        None,
    );

    allocator.reset();
    let parser_ret = Parser::new(&allocator, &source_text, source_type).parse();
    let semantic_ret = SemanticBuilder::new().with_cfg(true).build(&parser_ret.program);
    let semantic = semantic_ret.semantic;
    let module_record = Arc::new(ModuleRecord::new(path, &parser_ret.module_record, &semantic));
    let context_sub_hosts = vec![ContextSubHost::new(
        semantic,
        Arc::clone(&module_record),
        0,
        ContextSubHostOptions::default(),
    )];
    let (_, directives) =
        linter.run_with_disable_directives(path, context_sub_hosts, &allocator, None);
    let directives =
        directives.expect("disable directives should always be available for JS input");
    assert!(create_unused_directives_diagnostics(&directives, AllowWarnDeny::Warn).is_empty());

    let all_used_directives = vec![directives; 1_000];

    group.bench_function(id, |b| {
        b.iter(|| {
            let diagnostics_count = all_used_directives
                .iter()
                .map(|directives| {
                    create_unused_directives_diagnostics(directives, AllowWarnDeny::Warn).len()
                })
                .sum::<usize>();

            black_box(diagnostics_count);
        });
    });

    group.finish();
}

criterion_group!(
    linter,
    bench_linter,
    bench_unused_disable_directives,
    bench_unused_disable_directives_many_files
);
criterion_main!(linter);
