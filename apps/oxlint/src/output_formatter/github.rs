use std::borrow::Cow;
use std::path::Path;

use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};

use super::default::get_diagnostic_result_output;
use crate::output_formatter::InternalFormatter;

#[derive(Debug)]
pub struct GithubOutputFormatter;

impl InternalFormatter for GithubOutputFormatter {
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
    let filename_for_message = display_filename(&filename);
    let filename = escape_property(&filename);
    let message = escape_data(&format_message(&filename_for_message, start.line, &message));
    format!(
        "::{severity} file={filename},line={},endLine={},col={},endColumn={},title={title}::{message}\n",
        start.line, end.line, start.column, end.column
    )
}

fn format_message(filename: &str, line: usize, message: &str) -> String {
    if filename.is_empty() || line == 0 {
        return message.to_string();
    }

    format!("{filename}:{line} {message}")
}

fn display_filename(filename: &str) -> Cow<'_, str> {
    let path = Path::new(filename);
    if filename.contains("://") || path.is_absolute() {
        return path
            .file_name()
            .and_then(|name| name.to_str())
            .map_or(Cow::Borrowed(filename), |name| Cow::Owned(name.to_string()));
    }

    Cow::Borrowed(filename)
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
    use oxc_diagnostics::{
        NamedSource, OxcDiagnostic,
        reporter::{DiagnosticReporter, DiagnosticResult},
    };
    use oxc_span::Span;

    use super::GithubReporter;

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
            "::warning file=file%3A//test.ts,line=1,endLine=1,col=1,endColumn=9,title=oxlint::test.ts:1 error message\n"
        );
    }
}
