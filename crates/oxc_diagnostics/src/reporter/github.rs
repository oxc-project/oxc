use std::{
    borrow::Cow,
    io::{BufWriter, Stdout, Write},
};

use crate::{Error, Severity};

use super::{writer, DiagnosticReporter, Info};

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
    format!("::{severity} file={filename},line={line},endLine={line},col={column},endColumn={column},title={title}::{message}\n")
}
