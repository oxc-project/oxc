use oxc_formatter_core::{
    Buffer, Format, SourceText, SpanCursor,
    builders::{empty_line, expand_parent, hard_line_break, line_suffix, maybe_space, space, text},
    spec::is_suppression_marker,
    write,
};
use oxc_span::{GetSpan, Span};

use crate::{
    context::CssFormatContext,
    print::{CssFormatter, format_with},
};

/// A source comment.
///
/// oxc-css-parser keeps comments out of the AST; `format()` collects them through
/// `ParserBuilder::comments()` and stores their spans here.
#[derive(Clone, Copy, Debug)]
pub struct CssComment {
    pub span: Span,
    /// `// ...` line comment (SCSS/Less only). Block comments are `/* ... */`.
    pub inline: bool,
}

impl GetSpan for CssComment {
    fn span(&self) -> Span {
        self.span
    }
}

/// Cursor over the sorted comment list.
pub type Comments<'a> = SpanCursor<'a, CssComment>;

pub use oxc_formatter_core::spec::{Gap, classify_gap};

/// Emit raw comment text, trimming trailing whitespace from `//`.
/// Private because it does not emit the line boundary `//` requires.
fn write_comment_text(comment: CssComment, f: &mut CssFormatter<'_, '_>) {
    let content = f.context().source_text().text_for(&comment.span);
    if comment.inline {
        write!(f, text(content.trim_end()));
    } else {
        write!(f, text(content));
    }
}

/// Spacing after a block comment. A `//` comment always hard-breaks instead.
#[derive(Clone, Copy, Debug)]
pub enum BlockCommentAfter {
    None,
    Space,
    HardLine,
}

/// A comment written in place: a `//` ends its line, a block comment is followed by `block_after`.
#[must_use = "formatted comments must be written to the formatter"]
#[derive(Clone, Copy, Debug)]
pub struct FormatCommentBeforeContent {
    comment: CssComment,
    block_after: BlockCommentAfter,
}

impl FormatCommentBeforeContent {
    pub const fn new(comment: CssComment, block_after: BlockCommentAfter) -> Self {
        Self { comment, block_after }
    }
}

impl<'a> Format<'a, CssFormatContext<'a>> for FormatCommentBeforeContent {
    fn fmt(&self, f: &mut CssFormatter<'_, 'a>) {
        write_comment_text(self.comment, f);
        if self.comment.inline {
            write!(f, hard_line_break());
            return;
        }
        match self.block_after {
            BlockCommentAfter::None => {}
            BlockCommentAfter::Space => write!(f, space()),
            BlockCommentAfter::HardLine => write!(f, hard_line_break()),
        }
    }
}

/// A `//` comment deferred through `line_suffix`: cannot swallow later tokens, not measured.
#[must_use = "formatted comments must be written to the formatter"]
#[derive(Clone, Copy, Debug)]
pub struct FormatLineCommentSuffix {
    comment: CssComment,
    leading_space: bool,
    expand_parent: bool,
}

impl FormatLineCommentSuffix {
    pub const fn new(comment: CssComment) -> Self {
        Self { comment, leading_space: false, expand_parent: false }
    }

    pub const fn with_leading_space(mut self) -> Self {
        self.leading_space = true;
        self
    }

    /// A `line_suffix` alone never breaks the enclosing group.
    pub const fn with_expand_parent(mut self) -> Self {
        self.expand_parent = true;
        self
    }
}

impl<'a> Format<'a, CssFormatContext<'a>> for FormatLineCommentSuffix {
    fn fmt(&self, f: &mut CssFormatter<'_, 'a>) {
        debug_assert!(self.comment.inline, "expected a line comment");
        let comment = self.comment;
        let leading_space = self.leading_space;
        let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            write!(f, maybe_space(leading_space));
            write_comment_text(comment, f);
        });
        write!(f, [line_suffix(&content), self.expand_parent.then_some(expand_parent())]);
    }
}

/// Emits the formatter element that reproduces the vertical spacing implied by `gap`.
pub fn write_gap(gap: &[u8], f: &mut CssFormatter<'_, '_>) {
    match classify_gap(gap) {
        Gap::None => write!(f, space()),
        Gap::Line => write!(f, hard_line_break()),
        Gap::Blank => write!(f, empty_line()),
    }
}

