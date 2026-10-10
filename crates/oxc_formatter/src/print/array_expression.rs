use oxc_ast::ast::*;
use oxc_formatter_core::Buffer;
use oxc_span::GetSpan;

use crate::{ast_nodes::AstNode, formatter::prelude::*, options::ArrayExpand, write};

use super::{array_element_list::ArrayElementList, array_like::ArrayLike};

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
            let array_expand = f.options().array_expand;
            // A line comment after the last element (e.g. after a trailing hole)
            // is printed right before the `]` and needs the array to break,
            // regardless of the configured expand mode
            let has_trailing_line_comment = || {
                self.array.elements().last().is_some_and(|last| {
                    f.comments()
                        .comments_in_range(last.span().end, self.array.span.end)
                        .iter()
                        .any(Comment::is_line)
                })
            };

            // `arrayWrap: "collapse"` opts out of Prettier's forced expansion
            let should_expand = !self.options.is_force_flat_mode
                && ((array_expand != ArrayExpand::Never && should_break(self.array))
                    || ArrayLike::ArrayExpression(self.array).should_wrap(f)
                    || has_trailing_line_comment());

            // Preserve-based modes never use the fill layout: a fill-printed
            // array would be re-detected as multiline and re-laid out one per
            // line on the next run, making formatting non-idempotent
            let force_one_per_line =
                matches!(array_expand, ArrayExpand::Preserve | ArrayExpand::ForceAboveThreshold(_));
            let elements = ArrayElementList::new(self.array.elements(), group_id)
                .with_force_one_per_line(force_one_per_line);

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
            match element {
                ArrayExpressionElement::ArrayExpression(array) => {
                    let next_is_array_or_end = matches!(
                        elements.peek(),
                        None | Some(ArrayExpressionElement::ArrayExpression(_))
                    );
                    if array.elements.len() < 2 || !next_is_array_or_end {
                        return false;
                    }
                }
                ArrayExpressionElement::ObjectExpression(object) => {
                    let next_is_object_or_empty = matches!(
                        elements.peek(),
                        None | Some(ArrayExpressionElement::ObjectExpression(_))
                    );

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
