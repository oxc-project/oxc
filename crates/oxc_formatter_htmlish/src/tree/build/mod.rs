use oxc_allocator::{Allocator, ArenaVec};
use oxc_formatter_core::spec::is_suppression_marker;
use oxc_htmlish_parser::{
    Span,
    ast::{Element, Namespace, Node, NodeId, NodeKind, Root, Whitespace},
};

use super::{Borrows, Facts, Tree, display::display_comment};
use crate::{HtmlishVariant, options::WhitespaceSensitivity, rules::DialectRules};

mod merge;
mod sensitivity;

impl<'a> Tree<'a> {
    pub fn build(
        allocator: &'a Allocator,
        root: &'a Root<'a>,
        source: &'a str,
        sensitivity: WhitespaceSensitivity,
        variant: HtmlishVariant,
        standalone: bool,
    ) -> Self {
        let facts = ArenaVec::from_iter_in(
            root.nodes.iter().map(|node| Facts { span: node.span, ..Facts::default() }),
            &allocator,
        );
        let mut builder = Builder {
            tree: Self {
                allocator,
                rules: DialectRules::new(variant, root.sfc, standalone),
                root,
                source,
                facts,
                merged_texts: ArenaVec::new_in(&allocator),
            },
            sensitivity,
        };
        builder.facts(NodeId::DOCUMENT).display = builder.display(NodeId::DOCUMENT);
        builder.children(NodeId::DOCUMENT);
        builder.facts(NodeId::DOCUMENT).borrows = builder.borrows(NodeId::DOCUMENT);
        // Merges push in post-order (children first)
        builder.tree.merged_texts.sort_unstable_by_key(|(text, _)| *text);
        builder.tree
    }
}

struct Builder<'a> {
    tree: Tree<'a>,
    sensitivity: WhitespaceSensitivity,
}

