use std::collections::VecDeque;

use miette::{MietteError, MietteSpanContents, SourceCode, SourceSpan};

/// Source text with a precomputed table of line start offsets.
///
/// The default [`SourceCode`] implementations locate a span by scanning the source from the
/// beginning on every `read_span` call, so rendering `n` diagnostics for one file costs
/// `O(n * file size)`. This type finds the line containing the span with a binary search and
/// only scans the span itself plus the requested context lines.
#[derive(Debug)]
pub struct LineIndexedSource {
    text: String,
    /// Byte offset of the start of each line. Line terminators are `\n`, `\r\n` and lone `\r`,
    /// matching the scanner in oxc-miette's `source_impls.rs`.
    line_starts: Vec<usize>,
}

impl LineIndexedSource {
    pub fn new(text: String) -> Self {
        let bytes = text.as_bytes();
        let mut line_starts = vec![0];
        let mut i = 0;
        while i < bytes.len() {
            let byte = bytes[i];
            if matches!(byte, b'\r' | b'\n') {
                if byte == b'\r' && bytes.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
                line_starts.push(i + 1);
            }
            i += 1;
        }
        Self { text, line_starts }
    }
}

impl SourceCode for LineIndexedSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<MietteSpanContents<'a>, MietteError> {
        let input = self.text.as_bytes();
        let span_offset = span.offset() as usize;
        let span_len = span.len() as usize;

        // Find the line containing the start of the span and begin the scan
        // `context_lines_before` lines above it instead of at the start of the file.
        // `line_starts` always contains 0, so `partition_point` returns at least 1.
        let span_line = self.line_starts.partition_point(|&start| start <= span_offset) - 1;
        let first_line = span_line.saturating_sub(context_lines_before);
        let scan_start = self.line_starts[first_line];

        // The rest mirrors `context_info` in oxc-miette's `source_impls.rs`. Starting at a line
        // boundary at most `context_lines_before` lines above the span leaves every value below
        // identical to what a scan from offset 0 would produce.
        let mut offset = scan_start;
        let mut line_count = first_line;
        let mut start_line = first_line;
        let mut start_column = 0usize;
        let mut before_lines_starts = VecDeque::new();
        let mut current_line_start = scan_start;
        let mut end_lines = 0usize;
        let mut post_span = false;
        let mut post_span_got_newline = false;

        // A zero-length span at the start of a line: the reference scan reaches the end anchor
        // (`span_offset - 1`) while processing the previous line's terminator, before `scan_start`.
        if scan_start > (span_offset + span_len).saturating_sub(1) {
            if context_lines_after == 0 {
                // The reference scan breaks during that terminator, ending at `span_offset`.
                let starting_offset =
                    if context_lines_before == 0 { span_offset } else { scan_start };
                return Ok(MietteSpanContents::new(
                    &input[starting_offset..span_offset],
                    #[expect(clippy::cast_possible_truncation)]
                    (starting_offset as u32, (span_offset - starting_offset) as u32).into(),
                    first_line,
                    0,
                    span_line,
                ));
            }
            post_span = true;
        }
        let mut iter = input[scan_start..].iter().copied().peekable();
        while let Some(char) = iter.next() {
            if matches!(char, b'\r' | b'\n') {
                line_count += 1;
                if char == b'\r' && iter.next_if_eq(&b'\n').is_some() {
                    offset += 1;
                }
                if offset < span_offset {
                    // We're before the start of the span.
                    start_column = 0;
                    before_lines_starts.push_back(current_line_start);
                    if before_lines_starts.len() > context_lines_before {
                        start_line += 1;
                        before_lines_starts.pop_front();
                    }
                } else if offset >= span_offset + span_len.saturating_sub(1) {
                    // We're after the end of the span, but haven't necessarily
                    // started collecting end lines yet (we might still be
                    // collecting context lines).
                    if post_span {
                        start_column = 0;
                        if post_span_got_newline {
                            end_lines += 1;
                        } else {
                            post_span_got_newline = true;
                        }
                        if end_lines >= context_lines_after {
                            offset += 1;
                            break;
                        }
                    }
                }
                current_line_start = offset + 1;
            } else if offset < span_offset {
                start_column += 1;
            }

            if offset >= (span_offset + span_len).saturating_sub(1) {
                post_span = true;
                if end_lines >= context_lines_after {
                    offset += 1;
                    break;
                }
            }

            offset += 1;
        }

        if offset >= (span_offset + span_len).saturating_sub(1) {
            let starting_offset = before_lines_starts
                .front()
                .copied()
                .unwrap_or(if context_lines_before == 0 { span_offset } else { scan_start });
            Ok(MietteSpanContents::new(
                &input[starting_offset..offset],
                #[expect(clippy::cast_possible_truncation)]
                (starting_offset as u32, (offset - starting_offset) as u32).into(),
                start_line,
                if context_lines_before == 0 { start_column } else { 0 },
                line_count,
            ))
        } else {
            Err(MietteError::OutOfBounds)
        }
    }
}

