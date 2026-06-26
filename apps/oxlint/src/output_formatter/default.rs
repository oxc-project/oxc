use std::fmt::Write;

use crate::output_formatter::InternalFormatter;
use oxc_diagnostics::{
    Error, GraphicalReportHandler,
    reporter::{DiagnosticReporter, DiagnosticResult},
};
use oxc_linter::{RuleTimingRecord, table::RuleTable};
use rustc_hash::FxHashSet;

#[derive(Debug)]
pub struct DefaultOutputFormatter;

impl InternalFormatter for DefaultOutputFormatter {
    fn all_rules(&self, enabled_rules: FxHashSet<&str>) -> Option<String> {
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

        if let Some(rule_timings) = &lint_command_info.rule_timings
            && !rule_timings.is_empty()
        {
            output.push('\n');
            output.push_str(&format_rule_timing_table(rule_timings));
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
        let mut output = String::new();
        self.handler.render_report(&mut output, error.as_ref()).unwrap();
        Some(output)
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
        InternalFormatter, LintCommandInfo, OxlintSuppressionFileAction,
        default::{DefaultOutputFormatter, GraphicalReporter},
    };
    use oxc_diagnostics::reporter::{DiagnosticReporter, DiagnosticResult};
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
            rule_timings: Some(vec![
                RuleTimingRecord {
                    source: RuleTimingSource::Native,
                    plugin_name: "eslint".to_string(),
                    rule_name: "no-debugger".to_string(),
                    duration: Duration::from_micros(1500),
                    calls: 3,
                },
                RuleTimingRecord {
                    source: RuleTimingSource::TypeAware,
                    plugin_name: "typescript".to_string(),
                    rule_name: "no-floating-promises".to_string(),
                    duration: Duration::from_micros(500),
                    calls: 0,
                },
            ]),
        });

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "Finished in 5ms on 1 file with 2 rules using 1 threads.\n\nRule timings:\nRule                              Time (ms)  Relative  Calls  Source\n-------------------------------  ----------  --------  -----  ----------\neslint/no-debugger                    1.500     75.0%      3  native\ntypescript/no-floating-promises       0.500     25.0%      0  type-aware\n"
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
