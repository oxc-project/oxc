//! [Reporters](DiagnosticReporter) for rendering and writing diagnostics.

use crate::{Error, Severity};

/// Reporters are responsible for rendering diagnostics to some format and writing them to some
/// form of output stream.
///
/// Reporters get used by [`DiagnosticService`](crate::service::DiagnosticService) when they
/// receive diagnostics.
///
/// ## Example
/// ```rust,ignore
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

    /// Whether [`DiagnosticService`](crate::service::DiagnosticService) should replace very long
    /// rendered lines with the synthetic "minified file" warning.
    ///
    /// Human-readable reporters generally want this behavior to avoid dumping unreadable output.
    /// Machine-readable or intentionally single-line reporters should disable it.
    fn supports_minified_file_fallback(&self) -> bool {
        true
    }

    /// Render diagnostics in order, delegating to [`render_error`](Self::render_error) by default.
    fn render_errors(&mut self, errors: Vec<Error>, emit: &mut dyn FnMut(&str)) {
        for error in errors {
            if let Some(rendered) = self.render_error(error) {
                emit(&rendered);
            }
        }
    }

    /// Render diagnostics until `keep` rejects a rendered report.
    fn render_errors_until(
        &mut self,
        errors: Vec<Error>,
        keep: &mut dyn FnMut(Option<&str>, &str) -> bool,
    ) {
        for error in errors {
            let source_name =
                error.source_code().and_then(|source| source.name()).map(ToString::to_string);
            if let Some(rendered) = self.render_error(error)
                && !keep(source_name.as_deref(), &rendered)
            {
                break;
            }
        }
    }

    /// Render a diagnostic into this reporter's desired format. For example, a JSONLinesReporter
    /// might return a stringified JSON object on a single line. Returns [`None`] to skip reporting
    /// of this diagnostic.
    ///
    /// Reporters should use this method to write diagnostics to their output stream.
    fn render_error(&mut self, error: Error) -> Option<String>;
}

/// DiagnosticResult will be submitted to the Reporter when the [`DiagnosticService`](crate::service::DiagnosticService)
/// is finished receiving all files
#[derive(Default, Debug)]
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

#[derive(Debug)]
pub struct Info {
    pub start: InfoPosition,
    pub end: InfoPosition,
    pub filename: String,
    pub message: String,
    pub severity: Severity,
    pub rule_id: Option<String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct InfoPosition {
    pub line: usize,
    pub column: usize,
}

fn line_column(data: &[u8], mut offset: usize) -> Option<InfoPosition> {
    if offset > data.len() {
        return None;
    }
    if offset > 0 && offset < data.len() && data[offset - 1] == b'\r' && data[offset] == b'\n' {
        offset -= 1;
    }

    let mut line = 1;
    let mut line_start = 0;
    for index in memchr::memchr2_iter(b'\r', b'\n', &data[..offset]) {
        if data[index] == b'\n' && index > 0 && data[index - 1] == b'\r' {
            continue;
        }
        line += 1;
        line_start = if data[index] == b'\r' && index + 1 < offset && data[index + 1] == b'\n' {
            index + 2
        } else {
            index + 1
        };
    }
    Some(InfoPosition { line, column: offset - line_start + 1 })
}

/// Index of line breaks in a source, for resolving byte offsets to line and column positions.
///
/// Resolves positions with a binary search, instead of scanning the source from the start as [`line_column`] does.
/// Produces identical results to [`line_column`].
struct LineBreaks {
    /// Pointer to start of source data this index was built for. Only used to identify the source.
    source_ptr: *const u8,
    /// Length of source data this index was built for
    source_len: usize,
    /// Byte offsets of line breaks (`\r`, `\n`, or the `\r` of a `\r\n` pair), in ascending order
    breaks: Vec<usize>,
}

impl LineBreaks {
    fn new(data: &[u8]) -> Self {
        let breaks = memchr::memchr2_iter(b'\r', b'\n', data)
            .filter(|&index| !(data[index] == b'\n' && index > 0 && data[index - 1] == b'\r'))
            .collect();
        Self { source_ptr: data.as_ptr(), source_len: data.len(), breaks }
    }

    fn is_for(&self, data: &[u8]) -> bool {
        std::ptr::eq(self.source_ptr, data.as_ptr()) && self.source_len == data.len()
    }

    fn line_column(&self, data: &[u8], mut offset: usize) -> Option<InfoPosition> {
        if offset > data.len() {
            return None;
        }
        if offset > 0 && offset < data.len() && data[offset - 1] == b'\r' && data[offset] == b'\n' {
            offset -= 1;
        }

        // Number of line breaks before `offset`
        let line_index = self.breaks.partition_point(|&index| index < offset);
        let line_start = match line_index.checked_sub(1) {
            None => 0,
            Some(break_index) => {
                let index = self.breaks[break_index];
                if data[index] == b'\r' && index + 1 < offset && data[index + 1] == b'\n' {
                    index + 2
                } else {
                    index + 1
                }
            }
        };
        Some(InfoPosition { line: line_index + 1, column: offset - line_start + 1 })
    }
}

impl Info {
    pub fn new(diagnostic: &Error) -> Self {
        Self::new_with(diagnostic, line_column)
    }

