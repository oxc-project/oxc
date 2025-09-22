use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    format_args,
    formatter::{FormatResult, Formatter, prelude::*},
    generated::ast_nodes::{AstNode, AstNodes},
    parentheses::NeedsParentheses,
    write,
    write::FormatWrite,
};

impl<'a> FormatWrite<'a> for AstNode<'a, TSIntersectionType<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let types = self.types();

        if types.len() == 1 {
            return write!(f, self.types().first());
        }

        let content = format_with(|f| {
            if self.needs_parentheses(f) {
                return write!(
                    f,
                    [
                        indent(&format_once(|f| format_intersection_types(types, f))),
                        soft_line_break()
                    ]
                );
            }

            let is_inside_complex_tuple_type = match self.parent {
                AstNodes::TSTupleType(tuple) => tuple.element_types().len() > 1,
                _ => false,
            };

            if is_inside_complex_tuple_type {
                write!(
                    f,
                    [
                        indent(&format_args!(
                            if_group_breaks(&format_args!(text("("), soft_line_break())),
                            format_once(|f| format_intersection_types(types, f))
                        )),
                        soft_line_break(),
                        if_group_breaks(&text(")"))
                    ]
                )
            } else {
                format_intersection_types(types, f)
            }
        });

        write!(f, [group(&content)])
    }
}

/// Check if a TSType is object-like (object literal, mapped type, etc.)
fn is_object_like_type(ty: &TSType) -> bool {
    matches!(ty, TSType::TSTypeLiteral(_) | TSType::TSMappedType(_))
}

// [Prettier applies]: https://github.com/prettier/prettier/blob/cd3e530c2e51fb8296c0fb7738a9afdd3a3a4410/src/language-js/print/type-annotation.js#L93-L120
fn format_intersection_types<'a>(
    node: &AstNode<'a, Vec<'a, TSType<'a>>>,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()> {
    let last_index = node.len().saturating_sub(1);
    let mut is_prev_object_like = false;
    let mut is_chain_indented = false;

    for (index, item) in node.iter().enumerate() {
        let is_object_like = is_object_like_type(item.as_ref());

        // always inline first element
        if index == 0 {
            write!(f, item)?;
        } else {
            // If no object is involved, go to the next line if it breaks
            if !is_prev_object_like && !is_object_like {
                write!(f, [indent(&format_args!(soft_line_break_or_space(), item))])?;
            } else {
                write!(f, space())?;

                if !is_prev_object_like || !is_object_like {
                    // indent if we move from object to non-object or vice versa, otherwise keep inline
                    is_chain_indented = index > 1;
                }

                if is_chain_indented {
                    write!(f, [indent(&item)])?;
                } else {
                    write!(f, item)?;
                }
            }
        }

        // Add separator if not the last element
        if index < last_index {
            write!(f, [space(), "&"])?;
        }

        is_prev_object_like = is_object_like;
    }

    Ok(())
}