/// Emit comments that precede a node. Comments are statement-level nodes in
/// postcss, so each one ends with a line break (a blank line is preserved);
/// same-line gaps still produce a hardline.
pub fn write_leading_comments(
    comments: &[CssComment],
    value_start: u32,
    f: &mut CssFormatter<'_, '_>,
) {
    let source = f.context().source_text();
    for (i, &comment) in comments.iter().enumerate() {
        write_comment_text(comment, f);
        match comments.get(i + 1) {
            // Comment followed by another comment: keep same-line pairs
            // (`*/ /*!`) together.
            Some(next) => write_gap(source.bytes_range(comment.span.end, next.span.start), f),
            // Comment followed by the node: always on its own line (a blank
            // line in the source is preserved, otherwise a single hardline).
            None => {
                if classify_gap(source.bytes_range(comment.span.end, value_start)) == Gap::Blank {
                    write!(f, empty_line());
                } else {
                    write!(f, hard_line_break());
                }
            }
        }
    }
}

/// Drains and emits all pending comments ending at or before `value_start` as leading comments.
pub fn flush_leading_comments(value_start: u32, f: &mut CssFormatter<'_, '_>) {
    let leading = f.context().comments().take_before(value_start);
    write_leading_comments(leading, value_start, f);
}

/// Drains and emits the run of pending comments sitting on the same line
/// as `prev_end` as trailing comments (`red; /* x */ /* y */`); a `//` comment ends the run.
/// Look-alike of `scss::write_same_line_trailing_comments`, which deliberately differs:
/// no `expand_parent` there (its map/config bodies already hard-break).
/// For the run on a list comma's line, see `value::flush_line_comment_after_comma`.
pub fn write_trailing_same_line_comments(
    mut prev_end: u32,
    upper: u32,
    f: &mut CssFormatter<'_, '_>,
) {
    let source = f.context().source_text();
    while let Some(comment) = f.context().comments().peek() {
        if comment.span.end > upper
            || classify_gap(source.bytes_range(prev_end, comment.span.start)) != Gap::None
        {
            return;
        }

        f.context().comments().take_before(comment.span.end);

        // NOTE: Prettier does not distinguish between `// c` and `/* c */` at EOL only for CSS/SCSS/Less.
        // All other formatters treat EOL-line comments as line suffixes, so we are consistent with them.
        if comment.inline {
            write!(
                f,
                FormatLineCommentSuffix::new(comment).with_leading_space().with_expand_parent()
            );
            return;
        }

        write!(f, [space(), FormatCommentBeforeContent::new(comment, BlockCommentAfter::None)]);
        prev_end = comment.span.end;
    }
}

/// Emit comments that sit between the last child of a container and its closing delimiter.
pub fn write_trailing_inside_comments<'a>(
    comments: &[CssComment],
    lower_bound: u32,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    let mut prev_end = lower_bound;
    for &comment in comments {
        let gap_start = prev_end;
        let content = format_with(move |f: &mut CssFormatter<'_, 'a>| {
            let gap = source.bytes_range(gap_start, comment.span.start);
            write_gap(gap, f);
            write_comment_text(comment, f);
        });
        write!(f, [line_suffix(&content), expand_parent()]);
        prev_end = comment.span.end;
    }
}

/// Drains comments before `upper_bound` (typically a closing-delimiter position) and
/// writes them via [`write_trailing_inside_comments`].
pub fn flush_trailing_inside_comments(
    lower_bound: u32,
    upper_bound: u32,
    f: &mut CssFormatter<'_, '_>,
) {
    let trailing = f.context().comments().take_before(upper_bound);
    write_trailing_inside_comments(trailing, lower_bound, f);
}

/// Returns `true` if `comment` is an ignore marker (`/* oxfmt-ignore */` / `/* prettier-ignore */`).
pub fn is_suppression_comment(source: SourceText<'_>, comment: CssComment) -> bool {
    let content = source.text_for(&comment.span);
    let content = content
        .strip_prefix("/*")
        .and_then(|c| c.strip_suffix("*/"))
        .or_else(|| content.strip_prefix("//"))
        .unwrap_or(content);
    is_suppression_marker(content)
}

/// Prettier's `lastLineHasInlineComment`: does the last line of a raw
/// prelude/selector slice carry a `//` comment? When it does, `{` drops to
/// the next line instead of following on the same one.
pub fn last_line_has_inline_comment(raw: &str) -> bool {
    raw.rsplit('\n').next().unwrap_or(raw).contains("//")
}
