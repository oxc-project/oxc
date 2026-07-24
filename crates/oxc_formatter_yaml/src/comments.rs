use std::cell::Cell;

use oxc_formatter_core::{
    Buffer, SourceText,
    builders::{align, empty_line, expand_parent, hard_line_break, line_suffix, space, text},
    spec::is_suppression_marker,
    write,
};
use oxc_span::Span;

use crate::print::{YamlFormatter, format_with};

/// Cursor over a sorted comment-span list that hands out unprinted slices in span order.
///
/// YAML comments are always single-line (`# ...` to end of line);
/// the parser collects them into a flat, source-ordered list and `format()` bridges them to [`Span`]s.
/// Comment placement (leading / trailing / end) is decided positionally at print sites.
///
/// `cursor` is a [`Cell`] so the API works through `&self`.
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

    /// `anchor`, moved past the most recently consumed comment when that lies beyond it.
    ///
    /// A nested container's end-comment flush consumes comments PAST the outer caller's anchor
    /// (a deeper-indented run belongs to the inner container),
    /// reproducing the vertical spacing in front of them as it prints.
    /// Gap measurement resuming from the unmoved anchor would observe that same spacing again
    /// and emit a second blank line.
    pub fn gap_anchor_after_consumed(&self, anchor: u32) -> u32 {
        let last_consumed_end = self.cursor.get().checked_sub(1).map(|i| self.inner[i].end);
        last_consumed_end.map_or(anchor, |end| anchor.max(end))
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
/// Tokens in the gap make their line non-blank,
/// so newline counting alone would over-report blank lines (e.g. an indicator such as `-` sitting on its own line).
///
/// The source is normalized to `\n` before parsing (see `format()`),
/// but the CR-handling is kept so the helper stays correct on raw slices in tests.
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

/// `true` when the source between `from` and `to` holds nothing but whitespace and comments
/// (every line blank or `#`-only after indentation).
fn gap_is_trivia_only(source: &str, from: u32, to: u32) -> bool {
    source[from as usize..to as usize].lines().all(|line| {
        let trimmed = line.trim_start();
        trimmed.is_empty() || trimmed.starts_with('#')
    })
}

/// `true` when only whitespace precedes `offset` on its line
/// (an own-line comment, as opposed to one trailing other content).
pub fn is_own_line(source: &str, offset: u32) -> bool {
    own_line_column(source, offset).is_some()
}

/// The 0-based column of `offset` when only whitespace precedes it on its line,
/// in a single backward scan; `None` when other content does.
fn own_line_column(source: &str, offset: u32) -> Option<u32> {
    let mut column = 0u32;
    for &byte in source.as_bytes()[..offset as usize].iter().rev() {
        match byte {
            b'\n' => break,
            b' ' | b'\t' => column += 1,
            _ => return None,
        }
    }
    Some(column)
}

/// One line break, widened to a blank line when the source gap holds one.
pub fn write_blank_preserving_break(
    prev_end: u32,
    upper_bound: u32,
    f: &mut YamlFormatter<'_, '_>,
) {
    let prev_end = f.context().comments().gap_anchor_after_consumed(prev_end);
    if prev_end < upper_bound
        && classify_gap(f.context().source_text().bytes_range(prev_end, upper_bound)) == Gap::Blank
    {
        write!(f, empty_line());
    } else {
        write!(f, hard_line_break());
    }
}

/// Emit a single comment verbatim (trailing whitespace trimmed).
/// The spacing after `#` is kept as authored, never normalized.
pub fn write_single_comment(span: Span, f: &mut YamlFormatter<'_, '_>) {
    let content = f.context().source_text().text_for(&span);
    write!(f, text(content.trim_end()));
}

/// Emits the formatter element that reproduces the vertical spacing implied by `gap`:
/// `space` for same-line, `hard_line_break` for a line break, `empty_line` for a blank line.
fn write_gap(gap: &[u8], f: &mut YamlFormatter<'_, '_>) {
    match classify_gap(gap) {
        Gap::None => write!(f, space()),
        Gap::Line => write!(f, hard_line_break()),
        Gap::Blank => write!(f, empty_line()),
    }
}

/// Emit comments that precede a node,
/// preserving the source's vertical spacing (0/1/blank) between each comment and the next position.
fn write_leading_comments(comments: &[Span], value_start: u32, f: &mut YamlFormatter<'_, '_>) {
    let source = f.context().source_text();
    for (i, &span) in comments.iter().enumerate() {
        write_single_comment(span, f);
        let next_pos = comments.get(i + 1).map_or(value_start, |c| c.start);
        write_gap(source.bytes_range(span.end, next_pos), f);
    }
}

/// Drains and emits all pending comments ending at or before `value_start` as leading comments.
pub fn flush_leading_comments(value_start: u32, f: &mut YamlFormatter<'_, '_>) {
    let leading = f.context().comments().take_before(value_start);
    write_leading_comments(leading, value_start, f);
}

/// The next pending comment when it sits on the same line after `pos`
/// (nothing but spaces/tabs between), without consuming it.
pub fn pending_same_line_comment(pos: u32, f: &YamlFormatter<'_, '_>) -> Option<Span> {
    pending_same_line_comment_over(pos, &[], f)
}

/// [pending_same_line_comment], additionally allowing the caller's
/// `gap_punctuation` bytes in the gap (see [write_trailing_same_line_comment]).
fn pending_same_line_comment_over(
    pos: u32,
    gap_punctuation: &[u8],
    f: &YamlFormatter<'_, '_>,
) -> Option<Span> {
    f.context().comments().peek().filter(|span| {
        span.start >= pos
            && f.context().source_text().all_bytes_match(pos, span.start, |b| {
                matches!(b, b' ' | b'\t') || gap_punctuation.contains(&b)
            })
    })
}

/// If the next pending comment sits on the same line as `prev_end`,
/// drain it and emit it as a trailing line-suffix comment (` # ...`).
/// `expand_parent()` keeps the enclosing container multi-line.
///
/// The gap may only contain whitespace and `gap_punctuation`:
/// the structural bytes the CALLER's syntax puts between the node end and its trailing comment
/// (`,` between flow entries, `:` after an implicit key), so syntax knowledge stays at the print site.
/// Any other content means the comment trails a LATER node on the same line
/// (`[a, b, c # comment` must not attach the comment to `a`).
pub fn write_trailing_same_line_comment<'a>(
    prev_end: u32,
    gap_punctuation: &[u8],
    f: &mut YamlFormatter<'_, 'a>,
) {
    let Some(span) = pending_same_line_comment_over(prev_end, gap_punctuation, f) else { return };
    f.context().comments().take_before(span.end);
    let content = format_with(move |f: &mut YamlFormatter<'_, 'a>| {
        write!(f, space());
        write_single_comment(span, f);
    });
    write!(f, [line_suffix(&content), expand_parent()]);
}

