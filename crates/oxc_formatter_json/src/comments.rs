use std::cell::Cell;

use oxc_allocator::StringBuilder;
use oxc_ast::Comment;
use oxc_formatter_core::{
    Buffer, Format, SourceText,
    builders::{empty_line, expand_parent, hard_line_break, space, text},
    util::is_suppression_marker,
    write,
};
use oxc_span::Span;
use oxc_syntax::line_terminator::LineTerminatorSplitter;

use crate::{context::JsonFormatContext, print::JsonFormatter};

/// Cursor over a sorted comment list that hands out unprinted slices in span order.
///
/// `cursor` is a [`Cell`] so the API works through `&self`, allowing simultaneous
/// borrows alongside other context fields. The `Format` trait dispatches via `&self`,
/// so a `&mut Comments` accessor would force every drain site to go through
/// `f.context_mut()` and conflict with read-only context accesses.
pub struct Comments<'a> {
    inner: &'a [Comment],
    cursor: Cell<usize>,
}

impl<'a> Comments<'a> {
    pub fn new(comments: &'a [Comment]) -> Self {
        Self { inner: comments, cursor: Cell::new(0) }
    }

    /// Returns unprinted comments whose `span.end <= upper_bound`,
    /// and advances the cursor past them so they won't be returned again.
    pub fn take_before(&self, upper_bound: u32) -> &'a [Comment] {
        let start = self.cursor.get();
        let mut end = start;
        while end < self.inner.len() && self.inner[end].span.end <= upper_bound {
            end += 1;
        }
        self.cursor.set(end);
        &self.inner[start..end]
    }

    /// Drains all remaining unprinted comments and returns them.
    pub fn take_remaining(&self) -> &'a [Comment] {
        let start = self.cursor.get();
        self.cursor.set(self.inner.len());
        &self.inner[start..]
    }

    /// Iterator over unprinted comments whose `span.end <= upper_bound`.
    /// Does NOT advance the cursor, callers that want to mark these as
    /// printed must call [`Self::take_before`] instead.
    ///
    /// Mirrors `oxc_formatter::formatter::comments::Comments::comments_before_iter`
    /// so suppression / leading-comment checks can compose `.any(...)` / `.next()` directly and short-circuit.
    pub fn iter_before(&self, upper_bound: u32) -> impl Iterator<Item = &'a Comment> {
        let start = self.cursor.get();
        self.inner[start..].iter().take_while(move |c| c.span.end <= upper_bound)
    }
}

/// Emit a single comment, re-aligning interior `*`-prefixed lines
/// so the stars line up with the opening `/*` regardless of the source's original indentation.
///
/// Mirrors `oxc_formatter`'s `impl Format for Comment` (`formatter/trivia.rs`):
/// - Single-line comments (line and one-line block) emit as-is (trim trailing whitespace).
/// - Multi-line block comments whose interior lines all start with `*` (an
///   "indentable" / JSDoc-shaped comment) split into lines; the first is emitted
///   trimmed, and each subsequent line as `[hard_line_break, " ", trimmed]` so
///   the surrounding indent context re-indents the stars.
/// - Other multi-line block comments normalize `\r\n` → `\n` but otherwise stay
///   verbatim; their first line still gets its trailing whitespace trimmed.
pub fn write_single_comment(comment: &Comment, f: &mut JsonFormatter<'_, '_>) {
    let content = f.context().source_text().text_for(&comment.span);

    if !comment.is_multiline_block() {
        write!(f, text(content.trim_end()));
        return;
    }

    let mut lines = LineTerminatorSplitter::new(content);
    if is_alignable_comment(content) {
        // `unwrap` is safe because `content` contains at least one line.
        let first_line = lines.next().unwrap();
        write!(f, text(first_line.trim_end()));
        for line in lines {
            write!(f, [hard_line_break(), " ", text(line.trim())]);
        }
    } else {
        // Normalize line endings (`\r\n` → `\n`) but otherwise preserve the body.
        let mut builder = StringBuilder::with_capacity_in(content.len(), f.allocator());
        // `unwrap` is safe because `content` contains at least one line.
        builder.push_str(lines.next().unwrap().trim_end());
        for line in lines {
            builder.push('\n');
            builder.push_str(line);
        }
        write!(f, text(builder.into_str()));
    }
}

/// Returns `true` if every line after the first starts with `*`.
/// (after stripping leading whitespace)
/// These comments are "alignable":
/// their interior lines can be re-indented so the stars line up with the opening `/*`.
fn is_alignable_comment(content: &str) -> bool {
    LineTerminatorSplitter::new(content).skip(1).all(|line| line.trim_start().starts_with('*'))
}

