use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    formatter::{Formatter, prelude::*},
    parentheses::NeedsParentheses,
    print::FormatWrite,
    utils::format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    utils::typescript::is_object_like_type,
    write,
};

impl<'a> FormatWrite<'a> for AstNode<'a, TSIntersectionType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        let content = format_with(|f| format_intersection_types(self.types(), f));
        write!(f, [group(&content)]);
    }
}

// [Prettier applies]: https://github.com/prettier/prettier/blob/cd3e530c2e51fb8296c0fb7738a9afdd3a3a4410/src/language-js/print/type-annotation.js#L93-L120
fn format_intersection_types<'a>(
    node: &AstNode<'a, Vec<'a, TSType<'a>>>,
    f: &mut Formatter<'_, 'a>,
) {
    let len = node.len();
    let last_index = len.saturating_sub(1);
    let mut is_prev_object_like = false;
    let mut is_chain_indented = false;
    let is_operator_start = f.options().experimental_operator_position.is_start();

    for (index, item) in node.iter().enumerate() {
        let is_object_like = is_object_like_type(item.as_ref());

        // Compute inter-item comment info (used for both suppression and separator spacing).
        // Only relevant in operator-start mode when there is a following item.
        let (has_inline_line_comment_between, has_own_line_comment_between) =
            if is_operator_start && index < last_index {
                let next = &node[index + 1];
                let comments_between =
                    f.comments().comments_in_range(item.span().end, next.span().start);
                (
                    comments_between
                        .iter()
                        .any(|comment| comment.is_line() && !comment.preceded_by_newline()),
                    comments_between
                        .iter()
                        .any(|comment| comment.is_line() && comment.preceded_by_newline()),
                )
            } else {
                (false, false)
            };

        // Case covered by Prettier conformance fixture:
        // `typescript/intersection/mutiple-comments/17192.ts` with
        // `{ experimentalOperatorPosition: "start" }`.
        //
        // When comments between `item` and `next` are mixed (inline + own-line),
        // suppress default trailing-comment emission for `item` and re-emit with exact spacing later.
        let suppress_item_trailing_comments =
            has_inline_line_comment_between && has_own_line_comment_between;

        // Formats item, suppressing trailing comments when needed for controlled re-emission.
        let write_item = format_with(|f| {
            if suppress_item_trailing_comments {
                write!(f, FormatNodeWithoutTrailingComments(item));
            } else {
                write!(f, item);
            }
        });

        // always inline first element
        if index == 0 {
            write!(f, write_item);
        } else {
            // If no object is involved, go to the next line if it breaks
            if !(is_prev_object_like || is_object_like)
                || f.comments().has_leading_own_line_comment(item.span().start)
            {
                let content = format_with(|f| {
                    if item.needs_parentheses(f) {
                        write!(f, format_leading_comments(item.span()));
                    }
                    write!(f, write_item);
                });

                if is_operator_start {
                    write!(
                        f,
                        [indent(&format_with(|f| {
                            write!(f, [soft_line_break_or_space(), "&", space(), &content]);
                        }))]
                    );
                } else {
                    write!(f, soft_line_indent_or_space(&content));
                }
            } else {
                write!(f, space());

                if !is_prev_object_like || !is_object_like {
                    // indent if we move from object to non-object or vice versa, otherwise keep inline
                    is_chain_indented = index > 1;
                }

                if is_chain_indented {
                    write!(f, [indent(&write_item)]);
                } else {
                    write!(f, write_item);
                }
            }
        }

        // Add separator if not the last element
        if index < last_index {
            let should_print_at_end = if !is_operator_start {
                true
            } else {
                let next = &node[index + 1];
                let next_is_object_like = is_object_like_type(next);
                let next_is_non_object_branch = !(is_object_like || next_is_object_like)
                    || f.comments().has_leading_own_line_comment(next.span().start);
                !next_is_non_object_branch
            };

            if should_print_at_end {
                write!(f, [space(), "&"]);
            } else if is_operator_start {
                let next = &node[index + 1];
                if suppress_item_trailing_comments {
                    // Companion branch for `typescript/intersection/mutiple-comments/17192.ts`.
                    // Re-emit the suppressed trailing comments with controlled spacing before
                    // moving `&` to the start of the next line.
                    write!(f, [space(), space()]);
                    match item.as_ast_nodes() {
                        AstNodes::TSTypeReference(r) => r.format_trailing_comments(f),
                        AstNodes::TSLiteralType(l) => l.format_trailing_comments(f),
                        AstNodes::TSIntersectionType(i) => i.format_trailing_comments(f),
                        AstNodes::TSUnionType(u) => u.format_trailing_comments(f),
                        _ => {}
                    }
                }
                if f.comments().has_comment_in_range(item.span().end, next.span().start) {
                    // Same fixture as above (`17192.ts`): add one extra space for the
                    // inline-line-comment transition to preserve Prettier's exact spacing.
                    let add_extra_space_before_inline_comment = has_inline_line_comment_between
                        && f.comments().has_leading_own_line_comment(next.span().start);
                    write!(f, [space(), add_extra_space_before_inline_comment.then_some(space())]);
                }
            }
        }

        is_prev_object_like = is_object_like;
    }
}
