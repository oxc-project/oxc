//! The facts Prettier's `print-preprocess` attaches to its tree, as a table keyed by the parser's [`NodeId`].
//!
//! The AST is not rewritten; names, kinds and the tag table's verdict are read from it.
//! What the table adds:
//! - the nodes that print, linked by `parent` / `prev` / `next` / `first_child` / `last_child`:
//!   a whitespace-only text of a whitespace-insensitive parent is skipped (it becomes `has_*_spaces` of its neighbors),
//!   and the content of an element that reads as self-closing (`<my-el />`) follows it as siblings;
//! - CSS display and whitespace sensitivity, which depend on the options;
//! - facts the printer asks repeatedly (self-closing, `ends_in_text_like`, `prettier-ignore`).

mod build;
mod display;
mod names;

use oxc_allocator::{Allocator, ArenaVec, StringBuilder};
use oxc_htmlish_parser::{
    Span,
    ast::{Element, Namespace, NodeId, NodeKind, Root},
};

use crate::rules::DialectRules;

pub use display::Display;

#[derive(Clone, Copy, Default)]
pub struct Facts {
    /// Prettier's `sourceSpan`: a text in a whitespace-insensitive parent is trimmed.
    pub span: Span,
    /// The parent, siblings and children that print.
    pub parent: Option<NodeId>,
    pub prev: Option<NodeId>,
    pub next: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub display: Display,
    /// Text and comments: printed without tags whose markers could be borrowed.
    pub text_like: bool,
    /// No children and no end tag: the start tag closes the node (Prettier's `isSelfClosing`).
    /// Text-like nodes count too.
    pub self_closing: bool,
    /// CSS `white-space: pre*`: `pre`, `textarea`, `listing`, `plaintext`, `xmp`, outside SVG.
    /// Children keep their whitespace and indentation.
    pub pre_like: bool,
    /// `script` / `style`: children keep their whitespace.
    pub script_like: bool,
    /// The deepest last descendant is text-like, or the node itself is
    /// (an interpolation ends in its text).
    pub ends_in_text_like: bool,
    /// The node is a `pre` without an end tag, or ends in one through its last children without end tags:
    /// what follows it up to the end tag that closes them is `pre` content.
    pub ends_in_pre: bool,
    /// A CDATA section, or a text that merged one: its sections are never re-wrapped.
    pub has_cdata: bool,
    /// The previous sibling is `<!-- prettier-ignore -->`.
    pub prettier_ignore: bool,
    /// An SVG / MathML element, or one under it (`DialectRules::lowercase_known_names` only).
    pub foreign: bool,
    /// The tag name prints lowercase, written otherwise in the start or end tag (`Tree::printed_name`).
    pub lowercase_name: bool,
    pub has_leading_spaces: bool,
    pub has_trailing_spaces: bool,
    pub leading_sensitive: bool,
    pub trailing_sensitive: bool,
    /// An element without children had only whitespace inside.
    pub has_dangling_spaces: bool,
    pub dangling_sensitive: bool,
    /// An element whose start tag absorbs `<!--[if X]><!-->` right before it and `<!--<![endif]-->` right after it
    /// (`<!--[if X]><!--><html><!--<![endif]-->`): those two comments, which do not print on their own.
    /// The start comment; the end comment is the element's first child.
    pub condition: Option<NodeId>,
    pub borrows: Borrows,
}

/// The tag markers a node prints for its neighbors (the borrow rules in `print/tag.rs`);
/// the owners omit them (`*_borrowed`).
#[derive(Clone, Copy, Default)]
pub struct Borrows {
    /// `<p\n  >123`: the first child prints its parent's opening `>`.
    pub parent_opening_end: bool,
    /// `<p></p\n>123`: the previous sibling's closing `>`.
    pub prev_closing_end: bool,
    /// `<p\n  ><a></a\n  ></p\n>`: the last child's closing `>`, before the node's own end tag.
    pub last_child_closing_end: bool,
    /// `123</p\n>`: the last text prints its parent's `</p`.
    pub parent_closing_start: bool,
    /// `123<p\n>`: the text prints its next sibling's `<p`.
    pub next_opening_start: bool,
}