/// Returns `true` if `span` is an ignore marker (`# oxfmt-ignore` / `# prettier-ignore`).
fn is_suppression_comment(source: SourceText<'_>, span: Span) -> bool {
    let content = source.text_for(&span);
    is_suppression_marker(content.strip_prefix('#').unwrap_or(content))
}

/// Returns `true` if the LAST pending comment up to `before` is a suppression
/// marker (Prettier's `hasPrettierIgnore` checks the last leading/end comment).
pub fn is_suppressed_last_before(f: &YamlFormatter<'_, '_>, before: u32) -> bool {
    suppression_marker_start_before(f, before).is_some()
}

/// The gap-measurement upper bound before `next_start`: the next pending
/// comment when it precedes it (so a blank line in front of a leading comment
/// is still measured), else `next_start` itself.
pub fn gap_upper_bound(next_start: u32, f: &YamlFormatter<'_, '_>) -> u32 {
    f.context().comments().peek().filter(|c| c.start < next_start).map_or(next_start, |c| c.start)
}

/// The start of the LAST pending comment up to `before`, when it is a suppression marker.
fn suppression_marker_start_before(f: &YamlFormatter<'_, '_>, before: u32) -> Option<u32> {
    let source = f.context().source_text();
    f.context()
        .comments()
        .iter_before(before)
        .last()
        .filter(|c| is_suppression_comment(source, *c))
        .map(|c| c.start)
}

