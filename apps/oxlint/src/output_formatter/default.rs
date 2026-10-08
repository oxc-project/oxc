use std::{fmt::Write, time::Duration};

use crate::output_formatter::{InternalFormatter, RuleTimings};
use oxc_diagnostics::{
    Error, GraphicalReportHandler,
    reporter::{DiagnosticReporter, DiagnosticResult},
};
use oxc_linter::{RuleTimingRecord, RuleTimingSource, table::RuleTable};
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct DefaultOutputFormatter;

impl InternalFormatter for DefaultOutputFormatter {
    fn all_rules(&self, enabled_rules: FxHashSet<(&str, &str)>) -> Option<String> {
        let mut output = String::new();
        let table = RuleTable::default();
        for section in &table.sections {
            output.push_str(&section.render_markdown_table_cli(&enabled_rules));
            output.push('\n');
        }
        output.push_str(format!("Default: {}\n", table.turned_on_by_default_count).as_str());
        output.push_str(format!("Total: {}\n", table.total).as_str());
        Some(output)
    }

    fn lint_command_info(&self, lint_command_info: &super::LintCommandInfo) -> Option<String> {
        let mut output = lint_command_info.format_execution_summary();

        if let Some(rule_timings) = &lint_command_info.rule_timings {
            if !rule_timings.records.is_empty() {
                output.push('\n');
                output.push_str(&format_rule_timing_table(&rule_timings.records));
            }
            if !rule_timings.js_plugin_runtime.is_zero() {
                output.push('\n');
                output.push_str(&format_js_plugin_timing_summary(rule_timings));
            }
        }

        #[cfg(feature = "memory")]
        if let Some(rule_memory) = &lint_command_info.rule_memory {
            output.push_str(&format_rule_memory_table(rule_memory));
        }

        Some(output)
    }

    #[cfg(not(any(test, feature = "testing")))]
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(GraphicalReporter::default())
    }

    #[cfg(any(test, feature = "testing"))]
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        use crate::output_formatter::default::test_implementation::GraphicalReporterTester;

        Box::new(GraphicalReporterTester::default())
    }
}

fn format_js_plugin_timing_summary(rule_timings: &RuleTimings) -> String {
    let runtime = rule_timings.js_plugin_runtime;
    let rule_callbacks = rule_timings
        .records
        .iter()
        .filter(|record| record.source == RuleTimingSource::JsPlugin)
        .map(|record| record.duration)
        .sum::<Duration>();
    let shared_overhead = runtime.saturating_sub(rule_callbacks);

    format!(
        "JS plugin runtime:\n  Total:           {:>10.3}ms\n  Rule callbacks:  {:>10.3}ms\n  Shared overhead: {:>10.3}ms\n",
        runtime.as_secs_f64() * 1000.0,
        rule_callbacks.as_secs_f64() * 1000.0,
        shared_overhead.as_secs_f64() * 1000.0,
    )
}

fn format_rule_timing_table(rule_timings: &[RuleTimingRecord]) -> String {
    let rule_names = rule_timings
        .iter()
        .map(|record| format!("{}/{}", record.plugin_name, record.rule_name))
        .collect::<Vec<_>>();

    let source_width = rule_timings
        .iter()
        .map(|record| record.source.as_str().len())
        .max()
        .unwrap_or("Source".len())
        .max("Source".len());
    let rule_width =
        rule_names.iter().map(String::len).max().unwrap_or("Rule".len()).max("Rule".len());
    let calls_width = rule_timings
        .iter()
        .map(|record| record.calls.to_string().len())
        .max()
        .unwrap_or("Calls".len())
        .max("Calls".len());
    let total_millis =
        rule_timings.iter().map(|record| record.duration.as_secs_f64() * 1000.0).sum::<f64>();

    let mut output = String::new();
    output.push_str("Rule timings:\n");
    writeln!(
        output,
        "{:<rule_width$}  {:>10}  {:>8}  {:>calls_width$}  Source",
        "Rule", "Time (ms)", "Relative", "Calls",
    )
    .unwrap();
    writeln!(
        output,
        "{:-<rule_width$}  {:-<10}  {:-<8}  {:-<calls_width$}  {:-<source_width$}",
        "", "", "", "", "",
    )
    .unwrap();

    for (record, rule_name) in rule_timings.iter().zip(rule_names) {
        let millis = record.duration.as_secs_f64() * 1000.0;
        let relative = if total_millis > 0.0 { millis / total_millis * 100.0 } else { 0.0 };
        writeln!(
            output,
            "{:<rule_width$}  {:>10.3}  {:>7.1}%  {:>calls_width$}  {}",
            rule_name,
            millis,
            relative,
            record.calls,
            record.source.as_str(),
        )
        .unwrap();
    }

    output
}