impl Facts {
    /// Children keep their whitespace.
    pub fn whitespace_sensitive(&self) -> bool {
        self.pre_like || self.script_like
    }

    /// Whitespace before the node is significant and there is none.
    pub fn glued_before(&self) -> bool {
        self.leading_sensitive && !self.has_leading_spaces
    }

    /// Whitespace after the node is significant and there is none.
    pub fn glued_after(&self) -> bool {
        self.trailing_sensitive && !self.has_trailing_spaces
    }

    /// Exactly one child prints.
    pub fn only_child(&self) -> Option<NodeId> {
        self.first_child.filter(|_| self.first_child == self.last_child)
    }
}

pub struct Tree<'a> {
    allocator: &'a Allocator,
    rules: DialectRules,
    root: &'a Root<'a>,
    source: &'a str,
    facts: ArenaVec<'a, Facts>,
    /// Texts whose merged value differs from their source
    /// (an element merged in had whitespace in a tag), sorted by `NodeId`.
    merged_texts: ArenaVec<'a, (NodeId, &'a str)>,
}

impl std::ops::Index<NodeId> for Tree<'_> {
    type Output = Facts;

    fn index(&self, id: NodeId) -> &Facts {
        &self.facts[id.index()]
    }
}

impl<'a> Tree<'a> {
    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn kind(&self, id: NodeId) -> &'a NodeKind<'a> {
        &self.root[id].kind
    }

    pub fn rules(&self) -> DialectRules {
        self.rules
    }

    /// An Angular control flow block: transparent, its markers print no box (`@if (a) {` … `}`).
    pub fn is_block(&self, id: NodeId) -> bool {
        self.rules.has_blocks && matches!(self.kind(id), NodeKind::Block(_))
    }

    /// A `display: none` element in a parent that renders: the whitespace on both its sides is one run
    /// (DIVERGENCES.md#script-display-none).
    #[inline]
    pub fn is_transparent(&self, id: NodeId) -> bool {
        self[id].display == Display::None
            && self.element(id).is_some()
            && self[self.parent(id)].display != Display::None
    }

    pub fn is_interpolation(&self, id: NodeId) -> bool {
        matches!(self.kind(id), NodeKind::Interpolation(_))
    }

    /// A text or a CDATA section, which Prettier merges into text (`mergeCdataIntoText`).
    pub fn is_text(&self, id: NodeId) -> bool {
        matches!(self.kind(id), NodeKind::Text(_) | NodeKind::Cdata)
    }

    pub fn element(&self, id: NodeId) -> Option<&'a Element<'a>> {
        match &self.root[id].kind {
            NodeKind::Element(el) => Some(el),
            _ => None,
        }
    }