/// Flush bound for a block collection's leading comments:
/// stops before a trailing suppression marker so it survives for the first item's own check
/// (an ignore right above the first item freezes that item, not the whole collection).
pub fn suppression_flush_bound(
    is_block_collection: bool,
    bound: u32,
    f: &YamlFormatter<'_, '_>,
) -> u32 {
    if is_block_collection {
        suppression_marker_start_before(f, bound).unwrap_or(bound)
    } else {
        bound
    }
}

/// Emits a node's leading comments, then the node's source verbatim,
/// then advances the comment cursor past the span.
///
/// The suppressed range covers exactly one node (Prettier bug #13008 — ignore
/// bleeding into ALL following nodes — is intentionally not reproduced).
pub fn write_suppressed_node(span: Span, f: &mut YamlFormatter<'_, '_>) {
    flush_leading_comments(span.start, f);
    // The source is already normalized to `\n`-only line breaks before parsing,
    // so the raw slice is safe for the IR (which forbids `\r`).
    let raw = f.context().source_text().text_for(&span);
    write!(f, text(raw.trim_end()));
    // The verbatim text already includes inside-span comments;
    // advance the cursor so they aren't re-emitted later.
    let _ = f.context().comments().take_before(span.end);
}

/// Claims pending comments indented strictly deeper than `item_column` as the preceding item's end comments,
/// printed one indent level in
/// (the placement effect of Prettier's `shouldOwnEndComment` + `mappingValue.endComments`, re-derived positionally).
/// Returns the position after the last claimed comment so the caller can keep measuring gaps from it.
pub fn flush_container_end_comments(
    item_column: u32,
    prev_end: u32,
    upper_bound: u32,
    f: &mut YamlFormatter<'_, '_>,
) -> u32 {
    let source = f.context().source_text();
    let tab_width = f.options().indent_width.value();
    let mut prev_end = f.context().comments().gap_anchor_after_consumed(prev_end);
    loop {
        let Some(span) = f.context().comments().peek() else { return prev_end };
        if span.end > upper_bound
            || own_line_column(&source, span.start).is_none_or(|column| column <= item_column)
            // An end-comment run directly follows its container;
            // other tokens in between mean the comment belongs to a LATER node
            // (a nested collection's unbounded tail flush must not jump over the parent's following items).
            || !gap_is_trivia_only(&source, prev_end, span.start)
        {
            return prev_end;
        }
        f.context().comments().take_before(span.end);
        let is_blank = classify_gap(source.bytes_range(prev_end, span.start)) == Gap::Blank;
        // The line break lives INSIDE `align` so the comment line is indented
        let inner = format_with(move |f: &mut YamlFormatter<'_, '_>| {
            if is_blank {
                write!(f, empty_line());
            } else {
                write!(f, hard_line_break());
            }
            write_single_comment(span, f);
        });
        write!(f, align(tab_width, &inner));
        prev_end = span.end;
    }
}

#[cfg(test)]
mod tests {
    use super::{Gap, classify_gap};

    #[test]
    fn classify_gap_counts_line_terminators() {
        assert_eq!(classify_gap(b" \t "), Gap::None);
        assert_eq!(classify_gap(b"a"), Gap::None);
        assert_eq!(classify_gap(b"\n"), Gap::Line);
        assert_eq!(classify_gap(b"\n  \n"), Gap::Blank);
        assert_eq!(classify_gap(b"\r\n"), Gap::Line);
        assert_eq!(classify_gap(b"\r\n\r\n"), Gap::Blank);
    }

    #[test]
    fn classify_gap_treats_tokens_as_content() {
        // An indicator on its own line (e.g. `-` of a sequence item) is not a blank line.
        assert_eq!(classify_gap(b"\n-\n"), Gap::Line);
        // Content on the tail of the first or last line is not "inside" the gap.
        assert_eq!(classify_gap(b"-\n  "), Gap::Line);
    }
}
