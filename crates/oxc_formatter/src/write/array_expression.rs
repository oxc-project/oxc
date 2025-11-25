use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    formatter::{Buffer, Formatter, prelude::*},
    options::Expand,
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
            let should_expand = (!self.options.is_force_flat_mode && should_break(self.array))
                || f.options().expand == Expand::Always;

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
