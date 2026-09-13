use oxc_ast::Comment;
use oxc_formatter_core::SourceText;
use oxc_span::{GetSpan, SPAN, Span};

use crate::{
    formatter::{prelude::*, trivia::FormatCommentBeforeContent},
    options::FormatTrailingCommas,
    write,
};

/// Utility function to print array-like nodes (array expressions, array bindings and assignment patterns)
///
/// `span` is the bracketed node's span, used to locate the `,` of a leading hole.
pub fn write_array_node<'a, 'b, N>(
    span: Span,
    len: usize,
    array: impl IntoIterator<Item = Option<&'a N>> + 'b,
    f: &mut JsFormatter<'_, 'a>,
) where
    N: Format<'a, JsFormatContext<'a>> + GetSpan + std::fmt::Debug + 'a,
{
    // Specifically do not use format_separated as arrays need separators
    // inserted after holes regardless of the formatting since this makes a
    // semantic difference

    let last_index = len - 1;
    let source_text = f.context().source_text();
    let mut join = f.join_nodes_with_soft_line();
    let mut has_seen_elision = false;

    // Position after the previous slot (the `[`, an element's separator `,`, or a hole's `,`),
    // maintained so each hole's own `,` can be located in the source.
    // Only holes read it, so it is refreshed lazily: when a hole is printed,
    // and when an element knows a hole follows it.
    let mut cursor = span.start;

    let mut array_iter = array.into_iter().enumerate().peekable();

    while let Some((index, element)) = array_iter.next() {
        let separator_mode = if element.is_none() {
            TrailingSeparatorMode::Force
        } else {
            TrailingSeparatorMode::Auto
        };

        let is_disallow = matches!(separator_mode, TrailingSeparatorMode::Disallow);
        let is_force = matches!(separator_mode, TrailingSeparatorMode::Force);
        let next_is_hole = array_iter.peek().is_some_and(|(_, next)| next.is_none());

        // A hole owns the comments in its slot (between the previous `,` and its own `,`):
        // they print as its leading comments, keeping their side of both commas
        // (`[a, /* c */, b]` stays `[a, /* c */, b]`).
        let hole_comma = if element.is_none() {
            let comma = next_comma_position(
                source_text,
                join.fmt().comments().unprinted_comments(),
                cursor,
            );
            if let Some(comma) = comma {
                cursor = comma + 1;
            }
            comma
        } else {
            None
        };

        // A following hole's slot begins at the first same-line block comment behind this
        // element's separator `,`; hide the slot from this element's trailing pass so its
        // comments are not claimed backward across the separator.
        // A same-line line comment stays trailing: it rides the separator on a `line_suffix`
        // and never leaves its line (`[a, // c` stays `a, // c`).
        let trailing_limit = if let Some(element) = element
            && next_is_hole
        {
            let comments = join.fmt().comments().unprinted_comments();
            next_comma_position(source_text, comments, element.span().end).and_then(|sep_comma| {
                cursor = sep_comma + 1;
                let hole_comma = next_comma_position(source_text, comments, cursor)?;
                comments
                    .iter()
                    .filter(|c| c.span.start > sep_comma && c.span.end <= hole_comma)
                    .find(|c| c.is_block() && !c.preceded_by_newline())
                    .map(|c| c.span.start)
            })
        } else {
            None
        };

        join.entry(
            // Note(different-with-Biome): this implementation isn't the same as Biome, because its output doesn't exactly match Prettier.
            if has_seen_elision {
                // Use fake span to avoid add any empty line between elision and expression element.
                SPAN
            } else {
                element.map_or_else(
                    || hole_comma.map_or(SPAN, |comma| Span::sized(comma, 1)),
                    GetSpan::span,
                )
            },
            &format_once(|f| {
                if let Some(element) = element {
                    if let Some(limit) = trailing_limit {
                        let saved = f.context_mut().comments_mut().limit_comments_up_to(limit);
                        write!(f, group(&element));
                        f.context_mut().comments_mut().restore_view_limit(saved);
                    } else {
                        write!(f, group(&element));
                    }

                    if is_disallow {
                    } else if is_force || index != last_index {
                        ",".fmt(f);
                    } else {
                        write!(f, FormatTrailingCommas::ES5);
                    }
                } else {
                    has_seen_elision = true;
                    if let Some(comma) = hole_comma {
                        write_hole_leading_comments(comma, f);
                    }
                    write!(f, ",");
                }
            }),
        );
    }
}

/// Prints the comments sitting in a hole's slot, right before the hole's `,`.
fn write_hole_leading_comments(hole_comma: u32, f: &mut JsFormatter<'_, '_>) {
    let comments = f.context().comments().comments_before(hole_comma);
    let mut comments_iter = comments.iter().peekable();
    while let Some(comment) = comments_iter.next() {
        // Marks the comment printed; a line comment ends its line, so the `,` breaks below it.
        FormatCommentBeforeContent::new(comment).fmt(f);
        if comment.is_block() {
            if f.source_text().has_line_terminator_after(comment.span.end) {
                write!(f, hard_line_break());
            } else if comments_iter.peek().is_some() {
                write!(f, space());
            }
        }
    }
}

/// Position of the first `,` at or after `pos` that is not inside a comment.
fn next_comma_position(
    source_text: SourceText<'_>,
    comments: &[Comment],
    mut pos: u32,
) -> Option<u32> {
    #[expect(clippy::cast_possible_truncation)]
    let find = |start: u32, bytes: &[u8]| {
        bytes.iter().position(|&b| b == b',').map(|offset| start + offset as u32)
    };
    for comment in comments {
        if comment.span.end <= pos {
            continue;
        }
        if let Some(comma) = find(pos, source_text.bytes_range(pos, comment.span.start)) {
            return Some(comma);
        }
        pos = comment.span.end;
    }
    let text: &str = &source_text;
    find(pos, &text.as_bytes()[pos as usize..])
}

/// Determines if a trailing separator should be inserted after an array element
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TrailingSeparatorMode {
    /// Trailing separators are not allowed after this element (eg. rest elements)
    #[expect(unused)]
    Disallow,
    /// Trailing separators are inserted after this element except if its the
    /// last element and the group is not breaking
    Auto,
    /// Trailing separators will always be inserted after this element (eg. hole elements)
    Force,
}
