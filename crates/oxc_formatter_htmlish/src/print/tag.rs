//! Tags and their markers, and the "borrow" rules that move a marker to a neighbor
//! so that a line break can sit inside a tag where whitespace would be significant:
//!
//! ```html
//! <span>text</span
//! ><b>
//! ```
//!
//! The borrower side is [`Borrows`](crate::tree::Borrows), computed with the tree;
//! the owner side here (`*_borrowed`) omits the marker.
//!
//! The rules end up shaped like Prettier's `print/tag.js`, not ported from it:
//! which neighbor prints a marker is the output itself, and matching that output leaves no other structure.

use oxc_formatter_core::{Format, builders::text, write};

use super::{HtmlishFormatter, attribute::write_attributes};
use oxc_htmlish_parser::{
    Span,
    ast::{NodeId, NodeKind},
};

use crate::{context::HtmlishFormatContext, tree::Tree};

/// A tag marker, written as up to four pieces (`<` + name, `<!--[if ` + condition + `]><!--><` + name, ...).
#[derive(Clone, Copy, Default)]
pub struct Marker<'a>([&'a str; 4]);

impl<'a> Marker<'a> {
    const fn one(piece: &'a str) -> Self {
        Self([piece, "", "", ""])
    }

    const fn two(piece: &'a str, name: &'a str) -> Self {
        Self([piece, name, "", ""])
    }

    pub fn len(self) -> usize {
        self.0.iter().map(|piece| piece.len()).sum()
    }
}

impl<'a> Format<'a, HtmlishFormatContext<'a>> for Marker<'a> {
    fn fmt(&self, f: &mut HtmlishFormatter<'_, 'a>) {
        for piece in self.0 {
            if !piece.is_empty() {
                write!(f, text(piece));
            }
        }
    }
}

// ---
// Owner side

/// The node's closing `>` is printed by its next sibling or its parent.
pub fn closing_end_borrowed(t: &Tree<'_>, id: NodeId) -> bool {
    match t[id].next {
        Some(next) => t[next].borrows.prev_closing_end,
        None => t[t.parent(id)].borrows.last_child_closing_end,
    }
}

/// The node's `<name` is printed by its previous sibling.
pub fn opening_start_borrowed(t: &Tree<'_>, id: NodeId) -> bool {
    t[id].prev.is_some_and(|prev| t[prev].borrows.next_opening_start)
}

/// The node's opening `>` is printed by its first child.
pub fn opening_end_borrowed(t: &Tree<'_>, id: NodeId) -> bool {
    t[id].first_child.is_some_and(|first| t[first].borrows.parent_opening_end)
}

/// The node's `</name` is printed by its last child.
pub fn closing_start_borrowed(t: &Tree<'_>, id: NodeId) -> bool {
    t[id].last_child.is_some_and(|last| t[last].borrows.parent_closing_start)
}

// ---
// Markers

pub fn opening_start_marker<'a>(t: &'a Tree<'a>, id: NodeId) -> Marker<'a> {
    match t.kind(id) {
        NodeKind::ConditionalComment(comment) => {
            Marker::two("<!--[if ", t.collapsed(comment.condition))
        }
        NodeKind::ConditionalStart(start) => Marker::two("<!--[if ", t.collapsed(start.condition)),
        NodeKind::ConditionalEnd => Marker::one("<!--<!"),
        NodeKind::Interpolation(_) => Marker::one("{{"),
        // `<!doctype html>` but where an XML parser may read it, otherwise as written
        // (DIVERGENCES.md#doctype-case, DIVERGENCES.md#doctype-html-in-one-pass)
        NodeKind::DocType(doctype) => {
            if t.rules().lowercase_doctype
                && after_html_word(doctype.value.slice(t.source()).trim_ascii()) == Some("")
            {
                Marker::one("<!doctype")
            } else {
                let start = t[id].span.start;
                Marker::one(Span::new(start, start + 9).slice(t.source()))
            }
        }
        NodeKind::Element(_) => match t[id].condition.map(|start| t.kind(start)) {
            Some(NodeKind::ConditionalStart(start)) => {
                Marker(["<!--[if ", t.collapsed(start.condition), "]><!--><", t.name(id)])
            }
            _ => Marker::two("<", t.name(id)),
        },
        _ => Marker::two("<", t.name(id)),
    }
}