    /// The tag name to print (DIVERGENCES.md#tag-name-case); empty for other nodes.
    #[inline]
    pub fn name(&self, id: NodeId) -> &'a str {
        self.element(id).map_or("", |el| self.printed_name(id, el, el.name.slice(self.source)))
    }

    /// A start or end tag's name `written`, or the element's lowercase name where it prints so (DIVERGENCES.md#tag-name-case);
    /// htm's `<//>` closes as written.
    #[inline]
    pub fn printed_name(&self, id: NodeId, el: &Element<'a>, written: &'a str) -> &'a str {
        if self[id].lowercase_name && written != "/" { el.key } else { written }
    }

    /// An attribute's name written `written` to print: as written, or Prettier's known name in lowercase
    /// (DIVERGENCES.md#tag-name-case).
    #[inline]
    pub fn attribute_name(&self, element: NodeId, written: &'a str) -> &'a str {
        if !self.rules.lowercase_known_names || !written.bytes().any(|b| b.is_ascii_uppercase()) {
            return written;
        }
        match self.element(element) {
            Some(el) if self.lowercases_names(element, el) => {
                names::known_attribute(el.key, written).unwrap_or(written)
            }
            _ => written,
        }
    }

    /// Whether the element's known names print lowercase: a known HTML element outside foreign content.
    /// `svg` / `math` themselves are known; under them Prettier keeps every name, `foreignObject`'s HTML too.
    pub fn lowercases_names(&self, id: NodeId, el: &Element<'a>) -> bool {
        !self[id].parent.is_some_and(|parent| self[parent].foreign)
            && (el.namespace == Namespace::Html || matches!(el.key, "svg" | "math"))
            && names::is_known_element(el.key)
    }

    /// Whether an attribute written `written` is the one named `name` (lowercase).
    /// Prettier lowercases only the attributes it knows for the element; any name here is close enough.
    pub fn attribute_is(&self, written: &str, name: &str) -> bool {
        if self.rules.names_ignore_case {
            written.eq_ignore_ascii_case(name)
        } else {
            written == name
        }
    }

    /// A text's value: its source, unless a merge rewrote it.
    pub fn text(&self, id: NodeId) -> &'a str {
        match self.merged_texts.binary_search_by_key(&id, |(text, _)| *text) {
            Ok(index) => self.merged_texts[index].1,
            Err(_) => self[id].span.slice(self.source),
        }
    }

    /// A conditional comment's condition or a doctype's value, whitespace collapsed.
    pub fn collapsed(&self, span: Span) -> &'a str {
        collapse_whitespace(self.allocator, span.slice(self.source))
    }

    /// The name the HTML tables are keyed by (the parser's, per dialect); empty for other nodes.
    pub fn tag(&self, id: NodeId) -> &'a str {
        self.element(id).map_or("", |el| el.key)
    }

    pub fn parent(&self, id: NodeId) -> NodeId {
        self[id].parent.expect("only the document has no parent")
    }

    /// The printed children, in order.
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(self[id].first_child, move |&child| self[child].next)
    }

    /// Where the content between the tags starts (Prettier's `startSourceSpan.end`).
    pub fn content_start(&self, id: NodeId) -> u32 {
        if self[id].condition.is_some()
            && let Some(end) = self.root[id].first_child
        {
            return self.root[end].span.end;
        }
        match &self.root[id].kind {
            NodeKind::Element(el) => el.start_tag.end,
            NodeKind::ConditionalComment(comment) => comment.content.start,
            NodeKind::Block(block) => block.open.end,
            _ => self[id].span.start,
        }
    }

    /// Where the end tag starts, when there is one (Prettier's `endSourceSpan.start`).
    pub fn end_tag_start(&self, id: NodeId) -> Option<u32> {
        match &self.root[id].kind {
            NodeKind::Element(el) => el.end_tag.map(|tag| tag.span.start),
            NodeKind::ConditionalComment(comment) => Some(comment.content.end),
            NodeKind::Block(block) => block.close.map(|close| close.start),
            _ => None,
        }
    }

    /// An element without an end tag (closed by an ancestor's, a sibling's or the end of input)
    /// that does not read as self-closing.
    pub fn is_unclosed(&self, id: NodeId) -> bool {
        self.element(id).is_some_and(|el| el.end_tag.is_none()) && !self[id].self_closing
    }

    /// Where Prettier's `sourceSpan` of the node ends:
    /// an element without an end tag ends at its start tag, children aside.
    pub fn source_end(&self, id: NodeId) -> u32 {
        match &self.root[id].kind {
            NodeKind::Element(el) if el.end_tag.is_none() => el.start_tag.end,
            _ => self[id].span.end,
        }
    }

    /// Line breaks in `[from, to)`, counted up to 2:
    /// Prettier compares line numbers, so a gap is on the same line, the next line, or further.
    pub fn line_breaks(&self, from: u32, to: u32) -> usize {
        if from >= to {
            return 0;
        }
        memchr::memchr_iter(b'\n', &self.source.as_bytes()[from as usize..to as usize])
            .take(2)
            .count()
    }
}

/// `text` trimmed, with its whitespace runs collapsed to single spaces; borrowed when already so.
pub fn collapse_whitespace<'a>(allocator: &'a Allocator, text: &'a str) -> &'a str {
    let text = text.trim_ascii();
    let collapsed = text
        .as_bytes()
        .windows(2)
        .all(|pair| !pair[1].is_ascii_whitespace() || (pair[1] == b' ' && pair[0] != b' '));
    if collapsed {
        return text;
    }
    let mut out = StringBuilder::new_in(allocator);
    for (i, word) in text.split_ascii_whitespace().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(word);
    }
    out.into_str()
}
