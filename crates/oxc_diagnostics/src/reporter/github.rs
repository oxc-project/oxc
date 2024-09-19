use std::{
    borrow::Cow,
    io::{BufWriter, Stdout, Write},
};

use super::{writer, DiagnosticReporter, Info};
use crate::{Error, Severity};

/// Formats reports using [GitHub Actions
/// annotations](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions#setting-an-error-message). Useful for reporting in CI.
pub struct GithubReporter {
    writer: BufWriter<Stdout>,
}

impl Default for GithubReporter {
    fn default() -> Self {
        Self { writer: writer() }
    }
}

impl DiagnosticReporter for GithubReporter {
    fn finish(&mut self) {
        self.writer.flush().unwrap();
    }

    fn render_diagnostics(&mut self, _s: &[u8]) {}

    fn render_error(&mut self, error: Error) -> Option<String> {
        let message = format_github(&error);
        self.writer.write_all(message.as_bytes()).unwrap();
        None
    }
}

fn format_github(diagnostic: &Error) -> String {
    let Info { line, column, filename, message, severity, rule_id } = Info::new(diagnostic);
    let severity = match severity {
        Severity::Error => "error",
        Severity::Warning | miette::Severity::Advice => "warning",
    };
    let title = rule_id.map_or(Cow::Borrowed("oxlint"), Cow::Owned);
    let filename = escape_property(&filename);
    let message = escape_data(&message);
    format!(
        "::{severity} file={filename},line={line},endLine={line},col={column},endColumn={column},title={title}::{message}\n"
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