#[cfg(feature = "memory")]
fn format_rule_memory_table(records: &[RuleTimingRecord]) -> String {
    let mut records = records
        .iter()
        .filter(|record| record.source == RuleTimingSource::Native)
        .collect::<Vec<_>>();
    records.sort_unstable_by(|left, right| {
        right
            .memory
            .allocated_bytes
            .cmp(&left.memory.allocated_bytes)
            .then_with(|| left.plugin_name.cmp(&right.plugin_name))
            .then_with(|| left.rule_name.cmp(&right.rule_name))
    });

    let rule_width = records
        .iter()
        .map(|record| record.plugin_name.len() + 1 + record.rule_name.len())
        .max()
        .unwrap_or(0)
        .max("Total".len());
    let allocations = records.iter().map(|record| record.memory.allocations).sum::<u64>();
    let reallocations = records.iter().map(|record| record.memory.reallocations).sum::<u64>();
    let bytes = records.iter().map(|record| record.memory.allocated_bytes).sum::<u64>();
    let calls = records.iter().map(|record| record.calls).sum::<u64>();
    let allocs_width = allocations.to_string().len().max("Allocs".len());
    let reallocs_width = reallocations.to_string().len().max("Reallocs".len());
    let bytes_width = bytes.to_string().len().max("Bytes".len());
    let calls_width = calls.to_string().len().max("Calls".len());
    let mut output = String::from("\nRule memory (in native rules only):\n");
    writeln!(
        output,
        "{:<rule_width$}  {:>allocs_width$}  {:>reallocs_width$}  {:>bytes_width$}  {:>calls_width$}",
        "Rule", "Allocs", "Reallocs", "Bytes", "Calls",
    )
    .unwrap();
    writeln!(
        output,
        "{:-<rule_width$}  {:-<allocs_width$}  {:-<reallocs_width$}  {:-<bytes_width$}  {:-<calls_width$}",
        "", "", "", "", "",
    )
    .unwrap();

    for record in records {
        let name = format!("{}/{}", record.plugin_name, record.rule_name);
        writeln!(
            output,
            "{name:<rule_width$}  {:>allocs_width$}  {:>reallocs_width$}  {:>bytes_width$}  {:>calls_width$}",
            record.memory.allocations,
            record.memory.reallocations,
            record.memory.allocated_bytes,
            record.calls,
        )
        .unwrap();
    }
    writeln!(
        output,
        "{:<rule_width$}  {allocations:>allocs_width$}  {reallocations:>reallocs_width$}  {bytes:>bytes_width$}  {calls:>calls_width$}",
        "Total",
    )
    .unwrap();
    output
}

/// Pretty-prints diagnostics. Primarily meant for human-readable output in a terminal.
///
/// See [`GraphicalReportHandler`] for how to configure colors, context lines, etc.
#[cfg_attr(all(not(test), feature = "testing"), expect(dead_code))]
struct GraphicalReporter {
    handler: GraphicalReportHandler,
}

impl Default for GraphicalReporter {
    fn default() -> Self {
        Self { handler: GraphicalReportHandler::new() }
    }
}

impl DiagnosticReporter for GraphicalReporter {
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String> {
        Some(get_diagnostic_result_output(result))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let mut output = String::with_capacity(384);
        self.handler.render_report(&mut output, error.as_ref()).unwrap();
        Some(output)
    }

    fn render_errors(&mut self, errors: Vec<Error>, emit: &mut dyn FnMut(&str)) {
        let capacity = errors.len().checked_mul(384).unwrap_or(0);
        let mut output = String::with_capacity(capacity);
        self.handler
            .render_reports(
                &mut output,
                errors.iter().map(|error| error.as_ref() as &dyn oxc_diagnostics::Diagnostic),
            )
            .unwrap();
        emit(&output);
    }

    fn render_errors_until(
        &mut self,
        errors: Vec<Error>,
        keep: &mut dyn FnMut(Option<&str>, &str) -> bool,
    ) {
        self.handler
            .render_reports_until(
                errors.iter().map(|error| error.as_ref() as &dyn oxc_diagnostics::Diagnostic),
                &mut |diagnostic, rendered| {
                    let source_name = diagnostic.source_code().and_then(|source| source.name());
                    keep(source_name, rendered)
                },
            )
            .unwrap();
    }
}

pub(super) fn get_diagnostic_result_output(result: &DiagnosticResult) -> String {
    let mut output = String::new();

    if result.warnings_count() + result.errors_count() > 0 {
        output.push('\n');
    }

    output.push_str(
        format!(
            "Found {} warning{} and {} error{}.\n",
            result.warnings_count(),
            if result.warnings_count() == 1 { "" } else { "s" },
            result.errors_count(),
            if result.errors_count() == 1 { "" } else { "s" },
        )
        .as_str(),
    );

    if result.max_warnings_exceeded() {
        output.push_str(
            format!("Exceeded maximum number of warnings. Found {}.\n", result.warnings_count())
                .as_str(),
        );
    }

    output
}

