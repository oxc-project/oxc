use std::time::Duration;

use crate::output_formatter::InternalFormatter;
use oxc_diagnostics::{
    Error, GraphicalReportHandler, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};
use oxc_linter::table::RuleTable;
use rustc_hash::{FxHashMap, FxHashSet};

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
        let time = Self::get_execution_time(&lint_command_info.start_time);
        let s = if lint_command_info.number_of_files == 1 { "" } else { "s" };

        if let Some(number_of_rules) = lint_command_info.number_of_rules {
            Some(format!(
                "Finished in {time} on {} file{s} with {} rules using {} threads.\n",
                lint_command_info.number_of_files, number_of_rules, lint_command_info.threads_count
            ))
        } else {
            Some(format!(
                "Finished in {time} on {} file{s} using {} threads.\n",
                lint_command_info.number_of_files, lint_command_info.threads_count
            ))
        }
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

impl DefaultOutputFormatter {
    fn get_execution_time(duration: &Duration) -> String {
        let ms = duration.as_millis();
        if ms < 1000 { format!("{ms}ms") } else { format!("{:.1}s", duration.as_secs_f64()) }
    }
}

/// Pretty-prints diagnostics. Primarily meant for human-readable output in a terminal.
///
/// See [`GraphicalReportHandler`] for how to configure colors, context lines, etc.
#[cfg_attr(all(not(test), feature = "testing"), expect(dead_code))]
struct GraphicalReporter {
    handler: GraphicalReportHandler,
    file_summary: FxHashMap<String, FileDiagnosticCount>,
}

impl Default for GraphicalReporter {
    fn default() -> Self {
        Self { handler: GraphicalReportHandler::new(), file_summary: FxHashMap::default() }
    }
}

impl DiagnosticReporter for GraphicalReporter {
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String> {
        Some(get_diagnostic_result_output(result, &self.file_summary))
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        update_file_summary(&error, &mut self.file_summary);
        let mut output = String::new();
        self.handler.render_report(&mut output, error.as_ref()).unwrap();
        Some(output)
    }
}

const UNKNOWN_FILENAME: &str = "<unknown>";

#[derive(Clone, Copy, Debug, Default)]
struct FileDiagnosticCount {
    warnings: usize,
    errors: usize,
    first_error_line: Option<usize>,
}

fn update_file_summary(error: &Error, file_summary: &mut FxHashMap<String, FileDiagnosticCount>) {
    let (warning_count, error_count) = match error.severity() {
        Some(Severity::Warning) => (1, 0),
        Some(Severity::Error) | None => (0, 1),
        _ => (0, 0),
    };

    if warning_count == 0 && error_count == 0 {
        return;
    }

    let info = Info::new(error);
    let filename =
        if info.filename.is_empty() { UNKNOWN_FILENAME.to_string() } else { info.filename };

    let count = file_summary.entry(filename).or_default();
    count.warnings += warning_count;
    count.errors += error_count;
    if error_count > 0 && info.start.line > 0 {
        count.first_error_line = match count.first_error_line {
            Some(existing) => Some(existing.min(info.start.line)),
            None => Some(info.start.line),
        };
    }
}

fn get_diagnostic_result_output(
    result: &DiagnosticResult,
    file_summary: &FxHashMap<String, FileDiagnosticCount>,
) -> String {
    let mut output = String::new();
    let warnings_count = result.warnings_count();
    let errors_count = result.errors_count();
    let problems_count = warnings_count + errors_count;

    if problems_count > 0 {
        output.push('\n');
    }

    let error_file_count = file_summary.values().filter(|count| count.errors > 0).count();

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
    if let Some(summary) = format_error_summary(file_summary) {
        output.push('\n');
        output.push_str(summary.as_str());
    }

    if result.max_warnings_exceeded() {
        output.push_str(
            format!("Exceeded maximum number of warnings. Found {}.\n", result.warnings_count())
                .as_str(),
        );
    }

    output
}

