use std::borrow::Cow;

use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};

use super::default::get_diagnostic_result_output;
use crate::output_formatter::InternalFormatter;

#[derive(Debug)]
pub struct GithubOutputFormatter;

impl InternalFormatter for GithubOutputFormatter {
    fn lint_command_info(&self, lint_command_info: &super::LintCommandInfo) -> Option<String> {
        Some(lint_command_info.format_execution_summary())
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(GithubReporter)
    }
}

/// Formats reports using [GitHub Actions
/// annotations](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions#setting-an-error-message). Useful for reporting in CI.
struct GithubReporter;

impl DiagnosticReporter for GithubReporter {
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String> {
        Some(get_diagnostic_result_output(result))
    }

    fn supports_minified_file_fallback(&self) -> bool {
        false
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        Some(format_github(&error))
    }
}

fn format_github(diagnostic: &Error) -> String {
    let Info { start, end, filename, message, severity, rule_id } = Info::new(diagnostic);
    let severity = match severity {
        Severity::Error => "error",
        Severity::Warning | miette::Severity::Advice => "warning",
    };
    let title = rule_id.map_or(Cow::Borrowed("oxlint"), Cow::Owned);
    let filename = escape_property(&filename);

    if filename.is_empty() {
        let severity = match diagnostic.severity() {
            Some(Severity::Error) | None => "error",
            Some(Severity::Warning | miette::Severity::Advice) => "warning",
        };
        let message = diagnostic.to_string();
        let message = escape_data(&message);
        format!("::{severity} title={title}::{message}\n")
    } else {
        let message = escape_data(&message);
        format!(
            "::{severity} file={filename},line={},endLine={},col={},endColumn={},title={title}::{message}\n",
            start.line, end.line, start.column, end.column
        )
    }
}

fn escape_data(value: &str) -> String {
    // Refs:
    // - https://github.com/actions/runner/blob/a4c57f27477077e57545af79851551ff7f5632bd/src/Runner.Common/ActionCommand.cs#L18-L22
    // - https://github.com/actions/toolkit/blob/fe3e7ce9a7f995d29d1fcfd226a32bca407f9dc8/packages/core/src/command.ts#L80-L94
    let mut result = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\r' => result.push_str("%0D"),
            '\n' => result.push_str("%0A"),
            '%' => result.push_str("%25"),
            _ => result.push(c),
        }
    }
    result
}

fn escape_property(value: &str) -> String {
    // Refs:
    // - https://github.com/actions/runner/blob/a4c57f27477077e57545af79851551ff7f5632bd/src/Runner.Common/ActionCommand.cs#L25-L32
    // - https://github.com/actions/toolkit/blob/fe3e7ce9a7f995d29d1fcfd226a32bca407f9dc8/packages/core/src/command.ts#L80-L94
    let mut result = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\r' => result.push_str("%0D"),
            '\n' => result.push_str("%0A"),
            ':' => result.push_str("%3A"),
            ',' => result.push_str("%2C"),
            '%' => result.push_str("%25"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use oxc_diagnostics::{
        DiagnosticService, NamedSource, OxcDiagnostic,
        reporter::{DiagnosticReporter, DiagnosticResult},
    };
    use oxc_span::Span;

    use super::{GithubOutputFormatter, GithubReporter};
    use crate::output_formatter::{InternalFormatter, LintCommandInfo};

    #[test]
    fn lint_command_info() {
        let formatter = GithubOutputFormatter;
        let result = formatter.lint_command_info(&LintCommandInfo {
            number_of_files: 5,
            number_of_rules: Some(10),
            threads_count: 12,
            start_time: Duration::new(1, 0),
        });

        assert_eq!(
            result.unwrap(),
            "Finished in 1.0s on 5 files with 10 rules using 12 threads.\n"
        );
    }

    #[test]
    fn lint_command_info_unknown_rules() {
        let formatter = GithubOutputFormatter;
        let result = formatter.lint_command_info(&LintCommandInfo {
            number_of_files: 5,
            number_of_rules: None,
            threads_count: 12,
            start_time: Duration::new(1, 0),
        });

        assert_eq!(result.unwrap(), "Finished in 1.0s on 5 files using 12 threads.\n");
    }

    #[test]
    fn reporter_finish() {
        let mut reporter = GithubReporter;

        let result = reporter.finish(&DiagnosticResult::default());

        assert_eq!(result.unwrap(), "Found 0 warnings and 0 errors.\n");
    }

    #[test]
    fn reporter_finish_with_errors() {
        let mut reporter = GithubReporter;

        let result = reporter.finish(&DiagnosticResult::new(2, 1, false));

        assert_eq!(result.unwrap(), "\nFound 2 warnings and 1 error.\n");
    }

    #[test]
    fn reporter_error() {
        let mut reporter = GithubReporter;
        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let result = reporter.render_error(error);

        assert!(result.is_some());
        assert_eq!(
            result.unwrap(),
            "::warning file=file%3A//test.ts,line=1,endLine=1,col=1,endColumn=9,title=oxlint::error message\n"
        );
    }

    #[test]
    fn reporter_error_without_labels_omits_file_and_location() {
        let mut reporter = GithubReporter;
        let error = OxcDiagnostic::warn("warning message")
            .with_error_code("scope", "rule")
            .with_help("help message")
            .with_note("note message");

        let result = reporter.render_error(error.into());

        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap(), "::warning title=scope(rule)::warning message\n");
    }

    #[test]
    fn reporter_fileless_error_uses_error_annotation() {
        let mut reporter = GithubReporter;
        let error = OxcDiagnostic::error("error message")
            .with_error_code("scope", "rule")
            .with_help("help message")
            .with_note("note message");

        let result = reporter.render_error(error.into());

        assert!(result.is_some());
        assert_eq!(result.as_ref().unwrap(), "::error title=scope(rule)::error message\n");
    }

    #[test]
    fn reporter_does_not_use_minified_fallback_for_long_annotations() {
        let source_text = format!("{}\n", "a".repeat(1300));
        let diagnostic = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 1300))
            .with_source_code(NamedSource::new("file://test.ts", source_text));

        let (mut service, sender) = DiagnosticService::new(Box::new(GithubReporter));
        sender.send(vec![diagnostic]).unwrap();
        drop(sender);

        let mut output = Vec::new();
        service.run(&mut output);
        let output = String::from_utf8(output).unwrap();

        assert!(output.starts_with("::warning file=file%3A//test.ts,line=1,endLine=1,col=1,"));
        assert!(output.contains("title=oxlint::error message"));
        assert!(!output.contains("File is too long to fit on the screen"));
        assert!(!output.contains("file=,line=0,endLine=0,col=0,endColumn=0"));
    }
}
