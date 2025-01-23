use std::{borrow::Cow, io::Write};

use oxc_diagnostics::{
    reporter::{DiagnosticReporter, Info},
    Error, Severity,
};

use crate::output_formatter::InternalFormatter;

#[derive(Debug)]
pub struct GithubOutputFormatter;

impl InternalFormatter for GithubOutputFormatter {
    fn all_rules(&mut self, writer: &mut dyn Write) {
        writeln!(writer, "flag --rules with flag --format=github is not allowed").unwrap();
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(GithubReporter)
    }
}

/// Formats reports using [GitHub Actions
/// annotations](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions#setting-an-error-message). Useful for reporting in CI.
struct GithubReporter;

impl DiagnosticReporter for GithubReporter {
    fn finish(&mut self) -> Option<String> {
        None
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
    let message = escape_data(&message);
    format!(
        "::{severity} file={filename},line={},endLine={},col={},endColumn={},title={title}::{message}\n",
        start.line,
        end.line,
        start.column,
        end.column
    )
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
    use oxc_diagnostics::{reporter::DiagnosticReporter, NamedSource, OxcDiagnostic};
    use oxc_span::Span;

    use super::GithubReporter;

    #[test]
    fn reporter_finish() {
        let mut reporter = GithubReporter;

        let result = reporter.finish();

        assert!(result.is_none());
    }

    #[test]
    fn reporter_error() {
        let mut reporter = GithubReporter;
        let error = OxcDiagnostic::warn("error message")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("file://test.ts", "debugger;"));

        let result = reporter.render_error(error);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "::warning file=file%3A//test.ts,line=1,endLine=1,col=1,endColumn=9,title=oxlint::error message\n");
    }
}