fn format_error_summary(file_summary: &FxHashMap<String, FileDiagnosticCount>) -> Option<String> {
    let mut rows: Vec<(&str, FileDiagnosticCount)> =
        file_summary.iter().map(|(file, count)| (file.as_str(), *count)).collect();
    rows.retain(|(_, count)| count.errors > 0);
    rows.sort_unstable_by(|(left_file, left_count), (right_file, right_count)| {
        right_count.errors.cmp(&left_count.errors).then_with(|| left_file.cmp(right_file))
    });

    if rows.is_empty() {
        return None;
    }

    let error_width = rows
        .iter()
        .map(|(_, count)| count.errors.to_string().len())
        .max()
        .unwrap_or(0)
        .max("Errors".len());

    let mut output = String::new();
    output.push_str(format!("{:>error_width$}  Files\n", "Errors").as_str());

    for (filename, count) in rows {
        let file_location = match count.first_error_line {
            Some(line) => format!("{filename}:{line}"),
            None => filename.to_string(),
        };
        output.push_str(format!("{:>error_width$}  {file_location}\n", count.errors).as_str());
    }

    output.push('\n');

    Some(output)
}

fn get_plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(any(test, feature = "testing"))]
mod test_implementation {
    use rustc_hash::FxHashMap;

    use super::{FileDiagnosticCount, get_diagnostic_result_output, update_file_summary};
    use oxc_diagnostics::{
        Error, GraphicalReportHandler, GraphicalTheme,
        reporter::{DiagnosticReporter, DiagnosticResult, Info},
    };

    #[derive(Default)]
    pub struct GraphicalReporterTester {
        diagnostics: Vec<Error>,
        file_summary: FxHashMap<String, FileDiagnosticCount>,
        seen_warnings: usize,
        seen_errors: usize,
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

            output.push_str(&get_diagnostic_result_output(result, &self.file_summary));

            Some(output)
        }

        fn render_error(&mut self, error: Error) -> Option<String> {
            update_file_summary(&error, &mut self.file_summary);
            self.diagnostics.push(error);
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use rustc_hash::{FxHashMap, FxHashSet};

    use super::{
        DefaultOutputFormatter, FileDiagnosticCount, GraphicalReporter,
        get_diagnostic_result_output,
    };
    use crate::output_formatter::{InternalFormatter, LintCommandInfo};
    use oxc_diagnostics::reporter::{DiagnosticReporter, DiagnosticResult};

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
        });

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "Finished in 1.0s on 5 files using 12 threads.\n");
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

    // #[test]
    // fn summary_with_file_table() {
    //     let mut file_summary = FxHashMap::default();
    //     file_summary.insert(
    //         "src/bar.ts".to_string(),
    //         FileDiagnosticCount { warnings: 2, errors: 2, first_error_line: Some(3) },
    //     );
    //     file_summary.insert(
    //         "src/foo.ts".to_string(),
    //         FileDiagnosticCount { warnings: 0, errors: 1, first_error_line: Some(17) },
    //     );
    //     file_summary.insert(
    //         "src/warn-only.ts".to_string(),
    //         FileDiagnosticCount { warnings: 3, errors: 0, first_error_line: None },
    //     );

    //     let output =
    //         get_diagnostic_result_output(&DiagnosticResult::new(5, 3, false), &file_summary, 5, 3);

    //     assert_eq!(
    //         output,
    //         "\nFound 3 errors in 2 files.\n\nErrors  Files\n     2  src/bar.ts:3\n     1  src/foo.ts:17\n\n"
    //     );
    // }

    // #[test]
    // fn summary_with_file_table_lists_all_files() {
    //     let mut file_summary = FxHashMap::default();
    //     for index in 0..12 {
    //         file_summary.insert(
    //             format!("src/file-{index:02}.ts"),
    //             FileDiagnosticCount { warnings: 0, errors: 1, first_error_line: Some(index + 1) },
    //         );
    //     }

    //     let output = get_diagnostic_result_output(
    //         &DiagnosticResult::new(0, 12, false),
    //         &file_summary,
    //         0,
    //         12,
    //     );

    //     assert!(output.contains("Found 12 errors in 12 files."));
    //     assert!(output.contains("src/file-00.ts"));
    //     assert!(output.contains("src/file-11.ts"));
    // }

    // #[test]
    // fn summary_falls_back_to_legacy_when_counts_are_incomplete() {
    //     let mut file_summary = FxHashMap::default();
    //     file_summary.insert(
    //         "src/file.ts".to_string(),
    //         FileDiagnosticCount { warnings: 0, errors: 1, first_error_line: Some(4) },
    //     );

    //     let output =
    //         get_diagnostic_result_output(&DiagnosticResult::new(3, 1, false), &file_summary, 0, 1);

    //     assert_eq!(output, "\nFound 3 warnings and 1 error.\n");
    // }
}
