//! The line model and its measurements.
//!
//! [`get_lines`](GraphicalReportHandler::get_lines) splits already-read span
//! contents into [`Line`]s. Each `Line` knows its position in the source so it
//! can answer geometry questions about a [`FancySpan`] — does the span start,
//! end, or fly by on this line — which drives gutter and underline rendering.
//!
//! The [`visual_offset`](GraphicalReportHandler::visual_offset) /
//! [`line_visual_char_width`](GraphicalReportHandler::line_visual_char_width)
//! helpers translate byte offsets into terminal columns, accounting for tabs,
//! ANSI escapes, and wide/combining Unicode graphemes.

use std::{
    iter::Peekable,
    str::{CharIndices, from_utf8},
};

use smallvec::SmallVec;
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

use super::{handler::GraphicalReportHandler, span::FancySpan};
use crate::source_impls::SpanContents;

#[derive(Debug)]
pub(super) struct Line<'a> {
    pub(super) number: usize,
    pub(super) offset: usize,
    pub(super) length: usize,
    pub(super) text: &'a str,
}

impl Line<'_> {
    pub(super) fn span_line_only(&self, span: &FancySpan) -> bool {
        span.offset() >= self.offset && span.offset() + span.len() <= self.offset + self.length
    }

    /// Returns whether `span` should be visible on this line, either in the gutter or under the
    /// text on this line.
    pub(super) fn span_applies(&self, span: &FancySpan) -> bool {
        let span_len = span.len().max(1);
        let span_end = span.offset() + span_len;
        let line_end = self.offset + self.length;

        (span.offset() >= self.offset && span.offset() < self.offset + self.length)
            // Span passes through this line
            || (span.offset() < self.offset && span_end > line_end)
            // Span ends on this line
            || (span_end > self.offset && span_end <= line_end)
    }

    /// Returns whether `span` should be visible on this line in the gutter (so this excludes spans
    /// that are only visible on this line and do not span multiple lines).
    pub(super) fn span_applies_gutter(&self, span: &FancySpan) -> bool {
        let span_len = span.len().max(1);
        let span_end = span.offset() + span_len;
        let line_end = self.offset + self.length;
        let starts_on_line = span.offset() >= self.offset && span.offset() < line_end;
        let ends_on_line = span_end > self.offset && span_end <= line_end;
        self.span_applies(span)
            // Exclude spans that start and end on this line.
            && !(starts_on_line && ends_on_line)
    }

    // A 'flyby' is a multi-line span that technically covers this line, but
    // does not begin or end within the line itself. This method is used to
    // calculate gutters.
    pub(super) fn span_flyby(&self, span: &FancySpan) -> bool {
        // The span itself starts before this line's starting offset (so, in a
        // prev line).
        span.offset() < self.offset
            // ...and it stops after this line's end.
            && span.offset() + span.len() > self.offset + self.length
    }

    // Does this line contain the *beginning* of this multiline span?
    // This assumes self.span_applies() is true already.
    pub(super) fn span_starts(&self, span: &FancySpan) -> bool {
        span.offset() >= self.offset
    }

    // Does this line contain the *end* of this multiline span?
    // This assumes self.span_applies() is true already.
    pub(super) fn span_ends(&self, span: &FancySpan) -> bool {
        span.offset() + span.len() >= self.offset
            && span.offset() + span.len() <= self.offset + self.length
    }
}

/// Iterator over the visual (terminal-column) width of each `char` in a line.
///
/// ASCII text takes a fast path where every printable char is width 1. For
/// non-ASCII text we lazily visit graphemes and only charge the grapheme's
/// width to its first `char`, so combining marks contribute 0.
/// ANSI escape sequences (`\x1b … m`) are consumed at zero width.
struct CharWidthIterator<'a> {
    chars: CharIndices<'a>,
    graphemes: Option<Peekable<GraphemeIndices<'a>>>,
    column: usize,
    escaped: bool,
}

impl Iterator for CharWidthIterator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let (byte_pos, c) = self.chars.next()?;

        let width = match (self.escaped, c) {
            (false, '\t') => 4 - self.column % 4,
            (false, '\x1b') => {
                self.escaped = true;
                0
            }
            (false, _) => {
                if let Some(graphemes) = &mut self.graphemes {
                    graphemes
                        .next_if(|(pos, _)| *pos == byte_pos)
                        .map_or(0, |(_, grapheme)| grapheme.width())
                } else {
                    // ASCII path: all non-control chars are width 1
                    1
                }
            }
            (true, 'm') => {
                self.escaped = false;
                0
            }
            (true, _) => 0,
        };

        self.column += width;
        Some(width)
    }
}

impl GraphicalReportHandler {
    /// Returns an iterator over the visual width of each character in a line.
    pub(super) fn line_visual_char_width(text: &str) -> impl Iterator<Item = usize> + '_ + use<'_> {
        let graphemes = (!text.is_ascii()).then(|| text.grapheme_indices(true).peekable());

        CharWidthIterator { chars: text.char_indices(), graphemes, column: 0, escaped: false }
    }

