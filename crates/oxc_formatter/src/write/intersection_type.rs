use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{Formatter, prelude::*},
    utils::typescript::is_object_like_type,
    write,
    write::FormatWrite,
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
    let last_index = node.len().saturating_sub(1);
    let mut is_prev_object_like = false;
    let mut is_chain_indented = false;

    for (index, item) in node.iter().enumerate() {
        let is_object_like = is_object_like_type(item.as_ref());

        // always inline first element
        if index == 0 {
            write!(f, item);
        } else {
            // If no object is involved, go to the next line if it breaks
            if !(is_prev_object_like || is_object_like)
                || f.comments().has_leading_own_line_comment(item.span().start)
            {
                write!(f, soft_line_indent_or_space(item));
            } else {
                write!(f, space());

                if !is_prev_object_like || !is_object_like {
                    // indent if we move from object to non-object or vice versa, otherwise keep inline
                    is_chain_indented = index > 1;
                }

                if is_chain_indented {
                    write!(f, [indent(&item)]);
                } else {
                    write!(f, item);
                }
            }
        }

        // Add separator if not the last element
        if index < last_index {
            write!(f, [space(), "&"]);
        }

        is_prev_object_like = is_object_like;
    }
}