#[cfg(any(test, feature = "testing"))]
mod test_implementation {
    use oxc_diagnostics::{
        Error, GraphicalReportHandler, GraphicalTheme,
        reporter::{DiagnosticReporter, DiagnosticResult, Info},
    };

    use crate::output_formatter::default::get_diagnostic_result_output;

    #[derive(Default)]
    pub struct GraphicalReporterTester {
        diagnostics: Vec<Error>,
    }

    impl DiagnosticReporter for GraphicalReporterTester {
        fn finish(&mut self, result: &DiagnosticResult) -> Option<String> {
            let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none())
                // links print ansi escape codes, which makes snapshots harder to read
                .with_links(false);
            let mut output = String::new();

            self.diagnostics.sort_by_cached_key(|diagnostic| {
                let info = Info::new(diagnostic);
                (info.filename, info.start, info.end, info.rule_id, info.message)
            });

            for diagnostic in &self.diagnostics {
                handler.render_report(&mut output, diagnostic.as_ref()).unwrap();
            }

            output.push_str(&get_diagnostic_result_output(result));

            Some(output)
        }

        fn render_error(&mut self, error: Error) -> Option<String> {
            self.diagnostics.push(error);
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::output_formatter::{
        InternalFormatter, LintCommandInfo, OxlintSuppressionFileAction, RuleTimings,
        default::{DefaultOutputFormatter, GraphicalReporter, format_js_plugin_timing_summary},
    };
    use oxc_diagnostics::reporter::{DiagnosticReporter, DiagnosticResult};
    #[cfg(feature = "memory")]
    use oxc_linter::memory::AllocationStats;
    use oxc_linter::{RuleTimingRecord, RuleTimingSource};
    use rustc_hash::FxHashSet;

    #[test]
    fn all_rules() {
        let formatter = DefaultOutputFormatter;
        let result = formatter.all_rules(FxHashSet::default());

        assert!(result.is_some());
    }

    #[test]
    fn lint_command_info() {
        let formatter = DefaultOutputFormatter;
        let result = formatter.lint_command_info(&LintCommandInfo {
            number_of_files: 5,
            number_of_rules: Some(10),
            threads_count: 12,
            start_time: Duration::new(1, 0),
            oxlint_suppression_file_action: OxlintSuppressionFileAction::None,
            rule_timings: None,
            #[cfg(feature = "memory")]
            rule_memory: None,
        });

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "Finished in 1.0s on 5 files with 10 rules using 12 threads.\n"
        );
    }

    #[test]
    fn lint_command_info_unknown_rules() {
        let formatter = DefaultOutputFormatter;
        let result = formatter.lint_command_info(&LintCommandInfo {
            number_of_files: 5,
            number_of_rules: None,
            threads_count: 12,
            start_time: Duration::new(1, 0),
            oxlint_suppression_file_action: OxlintSuppressionFileAction::None,
            rule_timings: None,
            #[cfg(feature = "memory")]
            rule_memory: None,
        });

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Finished in 1.0s on 5 files using 12 threads.\n");
    }

