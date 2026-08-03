use oxc_formatter_core::{
    Buffer, SourceText, SpanCursor,
    builders::{empty_line, expand_parent, hard_line_break, line_suffix, space, text},
    spec::is_suppression_marker,
    write,
};
use oxc_span::{GetSpan, Span};

use crate::print::{CssFormatter, format_with};

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

/// Emit a single comment verbatim.
/// Mirrors Prettier's `css-comment` case: the original text slice,
/// with trailing whitespace trimmed for inline (`//`) comments.
pub fn write_single_comment(comment: CssComment, f: &mut CssFormatter<'_, '_>) {
    let content = f.context().source_text().text_for(&comment.span);
    if comment.inline {
        write!(f, text(content.trim_end()));
    } else {
        write!(f, text(content));
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
        write_single_comment(comment, f);
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
        let content = format_with(move |f: &mut CssFormatter<'_, '_>| {
            write!(f, space());
            write_single_comment(comment, f);
        });

        // NOTE: Prettier does not distinguish between `// c` and `/* c */` at EOL only for CSS/SCSS/Less.
        // All other formatters treat EOL-line comments as line suffixes, so we are consistent with them.
        if comment.inline {
            write!(f, [line_suffix(&content), expand_parent()]);
            return;
        }

        write!(f, [content]);
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
            write_single_comment(comment, f);
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
