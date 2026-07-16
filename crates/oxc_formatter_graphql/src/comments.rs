use std::cell::Cell;

use oxc_formatter_core::{
    Buffer, LINE_TERMINATORS, SourceText, arena_cow_str,
    builders::{empty_line, expand_parent, hard_line_break, line_suffix, space, text},
    normalize_newlines,
    spec::is_suppression_marker,
    write,
};
use oxc_span::Span;

use crate::print::{GraphqlFormatter, format_with};

/// Cursor over a sorted comment-span list that hands out unprinted slices in span order.
///
/// GraphQL comments are always single-line (`# ...` to end of line) and are collected
/// from the CST's trivia tokens by `format()`.
///
/// `cursor` is a [`Cell`] so the API works through `&self` (mirrors `oxc_formatter_json`'s `Comments`).
pub struct Comments<'a> {
    inner: &'a [Span],
    cursor: Cell<usize>,
}

impl<'a> Comments<'a> {
    pub fn new(comments: &'a [Span]) -> Self {
        Self { inner: comments, cursor: Cell::new(0) }
    }

    /// Returns the next unprinted comment without consuming it.
    pub fn peek(&self) -> Option<Span> {
        self.inner.get(self.cursor.get()).copied()
    }

    /// Returns unprinted comments whose `span.end <= upper_bound`,
    /// and advances the cursor past them so they won't be returned again.
    pub fn take_before(&self, upper_bound: u32) -> &'a [Span] {
        let start = self.cursor.get();
        let mut end = start;
        while end < self.inner.len() && self.inner[end].end <= upper_bound {
            end += 1;
        }
        self.cursor.set(end);
        &self.inner[start..end]
    }

    /// Drains all remaining unprinted comments and returns them.
    pub fn take_remaining(&self) -> &'a [Span] {
        let start = self.cursor.get();
        self.cursor.set(self.inner.len());
        &self.inner[start..]
    }

    /// Iterator over unprinted comments whose `span.end <= upper_bound`.
    /// Does NOT advance the cursor.
    pub fn iter_before(&self, upper_bound: u32) -> impl Iterator<Item = Span> {
        let start = self.cursor.get();
        self.inner[start..].iter().copied().take_while(move |c| c.end <= upper_bound)
    }
}

/// Vertical spacing implied by an inter-token source gap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gap {
    /// Same line (no line terminator).
    None,
    /// One or more line breaks, but no blank line.
    Line,
    /// At least one blank line.
    Blank,
}

/// Classifies the gap `slice` between two source positions.
///
/// A blank line is a line strictly inside the gap consisting solely of whitespace.
/// Tokens in the gap make their line non-blank, so newline counting alone would
/// over-report blank lines. This includes insignificant commas: Prettier's
/// `isNextLineEmpty` skips commas only on the current line's tail, so a comma-only
/// line still counts as content (`a\n,\nb` has no blank line).
/// Recognizes GraphQL line terminators: `\n`, lone `\r`, `\r\n`.
pub fn classify_gap(slice: &[u8]) -> Gap {
    let mut newline_count = 0;
    let mut line_has_content = false;
    let mut blank = false;
    let mut i = 0;
    while i < slice.len() {
        match slice[i] {
            b'\r' | b'\n' => {
                // A line strictly between two terminators with no content is blank.
                if newline_count > 0 && !line_has_content {
                    blank = true;
                }
                newline_count += 1;
                line_has_content = false;
                // Collapse `\r\n` into one break.
                if slice[i] == b'\r' && slice.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
            }
            b' ' | b'\t' => {}
            _ => line_has_content = true,
        }
        i += 1;
    }
    if blank {
        Gap::Blank
    } else if newline_count > 0 {
        Gap::Line
    } else {
        Gap::None
    }
}

/// Emit a single comment verbatim (trailing whitespace trimmed).
/// Mirrors Prettier's `printComment`: `"#" + comment.value.trimEnd()`.
fn write_single_comment(span: Span, f: &mut GraphqlFormatter<'_, '_>) {
    let content = f.context().source_text().text_for(&span);
    write!(f, text(content.trim_end()));
}

/// Emits the formatter element that reproduces the vertical spacing implied by `gap`:
/// `space` for same-line, `hard_line_break` for a line break, `empty_line` for a blank line.
fn write_gap(gap: &[u8], f: &mut GraphqlFormatter<'_, '_>) {
    match classify_gap(gap) {
        Gap::None => write!(f, space()),
        Gap::Line => write!(f, hard_line_break()),
        Gap::Blank => write!(f, empty_line()),
    }
}

/// Emit comments that precede a node,
/// preserving the source's vertical spacing (0/1/blank) between each comment and the next position.
fn write_leading_comments(comments: &[Span], value_start: u32, f: &mut GraphqlFormatter<'_, '_>) {
    let source = f.context().source_text();
    for (i, &span) in comments.iter().enumerate() {
        write_single_comment(span, f);
        let next_pos = comments.get(i + 1).map_or(value_start, |c| c.start);
        write_gap(source.bytes_range(span.end, next_pos), f);
    }
}

/// Drains and emits all pending comments ending at or before `value_start` as leading comments.
pub fn flush_leading_comments(value_start: u32, f: &mut GraphqlFormatter<'_, '_>) {
    let leading = f.context().comments().take_before(value_start);
    write_leading_comments(leading, value_start, f);
}

/// Emit dangling comments inside an otherwise empty container (the caller wraps the result in
/// [`oxc_formatter_core::builders::block_indent`] or similar).
pub fn write_dangling_comments(comments: &[Span], f: &mut GraphqlFormatter<'_, '_>) {
    for (i, &span) in comments.iter().enumerate() {
        if i > 0 {
            write!(f, hard_line_break());
        }
        write_single_comment(span, f);
    }
}

