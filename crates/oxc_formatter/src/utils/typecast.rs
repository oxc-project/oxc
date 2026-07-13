use oxc_ast::Comment;
use oxc_span::{GetSpan, Span};

use crate::{
    Buffer, Format, format_args,
    formatter::{
        JsFormatter,
        prelude::*,
        trivia::{FormatLeadingComments, format_leading_comments},
    },
    write,
};

/// How a JSDoc type cast comment relates to a node.
///
/// A cast is a `/** @type */`-like comment immediately followed by a
/// parenthesized expression; where that parenthesis closes decides the binding.
pub enum TypeCast<'a> {
    /// The node itself is the cast target
    /// (the parenthesis right after the comment closes right after the node):
    /// ```js
    /// /** @type {string} */ (value).length
    ///                        ^^^^^ Target
    /// /** @type {Document} */ (root.head ?? fallback)
    ///                          ^^^^^^^^^^^^^^^^^^^^^ Target
    /// ```
    /// The slice holds the comments still to be printed;
    /// it is empty when the cast comment was already printed
    /// (re-entry from [`format_type_cast_comment_node`], or printed by an ancestor).
    Target(&'a [Comment]),
    /// The node's unprinted leading comments end with a cast comment that binds
    /// to an inner expression (its parenthesis closes before the node ends):
    /// ```js
    /// /** @type {Number} */ (bar).zoo
    ///                       ^^^^^^^^^ BindsInner (the member expression)
    ///                       ^^^^^ the cast target is inside it
    /// x ? y : /** @type {D} */ (a).b ?? c
    ///                          ^^^^^^^^^^ BindsInner (the `??` expression)
    /// ```
    /// Formatter-added parentheses around the node must not separate the
    /// comment from its target (see [`format_leading_comments_and_open_paren`]).
    /// The slice holds the node's leading comments, ending with the cast comment.
    BindsInner(&'a [Comment]),
    /// No cast is involved with this node:
    /// no adjacent cast comment, a cast-shaped comment without following `(`
    /// (`/** @type {N} */ value`), or an already-printed cast settled at an
    /// inner level (`.zoo` in the `BindsInner` example above, once the comment is printed).
    None,
}

impl TypeCast<'_> {
    pub fn is_target(&self) -> bool {
        matches!(self, TypeCast::Target(_))
    }
}

/// Classifies how a type cast comment relates to the node at `span`.
/// This is the single source of truth for cast binding;
/// both [`format_type_cast_comment_node`] and [`format_leading_comments_and_open_paren`] consume it.
pub fn classify_type_cast<'a>(span: Span, f: &JsFormatter<'_, 'a>) -> TypeCast<'a> {
    let comments = f.context().comments();
    let source = f.source_text();

    // The cast comment may already be printed:
    // by the re-entry from `format_type_cast_comment_node`,
    // or as the leading comment of an ancestor starting at the same position.
    if !comments.is_handled_type_cast_comment()
        && let Some(last_printed_comment) = comments.printed_comments().last()
        && last_printed_comment.span.end <= span.start
        && comments.is_type_cast_comment_followed_by_paren(last_printed_comment)
    {
        // Cheap gate first: a cast target must be followed by `)`.
        // Every node formatted after a printed cast comment re-enters this branch
        // (e.g. each member chain link), and `has_closed_parentheses` scans the source up to the node end.
        return if !is_followed_by_closing_paren(span, f) {
            TypeCast::None
        } else if has_closed_parentheses(
            source.bytes_range(last_printed_comment.span.end, span.end),
        ) {
            // The cast binds to an inner node; since the comment is already printed,
            // there is nothing left to keep adjacent at this level.
            TypeCast::None
        } else {
            TypeCast::Target(&[])
        };
    }

    if let Some(type_cast_comment_index) = comments.get_type_cast_comment_index(span) {
        let unprinted_comments = comments.unprinted_comments();
        let type_cast_comment = &unprinted_comments[type_cast_comment_index];

        return if has_closed_parentheses(source.bytes_range(type_cast_comment.span.end, span.end)) {
            TypeCast::BindsInner(&unprinted_comments[..=type_cast_comment_index])
        } else if is_followed_by_closing_paren(span, f) {
            TypeCast::Target(&unprinted_comments[..=type_cast_comment_index])
        } else {
            TypeCast::None
        };
    }

    TypeCast::None
}

