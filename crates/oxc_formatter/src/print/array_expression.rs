use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{Buffer, Formatter, prelude::*},
    options::ArrayExpand,
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

impl<'a> Format<'a> for FormatArrayExpression<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, "[");

        if self.array.elements().is_empty() {
            write!(f, format_dangling_comments(self.array.span).with_block_indent());
        } else {
            let group_id = f.group_id("array");
            let array_expand = f.options().array_expand;

            let force_above_threshold = matches!(array_expand, ArrayExpand::ForceAboveThreshold(threshold) if self.array.elements().len() >= threshold as usize);

            let preserve_multiline = !force_above_threshold
                && matches!(array_expand, ArrayExpand::Auto | ArrayExpand::ForceAboveThreshold(_))
                && elements_have_leading_newline(self.array, f);

            let should_expand = !self.options.is_force_flat_mode
                && match array_expand {
                    ArrayExpand::Auto => should_break(self.array) || preserve_multiline,
                    ArrayExpand::Never => false,
                    ArrayExpand::ForceAboveThreshold(_) => {
                        force_above_threshold || preserve_multiline
                    }
                };

            let force_one_per_line = force_above_threshold || preserve_multiline;
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

fn elements_have_leading_newline(array: &AstNode<'_, ArrayExpression<'_>>, f: &Formatter) -> bool {
    array.elements().first().is_some_and(|e| {
        f.source_text().contains_newline_between(array.span.start, e.span().start)
    })
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
