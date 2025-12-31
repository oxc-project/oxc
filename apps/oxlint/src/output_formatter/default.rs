use std::time::Duration;

use crate::output_formatter::InternalFormatter;
use oxc_diagnostics::{
    Error, GraphicalReportHandler,
    reporter::{DiagnosticReporter, DiagnosticResult},
};
use oxc_linter::table::RuleTable;

#[derive(Debug)]
pub struct DefaultOutputFormatter;

impl InternalFormatter for DefaultOutputFormatter {
    fn all_rules(&self) -> Option<String> {
        let mut output = String::new();
        let table = RuleTable::default();
        for section in table.sections {
            output.push_str(section.render_markdown_table(None).as_str());
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

fn get_diagnostic_result_output(result: &DiagnosticResult) -> String {
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

    let total_fixes = result.safe_fixes_available()
        + result.dangerous_fixes_available()
        + result.suggestions_available();
    if total_fixes > 0 {
        let mut fix_option = "--fix".to_string();
        if result.dangerous_fixes_available() > 0 {
            fix_option.push_str(" --fix-dangerously");
        }
        if result.suggestions_available() > 0 {
            fix_option.push_str(" --fix-suggestions");
        }
        output.push_str(
            format!(
                "╰ {} {} available with '{}'\n",
                total_fixes,
                if total_fixes == result.suggestions_available() {
                    if total_fixes == 1 { "suggestion is" } else { "suggestions are" }
                } else if total_fixes == 1 {
                    "fix is"
                } else {
                    "fixes are"
                },
                fix_option
            )
            .as_str(),
        );
    }

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
            let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none());
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
        InternalFormatter, LintCommandInfo,
        default::{DefaultOutputFormatter, GraphicalReporter},
    };
    use oxc_diagnostics::reporter::{DiagnosticReporter, DiagnosticResult};

    #[test]
    fn all_rules() {
        let formatter = DefaultOutputFormatter;
        let result = formatter.all_rules();

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

        let result = reporter.finish(&DiagnosticResult::new(1, 1, 0, 0, 0, false));

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "\nFound 1 warning and 1 error.\n");
    }

    #[test]
    fn reporter_finish_multiple_warning_and_errors() {
        let mut reporter = GraphicalReporter::default();

        let result = reporter.finish(&DiagnosticResult::new(6, 4, 0, 0, 0, false));

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "\nFound 6 warnings and 4 errors.\n");
    }

    #[test]
    fn reporter_finish_exceeded_warnings() {
        let mut reporter = GraphicalReporter::default();

        let result = reporter.finish(&DiagnosticResult::new(6, 4, 0, 0, 0, true));

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "\nFound 6 warnings and 4 errors.\nExceeded maximum number of warnings. Found 6.\n"
        );
    }

    #[test]
    fn reporter_finish_with_fixes_and_suggestions() {
        let result =
            GraphicalReporter::default().finish(&DiagnosticResult::new(2, 3, 5, 1, 4, false));

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "\nFound 2 warnings and 3 errors.\n╰ 10 fixes are available with '--fix --fix-dangerously --fix-suggestions'"
        );

        let result =
            GraphicalReporter::default().finish(&DiagnosticResult::new(2, 3, 0, 1, 4, false));

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "\nFound 2 warnings and 3 errors.\n╰ 5 fixes are available with '--fix --fix-dangerously --fix-suggestions'"
        );

        let result =
            GraphicalReporter::default().finish(&DiagnosticResult::new(2, 3, 0, 1, 0, false));

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "\nFound 2 warnings and 3 errors.\n╰ 1 fix is available with '--fix --fix-dangerously'"
        );

        let result =
            GraphicalReporter::default().finish(&DiagnosticResult::new(2, 3, 0, 0, 1, false));

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "\nFound 2 warnings and 3 errors.\n╰ 1 suggestion is available with '--fix --fix-suggestions'"
        );

        let result =
            GraphicalReporter::default().finish(&DiagnosticResult::new(2, 3, 0, 0, 2, false));

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "\nFound 2 warnings and 3 errors.\n╰ 2 suggestions are available with '--fix --fix-suggestions'"
        );
    }
}
