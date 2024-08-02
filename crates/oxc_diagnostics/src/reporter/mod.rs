mod checkstyle;
mod github;
mod graphical;
mod json;
mod unix;

use std::io::{BufWriter, Stdout};

pub use self::{
    checkstyle::CheckstyleReporter, github::GithubReporter, graphical::GraphicalReporter,
    json::JsonReporter, unix::UnixReporter,
};
use crate::{Error, Severity};

/// stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
/// See `https://github.com/rust-lang/rust/issues/60673`.
fn writer() -> BufWriter<Stdout> {
    BufWriter::new(std::io::stdout())
}

pub trait DiagnosticReporter {
    fn finish(&mut self);
    fn render_diagnostics(&mut self, s: &[u8]);
    fn render_error(&mut self, error: Error) -> Option<String>;
}

struct Info {
    line: usize,
    column: usize,
    filename: String,
    message: String,
    severity: Severity,
    rule_id: Option<String>,
}

impl Info {
    fn new(diagnostic: &Error) -> Self {
        let mut line = 0;
        let mut column = 0;
        let mut filename = String::new();
        let mut message = String::new();
        let mut severity = Severity::Warning;
        let mut rule_id = None;
        if let Some(mut labels) = diagnostic.labels() {
            if let Some(source) = diagnostic.source_code() {
                if let Some(label) = labels.next() {
                    if let Ok(span_content) = source.read_span(label.inner(), 0, 0) {
                        line = span_content.line() + 1;
                        column = span_content.column() + 1;
                        if let Some(name) = span_content.name() {
                            filename = name.to_string();
                        };
                        if matches!(diagnostic.severity(), Some(Severity::Error)) {
                            severity = Severity::Error;
                        }
                        let msg = diagnostic.to_string();
                        // Our messages usually comes with `eslint(rule): message`
                        (rule_id, message) = msg.split_once(':').map_or_else(
                            || (None, msg.to_string()),
                            |(id, msg)| (Some(id.to_string()), msg.trim().to_string()),
                        );
                    }
                }
            }
        }
        Self { line, column, filename, message, severity, rule_id }
    }
}