pub fn opening_end_marker<'a>(t: &'a Tree<'a>, id: NodeId) -> Marker<'a> {
    match t.kind(id) {
        NodeKind::ConditionalComment(_) => Marker::one("]>"),
        NodeKind::Element(_) if t[id].condition.is_some() => Marker::one("><!--<![endif]-->"),
        NodeKind::Element(el) if el.self_closing => Marker::one("/>"),
        _ => Marker::one(">"),
    }
}

pub fn closing_start_marker<'a>(t: &'a Tree<'a>, id: NodeId) -> Marker<'a> {
    match t.kind(id) {
        NodeKind::ConditionalComment(_) => Marker::one("<!"),
        // The end tag as written, its name lowercase with the start tag's (DIVERGENCES.md#tag-name-case);
        // an omitted one is never added (DIVERGENCES.md#omitted-end-tag).
        // What HTML ignores after the name (`</div foo>`) is kept too, and so is htm's `<//>`.
        NodeKind::Element(el) => el.end_tag.map_or(Marker::default(), |tag| {
            let name = t.printed_name(id, el, tag.name.slice(t.source()));
            let ignored = tag
                .ignored
                .map_or("", |ignored| Span::new(tag.name.end, ignored.end).slice(t.source()));
            Marker(["</", name, ignored, ""])
        }),
        _ => Marker::two("</", t.name(id)),
    }
}

pub fn closing_end_marker<'a>(t: &'a Tree<'a>, id: NodeId) -> Marker<'a> {
    let item = &t[id];
    match t.kind(id) {
        NodeKind::ConditionalComment(_) | NodeKind::ConditionalEnd => Marker::one("[endif]-->"),
        NodeKind::ConditionalStart(_) => Marker::one("]><!-->"),
        NodeKind::Interpolation(_) => Marker::one("}}"),
        // A void element prints `/>` too
        NodeKind::Element(_) if item.self_closing => Marker::one("/>"),
        NodeKind::Element(el) if el.end_tag.is_none() => Marker::default(),
        _ => Marker::one(">"),
    }
}

/// What follows a leading `html` word in a doctype's value (`HTML PUBLIC …`), which HTML reads case-insensitively.
pub fn after_html_word(value: &str) -> Option<&str> {
    let rest = value.get(4..).filter(|_| value[..4].eq_ignore_ascii_case("html"))?;
    (!rest.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')).then_some(rest)
}

// ---
// Prefixes and suffixes: what a node prints for its neighbors

pub fn opening_tag_prefix<'a>(t: &'a Tree<'a>, id: NodeId) -> Marker<'a> {
    if t[id].borrows.parent_opening_end {
        opening_end_marker(t, t.parent(id))
    } else if t[id].borrows.prev_closing_end {
        closing_end_marker(t, t[id].prev.expect("borrowed from the previous sibling"))
    } else {
        Marker::default()
    }
}

pub fn closing_tag_suffix<'a>(t: &'a Tree<'a>, id: NodeId) -> Marker<'a> {
    if t[id].borrows.parent_closing_start {
        closing_start_marker(t, t.parent(id))
    } else if t[id].borrows.next_opening_start {
        opening_start_marker(t, t[id].next.expect("borrowed from the next sibling"))
    } else {
        Marker::default()
    }
}

// ---
// Tags

pub fn write_opening_tag_start<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    if !opening_start_borrowed(t, id) {
        write!(f, [opening_tag_prefix(t, id), opening_start_marker(t, id)]);
    }
}

pub fn write_opening_tag<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    write_opening_tag_start(t, id, f);
    write_attributes(t, id, f);
    if !t[id].self_closing && !opening_end_borrowed(t, id) {
        write!(f, opening_end_marker(t, id));
    }
}

pub fn write_closing_tag<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    if !t[id].self_closing && !closing_start_borrowed(t, id) {
        if t[id].borrows.last_child_closing_end {
            write!(f, closing_end_marker(t, t[id].last_child.expect("borrowed by the last child")));
        }
        write!(f, closing_start_marker(t, id));
    }
    write_closing_tag_end(t, id, f);
}

pub fn write_closing_tag_end<'a>(t: &'a Tree<'a>, id: NodeId, f: &mut HtmlishFormatter<'_, 'a>) {
    if !closing_end_borrowed(t, id) {
        write!(f, [closing_end_marker(t, id), closing_tag_suffix(t, id)]);
    }
}