    /// Returns the visual column position of a byte offset on a specific line.
    ///
    /// If the offset occurs in the middle of a character, the returned column
    /// corresponds to that character's first column in `start` is true, or its
    /// last column if `start` is false.
    pub(super) fn visual_offset(line: &Line<'_>, offset: usize, start: bool) -> usize {
        let line_range = line.offset..=(line.offset + line.length);
        assert!(line_range.contains(&offset));

        let mut text_index = offset - line.offset;
        while text_index <= line.text.len() && !line.text.is_char_boundary(text_index) {
            if start {
                text_index -= 1;
            } else {
                text_index += 1;
            }
        }
        let text = &line.text[..text_index.min(line.text.len())];
        // Plain ASCII is exactly one terminal column per byte.
        let text_width =
            if text.is_ascii() && memchr::memchr2(b'\t', b'\x1b', text.as_bytes()).is_none() {
                text.len()
            } else {
                Self::line_visual_char_width(text).sum()
            };
        if text_index > line.text.len() {
            // Spans extending past the end of the line are always rendered as
            // one column past the end of the visible line.
            //
            // This doesn't necessarily correspond to a specific byte-offset,
            // since a span extending past the end of the line could contain:
            //  - an actual \n character (1 byte)
            //  - a CRLF (2 bytes)
            //  - EOF (0 bytes)
            text_width + 1
        } else {
            text_width
        }
    }

    /// Splits already-scanned span contents into [`Line`]s.
    pub(super) fn get_lines<'a>(context_data: &SpanContents<'a>) -> SmallVec<[Line<'a>; 3]> {
        let context = from_utf8(context_data.data()).expect("Bad utf8 detected");
        let mut line = context_data.line();
        let base = context_data.span().start as usize;
        let bytes = context.as_bytes();
        // The built-in readers advance `line_count` from the payload's first
        // line, which gives the number of newline-terminated `Line`s here.
        // Cap the hint by byte length because custom sources own this metadata.
        let capacity =
            context_data.line_count().saturating_sub(context_data.line()).max(1).min(bytes.len());
        let mut lines = SmallVec::with_capacity(capacity);
        let mut start = 0;
        for newline in memchr::memchr_iter(b'\n', bytes) {
            let end = newline + 1;
            let text_end =
                if newline > start && bytes[newline - 1] == b'\r' { newline - 1 } else { newline };
            line += 1;
            lines.push(Line {
                number: line,
                offset: base + start,
                length: end - start,
                text: &context[start..text_end],
            });
            start = end;
        }
        if start < bytes.len() {
            // Preserve the historical line number for a payload ending in a
            // lone carriage return, which is rendered as visible text.
            if bytes.last() != Some(&b'\r') {
                line += 1;
            }
            lines.push(Line {
                number: line,
                offset: base + start,
                length: bytes.len() - start,
                text: &context[start..],
            });
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::cast_possible_truncation,
        reason = "test fixtures are much smaller than u32::MAX"
    )]

    use super::*;
    use crate::source_impls::SpanScanner;
    use oxc_span::Span;

    type ExpectedLine<'a> = (usize, usize, usize, &'a str);

    #[test]
    fn visual_widths_preserve_grapheme_geometry() {
        let widths =
            GraphicalReportHandler::line_visual_char_width("e\u{301}火").collect::<Vec<_>>();
        assert_eq!(widths, [1, 0, 2]);
    }

    #[test]
    fn get_lines_preserves_line_geometry() {
        const BASE: usize = 10;
        let cases: &[(&str, &[ExpectedLine<'_>])] = &[
            ("", &[]),
            ("abc", &[(5, BASE, 3, "abc")]),
            ("a\nb", &[(5, BASE, 2, "a"), (6, BASE + 2, 1, "b")]),
            ("a\n", &[(5, BASE, 2, "a")]),
            ("\n", &[(5, BASE, 1, "")]),
            ("a\r\nb", &[(5, BASE, 3, "a"), (6, BASE + 3, 1, "b")]),
            ("a\rb", &[(5, BASE, 3, "a\rb")]),
            ("a\r", &[(4, BASE, 2, "a\r")]),
            ("é\n火", &[(5, BASE, 3, "é"), (6, BASE + 3, 3, "火")]),
        ];
        for &(text, expected) in cases {
            let contents = SpanContents::new(
                text.as_bytes(),
                Span::sized(BASE as u32, text.len() as u32),
                4,
                2,
                expected.len(),
            );
            let actual = GraphicalReportHandler::get_lines(&contents)
                .iter()
                .map(|line| (line.number, line.offset, line.length, line.text))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "text={text:?}");
        }
    }

    #[test]
    fn get_lines_preallocates_the_source_window() {
        let source = "before\ntarget\nafter\nrest";
        let mut scanner = SpanScanner::new(source.as_bytes(), 1, 1);
        let contents = scanner.read_span(Span::sized(7, 6)).unwrap();
        let lines = GraphicalReportHandler::get_lines(&contents);

        assert_eq!(lines.len(), 3);
        assert_eq!(lines.capacity(), 3);
    }
}
