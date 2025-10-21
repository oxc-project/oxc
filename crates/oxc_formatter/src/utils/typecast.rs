use oxc_ast::{
    Comment,
    ast::{ArrowFunctionExpression, Function},
};
use oxc_span::GetSpan;

use crate::{
    Buffer, Format, FormatResult,
    ast_nodes::AstNode,
    format_args,
    formatter::{Formatter, SourceText, prelude::*, trivia::FormatLeadingComments},
    write,
    write::{
        FormatFunctionOptions, FormatJsArrowFunctionExpression,
        FormatJsArrowFunctionExpressionOptions,
    },
};

/// Checks if a node is a type cast node and returns the comments to be printed.
///
/// This function detects if a node is part of a TypeScript type cast pattern
/// by checking for JSDoc type cast comments and proper parenthesis structure.
///
/// Returns:
/// - `Some(&[])` if the node is a type cast node but no comments need to be printed
/// - `Some(&[Comment, ...])` if the node is a type cast node with comments to print
/// - `None` if the node is not a type cast node
pub fn is_type_cast_node<'a>(node: &impl GetSpan, f: &Formatter<'_, 'a>) -> Option<&'a [Comment]> {
    let comments = f.context().comments();
    let span = node.span();
    let source = f.source_text();

    // Check if there's a closing parenthesis after the node (possibly after comments)
    if !source.next_non_whitespace_byte_is(span.end, b')') {
        let comments_after_node = comments.comments_after(span.end);
        let mut start = span.end;
        // Skip comments after the node to find the next non-whitespace byte whether it's a `)`
        for comment in comments_after_node {
            if !source.bytes_range(start, comment.span.start).trim_ascii_start().is_empty() {
                break;
            }
            start = comment.span.end;
        }
        // Still not a `)`, return early because it's not a type cast
        if !source.next_non_whitespace_byte_is(start, b')') {
            return None;
        }
    }

    // Check for type cast comment in printed or unprinted comments
    if !comments.is_handled_type_cast_comment()
        && let Some(last_printed_comment) = comments.printed_comments().last()
        && last_printed_comment.span.end <= span.start
        && source.next_non_whitespace_byte_is(last_printed_comment.span.end, b'(')
        && f.comments().is_type_cast_comment(last_printed_comment)
    {
        // Get the source text from the end of type cast comment to the node span
        let node_source_text = source.bytes_range(last_printed_comment.span.end, span.end);

        // `(/** @type {Number} */ (bar).zoo)`
        //                         ^^^^
        // Should wrap for `baz` rather than `baz.zoo`
        if has_closed_parentheses(node_source_text) {
            None
        } else {
            // Type cast node, but comment was already printed
            Some(&[])
        }
    } else if let Some(type_cast_comment_index) = comments.get_type_cast_comment_index(span) {
        let comments = f.context().comments().unprinted_comments();
        let type_cast_comment = &comments[type_cast_comment_index];

        // Get the source text from the end of type cast comment to the node span
        let node_source_text = source.bytes_range(type_cast_comment.span.end, span.end);

        // `(/** @type {Number} */ (bar).zoo)`
        //                         ^^^^
        // Should wrap for `baz` rather than `baz.zoo`
        if has_closed_parentheses(node_source_text) {
            None
        } else {
            // Type cast node with comments to print
            Some(&comments[..=type_cast_comment_index])
        }
    } else {
        // No typecast comment
        None
    }
}

/// Formats a node with TypeScript type cast comments if present.
///
/// This function handles the formatting of JSDoc type cast comments that appear
/// immediately before parenthesized expressions, creating patterns like:
/// `(/** @type {string} */ value)` or `(/** @type {number} */ (expression))`
///
/// The function:
/// 1. Checks if there's a closing parenthesis after the node (indicating a type cast)
/// 2. Looks for associated type cast comments that precede the node
/// 3. Wraps the node in parentheses with proper formatting and indentation
/// 4. Handles both object/array expressions and other expression types differently
///
/// Returns `Ok(true)` if the node was formatted as a type cast, `Ok(false)` otherwise.
/// This allows callers to know whether they need to apply their own formatting.
pub fn format_type_cast_comment_node<'a, T>(
    node: &(impl Format<'a, T> + GetSpan),
    is_object_or_array_expression: bool,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<bool> {
    // Check if this is a type cast node and get the comments to print
    let Some(type_cast_comments) = is_type_cast_node(node, f) else {
        return Ok(false);
    };

    // Print the type cast comments if any
    if !type_cast_comments.is_empty() {
        write!(f, [FormatLeadingComments::Comments(type_cast_comments)])?;
    }

    let span = node.span();
    f.context_mut().comments_mut().mark_as_type_cast_node(node);

    // https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/print/estree.js#L117-L120
    if is_object_or_array_expression && !f.comments().has_comment_before(span.start) {
        write!(f, group(&format_args!("(", &format_once(|f| node.fmt(f)), ")")))?;
    } else {
        write!(
            f,
            group(&format_args!("(", soft_block_indent(&format_once(|f| node.fmt(f))), ")"))
        )?;
    }

    Ok(true)
}

/// Check if the source text has properly closed parentheses starting with '('.
/// Returns true if the text starts with '(' and all parentheses are balanced.
fn has_closed_parentheses(source: &[u8]) -> bool {
    let mut paren_count = 0i32;
    let mut i = 0;

    while i < source.len() {
        match source[i] {
            b'(' => paren_count += 1,
            b')' => {
                paren_count -= 1;
                if paren_count == 0 {
                    return true;
                }
            }
            b'/' if i + 1 < source.len() => {
                match source[i + 1] {
                    b'/' => {
                        // Skip to end of line comment
                        i += 2;
                        while i < source.len() && source[i] != b'\n' {
                            i += 1;
                        }
                        continue;
                    }
                    b'*' => {
                        // Skip to end of block comment
                        i += 2;
                        while i + 1 < source.len() {
                            if source[i] == b'*' && source[i + 1] == b'/' {
                                i += 2;
                                break;
                            }
                            i += 1;
                        }
                        continue;
                    }
                    _ => {}
                }
            }
            quote @ (b'"' | b'\'' | b'`') => {
                // Skip string literal (double-quoted, single-quoted, or template)
                i += 1;
                while i < source.len() {
                    match source[i] {
                        b if b == quote => break,
                        b'\\' if i + 1 < source.len() => i += 1, // Skip escaped character
                        _ => {}
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Return true only if parentheses are properly balanced
    paren_count == 0
}
