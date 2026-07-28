use oxc_formatter_core::{
    Buffer, Format, Formatter,
    builders::{FormatWith, empty_line, hard_line_break, if_group_fits_on_line, soft_line_break},
    write,
};
use oxc_graphql_parser::ast::Document;

use crate::{
    comments::{
        Gap, classify_gap, flush_leading_comments, flush_overlooked_inside_comments,
        flush_trailing_comment_before, flush_trailing_comment_before_break,
        flush_trailing_inside_comments, is_suppressed_before, write_dangling_comments,
        write_suppressed_node, write_trailing_same_line_comment,
    },
    context::GraphqlFormatContext,
};

mod common;
mod definition;
mod selection;
mod span;
mod string;
mod value;

use span::Spanned;
pub use span::to_span;

pub type GraphqlFormatter<'buf, 'a> = Formatter<'buf, 'a, GraphqlFormatContext<'a>>;

/// `Format` impl for `&'static str` specialized to `GraphqlFormatContext`.
///
/// Hardcoded to `GraphqlFormatContext` rather than generic over `C` so the blanket
/// `&T where T: Format` doesn't overlap.
impl<'a> Format<'a, GraphqlFormatContext<'a>> for &'static str {
    #[inline]
    fn fmt(&self, f: &mut GraphqlFormatter<'_, 'a>) {
        write!(f, oxc_formatter_core::builders::token(self));
    }
}

/// Wraps a re-entrant GraphQL closure in a [`FormatWith`]. The closure's context is
/// pinned to [`GraphqlFormatContext`] so call sites don't have to annotate it.
#[inline]
pub const fn format_with<'a, T>(formatter: T) -> FormatWith<T>
where
    T: Fn(&mut GraphqlFormatter<'_, 'a>),
{
    FormatWith::new(formatter)
}

/// [`flush_trailing_comment_before`] bounded at the next significant source token after `prev_end`:
/// the claim boundary for comments in front of formatter literals the AST carries no node for
/// (`:`, `repeatable`, the schema `{`, etc..., same as `oxc_formatter`), where punctuation positions are resolved by source scans.
/// Bounding at the literal keeps a comment that trails the literal itself (`g: # c`) from being pulled backwards across it.
///
/// The scan only runs with a comment pending; comment-free documents pay one peek.
fn flush_trailing_before_literal(prev_end: u32, f: &mut GraphqlFormatter<'_, '_>) {
    if f.context().comments().peek().is_none() {
        return;
    }
    let upper = span::next_token_start_after(&f.context().source_text(), prev_end);
    flush_trailing_comment_before(upper, f);
}

/// [`flush_trailing_before_literal`]'s sibling for callers whose next element already begins
/// with a line break (expanded separators): claims without the boundary, see [`flush_trailing_comment_before_break`].
fn flush_trailing_before_literal_break(prev_end: u32, f: &mut GraphqlFormatter<'_, '_>) {
    if f.context().comments().peek().is_none() {
        return;
    }
    let upper = span::next_token_start_after(&f.context().source_text(), prev_end);
    flush_trailing_comment_before_break(upper, f);
}

/// How consecutive sequence items are separated.
#[derive(Clone, Copy, Eq, PartialEq)]
enum SeparatorKind {
    /// One item per line (definitions, selections, fields, enum values, ...).
    Hard,
    /// Prettier's `join([ifBreak("", ", "), softline], items)`
    /// (arguments, variable definitions, list values, object fields).
    CommaSoftline,
}

/// Core sequence engine: emits `items` with separators, source blank-line preservation
/// (Prettier's `printSequence`), leading-comment flushing, trailing same-line comments,
/// and `# prettier-ignore` suppression.
///
/// Returns the significant end of the last item (`None` for an empty sequence),
/// so callers can seed `flush_trailing_inside_comments` without re-scanning.
fn write_sequence<'a, T, F>(
    f: &mut GraphqlFormatter<'_, 'a>,
    items: &[T],
    separator: SeparatorKind,
    preserve_blank: bool,
    mut write_item: F,
) -> Option<u32>
where
    T: Spanned,
    F: FnMut(usize, &mut GraphqlFormatter<'_, 'a>),
{
    let mut prev_end: Option<u32> = None;
    for (i, item) in items.iter().enumerate() {
        let item_span = item.span();
        let start = item_span.start;
        if let Some(pe) = prev_end {
            write_trailing_same_line_comment(pe, start, f);
            // Measure the gap up to the next pending comment (if it precedes the item),
            // so a blank line in front of a leading comment is still preserved.
            let anchor = f
                .context()
                .comments()
                .peek()
                .filter(|c| c.start < start)
                .map_or(start, |c| c.start);
            let is_blank = preserve_blank
                && classify_gap(f.context().source_text().bytes_range(pe, anchor)) == Gap::Blank;
            match separator {
                SeparatorKind::Hard => {
                    if is_blank {
                        write!(f, empty_line());
                    } else {
                        write!(f, hard_line_break());
                    }
                }
                SeparatorKind::CommaSoftline => {
                    // Mirrors Prettier's `[printed, hardline]` + `[ifBreak("", ", "), softline]`.
                    // The hard line forces the group to break, so the `ifBreak` comma never
                    // materializes; an `empty_line` reproduces the blank line directly.
                    if is_blank {
                        write!(f, empty_line());
                    } else {
                        write!(f, [if_group_fits_on_line(&", "), soft_line_break()]);
                    }
                }
            }
        }
        let end = item_span.end;
        if is_suppressed_before(f, start) {
            write_suppressed_node(item_span, f);
        } else {
            flush_leading_comments(start, f);
            write_item(i, f);
            // Drain positions no inner printer claims (e.g. between a type and its `!`)
            flush_overlooked_inside_comments(end, f);
        }
        prev_end = Some(end);
    }
    prev_end
}

/// Top-level document: definitions joined by hard lines (blank lines preserved),
/// then any comments trailing the last definition.
pub fn write_document<'a>(document: &'a Document<'a>, f: &mut GraphqlFormatter<'_, 'a>) {
    let defs = document.definitions.as_slice();
    if defs.is_empty() {
        // Defensive: oxc-graphql-parser errors on empty/comments-only documents,
        // so this branch is unreachable on the normal path.
        let remaining = f.context().comments().take_remaining();
        write_dangling_comments(remaining, f);
        return;
    }

    let last_end = write_sequence(f, defs, SeparatorKind::Hard, true, |i, f| {
        definition::write_definition(&defs[i], f);
    });
    if let Some(last_end) = last_end {
        flush_trailing_inside_comments(last_end, u32::MAX, f);
    }
}
