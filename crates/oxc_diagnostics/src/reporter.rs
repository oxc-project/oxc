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

/// Reusable line/column state for a single source.
///
/// [`Info::new`] resolves line/column by scanning the source from the start for every offset it
/// is asked about, so rendering `m` diagnostics of an `n` byte file scans the source `2 * m` times.
/// Reporters that render many diagnostics of the same source can reuse one `LineIndex` instead,
/// which continues scanning from where the previous lookup stopped.
///
/// Only diagnostics of the same source may share an index; [`is_for`](Self::is_for) checks that.
/// Lookups that move backwards rewind and rescan, costing the same as [`Info::new`] would.
#[derive(Debug)]
pub struct LineIndex<'a> {
    data: &'a [u8],
    /// Offset the scan has consumed. Bytes before this are accounted for in `line`/`line_start`.
    scanned: usize,
    /// 1-based line number of the line containing `scanned`.
    line: usize,
    /// Offset of the start of that line.
    line_start: usize,
}

impl<'a> LineIndex<'a> {
    /// Create an index for `data`.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, scanned: 0, line: 1, line_start: 0 }
    }

    /// Whether this index was created for `data`.
    ///
    /// Uses the same pointer check as the graphical reporter's `SpanScanner`. Reusing an index for
    /// a different source would produce incorrect line numbers, so callers must check this before
    /// sharing an index between diagnostics.
    pub fn is_for(&self, data: &[u8]) -> bool {
        std::ptr::eq(self.data, data)
    }

    /// Resolve the 1-based line and column of `offset`.
    ///
    /// Returns `None` when `offset` is beyond the end of the source. Equivalent to the lookup
    /// [`Info::new`] performs, but reuses the scan already done for previous offsets.
    pub fn position(&mut self, offset: usize) -> Option<InfoPosition> {
        if offset > self.data.len() {
            return None;
        }

        let mut offset = offset;
        if offset > 0
            && offset < self.data.len()
            && self.data[offset - 1] == b'\r'
            && self.data[offset] == b'\n'
        {
            offset -= 1;
        }

        if offset < self.scanned {
            // Moving backwards: start over, exactly as a fresh scan would.
            self.scanned = 0;
            self.line = 1;
            self.line_start = 0;
        }

        self.scan_to(offset);

        Some(InfoPosition { line: self.line, column: offset - self.line_start + 1 })
    }

    /// Advance the scan to `offset`, so that `line`/`line_start` describe the line containing it.
    fn scan_to(&mut self, offset: usize) {
        let data = self.data;

        // If the previous scan stopped on the `\r` of a `\r\n` pair, whether that pair counts as one
        // break depended on the offset then in effect. When the scan continues past it, the `\n` is
        // now inside the scanned range, so the line starts after it instead of on it.
        let mut scanned = self.scanned;
        if scanned > 0
            && self.line_start == scanned
            && scanned < offset
            && data[scanned - 1] == b'\r'
            && data[scanned] == b'\n'
        {
            self.line_start = scanned + 1;
            scanned += 1;
        }

        for index in memchr::memchr2_iter(b'\r', b'\n', &data[scanned..offset]) {
            let index = scanned + index;
            if data[index] == b'\n' && index > 0 && data[index - 1] == b'\r' {
                continue;
            }
            self.line += 1;
            self.line_start =
                if data[index] == b'\r' && index + 1 < offset && data[index + 1] == b'\n' {
                    index + 2
                } else {
                    index + 1
                };
        }

        self.scanned = offset;
    }
}

impl Info {
    pub fn new(diagnostic: &Error) -> Self {
        let mut index = diagnostic.source_code().map(|source| LineIndex::new(source.data()));
        Self::with_line_index(diagnostic, index.as_mut())
    }

