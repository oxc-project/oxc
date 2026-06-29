use std::cell::Cell;

use oxc_formatter_core::{
    Buffer, SourceText,
    builders::{empty_line, expand_parent, hard_line_break, line_suffix, space, text},
    spec::is_suppression_marker,
    write,
};
use oxc_span::Span;

use crate::print::{CssFormatter, format_with};

/// A source comment.
///
/// raffia keeps comments out of the AST; `format()` collects them through
/// `ParserBuilder::comments()` and stores their spans here.
#[derive(Clone, Copy, Debug)]
pub struct CssComment {
    pub span: Span,
    /// `// ...` line comment (SCSS/Less only). Block comments are `/* ... */`.
    pub inline: bool,
}

/// Cursor over a sorted comment list that hands out unprinted comments in span order.
///
/// `cursor` is a [`Cell`] so the API works through `&self`
/// (mirrors `oxc_formatter_graphql`'s `Comments`).
pub struct Comments<'a> {
    inner: &'a [CssComment],
    cursor: Cell<usize>,
}

impl<'a> Comments<'a> {
    pub fn new(comments: &'a [CssComment]) -> Self {
        Self { inner: comments, cursor: Cell::new(0) }
    }

    /// Returns the next unprinted comment without consuming it.
    pub fn peek(&self) -> Option<CssComment> {
        self.inner.get(self.cursor.get()).copied()
    }

    /// Returns unprinted comments whose `span.end <= upper_bound`,
    /// and advances the cursor past them so they won't be returned again.
    pub fn take_before(&self, upper_bound: u32) -> &'a [CssComment] {
        let start = self.cursor.get();
        let mut end = start;
        while end < self.inner.len() && self.inner[end].span.end <= upper_bound {
            end += 1;
        }
        self.cursor.set(end);
        &self.inner[start..end]
    }

    /// Drains all remaining unprinted comments and returns them.
    pub fn take_remaining(&self) -> &'a [CssComment] {
        let start = self.cursor.get();
        self.cursor.set(self.inner.len());
        &self.inner[start..]
    }

    /// Iterator over unprinted comments whose `span.end <= upper_bound`.
    /// Does NOT advance the cursor.
    pub fn iter_before(&self, upper_bound: u32) -> impl Iterator<Item = CssComment> {
        let start = self.cursor.get();
        self.inner[start..].iter().copied().take_while(move |c| c.span.end <= upper_bound)
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
/// Recognizes `\n`, lone `\r`, and `\r\n` line terminators.
pub fn classify_gap(slice: &[u8]) -> Gap {
    let mut newline_count = 0;
    let mut line_has_content = false;
    let mut blank = false;
    let mut i = 0;
    while i < slice.len() {
        match slice[i] {
            b'\r' | b'\n' => {
                if newline_count > 0 && !line_has_content {
                    blank = true;
                }
                newline_count += 1;
                line_has_content = false;
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
fn write_gap(gap: &[u8], f: &mut CssFormatter<'_, '_>) {
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
            Some(next) => {
                match classify_gap(source.bytes_range(comment.span.end, next.span.start)) {
                    Gap::None => write!(f, space()),
                    Gap::Line => write!(f, hard_line_break()),
                    Gap::Blank => write!(f, empty_line()),
                }
            }
            // Comment followed by the node: always on its own line.
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

/// If the next pending comment sits on the same line as `prev_end`,
/// drain it and emit it as a trailing comment.
pub fn write_trailing_same_line_comment(prev_end: u32, upper: u32, f: &mut CssFormatter<'_, '_>) {
    let Some(comment) = f.context().comments().peek() else { return };
    if comment.span.end > upper {
        return;
    }
    let source = f.context().source_text();
    if classify_gap(source.bytes_range(prev_end, comment.span.start)) != Gap::None {
        return;
    }
    f.context().comments().take_before(comment.span.end);
    // Plain content, NOT a line suffix: Prettier prints trailing comments as
    // regular doc parts, so they count towards the line width during the
    // preceding value group's fits measurement (`x: min(...); /* long */`
    // breaks the `min(` group even when the value alone would fit).
    write!(f, space());
    write_single_comment(comment, f);
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

#[cfg(test)]
mod tests {
    use super::{Gap, classify_gap};

    #[test]
    fn classify_gap_counts_line_terminators() {
        assert_eq!(classify_gap(b" \t "), Gap::None);
        assert_eq!(classify_gap(b"\n"), Gap::Line);
        assert_eq!(classify_gap(b"\n  \n"), Gap::Blank);
        assert_eq!(classify_gap(b"\r\n"), Gap::Line);
        assert_eq!(classify_gap(b"\r\n\r\n"), Gap::Blank);
        assert_eq!(classify_gap(b"\r"), Gap::Line);
        assert_eq!(classify_gap(b"\r\r"), Gap::Blank);
    }
}