    /// Create [`Info`] for each of `diagnostics`.
    ///
    /// Produces the same results as calling [`Info::new`] for each diagnostic, but is much faster for large files
    /// with many diagnostics. [`Info::new`] scans the source from the start to find each diagnostic's position,
    /// so total cost is proportional to file size multiplied by number of diagnostics. This function instead
    /// indexes the line breaks in a source once, and reuses the index for consecutive diagnostics with the same source.
    ///
    /// Sources are identified by address and length of their data. All sources are borrowed for the duration of
    /// this call, and two slices which are alive at the same time with the same address and length have the same
    /// contents, so a cached index is always correct for a source it's reused for.
    pub fn new_all(diagnostics: &[Error]) -> Vec<Self> {
        let mut line_breaks: Option<LineBreaks> = None;
        diagnostics
            .iter()
            .map(|diagnostic| {
                Self::new_with(diagnostic, |data, offset| {
                    if let Some(line_breaks) = &line_breaks
                        && line_breaks.is_for(data)
                    {
                        return line_breaks.line_column(data, offset);
                    }
                    line_breaks.insert(LineBreaks::new(data)).line_column(data, offset)
                })
            })
            .collect()
    }

    fn new_with(
        diagnostic: &Error,
        mut line_column: impl FnMut(&[u8], usize) -> Option<InfoPosition>,
    ) -> Self {
        let mut start = InfoPosition { line: 0, column: 0 };
        let mut end = InfoPosition { line: 0, column: 0 };
        let mut filename = String::new();
        let mut message = String::new();
        let mut severity = Severity::Warning;
        let rule_id = diagnostic.code().map(|code| code.to_string());

        if let Some(source) = diagnostic.source_code() {
            if let Some(name) = source.name() {
                filename = name.to_string();
            }

            if let Some(label) = diagnostic.labels().first()
                && let Ok(start_offset) = usize::try_from(label.offset())
                && let Some(end_offset) = label.offset().checked_add(label.len())
                && let Ok(end_offset) = usize::try_from(end_offset)
                && source.data().get(start_offset..end_offset).is_some()
                && let Some(start_position) = line_column(source.data(), start_offset)
            {
                start = start_position;
                if let Some(end_position) = line_column(source.data(), end_offset) {
                    end = end_position;
                }

                if matches!(diagnostic.severity(), Some(Severity::Error)) {
                    severity = Severity::Error;
                }

                message = diagnostic.to_string();
            }
        }

        Self { start, end, filename, message, severity, rule_id }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use oxc_span::Span;

    use crate::{Error, NamedSource, OxcDiagnostic};

    use super::{Info, LineBreaks, line_column};

    #[test]
    fn includes_source_name_for_unlabelled_diagnostic() {
        let error: Error = OxcDiagnostic::error("Something went wrong")
            .with_source_code(NamedSource::new("file.js", ""));

        assert_eq!(Info::new(&error).filename, "file.js");
    }

    #[test]
    fn line_breaks_matches_line_column() {
        let sources = [
            "",
            "a",
            "\n",
            "\r",
            "\r\n",
            "\n\r",
            "a\nb\rc\r\nd",
            "\r\n\r\n",
            "\r\r\n\n",
            "ab\r\ncd\n\nef\r\rgh\r\n",
            "x\r",
            "\u{2028}a\u{2029}b\nc",
        ];
        for source in sources {
            let data = source.as_bytes();
            let line_breaks = LineBreaks::new(data);
            for offset in 0..=data.len() + 1 {
                assert_eq!(
                    line_breaks.line_column(data, offset),
                    line_column(data, offset),
                    "source {source:?}, offset {offset}"
                );
            }
        }
    }

    #[test]
    fn new_all_matches_new() {
        let source_a = "let a = 1;\r\nlet b = 2;\nlet c = 3;\r\n";
        let source_b = "x\ny\nz";
        let named_a = Arc::new(NamedSource::new("a.js", source_a.to_string()));
        let named_b = Arc::new(NamedSource::new("b.js", source_b.to_string()));
        let diagnostic = |source: &Arc<NamedSource<String>>, start: u32, end: u32| -> Error {
            OxcDiagnostic::error("message")
                .with_label(Span::new(start, end))
                .with_source_code(Arc::clone(source))
        };
        let diagnostics = vec![
            diagnostic(&named_a, 0, 3),
            diagnostic(&named_a, 12, 15),
            diagnostic(&named_a, 23, 33),
            diagnostic(&named_b, 2, 3),
            diagnostic(&named_a, 11, 12),
            diagnostic(&named_b, 4, 5),
            OxcDiagnostic::error("no label").with_source_code(Arc::clone(&named_b)),
            OxcDiagnostic::error("no source").into(),
        ];

        let all = Info::new_all(&diagnostics);
        assert_eq!(all.len(), diagnostics.len());
        for (info, diagnostic) in all.iter().zip(&diagnostics) {
            let expected = Info::new(diagnostic);
            assert_eq!(info.start, expected.start);
            assert_eq!(info.end, expected.end);
            assert_eq!(info.filename, expected.filename);
            assert_eq!(info.message, expected.message);
            assert_eq!(info.rule_id, expected.rule_id);
        }
    }
}
