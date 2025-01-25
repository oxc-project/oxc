//! [Reporters](DiagnosticReporter) for rendering and writing diagnostics.

use miette::SourceSpan;

use crate::{Error, Severity};

/// Reporters are responsible for rendering diagnostics to some format and writing them to some
/// form of output stream.
///
/// Reporters get used by [`DiagnosticService`](crate::service::DiagnosticService) when they
/// receive diagnostics.
///
/// ## Example
/// ```
/// use oxc_diagnostics::{DiagnosticReporter, Error, Severity};
///
/// #[derive(Default)]
/// pub struct BufferedReporter;
///
/// impl DiagnosticReporter for BufferedReporter {
///     // render the finished output, some reporters will store the errors in memory
///     // to output all diagnostics at the end
///     fn finish(&mut self) -> Option<String> {
///         None
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
    /// Some reporters (e.g. `JSONReporter`) store all diagnostics in memory, then write them
    /// all at once.
    ///
    /// While this method _should_ only ever be called a single time, this is not a guarantee
    /// upheld in Oxc's API. Do not rely on this behavior.
    fn finish(&mut self, result: &DiagnosticResult) -> Option<String>;

    /// Render a diagnostic into this reporter's desired format. For example, a JSONLinesReporter
    /// might return a stringified JSON object on a single line. Returns [`None`] to skip reporting
    /// of this diagnostic.
    ///
    /// Reporters should use this method to write diagnostics to their output stream.
    fn render_error(&mut self, error: Error) -> Option<String>;
}

/// DiagnosticResult will be submitted to the Reporter when the [`DiagnosticService`](crate::service::DiagnosticService)
/// is finished receiving all files
#[derive(Default)]
pub struct DiagnosticResult {
    /// Total number of warnings received
    warnings_count: usize,

    /// Total number of errors received
    errors_count: usize,

    /// Did the threshold for warnings exceeded the max_warnings?
    /// ToDo: We giving the input from outside, let the owner calculate the result
    max_warnings_exceeded: bool,
}

impl DiagnosticResult {
    pub fn new(warnings_count: usize, errors_count: usize, max_warnings_exceeded: bool) -> Self {
        Self { warnings_count, errors_count, max_warnings_exceeded }
    }

    /// Get the number of warning-level diagnostics received.
    pub fn warnings_count(&self) -> usize {
        self.warnings_count
    }

    /// Get the number of error-level diagnostics received.
    pub fn errors_count(&self) -> usize {
        self.errors_count
    }

    /// Did the threshold for warnings exceeded the max_warnings?
    pub fn max_warnings_exceeded(&self) -> bool {
        self.max_warnings_exceeded
    }
}

pub struct Info {
    pub start: InfoPosition,
    pub end: InfoPosition,
    pub filename: String,
    pub message: String,
    pub severity: Severity,
    pub rule_id: Option<String>,
}

pub struct InfoPosition {
    pub line: usize,
    pub column: usize,
}

impl Info {
    pub fn new(diagnostic: &Error) -> Self {
        let mut start = InfoPosition { line: 0, column: 0 };
        let mut end = InfoPosition { line: 0, column: 0 };
        let mut filename = String::new();
        let mut message = String::new();
        let mut severity = Severity::Warning;
        let mut rule_id = None;
        if let Some(mut labels) = diagnostic.labels() {
            if let Some(source) = diagnostic.source_code() {
                if let Some(label) = labels.next() {
                    if let Ok(span_content) = source.read_span(label.inner(), 0, 0) {
                        start.line = span_content.line() + 1;
                        start.column = span_content.column() + 1;

                        let end_offset = label.inner().offset() + label.inner().len();

                        if let Ok(span_content) =
                            source.read_span(&SourceSpan::from((end_offset, 0)), 0, 0)
                        {
                            end.line = span_content.line() + 1;
                            end.column = span_content.column() + 1;
                        }

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

        Self { start, end, filename, message, severity, rule_id }
    }
}
