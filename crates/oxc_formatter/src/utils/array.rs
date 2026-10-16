use oxc_span::{GetSpan, SPAN, Span};

use crate::{
    ast_nodes::AstNode,
    format_args,
    formatter::{
        prelude::*,
        trivia::{FormatCommentBeforeContent, format_trailing_comments},
    },
    options::FormatTrailingCommas,
    utils::{
        format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
        statement_body::write_trailing_comments_before,
    },
    write,
};

/// Utility function to print array-like nodes (array expressions, array bindings and assignment patterns)
///
/// `rest_start` is the start of the rest element (`[a, ...rest]`) printed by the caller after this.
///
/// A hole's `,` is content (it keeps the hole count), so comments around it stay on their source side:
/// the hole prints its leading comments, its `,`, then its same-line trailing comments.
/// Comments after the last entry print before the `]` ([`write_unclaimed_comments`]), or lead the rest element.
pub fn write_array_node<'a, T>(
    array_span: Span,
    rest_start: Option<u32>,
    array: impl IntoIterator<Item = Option<&'a AstNode<'a, T>>>,
    f: &mut JsFormatter<'_, 'a>,
) where
    T: 'a,
    AstNode<'a, T>: Format<'a, JsFormatContext<'a>> + GetSpan,
{
    // The elements: from the `[` up to the `]` (exclusive) or the rest element
    let range = Span::new(array_span.start, rest_start.unwrap_or(array_span.end - 1));
    let has_rest = rest_start.is_some();

    // Specifically do not use `format_separated` as arrays need separators inserted after holes
    // regardless of the formatting since this makes a semantic difference.

    let mut join = f.join_nodes_with_soft_line();
    let mut has_seen_elision = false;
    // Holes come as `None` without a span (patterns have no `Elision`),
    // so each hole's `,` is found in the source, once, by the entry before it (or at the `[` for a leading hole).
    //
    // TODO: Once `ArrayPattern` / `ArrayAssignmentTarget` hold an `Elision` (a span-only hole) like `ArrayExpression`,
    // pass the holes' spans here instead of this scan, and format the element before a hole as is
    // (the iterator would see the hole as the following sibling).
    let comma_after =
        |f: &JsFormatter<'_, 'a>, pos: u32| f.comments().position_after_character(pos, b',') - 1;
    let mut hole_comma = None;

    let mut array_iter = array.into_iter().peekable();

    while let Some(element) = array_iter.next() {
        let next = array_iter.peek().copied();

        let span = if let Some(element) = element {
            element.span()
        } else {
            Span::sized(
                hole_comma.take().unwrap_or_else(|| comma_after(join.fmt(), range.start)),
                1,
            )
        };

        // Use fake span to avoid add any empty line between elision and expression element
        let entry_span = if has_seen_elision { SPAN } else { span };

        // The `,` of the next hole, past this entry's own `,`
        if matches!(next, Some(None)) {
            let own_comma =
                if element.is_some() { comma_after(join.fmt(), span.end) } else { span.start };
            hole_comma = Some(comma_after(join.fmt(), own_comma + 1));
        }

        if let Some(element) = element {
            let is_last = next.is_none() && !has_rest;
            join.entry(
                entry_span,
                &format_once(|f| {
                    if let Some(comma) = hole_comma {
                        // The iterator skips holes, so bound the trailing pass at the next hole's `,`
                        let trailing =
                            format_trailing_comments(element.parent().span(), span, comma);
                        write!(
                            f,
                            group(&format_args!(
                                FormatNodeWithoutTrailingComments(element),
                                trailing
                            ))
                        );
                    } else {
                        write!(f, group(&element));
                    }
                    if is_last {
                        write!(f, FormatTrailingCommas::ES5);
                    } else {
                        ",".fmt(f);
                    }
                }),
            );
        } else {
            // `None` after the last hole: see the unclaimed comments below
            let following = match next {
                Some(Some(next)) => Some(next.span().start),
                _ => hole_comma,
            };
            join.entry(
                entry_span,
                &format_once(|f| {
                    write_hole_slot_comments(span.start, f);
                    write!(f, ",");
                    if let Some(following) = following {
                        write!(f, format_trailing_comments(range, span, following));
                    }
                }),
            );
            has_seen_elision = true;
        }
    }

    if !has_rest {
        write_unclaimed_comments(range.end, join.fmt_mut());
    }
}

/// Comments in a hole's slot, before its `,` at `comma`.
/// A hole has no content, so the `,` follows a block comment directly (`[a, /* c */, b]`),
/// as a separator follows any content (`[a /* c */, b]`); an own-line comment keeps its line.
///
/// Not [`write_trailing_comments_before`] like an empty for-head slot:
/// it would space a comment away from the `[` (`[ /* c */, b]`),
/// and let a line comment right after the `[` ride past the hole's `,` (`[// c` + break + `, b]` -> `, // c`).
fn write_hole_slot_comments(comma: u32, f: &mut JsFormatter<'_, '_>) {
    let comments = f.context().comments().comments_before(comma);
    for (index, comment) in comments.iter().enumerate() {
        // A line comment ends with a hard break
        write!(f, FormatCommentBeforeContent::new(comment));
        if comment.is_block() {
            if comment.followed_by_newline() {
                write!(f, hard_line_break());
            } else if index + 1 < comments.len() {
                write!(f, space());
            }
        }
    }
}

/// Comments that no element consumed (e.g. after a trailing hole: `[,, /* comment */]`)
/// would otherwise escape the brackets; print them before the `]` at `end`,
/// trailing the last `,` like after any separator (`[, , /* comment */]`).
pub fn write_unclaimed_comments(end: u32, f: &mut JsFormatter<'_, '_>) {
    let Some(first) = f.context().comments().comments_before(end).first() else { return };
    // Discarded flush obligation:
    // a riding line comment's `expand_parent` breaks the array group,
    // so a real break already follows before the `]`
    let _line_comment_riding = write_trailing_comments_before(first.span.start, b']', f);
}