/// Whether the next non-whitespace byte after the node (skipping comments adjacent to it) is `)`.
/// i.e. source parentheses close right after the node, as a cast target requires.
fn is_followed_by_closing_paren(span: Span, f: &JsFormatter<'_, '_>) -> bool {
    f.source_text().next_non_whitespace_byte_is(span.end, b')')
        || f.context().comments().comments_before_closing_paren(span.end).is_some()
}

/// Formats a node that is the target of a JSDoc type cast (see [`TypeCast::Target`]):
/// prints the pending cast comments, marks the node
/// (so `NeedsParentheses` rules skip their own parentheses), and wraps it in parentheses,
/// like `/** @type {string} */ (value)` or `/** @type {number} */ ((expression))`.
///
/// Returns `true` if the node was formatted as a cast target, `false` otherwise;
/// callers apply their own formatting on `false`.
pub fn format_type_cast_comment_node<'a>(
    node: &(impl Format<'a, JsFormatContext<'a>> + GetSpan),
    is_object_or_array_expression: bool,
    f: &mut JsFormatter<'_, 'a>,
) -> bool {
    // Check if this node is a cast target and get the comments to print
    let TypeCast::Target(type_cast_comments) = classify_type_cast(node.span(), f) else {
        return false;
    };

    // Print the type cast comments if any
    if !type_cast_comments.is_empty() {
        write!(f, [FormatLeadingComments::Comments(type_cast_comments)]);
    }

    let span = node.span();
    f.context_mut().comments_mut().mark_as_type_cast_node(node);

    // https://github.com/prettier/prettier/blob/7584432401a47a26943dd7a9ca9a8e032ead7285/src/language-js/print/estree.js#L117-L120
    if is_object_or_array_expression && !f.comments().has_comment_before(span.start) {
        write!(f, group(&format_args!("(", &format_with(|f| node.fmt(f)), ")")));
    } else {
        write!(f, group(&format_args!("(", soft_block_indent(&format_with(|f| node.fmt(f))), ")")));
    }

    true
}

/// Prints a node's leading comments and the formatter-added `(` in the correct order.
/// The caller prints the matching `)` when `needs_parentheses` is true.
///
/// When the leading comments end with a cast comment binding into the node (see [`TypeCast::BindsInner`]),
/// printing them all first would insert the added `(` between the comment and its cast target,
/// rebinding the cast and changing the type semantics:
/// ```js
/// x ? y : /** @type {D} */ (a).b ?? c
/// ```
/// must become:
/// ```js
/// x ? y : (/** @type {D} */ (a).b ?? c)
/// ```
/// Not
/// ```js
/// x ? y : /** @type {D} */ ((a).b ?? c)
/// ```
/// So the cast comment is printed inside the added parenthesis.
pub fn format_leading_comments_and_open_paren(
    span: Span,
    needs_parentheses: bool,
    f: &mut JsFormatter<'_, '_>,
) {
    if needs_parentheses {
        if let TypeCast::BindsInner(comments) = classify_type_cast(span, f)
            && let Some((cast_comment, rest)) = comments.split_last()
        {
            // Only the cast comment moves inside; earlier comments stay outside.
            write!(
                f,
                [
                    FormatLeadingComments::Comments(rest),
                    "(",
                    FormatLeadingComments::Comments(std::slice::from_ref(cast_comment))
                ]
            );
        } else {
            write!(f, [format_leading_comments(span), "("]);
        }
    } else {
        write!(f, format_leading_comments(span));
    }
}

/// Check if the source text has properly closed parentheses starting with '('.
/// Returns true if the text starts with '(' and all parentheses are balanced.
///
/// NOTE: This lexical scan skips strings and comments but NOT regex literals,
/// so a `)` byte inside a regex (`/\)/`, `/[)]/`) is miscounted as a closing parenthesis
/// and the cast target is misidentified:
/// `/** @type {D} */ (s.replace(/\)/g, ""))` loses its cast parentheses.
///
/// This interior scan can be replaced by a span-based check with
/// no scanning of expression bytes at all: even with `preserve_parens: false`,
/// the parser excludes a node's own wrapping parentheses from its span
/// but includes a leftmost descendant's parentheses (`(a).b ?? c` spans from `(`).
/// So: span starts at the `(` right after the cast comment → the cast binds inner;
/// the gap between the comment and span start holds only `(`s/trivia (and the node is followed by `)`)
/// → the node is the cast target.
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
