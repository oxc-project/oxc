use std::borrow::Cow;
use std::fmt::Write;

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
    let message = escape_message(&display_filename(&filename), start.line, &message);
    let filename = escape_property(&filename);
    format!(
        "::{severity} file={filename},line={},endLine={},col={},endColumn={},title={title}::{message}\n",
        start.line, end.line, start.column, end.column
    )
}

fn escape_message(filename: &str, line: usize, message: &str) -> String {
    let mut result = String::with_capacity(filename.len() + message.len() + 16);

    if !filename.is_empty() && line > 0 {
        push_escaped_data(&mut result, filename);
        result.push(':');
        write!(&mut result, "{line} ").expect("writing to String should not fail");
    }

    push_escaped_data(&mut result, message);
    result
}

fn display_filename(filename: &str) -> Cow<'_, str> {
    if let Some((_, path)) = filename.split_once("://") {
        #[cfg(windows)]
        if let Some(path) = path.strip_prefix('/')
            && path.as_bytes().get(1) == Some(&b':')
        {
            return Cow::Borrowed(path);
        }

        return Cow::Borrowed(path);
    }

    Cow::Borrowed(filename)
}

fn push_escaped_data(result: &mut String, value: &str) {
    for c in value.chars() {
        match c {
            '\r' => result.push_str("%0D"),
            '\n' => result.push_str("%0A"),
            '%' => result.push_str("%25"),
            _ => result.push(c),
        }
    }
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

    use super::{GithubReporter, display_filename};

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

    #[test]
    fn display_filename_keeps_nested_relative_path() {
        assert_eq!(display_filename("packages/ui/main.ts"), "packages/ui/main.ts");
    }

    #[test]
    fn display_filename_strips_uri_scheme() {
        assert_eq!(display_filename("file://packages/ui/main.ts"), "packages/ui/main.ts");
    }

    #[cfg(not(windows))]
    #[test]
    fn display_filename_keeps_absolute_path() {
        assert_eq!(
            display_filename("/workspace/packages/ui/main.ts"),
            "/workspace/packages/ui/main.ts"
        );
    }

    #[cfg(windows)]
    #[test]
    fn display_filename_keeps_absolute_path() {
        assert_eq!(
            display_filename("C:/workspace/packages/ui/main.ts"),
            "C:/workspace/packages/ui/main.ts"
        );
    }

    #[cfg(windows)]
    #[test]
    fn display_filename_normalizes_windows_file_uri() {
        assert_eq!(
            display_filename("file:///C:/workspace/packages/ui/main.ts"),
            "C:/workspace/packages/ui/main.ts"
        );
    }
}