impl Builder<'_> {
    fn facts(&mut self, id: NodeId) -> &mut Facts {
        &mut self.tree.facts[id.index()]
    }

    /// The nearest box: `id`, or the nearest ancestor that is not a block.
    fn flow_parent(&self, mut id: NodeId) -> NodeId {
        while self.tree.is_block(id) {
            id = self.tree.parent(id);
        }
        id
    }

    fn children(&mut self, parent: NodeId) {
        let root = self.tree.root;
        let whitespace_sensitive = self.tree[self.flow_parent(parent)].whitespace_sensitive();
        // The end comment a conditional start tag absorbed
        let absorbed_end = self.tree[parent].condition.and(root[parent].first_child);
        let nodes = || {
            std::iter::successors(root[parent].first_child, move |&id| next_child(root, parent, id))
                .filter(move |&id| Some(id) != absorbed_end)
                .map(|id| (id, &root[id]))
        };

        // Prettier drops a lone whitespace-only text even where whitespace is content;
        // a whitespace-sensitive parent keeps it (DIVERGENCES.md#whitespace-only-pre), except `script` / `style`.
        let mut first_two = nodes();
        let (first, second) = (first_two.next(), first_two.next());
        let only_whitespace = (!whitespace_sensitive || self.tree[parent].script_like)
            && second.is_none()
            && first.is_some_and(|(_, node)| is_whitespace_text(node));
        if first.is_none() || only_whitespace {
            let facts = self.facts(parent);
            facts.has_dangling_spaces = only_whitespace;
            facts.dangling_sensitive = facts.display.edges_sensitive() && !facts.script_like;
            return;
        }

        let mut prev: Option<NodeId> = None;
        let mut pending_space = false;
        let mut any_cdata = false;
        let mut any_hidden = false;
        for (id, node) in nodes() {
            if let Some(element) = self.absorbing_element(id) {
                let facts = self.facts(element);
                facts.condition = Some(id);
                facts.span.start = node.span.start;
                continue;
            }
            let (leading, trailing, whitespace_only) = match &node.kind {
                NodeKind::Text(text) if !whitespace_sensitive => (
                    text.leading != Whitespace::None,
                    text.trailing != Whitespace::None,
                    text.is_whitespace_only(),
                ),
                // `<pre />⏎`: the line break HTML drops after the start tag is whitespace after a tag that reads self-closing
                NodeKind::Element(el)
                    if !whitespace_sensitive && el.leading_lf && closes_at_start_tag(node) =>
                {
                    (false, true, false)
                }
                _ => (false, false, false),
            };
            if leading && let Some(prev) = prev {
                self.facts(prev).has_trailing_spaces = true;
            }
            if whitespace_only {
                pending_space = true;
                continue;
            }

            self.node(id, whitespace_sensitive);
            if matches!(node.kind, NodeKind::Cdata) {
                any_cdata = true;
                self.facts(id).has_cdata = true;
            }
            let facts = self.facts(id);
            facts.parent = Some(parent);
            facts.prev = prev;
            facts.has_leading_spaces = leading || pending_space;
            facts.has_trailing_spaces = trailing;
            match prev {
                Some(prev) => {
                    if let NodeKind::Comment(comment) = &root[prev].kind
                        && !comment.bogus
                    {
                        let content = comment.content.slice(self.tree.source);
                        let facts = self.facts(id);
                        facts.prettier_ignore = is_suppression_marker(content);
                        if let Some(display) = display_comment(content) {
                            facts.display = display;
                        }
                    }
                    self.facts(prev).next = Some(id);
                }
                None => self.facts(parent).first_child = Some(id),
            }
            any_hidden |= self.is_hidden(id);
            if self.tree.rules.lowercase_known_names {
                self.mark_name_case(id, parent);
            }
            prev = Some(id);
            pending_space = trailing;
        }
        self.facts(parent).last_child = prev;
        self.side_sensitivities(parent, any_hidden);
        let mut next = self.tree[parent].first_child;
        while let Some(id) = next {
            next = self.tree[id].next;
            if printed_first_child(&root[id]).is_some() {
                self.children(id);
            }
            let facts = &self.tree[id];
            let ends_in_text_like = facts.last_child.map_or_else(
                || facts.text_like || self.tree.is_interpolation(id),
                |last| self.tree[last].ends_in_text_like,
            );
            let ends_in_pre = self.tree.is_unclosed(id)
                && (facts.pre_like
                    || facts.last_child.is_some_and(|last| self.tree[last].ends_in_pre));
            let facts = self.facts(id);
            facts.ends_in_text_like = ends_in_text_like;
            facts.ends_in_pre = ends_in_pre;
        }

        if any_cdata {
            self.merge_cdata(parent);
        }
        self.merge_simple_elements(parent);

        let mut next = self.tree[parent].first_child;
        while let Some(id) = next {
            self.facts(id).borrows = self.borrows(id);
            next = self.tree[id].next;
        }
    }

    /// Needs the node's final siblings and its children's facts.
    fn borrows(&self, id: NodeId) -> Borrows {
        let t = &self.tree;
        let item = &t[id];
        // A block's markers are no tags: nothing borrows them, and a block borrows nothing
        if t.is_block(id) {
            return Borrows::default();
        }
        let in_block = item.parent.is_some_and(|parent| t.is_block(parent));
        Borrows {
            parent_opening_end: item.prev.is_none() && item.glued_before() && !in_block,
            prev_closing_end: item.prev.is_some_and(|prev| {
                !matches!(t.kind(prev), NodeKind::DocType(_))
                    && !t.is_block(prev)
                    && !t[prev].text_like
            }) && item.glued_before(),
            last_child_closing_end: item.last_child.is_some_and(|last| {
                t[last].glued_after() && !t[last].ends_in_text_like && !t.is_block(last)
            }) && !item.pre_like,
            parent_closing_start: item.next.is_none()
                && item.glued_after()
                && item.ends_in_text_like
                && !in_block,
            next_opening_start: item
                .next
                .is_some_and(|next| !t[next].text_like && !t.is_block(next))
                && item.text_like
                && item.glued_after(),
        }
    }

    /// `<!--[if IE]><!--><html><!--<![endif]-->`: for the start comment, the element whose start tag absorbs it
    /// together with the end comment, its first child.
    fn absorbing_element(&self, id: NodeId) -> Option<NodeId> {
        let root = self.tree.root;
        let start = &root[id];
        if !matches!(start.kind, NodeKind::ConditionalStart(_)) {
            return None;
        }
        let element = start.next?;
        let NodeKind::Element(el) = &root[element].kind else { return None };
        let end = printed_first_child(&root[element])?;
        (root[element].span.start == start.span.end
            && matches!(root[end].kind, NodeKind::ConditionalEnd)
            && root[end].span.start == el.start_tag.end)
            .then_some(element)
    }

    /// The facts of a node that does not depend on its siblings.
    fn node(&mut self, id: NodeId, in_whitespace_sensitive: bool) {
        let node = &self.tree.root[id];
        let (mut self_closing, mut pre_like, mut script_like) = (true, false, false);
        match &node.kind {
            NodeKind::Text(text) if !in_whitespace_sensitive => self.facts(id).span = text.content,
            NodeKind::Interpolation(_) | NodeKind::ConditionalComment(_) | NodeKind::Block(_) => {
                self_closing = false;
            }
            NodeKind::Element(el) => {
                self_closing = closes_at_start_tag(node);
                pre_like = is_pre_like(el) || self.preserves_svg_text(id, el);
                script_like = matches!(el.key, "script" | "style");
            }
            _ => {}
        }
        // CDATA reads as text, merged with the texts around it,
        // and prints as written (DIVERGENCES.md#cdata-as-written).
        let text_like =
            matches!(node.kind, NodeKind::Text(_) | NodeKind::Comment(_) | NodeKind::Cdata);
        let display = self.display(id);
        let facts = self.facts(id);
        facts.self_closing = self_closing;
        facts.pre_like = pre_like;
        facts.script_like = script_like;
        facts.text_like = text_like;
        facts.display = display;
    }

    /// Where known names print lowercase (`DialectRules::lowercase_known_names`):
    /// marks foreign content, under which they keep their case, and a tag name written in another case.
    fn mark_name_case(&mut self, id: NodeId, parent: NodeId) {
        let t = &self.tree;
        let Some(el) = t.element(id) else { return };
        let foreign = t[parent].foreign || el.namespace != Namespace::Html;
        let uppercase = |name: Span| name.slice(t.source).bytes().any(|b| b.is_ascii_uppercase());
        let lowercase_name = (uppercase(el.name)
            || el.end_tag.is_some_and(|tag| uppercase(tag.name)))
            && t.lowercases_names(id, el);
        let facts = self.facts(id);
        facts.foreign = foreign;
        facts.lowercase_name = lowercase_name;
    }
}

