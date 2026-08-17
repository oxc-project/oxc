use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode, format_args, formatter::prelude::*, print::FormatWrite,
    utils::typescript::is_object_like_type, write,
};

impl<'a> FormatWrite<'a> for AstNode<'a, TSIntersectionType<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let content = format_with(|f| format_intersection_types(self.types(), f));
        write!(f, [group(&content)]);
    }
}

// [Prettier applies]: https://github.com/prettier/prettier/blob/cd3e530c2e51fb8296c0fb7738a9afdd3a3a4410/src/language-js/print/type-annotation.js#L93-L120
//
// NOTE: The trailing `&` here vs union's leading `|` is a long-standing, undecided Prettier inconsistency
// (prettier#3986: the 2018 unifying PR was rejected as "current behavior is desired"; maintainers are split again as of 2025-12, no decision).
//
// `operatorPosition: "start"` resolves only the operator half,
// the chain still hangs off the first line instead of breaking after `=` into a full union-style member list.
//
// Planned (product decision, pending the union 3.8-stay decision in `union_type.rs`): unify this printer into the union layout.
// One member per line with leading `&`, object members treated like any other member (NO hug special case).
// Until that lands, this follows Prettier's current output.
fn format_intersection_types<'a>(
    node: &AstNode<'a, ArenaVec<'a, TSType<'a>>>,
    f: &mut JsFormatter<'_, 'a>,
) {
    let operator_leads_break = f.options().operator_position.is_start();
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
                if operator_leads_break {
                    // The hoist keeps own-line comments own-line, like binary-like chains.
                    // NOTE: Prettier prints them behind `& `, losing that (and idempotency).
                    write!(
                        f,
                        soft_line_indent_or_space(&format_args!(
                            format_hoisted_leading_comments(item.span()),
                            "&",
                            space(),
                            item
                        ))
                    );
                } else {
                    write!(f, [space(), "&", soft_line_indent_or_space(&item)]);
                }
            } else {
                write!(f, [space(), "&", space()]);

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

        is_prev_object_like = is_object_like;
    }
}
