use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, SPAN};

use crate::{
    formatter::{FormatResult, Formatter, prelude::*},
    generated::ast_nodes::AstNode,
    options::FormatTrailingCommas,
    write,
};

/// Utility function to print array-like nodes (array expressions, array bindings and assignment patterns)
pub fn write_array_node<'a>(
    node: &AstNode<'a, Vec<'a, ArrayExpressionElement<'a>>>,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    let trailing_separator = FormatTrailingCommas::ES5.trailing_separator(f.options());

    // Specifically do not use format_separated as arrays need separators
    // inserted after holes regardless of the formatting since this makes a
    // semantic difference

    let source_text = f.source_text();
    let mut join = f.join_nodes_with_soft_line();
    let last_index = node.len().saturating_sub(1);

    let mut has_seen_elision = false;
    for (index, element) in node.iter().enumerate() {
        let separator_mode = match element.as_ref() {
            ArrayExpressionElement::Elision(_) => TrailingSeparatorMode::Force,
            _ => TrailingSeparatorMode::Auto,
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
                element.span()
            },
            &format_once(|f| {
                if element.is_elision() {
                    has_seen_elision = true;
                    return write!(f, ",");
                }

                write!(f, group(&element))?;

                if is_disallow {
                    // Trailing separators are disallowed, replace it with an empty element
                    // if let Some(separator) = element.trailing_separator()? {
                    // write!(f, [format_removed(separator)])?;
                    // }
                } else if is_force || index != last_index {
                    // In forced separator mode or if this element is not the last in the list, print the separator
                    // match element.trailing_separator()? {
                    // Some(trailing) => write!(f, [trailing.format()])?,
                    // None => text(",").fmt(f)?,
                    // };
                    ",".fmt(f)?;
                // } else if let Some(separator) = element.trailing_separator()? {
                // match trailing_separator {
                // TrailingSeparator::Omit => {
                // // write!(f, [format_removed(separator)])?;
                // }
                // _ => {
                // write!(f, format_only_if_breaks(SPAN, separator))?;
                // }
                // }
                } else {
                    write!(f, FormatTrailingCommas::ES5)?;
                }

                Ok(())
            }),
        );
    }

    join.finish()
}

/// Determines if a trailing separator should be inserted after an array element
pub enum TrailingSeparatorMode {
    /// Trailing separators are not allowed after this element (eg. rest elements)
    Disallow,
    /// Trailing separators are inserted after this element except if its the
    /// last element and the group is not breaking
    Auto,
    /// Trailing separators will always be inserted after this element (eg. hole elements)
    Force,
}
