use oxc_formatter_core::{
    Buffer, SourceText, SpanCursor,
    builders::{align, empty_line, expand_parent, hard_line_break, line_suffix, space, text},
    spec::is_suppression_marker,
    write,
};
use oxc_span::{GetSpan, Span};

use crate::print::{YamlFormatter, format_with};

/// A comment bridged from the parser: its span in the arena source,
/// plus the parser-recorded layout fact print sites branch on.
#[derive(Clone, Copy, Debug)]
pub struct SourceComment {
    pub span: Span,
    /// The `#`'s 0-based column when only whitespace precedes it on its line;
    /// `None` when the comment trails other content (see the parser's `Comment`).
    pub own_line_column: Option<u32>,
}

impl GetSpan for SourceComment {
    fn span(&self) -> Span {
        self.span
    }
}

/// Cursor over the sorted comment list.
///
/// YAML comments are always single-line (`# ...` to end of line);
/// the parser collects them into a flat, source-ordered list and `format()` bridges them to [`SourceComment`]s.
/// Comment placement (leading / trailing / end) is decided positionally at print sites.
pub type Comments<'a> = SpanCursor<'a, SourceComment>;

/// `anchor`, moved past the most recently consumed comment when that lies beyond it.
///
/// A nested container's end-comment flush consumes comments PAST the outer caller's anchor
/// (a deeper-indented run belongs to the inner container),
/// reproducing the vertical spacing in front of them as it prints.
/// Gap measurement resuming from the unmoved anchor would observe that same spacing again
/// and emit a second blank line.
pub fn gap_anchor_after_consumed(anchor: u32, f: &YamlFormatter<'_, '_>) -> u32 {
    f.context().comments().last_consumed().map_or(anchor, |c| anchor.max(c.span.end))
}

pub use oxc_formatter_core::spec::{Gap, classify_gap};

/// `true` when the source between `from` and `to` holds nothing but whitespace and comments
/// (every line blank or `#`-only after indentation).
fn gap_is_trivia_only(source: &str, from: u32, to: u32) -> bool {
    source[from as usize..to as usize].lines().all(|line| {
        let trimmed = line.trim_start();
        trimmed.is_empty() || trimmed.starts_with('#')
    })
}

/// One line break, widened to a blank line when the source gap holds one.
pub fn write_blank_preserving_break(
    prev_end: u32,
    upper_bound: u32,
    f: &mut YamlFormatter<'_, '_>,
) {
    let prev_end = gap_anchor_after_consumed(prev_end, f);
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
fn write_leading_comments(
    comments: &[SourceComment],
    value_start: u32,
    f: &mut YamlFormatter<'_, '_>,
) {
    let source = f.context().source_text();
    for (i, comment) in comments.iter().enumerate() {
        write_single_comment(comment.span, f);
        let next_pos = comments.get(i + 1).map_or(value_start, |c| c.span.start);
        write_gap(source.bytes_range(comment.span.end, next_pos), f);
    }
}

/// Drains and emits all pending comments ending at or before `value_start` as leading comments.
pub fn flush_leading_comments(value_start: u32, f: &mut YamlFormatter<'_, '_>) {
    let leading = f.context().comments().take_before(value_start);
    write_leading_comments(leading, value_start, f);
}

/// The next pending comment when it sits on the same line after `pos`
/// (nothing but spaces/tabs between), without consuming it.
pub fn pending_same_line_comment(pos: u32, f: &YamlFormatter<'_, '_>) -> Option<SourceComment> {
    pending_same_line_comment_over(pos, &[], f)
}

/// [pending_same_line_comment], additionally allowing the caller's
/// `gap_punctuation` bytes in the gap (see [write_trailing_same_line_comment]).
fn pending_same_line_comment_over(
    pos: u32,
    gap_punctuation: &[u8],
    f: &YamlFormatter<'_, '_>,
) -> Option<SourceComment> {
    f.context().comments().peek().filter(|comment| {
        comment.span.start >= pos
            && f.context().source_text().all_bytes_match(pos, comment.span.start, |b| {
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
pub fn write_trailing_same_line_comment(
    prev_end: u32,
    gap_punctuation: &[u8],
    f: &mut YamlFormatter<'_, '_>,
) {
    let Some(comment) = pending_same_line_comment_over(prev_end, gap_punctuation, f) else {
        return;
    };
    f.context().comments().take_before(comment.span.end);
    write_comment_line_suffix(comment.span, f);
    write!(f, expand_parent());
}

/// The ` # ...` emission of a same-line trailing comment: a `line_suffix`,
/// so it never counts toward the `fits` measurement (see the "trailing comment width" divergence).
/// Gating and consuming are the caller's.
pub fn write_comment_line_suffix<'a>(span: Span, f: &mut YamlFormatter<'_, 'a>) {
    let content = format_with(move |f: &mut YamlFormatter<'_, 'a>| {
        write!(f, space());
        write_single_comment(span, f);
    });
    write!(f, line_suffix(&content));
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
    f.context()
        .comments()
        .peek()
        .filter(|c| c.span.start < next_start)
        .map_or(next_start, |c| c.span.start)
}

/// The start of the LAST pending comment up to `before`, when it is a suppression marker.
fn suppression_marker_start_before(f: &YamlFormatter<'_, '_>, before: u32) -> Option<u32> {
    let source = f.context().source_text();
    f.context()
        .comments()
        .iter_before(before)
        .last()
        .filter(|c| is_suppression_comment(source, c.span))
        .map(|c| c.span.start)
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
/// printed at the container's item-content column: `align_width` in from the items,
/// the tab width for block mappings, the `- ` width (2) for block sequences
/// (the placement effect of Prettier's `shouldOwnEndComment` + `mappingValue.endComments`, re-derived positionally).
/// Its direct-block-scalar exclusion is the caller's gate: `ItemTail` in `print/block_collection.rs`.
/// Returns the position after the last claimed comment so the caller can keep measuring gaps from it.
pub fn flush_container_end_comments(
    item_column: u32,
    align_width: u8,
    prev_end: u32,
    upper_bound: u32,
    f: &mut YamlFormatter<'_, '_>,
) -> u32 {
    let source = f.context().source_text();
    let mut prev_end = gap_anchor_after_consumed(prev_end, f);
    loop {
        let Some(comment) = f.context().comments().peek() else { return prev_end };
        let span = comment.span;
        if span.end > upper_bound
            || comment.own_line_column.is_none_or(|column| column <= item_column)
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
        write!(f, align(align_width, &inner));
        prev_end = span.end;
    }
}
