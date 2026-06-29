use oxc_formatter_core::{
    Buffer, Format, Formatter,
    builders::{FormatWith, empty_line, hard_line_break},
    write,
};
use raffia::ast::Stylesheet;

use crate::{
    comments::{
        Gap, classify_gap, flush_leading_comments, write_leading_comments,
        write_trailing_same_line_comment,
    },
    context::CssFormatContext,
};

pub mod at_rule;
pub mod less;
pub mod scss;
pub mod selector;
pub mod statement;
pub mod value;

pub type CssFormatter<'buf, 'a> = Formatter<'buf, 'a, CssFormatContext<'a>>;

/// `Format` impl for `&'static str` specialized to `CssFormatContext`.
///
/// Hardcoded to `CssFormatContext` rather than generic over `C` so the blanket
/// `&T where T: Format` doesn't overlap.
impl<'a> Format<'a, CssFormatContext<'a>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut CssFormatter<'_, 'a>) {
        write!(f, oxc_formatter_core::builders::token(self));
    }
}

/// Wraps a re-entrant CSS closure in a [`FormatWith`]. The closure's context is
/// pinned to [`CssFormatContext`] so call sites don't have to annotate it.
#[inline]
pub const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut CssFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

/// Emits the whole stylesheet: top-level statements separated by hard lines
/// (blank lines preserved, max one), then any trailing comments.
///
/// Mirrors Prettier's `css-root` + `printSequence`.
pub fn write_stylesheet<'a>(stylesheet: &Stylesheet<'a>, f: &mut CssFormatter<'_, 'a>) {
    write_statement_sequence(&stylesheet.statements, f);

    // Comments after the last statement (or in an otherwise empty file).
    let remaining = f.context().comments().take_remaining();
    if !remaining.is_empty() {
        if !stylesheet.statements.is_empty() {
            let source = f.context().source_text();
            let prev_end = stylesheet.statements.last().map_or(0, |s| statement::stmt_end(s, f));
            match classify_gap(source.bytes_range(prev_end, remaining[0].span.start)) {
                Gap::None => write!(f, oxc_formatter_core::builders::space()),
                Gap::Line => write!(f, hard_line_break()),
                Gap::Blank => write!(f, empty_line()),
            }
        }
        let last_end = remaining.last().unwrap().span.end;
        write_leading_comments(remaining, last_end, f);
    }
}

/// Emits `statements` separated by hard lines, preserving at most one blank line
/// between consecutive statements, flushing comments at their source positions.
///
/// Mirrors Prettier's `printSequence`.
pub fn write_statement_sequence<'a>(
    statements: &[raffia::ast::Statement<'a>],
    f: &mut CssFormatter<'_, 'a>,
) {
    write_statement_sequence_bounded(statements, u32::MAX, f);
}

/// Like [`write_statement_sequence`], but trailing same-line comments are
/// only claimed when they end before `upper` (a block's closing `}`),
/// so inline rules don't steal comments that belong to the parent.
pub fn write_statement_sequence_bounded<'a>(
    statements: &[raffia::ast::Statement<'a>],
    upper: u32,
    f: &mut CssFormatter<'_, 'a>,
) {
    let source = f.context().source_text();
    for (i, stmt) in statements.iter().enumerate() {
        let start = statement::stmt_start(stmt);
        if i > 0 {
            let prev_end = statement::stmt_end(&statements[i - 1], f);
            // Trailing comment on the same line as the previous statement
            // (but not one that sits after the NEXT statement on that line).
            write_trailing_same_line_comment(prev_end, upper.min(start), f);
            write!(f, hard_line_break());
            // Preserve a single blank line. The gap considered is from the end of
            // the previous statement to the next printed position (comment or stmt).
            let next_start =
                f.context().comments().peek().map_or(start, |c| c.span.start.min(start));
            if classify_gap(source.bytes_range(prev_end, next_start)) == Gap::Blank {
                write!(f, empty_line());
            }
        }
        // `prettier-ignore` / `oxfmt-ignore`: print the statement verbatim.
        let suppressed = f
            .context()
            .comments()
            .iter_before(start)
            .last()
            .is_some_and(|c| crate::comments::is_suppression_comment(source, c));
        flush_leading_comments(start, f);
        if suppressed {
            let end = statement::stmt_end(stmt, f);
            write!(f, oxc_formatter_core::builders::text(source.slice_range(start, end)));
        } else {
            statement::write_statement(stmt, f);
        }
        // Discard comments inside spans the statement printer didn't claim
        // (e.g. inside selectors/values that are still printed verbatim),
        // so the cursor never points before an already-printed position.
        let _ = f.context().comments().take_before(statement::stmt_end(stmt, f));
    }
    if let Some(last) = statements.last() {
        write_trailing_same_line_comment(statement::stmt_end(last, f), upper, f);
    }
}
