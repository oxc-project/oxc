use oxc_formatter_core::{
    Buffer, LINE_TERMINATORS, SourceText, SpanCursor, arena_cow_str,
    builders::{
        empty_line, expand_parent, hard_line_break, line_suffix, line_suffix_boundary, space, text,
    },
    normalize_newlines,
    spec::is_suppression_marker,
    write,
};
use oxc_span::Span;

use crate::print::{GraphqlFormatter, format_with};

/// Comma note (why token-sensitivity matters for GraphQL):
/// Prettier's `isNextLineEmpty` skips commas only on the current line's tail,
/// so an insignificant comma-only line still counts as content (`a\n,\nb` has no blank line).
pub use oxc_formatter_core::spec::{Gap, classify_gap};

/// Cursor over the sorted comment-span list.
/// GraphQL comments are always single-line (`# ...` to end of line).
pub type Comments<'a> = SpanCursor<'a, Span>;

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
pub fn write_trailing_same_line_comment(
    prev_end: u32,
    upper: u32,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    let Some(span) = f.context().comments().peek() else { return };
    if span.end > upper {
        return;
    }
    let source = f.context().source_text();
    if classify_gap(source.bytes_range(prev_end, span.start)) != Gap::None {
        return;
    }
    take_and_write_line_suffix_comment(span, f);
}

/// If the next pending comment follows `prev_end` on the same line with only
/// whitespace between, drain it and emit it as a trailing line-suffix comment.
///
/// Unlike [`write_trailing_same_line_comment`], the whitespace-only guard keeps
/// the comment from being pulled backwards across source tokens the caller has
/// not printed yet (e.g. `"desc" type # c` must leave the comment to the keyword's
/// line, not attach it to the description).
///
/// Precondition: the caller emits a line break right after this call.
/// `line_suffix` defers the comment to the next printed break, so any token
/// written on the same line before that break would jump in front of the comment.
pub fn write_adjacent_trailing_comment(prev_end: u32, f: &mut GraphqlFormatter<'_, '_>) {
    let Some(span) = f.context().comments().peek() else { return };
    let source = f.context().source_text();
    if span.start < prev_end
        || !source.all_bytes_match(prev_end, span.start, |b| b == b' ' || b == b'\t')
    {
        return;
    }
    take_and_write_line_suffix_comment(span, f);
}

/// Shared emit tail of the trailing claims: drain `span` and defer ` # ...`
/// to the current line's end.
fn take_and_write_line_suffix_comment<'a>(span: Span, f: &mut GraphqlFormatter<'_, 'a>) {
    f.context().comments().take_before(span.end);
    let content = format_with(move |f: &mut GraphqlFormatter<'_, 'a>| {
        write!(f, space());
        write_single_comment(span, f);
    });
    write!(f, [line_suffix(&content), expand_parent()]);
}

/// `true` if only whitespace precedes `position` on its source line.
fn is_own_line(source: SourceText<'_>, position: u32) -> bool {
    source
        .bytes_to(position)
        .find(|&b| b != b' ' && b != b'\t')
        .is_none_or(|b| b == b'\n' || b == b'\r')
}

/// Claims the next pending comment when it ends at or before `upper`
/// (the source start of what the caller prints next)
/// and trails other content on its source line.
/// e.g. it is a trailing comment of an already-printed token,
/// about to be crossed forward by a formatter literal (` implements`, `": "`, `{`, ...).
/// Own-line comments are left pending (they are leading trivia of what follows).
///
/// Returns whether a comment was claimed (deferred to this line's end as a `line_suffix`).
fn claim_trailing_comment_before(upper: u32, f: &mut GraphqlFormatter<'_, '_>) -> bool {
    let Some(span) = f.context().comments().peek() else { return false };
    if span.end > upper || is_own_line(f.context().source_text(), span.start) {
        return false;
    }
    take_and_write_line_suffix_comment(span, f);
    true
}

/// [`claim_trailing_comment_before`] + `line_suffix_boundary`,
/// for callers that continue with same-line tokens:
/// the boundary hard-breaks in front of them so the claimed comment keeps its source line.
/// Layout-neutral when nothing is claimed.
pub fn flush_trailing_comment_before(upper: u32, f: &mut GraphqlFormatter<'_, '_>) {
    if claim_trailing_comment_before(upper, f) {
        write!(f, line_suffix_boundary());
    }
}

/// [`claim_trailing_comment_before`] alone,
/// for callers whose next element already begins with a line break (block indents, expanded separators):
/// the suffix flushes at that break, a boundary would double it.
pub fn flush_trailing_comment_before_break(upper: u32, f: &mut GraphqlFormatter<'_, '_>) {
    claim_trailing_comment_before(upper, f);
}

/// Emit comments that sit between the last child of a container and its closing delimiter.
///
/// Every GraphQL comment is a line comment, so each one is deferred to a `line_suffix()`
/// (its width must not count toward the `fits` measurement of the preceding group)
/// with `expand_parent()` so the enclosing container stays multi-line.
/// `lower_bound` seeds the gap measurement for the first comment;
/// `None` when the source gap is not measurable (the printed output has already moved past the comment's position),
/// the comment then goes on its own line.
fn write_trailing_inside_comments<'a>(
    comments: &[Span],
    lower_bound: Option<u32>,
    f: &mut GraphqlFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let mut prev_end = lower_bound;
    for &span in comments {
        // Positional-cursor invariant; see `flush_overlooked_inside_comments`
        debug_assert!(prev_end.is_none_or(|pe| pe <= span.start));
        let gap_start = prev_end;
        let content = format_with(move |f: &mut GraphqlFormatter<'_, 'a>| {
            if let Some(gap_start) = gap_start {
                write_gap(source.bytes_range(gap_start, span.start), f);
            } else {
                write!(f, hard_line_break());
            }
            write_single_comment(span, f);
        });
        write!(f, [line_suffix(&content), expand_parent()]);
        prev_end = Some(span.end);
    }
}

/// Fallback flush for comments still pending inside a just-printed node's span:
/// positions no printer claims (e.g. between a type and its `!`, or after the last argument's directives).
/// Draining them here keeps the positional cursor monotonic,
/// a later flush point would sit AFTER these comments in the source, making its gap range inverted (start > end).
pub fn flush_overlooked_inside_comments(upper_bound: u32, f: &mut GraphqlFormatter<'_, '_>) {
    let leftover = f.context().comments().take_before(upper_bound);
    write_trailing_inside_comments(leftover, None, f);
}

/// Drains comments before `upper_bound` (typically a closing-delimiter position)
/// and writes them via [`write_trailing_inside_comments`].
pub fn flush_trailing_inside_comments(
    lower_bound: u32,
    upper_bound: u32,
    f: &mut GraphqlFormatter<'_, '_>,
) {
    let trailing = f.context().comments().take_before(upper_bound);
    write_trailing_inside_comments(trailing, Some(lower_bound), f);
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