    #[test]
    fn lint_command_info_oxlint_suppression_file_created() {
        let formatter = DefaultOutputFormatter;
        let result = formatter.lint_command_info(&LintCommandInfo {
            number_of_files: 5,
            number_of_rules: None,
            threads_count: 12,
            start_time: Duration::new(1, 0),
            oxlint_suppression_file_action: OxlintSuppressionFileAction::Created,
            rule_timings: None,
            #[cfg(feature = "memory")]
            rule_memory: None,
        });

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "Created 'oxlint-suppressions.json' in the root folder.\nFinished in 1.0s on 5 files using 12 threads.\n"
        );
    }

    #[test]
    fn lint_command_info_oxlint_suppression_file_updated() {
        let formatter = DefaultOutputFormatter;
        let result = formatter.lint_command_info(&LintCommandInfo {
            number_of_files: 5,
            number_of_rules: None,
            threads_count: 12,
            start_time: Duration::new(1, 0),
            oxlint_suppression_file_action: OxlintSuppressionFileAction::Updated,
            rule_timings: None,
            #[cfg(feature = "memory")]
            rule_memory: None,
        });

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "Updated 'oxlint-suppressions.json'.\nFinished in 1.0s on 5 files using 12 threads.\n"
        );
    }

    #[test]
    fn lint_command_info_shows_rule_timings() {
        let formatter = DefaultOutputFormatter;
        let result = formatter.lint_command_info(&LintCommandInfo {
            number_of_files: 1,
            number_of_rules: Some(2),
            threads_count: 1,
            start_time: Duration::from_millis(5),
            oxlint_suppression_file_action: OxlintSuppressionFileAction::None,
            rule_timings: Some(RuleTimings {
                records: vec![
                    RuleTimingRecord {
                        source: RuleTimingSource::Native,
                        plugin_name: "eslint".to_string(),
                        rule_name: "no-debugger".to_string(),
                        duration: Duration::from_micros(1500),
                        calls: 3,
                        #[cfg(feature = "memory")]
                        memory: AllocationStats::default(),
                    },
                    RuleTimingRecord {
                        source: RuleTimingSource::TypeAware,
                        plugin_name: "typescript".to_string(),
                        rule_name: "no-floating-promises".to_string(),
                        duration: Duration::from_micros(500),
                        calls: 0,
                        #[cfg(feature = "memory")]
                        memory: AllocationStats::default(),
                    },
                ],
                js_plugin_runtime: Duration::ZERO,
            }),
            #[cfg(feature = "memory")]
            rule_memory: None,
        });

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "Finished in 5ms on 1 file with 2 rules using 1 threads.\n\nRule timings:\nRule                              Time (ms)  Relative  Calls  Source\n-------------------------------  ----------  --------  -----  ----------\neslint/no-debugger                    1.500     75.0%      3  native\ntypescript/no-floating-promises       0.500     25.0%      0  type-aware\n"
        );
    }

    #[cfg(feature = "memory")]
    #[test]
    fn memory_table_sorts_and_totals_native_rules() {
        let record = |source, plugin: &str, rule: &str, bytes| RuleTimingRecord {
            source,
            plugin_name: plugin.to_string(),
            rule_name: rule.to_string(),
            duration: Duration::ZERO,
            calls: 1,
            memory: AllocationStats { allocations: 2, reallocations: 3, allocated_bytes: bytes },
        };
        let output = super::format_rule_memory_table(&[
            record(RuleTimingSource::Native, "eslint", "z", 16),
            record(RuleTimingSource::JsPlugin, "external", "a", 100),
            record(RuleTimingSource::TypeAware, "typescript", "b", 200),
            record(RuleTimingSource::Native, "eslint", "a", 16),
            record(RuleTimingSource::Native, "jsdoc", "b", 32),
        ]);
        let rows = output.lines().skip(1).collect::<Vec<_>>();
        assert_eq!(
            rows,
            [
                "Rule memory (in native rules only):",
                "Rule      Allocs  Reallocs  Bytes  Calls",
                "--------  ------  --------  -----  -----",
                "jsdoc/b        2         3     32      1",
                "eslint/a       2         3     16      1",
                "eslint/z       2         3     16      1",
                "Total          6         9     64      3",
            ]
        );
        assert!(!output.contains("external/a"));
        assert!(!output.contains("typescript/b"));
    }

    #[test]
    fn js_plugin_timing_summary() {
        let result = format_js_plugin_timing_summary(&RuleTimings {
            records: vec![RuleTimingRecord {
                source: RuleTimingSource::JsPlugin,
                plugin_name: "example".to_string(),
                rule_name: "rule".to_string(),
                duration: Duration::from_micros(700),
                calls: 2,
                #[cfg(feature = "memory")]
                memory: AllocationStats::default(),
            }],
            js_plugin_runtime: Duration::from_millis(1),
        });

        assert_eq!(
            result,
            "JS plugin runtime:\n  Total:                1.000ms\n  Rule callbacks:       0.700ms\n  Shared overhead:      0.300ms\n"
        );
    }

    #[test]
    fn reporter_finish_no_results() {
        let mut reporter = GraphicalReporter::default();

        let result = reporter.finish(&DiagnosticResult::default());

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Found 0 warnings and 0 errors.\n");
    }

    #[test]
    fn reporter_finish_one_warning_and_one_error() {
        let mut reporter = GraphicalReporter::default();

        let result = reporter.finish(&DiagnosticResult::new(1, 1, false));

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "\nFound 1 warning and 1 error.\n");
    }

    #[test]
    fn reporter_finish_multiple_warning_and_errors() {
        let mut reporter = GraphicalReporter::default();

        let result = reporter.finish(&DiagnosticResult::new(6, 4, false));

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "\nFound 6 warnings and 4 errors.\n");
    }

    #[test]
    fn reporter_finish_exceeded_warnings() {
        let mut reporter = GraphicalReporter::default();

        let result = reporter.finish(&DiagnosticResult::new(6, 4, true));

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "\nFound 6 warnings and 4 errors.\nExceeded maximum number of warnings. Found 6.\n"
        );
    }
}
