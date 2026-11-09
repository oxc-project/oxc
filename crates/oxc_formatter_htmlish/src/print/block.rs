//! Angular control flow blocks: transparent, their markers print no box (`@if (a) {` … `}`).

use oxc_allocator::{Allocator, StringBuilder};
use oxc_formatter_core::{
    builders::{hard_line_break, indent, text},
    write,
};
use oxc_htmlish_parser::ast::{Block, NodeId, NodeKind};

use super::{
    HtmlishFormatter,
    children::{Between, write_children},
    format_with,
};

use crate::tree::{Tree, collapse_whitespace};

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
    } else if continues_block(t, prev, next) {
        Between::Space
    } else {
        Between::Hard
    }
}

/// `next` is a block that continues `prev` (`@if` … `@else`), printed after its `}`.
fn continues_block(t: &Tree<'_>, prev: NodeId, next: NodeId) -> bool {
    let (NodeKind::Block(prev), NodeKind::Block(next)) = (t.kind(prev), t.kind(next)) else {
        return false;
    };
    let next = block_name(t, next);
    match block_name(t, prev) {
        "if" | "else if" => matches!(next, "else if" | "else"),
        "for" => next == "empty",
        "defer" | "placeholder" | "loading" | "error" => {
            matches!(next, "placeholder" | "loading" | "error")
        }
        "boundary" => next == "error",
        _ => false,
    }
}

/// The block's name, whitespace collapsed unless it spans lines (`else\n if` is no `else if`, DIVERGENCES.md#angular-block-name-with-line-break).
fn block_name<'a>(t: &'a Tree<'a>, block: &Block<'_>) -> &'a str {
    let name = block.name.slice(t.source());
    if name.contains('\n') { name } else { t.collapsed(block.name) }
}

pub fn write_block<'a>(
    t: &'a Tree<'a>,
    id: NodeId,
    block: &Block<'a>,
    f: &mut HtmlishFormatter<'_, 'a>,
) {
    let item = &t[id];
    let name = block_name(t, block);
    let default_never = name == "default never";
    write!(f, ["@", text(name)]);
    if !block.parameters.is_empty() {
        // `@default never(state)`: Prettier glues the parameters to `never`
        write!(f, if default_never { "(" } else { " (" });
        for (i, parameter) in block.parameters.iter().enumerate() {
            if i > 0 {
                write!(f, "; ");
            }
            // As written until expressions are formatted, on one line
            let parameter =
                collapse_whitespace_outside_quotes(f.allocator(), parameter.slice(t.source()));
            write!(f, text(parameter));
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
    } else if item.next.is_some_and(|next| continues_block(t, id, next)) {
        // `@if (a) {⏎} @else {`, as Prettier prints a closing `}` before the next block
        write!(f, hard_line_break());
    }
    write!(f, "}");
}

/// [`collapse_whitespace`] outside string literals
/// (`'`, `"`, `` ` ``, a `\\` escaping the next character).
fn collapse_whitespace_outside_quotes<'a>(allocator: &'a Allocator, text: &'a str) -> &'a str {
    if !text.contains(['\'', '"', '`']) {
        return collapse_whitespace(allocator, text);
    }
    let mut out = StringBuilder::new_in(allocator);
    let (mut quote, mut escaped, mut space) = (None, false, false);
    for c in text.trim_ascii().chars() {
        if quote.is_none() && c.is_ascii_whitespace() {
            space = true;
            continue;
        }
        if std::mem::take(&mut space) {
            out.push(' ');
        }
        out.push(c);
        match (c, quote) {
            _ if std::mem::take(&mut escaped) => {}
            ('\\', _) => escaped = true,
            (c, Some(q)) if c == q => quote = None,
            ('\'' | '"' | '`', None) => quote = Some(c),
            _ => {}
        }
    }
    out.into_str()
}