/// If the next pending comment sits on the same line as `prev_end`
/// and ends at or before `upper` (the next piece of user content),
/// drain it and emit it as a trailing line-suffix comment (` # ...`).
///
/// The bound is what keeps a node from claiming a comment across later siblings:
/// in `f(a: 1, b: 2) # c` everything shares a line, but `# c` ends past `b`'s start,
/// so `a: 1` leaves it pending for the enclosing node's flush point.
pub fn write_trailing_same_line_comment<'a>(
    prev_end: u32,
    upper: u32,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let Some(span) = f.context().comments().peek() else { return };
    if span.end > upper {
        return;
    }
    let source = f.context().source_text();
    if classify_gap(source.bytes_range(prev_end, span.start)) != Gap::None {
        return;
    }
    f.context().comments().take_before(span.end);
    let content = format_with(move |f: &mut GraphqlFormatter<'_, 'a>| {
        write!(f, space());
        write_single_comment(span, f);
    });
    write!(f, [line_suffix(&content), expand_parent()]);
}

/// Emit comments that sit between the last child of a container and its closing delimiter.
///
/// Every GraphQL comment is a line comment, so each one is deferred to a `line_suffix()`
/// (its width must not count toward the `fits` measurement of the preceding group)
/// with `expand_parent()` so the enclosing container stays multi-line.
/// `lower_bound` seeds the gap measurement for the first comment.
fn write_trailing_inside_comments<'a>(
    comments: &[Span],
    lower_bound: u32,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let mut prev_end = lower_bound;
    for &span in comments {
        let gap_start = prev_end;
        let content = format_with(move |f: &mut GraphqlFormatter<'_, 'a>| {
            let gap = source.bytes_range(gap_start, span.start);
            write_gap(gap, f);
            write_single_comment(span, f);
        });
        write!(f, [line_suffix(&content), expand_parent()]);
        prev_end = span.end;
    }
}

/// Drains comments before `upper_bound` (typically a closing-delimiter position) and
/// writes them via [`write_trailing_inside_comments`].
pub fn flush_trailing_inside_comments(
    lower_bound: u32,
    upper_bound: u32,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    let trailing = f.context().comments().take_before(upper_bound);
    write_trailing_inside_comments(trailing, lower_bound, f);
}

/// Returns `true` if `span` is an ignore marker (`# oxfmt-ignore` / `# prettier-ignore`).
fn is_suppression_comment(source: SourceText<'_>, span: Span) -> bool {
    let content = source.text_for(&span);
    is_suppression_marker(content.strip_prefix('#').unwrap_or(content))
}

/// Returns `true` if any pending comment up to `before` is a suppression marker.
pub fn is_suppressed_before(f: &GraphqlFormatter<'_, '_>, before: u32) -> bool {
    let source = f.context().source_text();
    f.context().comments().iter_before(before).any(|c| is_suppression_comment(source, c))
}

/// Emits a node's leading comments, then the node's source verbatim,
/// then advances the comment cursor past the span.
pub fn write_suppressed_node(span: Span, f: &mut GraphqlFormatter<'_, '_>) {
    flush_leading_comments(span.start, f);
    // The IR only supports `\n` as a line break. Normalize CRLF / CR / LS / PS to LF;
    // the printer will re-emit the configured `LineEnding` at the final stage.
    let raw = f.context().source_text().text_for(&span);
    let normalized = normalize_newlines(raw, LINE_TERMINATORS);
    write!(f, text(arena_cow_str(&normalized, f)));
    // The verbatim text already includes inside-span comments;
    // advance the cursor so they aren't re-emitted later.
    let _ = f.context().comments().take_before(span.end);
}

#[cfg(test)]
mod tests {
    use super::{Gap, classify_gap};

    // Whitebox coverage of `classify_gap`'s line-terminator handling.
    // End-to-end CRLF behavior of the suppress path is covered by the
    // `tests/fixtures/graphql/crlf/` fixtures.
    #[test]
    fn classify_gap_counts_line_terminators() {
        assert_eq!(classify_gap(b" \t "), Gap::None);
        assert_eq!(classify_gap(b"a"), Gap::None);
        assert_eq!(classify_gap(b"\n"), Gap::Line);
        assert_eq!(classify_gap(b"\n  \n"), Gap::Blank);
        // CRLF must collapse to one break, never two (otherwise blank lines are invented).
        assert_eq!(classify_gap(b"\r\n"), Gap::Line);
        assert_eq!(classify_gap(b"\r\n\r\n"), Gap::Blank);
        // Lone CR is a GraphQL line terminator.
        assert_eq!(classify_gap(b"\r"), Gap::Line);
        assert_eq!(classify_gap(b"\r\r"), Gap::Blank);
        // Mixed endings.
        assert_eq!(classify_gap(b"\n\r\n"), Gap::Blank);
    }

    #[test]
    fn classify_gap_treats_tokens_as_content() {
        // A token on its own line (e.g. the `&` between two `implements` comments)
        // is not a blank line.
        assert_eq!(classify_gap(b"\n&\n"), Gap::Line);
        // Comma-only lines count as content too (mirrors Prettier's `isNextLineEmpty`,
        // which skips commas only on the current line's tail).
        assert_eq!(classify_gap(b"\n,\n"), Gap::Line);
        assert_eq!(classify_gap(b",\n\n"), Gap::Blank);
        // Content on the tail of the first or last line is not "inside" the gap.
        assert_eq!(classify_gap(b",\n  "), Gap::Line);
    }
}