fn is_whitespace_text(node: &Node<'_>) -> bool {
    matches!(&node.kind, NodeKind::Text(text) if text.is_whitespace_only())
}

/// An HTML element whose default style is `white-space: pre*`.
fn is_pre_like(el: &Element<'_>) -> bool {
    el.namespace != Namespace::Svg
        && matches!(el.key, "pre" | "textarea" | "listing" | "plaintext" | "xmp")
}

/// A void element, or one written `/>` without an end tag.
///
/// HTML ignores the `/` of `<my-el />` and an ancestor's end tag closes it, but the tag reads as self-closing (as Prettier reads it):
/// its content follows it as siblings (`next_child`).
/// The text prints the same either way; whitespace inside such an element is judged as whitespace between siblings,
/// so `<div><my-el />⏎</div>` drops the `⏎` HTML puts in `my-el`, collapsible at the end of a block and never rendered.
/// Except in `<pre />`, whose content keeps its whitespace: re-indented as siblings, it would render differently
/// (DIVERGENCES.md#pre-content-without-end-tag).
fn closes_at_start_tag(node: &Node<'_>) -> bool {
    let NodeKind::Element(el) = &node.kind else { return false };
    el.void
        || (el.self_closing
            && el.end_tag.is_none()
            && (!is_pre_like(el) || node.first_child.is_none()))
}

/// The parser's first child, unless the content follows the element as siblings (`next_child`).
fn printed_first_child(node: &Node<'_>) -> Option<NodeId> {
    node.first_child.filter(|_| !closes_at_start_tag(node))
}

/// The node after `id` among `parent`'s children,
/// with the content of an element whose start tag closes it following it as siblings.
fn next_child(root: &Root<'_>, parent: NodeId, mut id: NodeId) -> Option<NodeId> {
    if closes_at_start_tag(&root[id])
        && let Some(first) = root[id].first_child
    {
        return Some(first);
    }
    loop {
        if let Some(next) = root[id].next {
            return Some(next);
        }
        id = root[id].parent.filter(|&up| up != parent)?;
    }
}
