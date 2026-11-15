//! Angular control flow blocks: transparent, their markers print no box (`@if (a) {` … `}`).

use oxc_formatter_core::{
    builders::{hard_line_break, indent, text},
    write,
};
use oxc_htmlish_parser::ast::{Block, NodeId};

use super::{
    HtmlishFormatter,
    children::{Between, write_children},
    format_with,
};

use crate::tree::Tree;

/// Next to a block marker: glued where the run is significant and empty (DIVERGENCES.md#angular-glued-block-markers),
/// a space before a block that continues the previous one (`} @else {`), a line break otherwise.
pub fn between_block(t: &Tree<'_>, prev: NodeId, next: NodeId) -> Between {
    let (p, n) = (&t[prev], &t[next]);
    // The run continues into a block's content past its marker
    let spaces = p.has_trailing_spaces
        || n.has_leading_spaces
        || (t.is_block(prev) && p.last_child.is_some_and(|last| t[last].has_trailing_spaces))
        || (t.is_block(next) && n.first_child.is_some_and(|first| t[first].has_leading_spaces));
    if n.leading_sensitive && !spaces {
        Between::Nothing
    } else if t.continues_block(prev, next) {
        Between::Space
    } else {
        Between::Hard
    }
}

pub fn write_block<'a>(
    t: &'a Tree<'a>,
    id: NodeId,
    block: &Block<'a>,
    f: &mut HtmlishFormatter<'_, 'a>,
) {
    let item = &t[id];
    let name = t.block_name(block);
    let default_never = name == "default never";
    write!(f, ["@", text(name)]);
    if !block.parameters.is_empty() {
        // `@default never(state)`: Prettier glues the parameters to `never`
        write!(f, if default_never { "(" } else { " (" });
        for (i, parameter) in block.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, "; ");
            }
            // As written: no expression is formatted yet (Prettier keeps so the ones it does not format, `@defer`'s)
            write!(f, text(parameter.slice(t.source())));
        }
        write!(f, ")");
    }
    if block.close.is_none() {
        // `@default never;`; a fall-through `@case (a)` has nothing more
        if default_never {
            write!(f, ";");
        }
        return;
    }
    write!(f, " {");
    if let (Some(first), Some(last)) = (item.first_child, item.last_child) {
        let glued_before = t[first].glued_before() && !item.has_leading_spaces;
        let glued_after = t[last].glued_after() && !item.has_trailing_spaces;
        write!(
            f,
            indent(&format_with(|f| {
                if !glued_before {
                    write!(f, hard_line_break());
                }
                write_children(t, id, false, f);
            }))
        );
        if !glued_after {
            write!(f, hard_line_break());
        }
    } else if item.next.is_some_and(|next| t.continues_block(id, next)) {
        // `@if (a) {⏎} @else {`, as Prettier prints a closing `}` before the next block
        write!(f, hard_line_break());
    }
    write!(f, "}");
}
