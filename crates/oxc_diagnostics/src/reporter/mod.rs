//! [Reporters](DiagnosticReporter) for rendering and writing diagnostics.

use crate::{Error, Severity};

/// Reporters are responsible for rendering diagnostics to some format and writing them to some
/// form of output stream.
///
/// Reporters get used by [`DiagnosticService`](crate::service::DiagnosticService) when they
/// receive diagnostics.
///
/// ## Example
/// ```
/// use std::io::{self, Write, BufWriter, Stderr};
/// use oxc_diagnostics::{DiagnosticReporter, Error, Severity};
///
/// pub struct BufReporter {
///     writer: BufWriter<Stderr>,
/// }
///
/// impl Default for BufReporter {
///    fn default() -> Self {
///        Self { writer: BufWriter::new(io::stderr()) }
///    }
/// }
///
/// impl DiagnosticReporter for BufferedReporter {
///     // flush all remaining bytes when no more diagnostics will be reported
///     fn finish(&mut self) {
///         self.writer.flush().unwrap();
///     }
///
///     // write rendered reports to stderr
///     fn render_diagnostics(&mut self, s: &[u8]) {
///         self.writer.write_all(s).unwrap();
///     }
///
///     // render diagnostics to a simple Apache-like log format
///     fn render_error(&mut self, error: Error) -> Option<String> {
///         let level = match error.severity().unwrap_or_default() {
///             Severity::Error => "ERROR",
///             Severity::Warning => "WARN",
///             Severity::Advice => "INFO",
///         };
///         let rendered = format!("[{level}]: {error}");
///
///         Some(rendered)
///     }
/// }
/// ```
pub trait DiagnosticReporter {
    /// Lifecycle hook that gets called when no more diagnostics will be reported.
    ///
    /// Used primarily for flushing output stream buffers, but you don't just have to use it for
    /// that. Some reporters (e.g. `JSONReporter`) store all diagnostics in memory, then write them
    /// all at once.
    ///
    /// While this method _should_ only ever be called a single time, this is not a guarantee
    /// upheld in Oxc's API. Do not rely on this behavior.
    fn finish(&mut self) -> Option<String>;

    /// Render a diagnostic into this reporter's desired format. For example, a JSONLinesReporter
    /// might return a stringified JSON object on a single line. Returns [`None`] to skip reporting
    /// of this diagnostic.
    ///
    /// Reporters should not use this method to write diagnostics to their output stream. That
    /// should be done in [`render_diagnostics`](DiagnosticReporter::render_diagnostics).
    fn render_error(&mut self, error: Error) -> Option<String>;
}

pub struct Info {
    pub line: usize,
    pub column: usize,
    pub filename: String,
    pub message: String,
    pub severity: Severity,
    pub rule_id: Option<String>,
}

impl Info {
    pub fn new(diagnostic: &Error) -> Self {
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
