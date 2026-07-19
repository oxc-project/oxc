use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{Buffer, prelude::*},
    write,
};

use super::array_element_list::ArrayElementList;

#[derive(Default)]
pub struct FormatArrayExpressionOptions {
    pub is_force_flat_mode: bool,
}

pub struct FormatArrayExpression<'a, 'b> {
    array: &'b AstNode<'a, ArrayExpression<'a>>,
    options: FormatArrayExpressionOptions,
}

impl<'a, 'b> FormatArrayExpression<'a, 'b> {
    pub fn new(array: &'b AstNode<'a, ArrayExpression<'a>>) -> Self {
        Self { array, options: FormatArrayExpressionOptions::default() }
    }
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatArrayExpression<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        write!(f, "[");

        if self.array.elements().is_empty() {
            write!(f, format_dangling_comments(self.array.span).with_soft_block_indent());
        } else {
            let group_id = f.group_id("array");
            // A line comment after the last element (e.g. after a trailing hole)
            // is printed right before the `]` and needs the array to break
            let has_trailing_line_comment = || {
                self.array.elements().last().is_some_and(|last| {
                    f.comments()
                        .comments_in_range(last.span().end, self.array.span.end)
                        .iter()
                        .any(|comment| comment.is_line())
                })
            };
            let should_expand = !self.options.is_force_flat_mode
                && (should_break(self.array) || has_trailing_line_comment());

            let elements = ArrayElementList::new(self.array.elements(), group_id);

            write!(
                f,
                group(&soft_block_indent(&elements))
                    .with_group_id(Some(group_id))
                    .should_expand(should_expand)
            );
        }

        write!(f, "]");
    }
}

/// Returns `true` for arrays containing at least two elements if:
/// * all elements are either object or array expressions
/// * each child array expression has at least two elements, or each child object expression has at least two members.
fn should_break(array: &ArrayExpression<'_>) -> bool {
    if array.elements.len() < 2 {
        false
    } else {
        let mut elements = array.elements.iter().peekable();

        while let Some(element) = elements.next() {
            match element.as_expression().map(Expression::kind) {
                Some(ExpressionKind::ArrayExpression(array)) => {
                    let next_is_array_or_end = elements.peek().is_none_or(|next| {
                        next.as_expression().is_some_and(Expression::is_array_expression)
                    });
                    if array.elements.len() < 2 || !next_is_array_or_end {
                        return false;
                    }
                }
                Some(ExpressionKind::ObjectExpression(object)) => {
                    let next_is_object_or_empty = elements.peek().is_none_or(|next| {
                        next.as_expression().is_some_and(Expression::is_object_expression)
                    });

                    if object.properties.len() < 2 || !next_is_object_or_empty {
                        return false;
                    }
                }
                _ => {
                    return false;
                }
            }
        }

        true
    }
}