#[cfg(test)]
mod test {
    use miette::{SourceCode, SourceSpan, SpanContents};

    use super::LineIndexedSource;

    /// Compare `LineIndexedSource::read_span` against the default `String` implementation for
    /// every in-bounds and near-out-of-bounds span, with several context sizes.
    fn assert_equivalent(text: &str) {
        let indexed = LineIndexedSource::new(text.to_string());
        let plain = text.to_string();
        let len = text.len();

        // Spans starting past the end of the source make the reference implementation panic on
        // slicing, so only start offsets up to `len` are comparable. Span lengths deliberately
        // overshoot to exercise the out-of-bounds error path.
        for offset in 0..=len {
            for span_len in 0..=len - offset + 2 {
                for context in [(0, 0), (1, 1), (2, 2), (0, 3), (3, 0)] {
                    let span = SourceSpan::from((
                        u32::try_from(offset).unwrap(),
                        u32::try_from(span_len).unwrap(),
                    ));
                    let expected = plain.read_span(&span, context.0, context.1);
                    let actual = indexed.read_span(&span, context.0, context.1);
                    match (expected, actual) {
                        (Ok(expected), Ok(actual)) => {
                            let context_msg = format!(
                                "text {text:?}, span ({offset}, {span_len}), context {context:?}"
                            );
                            assert_eq!(expected.data(), actual.data(), "data: {context_msg}");
                            assert_eq!(expected.span(), actual.span(), "span: {context_msg}");
                            assert_eq!(expected.line(), actual.line(), "line: {context_msg}");
                            assert_eq!(expected.column(), actual.column(), "column: {context_msg}");
                            assert_eq!(
                                expected.line_count(),
                                actual.line_count(),
                                "line_count: {context_msg}"
                            );
                        }
                        (Err(_), Err(_)) => {}
                        (expected, actual) => {
                            panic!(
                                "result mismatch for text {text:?}, span ({offset}, {span_len}), \
                                 context {context:?}: expected {expected:?}, actual {actual:?}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn empty() {
        assert_equivalent("");
    }

    #[test]
    fn single_line() {
        assert_equivalent("debugger;");
    }

    #[test]
    fn lf_terminated() {
        assert_equivalent("foo;\nbar;\nbaz;\n");
    }

    #[test]
    fn lf_no_trailing_newline() {
        assert_equivalent("foo;\nbar;\nbaz;");
    }

    #[test]
    fn crlf() {
        assert_equivalent("foo;\r\nbar;\r\nbaz;\r\n");
    }

    #[test]
    fn lone_cr() {
        assert_equivalent("foo;\rbar;\rbaz;");
    }

    #[test]
    fn mixed_terminators() {
        assert_equivalent("a\nb\r\nc\rd\n\ne\r\n\r\nf");
    }

    #[test]
    fn blank_lines() {
        assert_equivalent("\n\n\nfoo\n\n\n");
    }

    #[test]
    fn only_newlines() {
        assert_equivalent("\n\r\n\r\n");
    }
}
