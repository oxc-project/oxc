use oxc_span::{GetSpan, SPAN, Span};

use crate::{
    formatter::{Formatter, prelude::*},
    options::FormatTrailingCommas,
    write,
};

/// Utility function to print array-like nodes (array expressions, array bindings and assignment patterns)
pub fn write_array_node<'a, 'b, N>(
    len: usize,
    array: impl IntoIterator<Item = Option<&'a N>> + 'b,
    f: &mut Formatter<'_, 'a>,
) where
    N: Format<'a> + GetSpan + std::fmt::Debug + 'a,
{
    // Specifically do not use format_separated as arrays need separators
    // inserted after holes regardless of the formatting since this makes a
    // semantic difference

    let last_index = len - 1;
    let source_text = f.context().source_text();
    let mut join = f.join_nodes_with_soft_line();
    let mut has_seen_elision = false;

    let mut array_iter = array.into_iter().enumerate().peekable();

    while let Some((index, element)) = array_iter.next() {
        let separator_mode = if element.is_none() {
            TrailingSeparatorMode::Force
        } else {
            TrailingSeparatorMode::Auto
        };

        let is_disallow = matches!(separator_mode, TrailingSeparatorMode::Disallow);
        let is_force = matches!(separator_mode, TrailingSeparatorMode::Force);

        join.entry(
            // Note(different-with-Biome): this implementation isn't the same as Biome, because its output doesn't exactly match Prettier.
            if has_seen_elision {
                // Use fake span to avoid add any empty line between elision and expression element.
                SPAN
            } else {
                has_seen_elision = false;
                element.map_or_else(
                    || {
                        // TODO: improve the `ArrayPattern` AST to simplify this logic.
                        // Since `ArrayPattern` doesn't have a elision node, so that we have to find the comma position
                        // by looking through the source text.
                        let next_span =
                            array_iter.peek().map_or(SPAN, |e| e.1.map_or(SPAN, GetSpan::span));

                        let comma_position =
                            source_text.bytes_to(next_span.start).position(|c| c == b',');

                        // comma span
                        #[expect(clippy::cast_possible_truncation)]
                        comma_position.map_or(SPAN, |pos| {
                            Span::new(
                                next_span.start - pos as u32,
                                next_span.start - pos as u32 + 1,
                            )
                        })
                    },
                    GetSpan::span,
                )
            },
            &format_once(|f| {
                if let Some(element) = element {
                    write!(f, group(&element));

                    if is_disallow {
                    } else if is_force || index != last_index {
                        ",".fmt(f);
                    } else {
                        write!(f, FormatTrailingCommas::ES5);
                    }
                } else {
                    has_seen_elision = true;
                    write!(f, ",");
                }
            }),
        );
    }
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
