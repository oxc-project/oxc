use std::{io::Write, time::Duration};

use oxc_diagnostics::{
    reporter::{DiagnosticReporter, DiagnosticResult},
    Error, GraphicalReportHandler,
};
use oxc_linter::table::RuleTable;

use crate::output_formatter::InternalFormatter;

#[derive(Debug)]
pub struct DefaultOutputFormatter;

impl InternalFormatter for DefaultOutputFormatter {
    fn all_rules(&mut self, writer: &mut dyn Write) {
        let table = RuleTable::new();
        for section in table.sections {
            writeln!(writer, "{}", section.render_markdown_table(None)).unwrap();
        }
        writeln!(writer, "Default: {}", table.turned_on_by_default_count).unwrap();
        writeln!(writer, "Total: {}", table.total).unwrap();
    }

    fn lint_command_info(&self, lint_command_info: &super::LintCommandInfo) -> Option<String> {
        let time = Self::get_execution_time(&lint_command_info.start_time);
        let s = if lint_command_info.number_of_files == 1 { "" } else { "s" };

        Some(format!(
            "Finished in {time} on {} file{s} with {} rules using {} threads.\n",
            lint_command_info.number_of_files,
            lint_command_info.number_of_rules,
            lint_command_info.threads_count
        ))
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(GraphicalReporter::default())
    }
}

impl DefaultOutputFormatter {
    fn get_execution_time(duration: &Duration) -> String {
        let ms = duration.as_millis();
        if ms < 1000 {
            format!("{ms}ms")
        } else {
            format!("{:.1}s", duration.as_secs_f64())
        }
    }
}

/// Pretty-prints diagnostics. Primarily meant for human-readable output in a terminal.
///
/// See [`GraphicalReportHandler`] for how to configure colors, context lines, etc.
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
                format!(
                    "Exceeded maximum number of warnings. Found {}.\n",
                    result.warnings_count()
                )
                .as_str(),
            );
        }

        Some(output)
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        let mut output = String::new();
        self.handler.render_report(&mut output, error.as_ref()).unwrap();
        Some(output)
    }
}
impl GraphicalReporter {
    #[cfg(test)]
    pub fn with_handler(mut self, handler: GraphicalReportHandler) -> Self {
        self.handler = handler;
        self
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::output_formatter::{
        default::{DefaultOutputFormatter, GraphicalReporter},
        InternalFormatter, LintCommandInfo,
    };
    use miette::{GraphicalReportHandler, GraphicalTheme, NamedSource};
    use oxc_diagnostics::{
        reporter::{DiagnosticReporter, DiagnosticResult},
        OxcDiagnostic,
    };
    use oxc_span::Span;

    #[test]
    fn all_rules() {
        let mut writer = Vec::new();
        let mut formatter = DefaultOutputFormatter;

        formatter.all_rules(&mut writer);
        assert!(!writer.is_empty());
    }

    #[test]
    fn lint_command_info() {
        let formatter = DefaultOutputFormatter;
        let result = formatter.lint_command_info(&LintCommandInfo {
            number_of_files: 5,
            number_of_rules: 10,
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

    #[test]
    fn reporter_error() {
        let mut reporter = GraphicalReporter::default().with_handler(
            GraphicalReportHandler::new_themed(GraphicalTheme::none()).with_links(false),
        );

        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let result = reporter.render_error(error);

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "\n  ! error message\n   ,-[file://test.ts:1:1]\n 1 | debugger;\n   : ^^^^^^^^\n   `----\n"
        );
    }
}