    /// Same as [`Info::new`], resolving line/column through `index` when it is given.
    ///
    /// Pass an index that [`LineIndex::is_for`] the diagnostic's source to avoid rescanning the
    /// source for every diagnostic of a file; pass `None` to look the positions up directly.
    pub fn with_line_index(diagnostic: &Error, mut index: Option<&mut LineIndex<'_>>) -> Self {
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

            let mut position = |offset: usize| match index.as_mut() {
                Some(index) if index.is_for(source.data()) => index.position(offset),
                _ => line_column(source.data(), offset),
            };

            if let Some(label) = diagnostic.labels().first()
                && let Ok(start_offset) = usize::try_from(label.offset())
                && let Some(end_offset) = label.offset().checked_add(label.len())
                && let Ok(end_offset) = usize::try_from(end_offset)
                && source.data().get(start_offset..end_offset).is_some()
                && let Some(start_position) = position(start_offset)
            {
                start = start_position;
                if let Some(end_position) = position(end_offset) {
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

/// Iterate diagnostics with their [`Info`], reusing a [`LineIndex`] for diagnostics that share a
/// source.
///
/// Reporters that render a whole batch should use this instead of calling [`Info::new`] per
/// diagnostic, so that a file's line/column lookups are resolved by one forward scan.
pub fn batch_infos(errors: &[Error]) -> BatchInfos<'_> {
    BatchInfos { errors, index: None, next: 0 }
}

/// Iterator returned by [`batch_infos`].
pub struct BatchInfos<'a> {
    errors: &'a [Error],
    index: Option<LineIndex<'a>>,
    next: usize,
}

impl<'a> Iterator for BatchInfos<'a> {
    type Item = (&'a Error, Info);

    fn next(&mut self) -> Option<Self::Item> {
        let error = self.errors.get(self.next)?;
        self.next += 1;

        let info = match error.source_code() {
            Some(source) => {
                if !self.index.as_ref().is_some_and(|index| index.is_for(source.data())) {
                    self.index = Some(LineIndex::new(source.data()));
                }
                Info::with_line_index(error, self.index.as_mut())
            }
            None => Info::new(error),
        };

        Some((error, info))
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use oxc_span::Span;

    use crate::{Error, NamedSource, OxcDiagnostic};

    use super::{Info, LineIndex, batch_infos, line_column};

    #[test]
    fn includes_source_name_for_unlabelled_diagnostic() {
        let error: Error = OxcDiagnostic::error("Something went wrong")
            .with_source_code(NamedSource::new("file.js", ""));

        assert_eq!(Info::new(&error).filename, "file.js");
    }

    /// Sources covering the line-break shapes `line_column` has to handle.
    const SOURCES: [&str; 12] = [
        "",
        "\n",
        "\r",
        "\r\n",
        "a",
        "a\n",
        "a\r\n",
        "a\r\nb",
        "a\nb\nc",
        "a\r\nb\rc\nd",
        "\r\n\r\n\r\n",
        "// 你好\nconst a = 1;\r\nconst b = 2;\rconst c = 3;\n",
    ];

    /// Exhaustively compare [`LineIndex`] against [`line_column`] for every offset, in every
    /// order of lookups (ascending, descending, and interleaved).
    #[test]
    fn line_index_matches_line_column() {
        for source in SOURCES {
            let data = source.as_bytes();
            let offsets: Vec<usize> = (0..=data.len() + 1).collect();

            let mut ascending = LineIndex::new(data);
            let mut descending = LineIndex::new(data);
            let mut interleaved = LineIndex::new(data);

            for &offset in &offsets {
                let expected = line_column(data, offset);
                assert_eq!(
                    ascending.position(offset),
                    expected,
                    "ascending lookup at {offset} in {source:?}"
                );
            }

            for &offset in offsets.iter().rev() {
                let expected = line_column(data, offset);
                assert_eq!(
                    descending.position(offset),
                    expected,
                    "descending lookup at {offset} in {source:?}"
                );
            }

            // Alternate directions so the index both advances and rewinds.
            let mut order = Vec::with_capacity(offsets.len() * 2);
            for (index, &offset) in offsets.iter().enumerate() {
                if index % 2 == 0 {
                    order.push(offset);
                } else {
                    order.push(offsets[offsets.len() - 1 - index]);
                }
            }
            for &offset in &order {
                let expected = line_column(data, offset);
                assert_eq!(
                    interleaved.position(offset),
                    expected,
                    "interleaved lookup at {offset} in {source:?}"
                );
            }
        }
    }

    /// Pseudo-random sources and lookup orders, to cover break arrangements the fixed sources miss.
    #[test]
    fn line_index_matches_line_column_randomly() {
        let alphabet = [b'a', b'\n', b'\r', b' ', b'\t'];
        let mut state = 0x2545_f491usize;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };

        for _ in 0..300 {
            let len = next() % 24;
            let mut data = Vec::with_capacity(len);
            for _ in 0..len {
                data.push(alphabet[next() % alphabet.len()]);
            }

            let mut index = LineIndex::new(&data);
            let mut offsets: Vec<usize> = (0..=data.len() + 1).collect();
            // Shuffle deterministically so lookups jump around.
            for i in (1..offsets.len()).rev() {
                let j = next() % (i + 1);
                offsets.swap(i, j);
            }

            for offset in offsets {
                assert_eq!(
                    index.position(offset),
                    line_column(&data, offset),
                    "lookup at {offset} in {:?}",
                    String::from_utf8_lossy(&data)
                );
            }
        }
    }

    /// Diagnostics of one file rendered as a batch must match rendering them one by one.
    #[test]
    fn batch_infos_matches_info_new() {
        let source =
            Arc::new(NamedSource::new("input.ts", "first();\nsecond();\r\nthird();\rfourth();\n"));
        let other = Arc::new(NamedSource::new("other.ts", "alpha();\nbeta();\n"));

        let offsets = [(0, 5), (9, 15), (18, 24), (28, 34), (0, 0), (34, 34)];
        let mut errors: Vec<Error> = Vec::new();
        for (start, end) in offsets {
            errors.push(
                OxcDiagnostic::warn("one")
                    .with_label(Span::new(start, end))
                    .with_source_code(Arc::clone(&source)),
            );
            errors.push(
                OxcDiagnostic::error("two")
                    .with_label(Span::new(start, end))
                    .with_source_code(Arc::clone(&other)),
            );
        }

        let batched: Vec<Info> = batch_infos(&errors).map(|(_, info)| info).collect();
        let individual: Vec<Info> = errors.iter().map(Info::new).collect();

        assert_eq!(batched.len(), individual.len());
        for (batched, individual) in batched.iter().zip(&individual) {
            assert_eq!(batched.start, individual.start);
            assert_eq!(batched.end, individual.end);
            assert_eq!(batched.filename, individual.filename);
        }
    }

    /// `Info::new` must keep resolving positions when no index is available.
    #[test]
    fn with_line_index_none_matches_new() {
        let source = Arc::new(NamedSource::new("input.ts", "one\ntwo\r\nthree\n"));
        let error: Error = OxcDiagnostic::warn("message")
            .with_label(Span::new(4, 7))
            .with_source_code(Arc::clone(&source));

        let without_index = Info::with_line_index(&error, None);
        let expected = Info::new(&error);

        assert_eq!(without_index.start, expected.start);
        assert_eq!(without_index.end, expected.end);
        assert_eq!(without_index.filename, expected.filename);
        assert_eq!(without_index.message, expected.message);
    }
}