/// Emit comments that precede an AST value,
/// preserving the source's vertical spacing (0/1/blank) between each comment and the next position.
/// Another comment for in-group separators, or `value_start` for the last comment's break to the value.
pub fn write_leading_comments(
    comments: &[Comment],
    value_start: u32,
    f: &mut JsonFormatter<'_, '_>,
) {
    let source = f.context().source_text();
    for (i, comment) in comments.iter().enumerate() {
        write_single_comment(comment, f);
        let next_pos = comments.get(i + 1).map_or(value_start, |c| c.span.start);
        write_gap(source.bytes_range(comment.span.end, next_pos), f);
    }
}

/// Counts `\n` bytes in `slice`.
#[expect(clippy::naive_bytecount, reason = "tiny slice, not worth a bytecount dep")]
pub fn count_newlines(slice: &[u8]) -> usize {
    slice.iter().filter(|&&b| b == b'\n').count()
}

/// Emits the formatter element that reproduces the vertical spacing implied by `gap`:
/// `space` for 0 newlines, `hard_line_break` for 1, `empty_line` for 2+ (blank line).
fn write_gap(gap: &[u8], f: &mut JsonFormatter<'_, '_>) {
    match count_newlines(gap) {
        0 => write!(f, space()),
        1 => write!(f, hard_line_break()),
        _ => write!(f, empty_line()),
    }
}

/// Emit dangling comments inside an empty container (the caller wraps the result in
/// [`oxc_formatter_core::builders::block_indent`] or similar).
pub fn write_dangling_comments(comments: &[Comment], f: &mut JsonFormatter<'_, '_>) {
    for (i, comment) in comments.iter().enumerate() {
        if i > 0 {
            write!(f, hard_line_break());
        }
        write_single_comment(comment, f);
    }
}

/// Emit comments that sit between the last child of a container and its closing delimiter.
///
/// Like [`write_leading_comments`], preserves the source's vertical spacing (0/1/blank)
/// in front of each comment.
/// `lower_bound` is the position immediately after the last emitted content
/// (typically the container's last child's `span.end`)
/// and seeds the gap measurement for the first comment.
pub fn write_trailing_inside_comments(
    comments: &[Comment],
    lower_bound: u32,
    f: &mut JsonFormatter<'_, '_>,
) {
    let source = f.context().source_text();
    let mut prev_end = lower_bound;
    for comment in comments {
        write_gap(source.bytes_range(prev_end, comment.span.start), f);
        write_single_comment(comment, f);
        // To prevent `[a, // this comment breaks -> ]`
        if comment.is_line() {
            write!(f, expand_parent());
        }
        prev_end = comment.span.end;
    }
}

/// `Format` adapter that drains and prints all pending comments ending at or before
/// `span.start`. Lets callers replace the 3-line `comments().take_before` + `if !empty`
/// dance with `write!(f, [FormatLeadingComments(span), value])`.
pub struct FormatLeadingComments(pub Span);

impl<'a> Format<'a, JsonFormatContext<'a>> for FormatLeadingComments {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        let leading = f.context().comments().take_before(self.0.start);
        write_leading_comments(leading, self.0.start, f);
    }
}

/// `Format` adapter that drains comments before `upper_bound`
/// (typically the container's closing-delimiter position) and writes them.
/// `lower_bound` is the position right after the last emitted child
/// so the first comment's gap can be measured for blank-line preservation;
/// pass `upper_bound` when there is no prior child.
pub struct FormatTrailingInsideComments {
    pub lower_bound: u32,
    pub upper_bound: u32,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FormatTrailingInsideComments {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        let trailing = f.context().comments().take_before(self.upper_bound);
        write_trailing_inside_comments(trailing, self.lower_bound, f);
    }
}

/// Returns `true` if `comment` is an ignore marker (`oxfmt-ignore` / `prettier-ignore`).
/// Mirrors `oxc_formatter`'s suppression rule so JSON honors the same authoring convention as JS/TS.
pub fn is_suppression_comment(source: SourceText<'_>, comment: &Comment) -> bool {
    let body = source.text_for(&comment.content_span());
    is_suppression_marker(body)
}

/// Returns `true` if any pending comment up to `before` is a suppression marker.
/// `before` is typically the next AST node's `span.start`.
pub fn is_suppressed_before(f: &JsonFormatter<'_, '_>, before: u32) -> bool {
    let source = f.context().source_text();
    f.context().comments().iter_before(before).any(|c| is_suppression_comment(source, c))
}

/// `Format` adapter that emits a node's leading comments, then the node's source
/// verbatim, then advances the comment cursor past the span. Used for both
/// `oxfmt-ignore` / `prettier-ignore` suppression and JSON-invalid fallback paths.
pub struct FormatSuppressedNode(pub Span);

impl<'a> Format<'a, JsonFormatContext<'a>> for FormatSuppressedNode {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        write!(f, FormatLeadingComments(self.0));
        write!(f, text(f.context().source_text().text_for(&self.0)));
        // The verbatim text already includes inside-span comments;
        // advance the cursor so they aren't re-emitted as leading comments of a later node.
        let _ = f.context().comments().take_before(self.0.end);
    }
}
