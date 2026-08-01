use std::{
    fmt::Write as _,
    fs::File,
    io::{self, Write},
    path::Path,
    sync::Arc,
};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_linter::{
    ConfigStore, ConfigStoreBuilder, ContextSubHost, ContextSubHostOptions, ExternalPluginStore,
    LintOptions, Linter, ModuleRecord, RuleTimingRecord, RuleTimingStore,
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_tasks_common::{TestFiles, project_root};

const SNAPSHOT_PATH: &str = "tasks/track_linter_timings/linter_timings.snap";

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    run().unwrap();
}

/// # Panics
/// # Errors
pub fn run() -> Result<(), io::Error> {
    let linter = create_linter();
    let rule_timing_store = RuleTimingStore::new();

    for file in TestFiles::complicated().files() {
        let allocator = Allocator::default();
        let parser_ret = Parser::new(&allocator, &file.source_text, file.source_type).parse();
        assert!(parser_ret.diagnostics.is_empty());

        let path = Path::new(&file.file_name);
        let semantic = SemanticBuilder::new_linter().build(&parser_ret.program).semantic;
        let module_record = Arc::new(ModuleRecord::new(path, &parser_ret.module_record, &semantic));

        let _ = linter.run_with_disable_directives::<true>(
            path,
            vec![ContextSubHost::new(
                semantic,
                Arc::clone(&module_record),
                0,
                ContextSubHostOptions::default(),
            )],
            &allocator,
            None,
            Some(&rule_timing_store),
        );
    }

    let mut records = rule_timing_store.collect();
    records.sort_unstable_by(|left, right| {
        left.plugin_name.cmp(&right.plugin_name).then_with(|| left.rule_name.cmp(&right.rule_name))
    });

    write_snapshot(SNAPSHOT_PATH, &format_snapshot(&records))
}

fn create_linter() -> Linter {
    let mut external_plugin_store = ExternalPluginStore::default();
    let lint_config = ConfigStoreBuilder::all().build(&mut external_plugin_store).unwrap();
    Linter::new(
        LintOptions::default(),
        ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store),
        None,
    )
}

fn format_snapshot(records: &[RuleTimingRecord]) -> String {
    let rule_names = records
        .iter()
        .map(|record| format!("{}/{}", record.plugin_name, record.rule_name))
        .collect::<Vec<_>>();
    let rule_width =
        rule_names.iter().map(String::len).max().unwrap_or("Rule".len()).max("Rule".len());
    let calls_width = records
        .iter()
        .map(|record| record.calls.to_string().len())
        .max()
        .unwrap_or("Calls".len())
        .max("Calls".len());

    let mut output = String::new();
    writeln!(output, "{:<rule_width$} | {:>calls_width$}", "Rule", "Calls").unwrap();
    writeln!(output, "{:-<rule_width$}-+-{:-<calls_width$}", "", "").unwrap();

    for (record, rule_name) in records.iter().zip(rule_names) {
        let calls = record.calls;
        writeln!(output, "{rule_name:<rule_width$} | {calls:>calls_width$}").unwrap();
    }

    output
}

fn write_snapshot(path: &str, out: &str) -> Result<(), io::Error> {
    let mut file = File::create(project_root().join(path))?;
    file.write_all(out.as_bytes())
}
