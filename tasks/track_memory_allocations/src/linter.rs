use std::{fmt::Write as _, fs, io, path::Path, rc::Rc, sync::Arc};

use oxc_allocator::Allocator;
use oxc_linter::{
    ConfigStore, ConfigStoreBuilder, ContextSubHost, ContextSubHostOptions, ExternalPluginStore,
    LintOptions, ModuleRecord, rules::ContextHost,
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::{TestFiles, project_root};
use rustc_hash::FxHashMap;

use super::{record_stats_in, render_file_stats};

pub fn run(files: &TestFiles) -> io::Result<()> {
    let mut external_plugin_store = ExternalPluginStore::default();
    let config = ConfigStoreBuilder::all().build(&mut external_plugin_store).unwrap();
    let config_store = ConfigStore::new(config, FxHashMap::default(), external_plugin_store);
    let snapshot_path = project_root().join("tasks/track_memory_allocations/allocs_linter.yaml");
    let mut out = String::new();

    for file in files.files() {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &file.source_text, file.source_type).parse();
        assert!(parsed.diagnostics.is_empty());
        let path = Path::new(&file.file_name);
        let semantic = SemanticBuilder::new_linter().build(&parsed.program).semantic;
        let module_record = Arc::new(ModuleRecord::new(path, &parsed.module_record, &semantic));
        let state = config_store.resolve(path);
        let host = Rc::new(ContextHost::new(
            path,
            vec![ContextSubHost::new(semantic, module_record, 0, ContextSubHostOptions::default())],
            &allocator,
            LintOptions::default(),
            state.config,
        ));

        let mut entries = Vec::new();
        for (rule, severity) in state.rules.iter().filter(|(rule, _)| !rule.is_tsgolint_rule()) {
            let rule_name = format!("{}/{}", rule.plugin_name(), rule.name());
            let arena_before = allocator.used_bytes();
            let ((), mut stats) = record_stats_in(&allocator, || {
                Rc::clone(&host).run_rule_for_allocations(rule, *severity);
            });
            stats.arena_used_bytes = allocator.used_bytes().saturating_sub(arena_before);
            entries.push((rule_name, stats));
        }

        entries.sort_unstable_by(|(left_name, left), (right_name, right)| {
            right
                .counters
                .heap
                .alloc_bytes
                .cmp(&left.counters.heap.alloc_bytes)
                .then_with(|| right.counters.heap.allocs.cmp(&left.counters.heap.allocs))
                .then_with(|| left_name.cmp(right_name))
        });

        writeln!(out, "{}:", file.file_name).unwrap();
        for (rule_name, stats) in entries {
            let mut metrics = stats.metrics(file.source_text.len());
            metrics.retain(|metric| metric.label != "file size");
            let mut rule_out = String::new();
            render_file_stats(&mut rule_out, &rule_name, &metrics, None);
            for line in rule_out.lines() {
                writeln!(out, "  {line}").unwrap();
            }
        }
        out.push('\n');
    }

    fs::write(snapshot_path, out)
}
