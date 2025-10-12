use oxc_ast::{
    Comment,
    ast::{ArrowFunctionExpression, Function},
};
use oxc_span::GetSpan;

use crate::{
    Buffer, Format, FormatResult, format_args,
    formatter::{Formatter, SourceText, prelude::*, trivia::FormatLeadingComments},
    generated::ast_nodes::AstNode,
    write,
    write::{
        FormatFunctionOptions, FormatJsArrowFunctionExpression,
        FormatJsArrowFunctionExpressionOptions,
    },
};

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
    let comments = f.context().comments();
    let span = node.span();
    let source = f.source_text();

    if !source.next_non_whitespace_byte_is(span.end, b')') {
        let comments_after_node = comments.comments_after(span.end);
        let mut start = span.end;
        // Skip comments after the node to find the next non-whitespace byte whether it's a `)`
        for comment in comments_after_node {
            if !source.all_bytes_match(start, comment.span.start, |b| b.is_ascii_whitespace()) {
                break;
            }
            start = comment.span.end;
        }
        // Still not a `)`, return early because it's not a type cast
        if !source.next_non_whitespace_byte_is(start, b')') {
            return Ok(false);
        }
    }

    if !comments.is_handled_type_cast_comment()
        && let Some(last_printed_comment) = comments.printed_comments().last()
        && last_printed_comment.span.end <= span.start
        && f.source_text().next_non_whitespace_byte_is(last_printed_comment.span.end, b'(')
        && f.comments().is_type_cast_comment(last_printed_comment)
    {
        // Get the source text from the end of type cast comment to the node span
        let node_source_text = source.bytes_range(last_printed_comment.span.end, span.end);

        // `(/** @type {Number} */ (bar).zoo)`
        //                         ^^^^
        // Should wrap for `baz` rather than `baz.zoo`
        if has_closed_parentheses(node_source_text) {
            return Ok(false);
        }

        f.context_mut().comments_mut().mark_as_type_cast_node(node);
    } else if let Some(type_cast_comment_index) = comments.get_type_cast_comment_index(span) {
        let comments = f.context().comments().unprinted_comments();
        let type_cast_comment = &comments[type_cast_comment_index];

        // Get the source text from the end of type cast comment to the node span
        let node_source_text = source.bytes_range(type_cast_comment.span.end, span.end);

        // `(/** @type {Number} */ (bar).zoo)`
        //                         ^^^^
        // Should wrap for `baz` rather than `baz.zoo`
        if has_closed_parentheses(node_source_text) {
            return Ok(false);
        }

        let type_cast_comments = &comments[..=type_cast_comment_index];

        write!(f, [FormatLeadingComments::Comments(type_cast_comments)])?;
        f.context_mut().comments_mut().mark_as_type_cast_node(node);
        // If the printed cast comment is already handled, return early to avoid infinite recursion.
    } else {
        // No typecast comment
        return Ok(false);
    }

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
