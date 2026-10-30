use oxc_allocator::{Allocator, ArenaVec, StringBuilder};
use oxc_formatter_core::spec::is_suppression_marker;
use oxc_htmlish_parser::{
    Span,
    ast::{Element, Namespace, Node, NodeId, NodeKind, Root, Whitespace},
};

use super::{
    Borrows, Display, Facts, Tree,
    display::{display_comment, ua_display},
};
use crate::{HtmlishVariant, options::WhitespaceSensitivity, rules::DialectRules};

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
            any_hidden |=
                self.tree[id].display == Display::None && matches!(node.kind, NodeKind::Element(_));
            if self.tree.rules.lowercase_known_names {
                self.mark_name_case(id, parent);
            }
            prev = Some(id);
            pending_space = trailing;
        }
        self.facts(parent).last_child = prev;
        // Inside a hidden parent (`head`) no whitespace renders: nothing to look past
        let past_hidden = any_hidden && self.tree[parent].display != Display::None;
        // Runs meet the nodes that render around them, past hidden ones (Angular's blocks look past them themselves):
        // the one before is the last seen, the one ahead is looked up again on reaching it, so each child is read once
        let look_past = past_hidden && !self.tree.rules.has_blocks;
        let mut before: Option<NodeId> = None;
        let mut ahead =
            if look_past { self.flow_past(self.tree[parent].first_child, true) } else { None };

        // A side is sensitive when both nodes facing each other say so
        let mut prev_trailing = false;
        let mut next = self.tree[parent].first_child;
        while let Some(id) = next {
            let (prev, after) = (self.tree[id].prev, self.tree[id].next);
            let (leading, trailing) = if look_past {
                let own = (ahead == Some(id)).then_some(id);
                if own.is_some() {
                    ahead = self.flow_past(after, true);
                }
                let sides = (
                    self.side_sensitive_through(id, parent, own.or(ahead), before, true),
                    self.side_sensitive_through(id, parent, own.or(before), ahead, false),
                );
                before = own.or(before);
                sides
            } else {
                (
                    self.side_sensitive(id, parent, prev, true),
                    self.side_sensitive(id, parent, after, false),
                )
            };
            if let Some(prev) = prev {
                self.facts(prev).trailing_sensitive &= leading;
            }
            let facts = self.facts(id);
            facts.leading_sensitive = leading && (prev.is_none() || prev_trailing);
            facts.trailing_sensitive = trailing;
            prev_trailing = trailing;
            next = after;
        }
        if past_hidden {
            self.spread_spaces_past_hidden(parent);
        }
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
    /// together with the end comment, its first child (Prettier's `mergeIfConditionalStartEndCommentIntoElementOpeningTag`).
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

    /// Texts and CDATA sections next to each other print as one text (Prettier's `mergeCdataIntoText`);
    /// the source between them, whitespace included, is the merged value.
    fn merge_cdata(&mut self, parent: NodeId) {
        let mut next = self.tree[parent].first_child;
        while let Some(id) = next {
            next = self.tree[id].next;
            let Some(after) = next else { break };
            // The parser never puts two texts side by side, so a pair here always holds a section
            if self.tree.is_text(id) && self.tree.is_text(after) {
                self.absorb(parent, id, after);
                // The merged text may absorb the next one too
                next = Some(id);
            }
        }
    }

    /// `keep` extends over its siblings up to `last`, which leave the printed links.
    fn absorb(&mut self, parent: NodeId, keep: NodeId, last: NodeId) {
        let last = self.tree[last];
        let facts = self.facts(keep);
        facts.span.end = last.span.end;
        facts.trailing_sensitive = last.trailing_sensitive;
        facts.has_trailing_spaces = last.has_trailing_spaces;
        facts.has_cdata |= last.has_cdata;
        facts.next = last.next;
        match last.next {
            Some(after_next) => self.facts(after_next).prev = Some(keep),
            None => self.facts(parent).last_child = Some(keep),
        }
    }

    /// `a<b>x</b>c` prints as one text (Prettier's `mergeSimpleElementIntoText`):
    /// the text before absorbs the element and the text after.
    fn merge_simple_elements(&mut self, parent: NodeId) {
        let mut next = self.tree[parent].first_child;
        while let Some(text) = next {
            // Prettier rebuilds the text from the parts (`<b >` prints `<b>`);
            // the source serves when it reads the same.
            // A chain's text is rebuilt once (`a<b >x</b>c<i>y</i>d`).
            // The end tag keeps its written name, htm's `<//>` included (DIVERGENCES.md#htm-closing-tag-as-written),
            // but where known names print lowercase (DIVERGENCES.md#tag-name-case).
            let mut rebuilt: Option<StringBuilder<'_>> = None;
            while let Some(id) = self.tree[text].next.filter(|&id| self.is_simple_element(id)) {
                let t = &self.tree;
                // `is_simple_element` checked each of these
                let (el, source) = (t.element(id).expect("an element"), t.source);
                let child = t[id].first_child.expect("one child");
                let after = t[id].next.expect("a text after");
                let end_tag = el.end_tag.expect("an end tag");
                let (written, written_end) = (el.name.slice(source), end_tag.name.slice(source));
                let (name, end_name) =
                    (t.printed_name(id, el, written), t.printed_name(id, el, written_end));
                let plain = el.start_tag.slice(source).len() == written.len() + 2
                    && end_tag.span.slice(source).len() == written_end.len() + 3
                    && name == written
                    && end_name == written_end;
                if !plain || rebuilt.is_some() {
                    let value = rebuilt.get_or_insert_with(|| {
                        StringBuilder::from_str_in(t[text].span.slice(source), t.allocator)
                    });
                    for piece in [
                        "<",
                        name,
                        ">",
                        t[child].span.slice(source),
                        "</",
                        end_name,
                        ">",
                        t[after].span.slice(source),
                    ] {
                        value.push_str(piece);
                    }
                }
                let child_has_cdata = t[child].has_cdata;
                self.absorb(parent, text, after);
                self.facts(text).has_cdata |= child_has_cdata;
            }
            if let Some(value) = rebuilt {
                self.tree.merged_texts.push((text, value.into_str()));
            }
            next = self.tree[text].next;
        }
    }

    /// An inline element with one word inside, glued to texts on both sides.
    fn is_simple_element(&self, id: NodeId) -> bool {
        let t = &self.tree;
        let (item, source) = (&t[id], t.source);
        let Some(el) = t.element(id) else { return false };
        let is_text = |id: NodeId| t.is_text(id);
        item.glued_before()
            && item.glued_after()
            && item.prev.is_some_and(is_text)
            && item.next.is_some_and(is_text)
            && el.attributes.is_empty()
            && el.end_tag.is_some_and(|tag| tag.ignored.is_none())
            && item.only_child().is_some_and(|child| {
                is_text(child)
                    && !t[child].has_leading_spaces
                    && !t[child].has_trailing_spaces
                    && !t[child].span.slice(source).bytes().any(|b| b.is_ascii_whitespace())
            })
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
                pre_like = is_pre_like(el);
                script_like = matches!(el.key, "script" | "style");
            }
            _ => {}
        }
        // CDATA is text to Prettier (`mergeCdataIntoText`),
        // printed as written here (DIVERGENCES.md#cdata-as-written).
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

    /// The display from the element's table and the option;
    /// a `<!-- display: -->` comment overrides it in `children`.
    fn display(&self, id: NodeId) -> Display {
        let Some(el) = self.tree.element(id) else {
            // Text, comments and the document: only the option applies
            return match self.sensitivity {
                WhitespaceSensitivity::Ignore => Display::Block,
                _ => Display::Inline,
            };
        };
        if el.namespace == Namespace::Svg {
            return if el.key == "svg" {
                Display::InlineBlock
            } else if self.sensitivity != WhitespaceSensitivity::Ignore && self.in_svg_text(id) {
                // Whitespace between a `<text>`'s children renders (DIVERGENCES.md#svg-text-content-inline)
                Display::Inline
            } else {
                Display::Block
            };
        }
        match self.sensitivity {
            WhitespaceSensitivity::Strict => Display::Inline,
            WhitespaceSensitivity::Ignore => Display::Block,
            WhitespaceSensitivity::Css => ua_display(el.key),
        }
    }

    /// Whether an SVG element is inside a `<text>`, whose content lays out as inline text.
    fn in_svg_text(&self, id: NodeId) -> bool {
        std::iter::successors(self.tree.root[id].parent, |&p| self.tree.root[p].parent)
            .map_while(|p| self.tree.element(p).filter(|el| el.namespace == Namespace::Svg))
            .any(|el| el.key == "text")
    }

    /// Whether whitespace on one side of the node is significant, before the neighbor's side is combined in:
    /// the leading side looks at the previous sibling as `neighbor`, the trailing side at the next one.
    fn side_sensitive(
        &self,
        id: NodeId,
        parent_id: NodeId,
        neighbor: Option<NodeId>,
        leading: bool,
    ) -> bool {
        if self.tree.rules.has_blocks {
            return self.side_sensitive_around_blocks(id, parent_id, neighbor, leading);
        }
        let display = neighbor.map(|n| self.tree[n].display);
        self.side_sensitive_with(id, id, parent_id, neighbor, display, leading)
    }

    /// Blocks are transparent: a whitespace run continues past their markers, and past hidden elements.
    fn side_sensitive_around_blocks(
        &self,
        id: NodeId,
        parent_id: NodeId,
        neighbor: Option<NodeId>,
        leading: bool,
    ) -> bool {
        // Across a block, the run meets the content at its near edge, past empty blocks
        let neighbor = self.flow_past(neighbor, !leading);
        self.side_sensitive_through(
            id,
            parent_id,
            self.flow_past(Some(id), leading),
            neighbor,
            leading,
        )
    }

    /// `flow_item` is the node the run meets on `id`'s side (`None` past the parent's edge), `neighbor` the one across it.
    fn side_sensitive_through(
        &self,
        id: NodeId,
        parent_id: NodeId,
        flow_item: Option<NodeId>,
        neighbor: Option<NodeId>,
        leading: bool,
    ) -> bool {
        let t = &self.tree;
        // An empty block passes the run: its sides are those of the content around it
        let (flow_item, neighbor, leading) = match (flow_item, neighbor) {
            (Some(flow_item), neighbor) => (flow_item, neighbor, leading),
            (None, Some(neighbor)) => (neighbor, None, !leading),
            (None, None) => (id, None, leading),
        };
        // At the content's edge, the run is the one the block's own side faces, decided outside it
        if neighbor.is_none() && t.is_block(parent_id) {
            let block = &t[parent_id];
            return if leading { block.leading_sensitive } else { block.trailing_sensitive };
        }
        // The rules are the nearest box's
        let display = neighbor.map(|n| self.display_of(n));
        self.side_sensitive_with(
            id,
            flow_item,
            self.flow_parent(parent_id),
            neighbor,
            display,
            leading,
        )
    }

    /// `flow_item` is the node the run meets on `id`'s side (`id` itself, unless it is a block),
    /// `display` the neighbor's.
    fn side_sensitive_with(
        &self,
        id: NodeId,
        flow_item: NodeId,
        parent_id: NodeId,
        neighbor: Option<NodeId>,
        display: Option<Display>,
        leading: bool,
    ) -> bool {
        let t = &self.tree;
        let (item, parent) = (&t[id], &t[parent_id]);
        let text_or_interpolation = |id: NodeId| t.is_text(id) || t.is_interpolation(id);
        let sensitive =
            if text_or_interpolation(flow_item) && neighbor.is_some_and(text_or_interpolation) {
                true
            } else if parent.display == Display::None {
                false
            } else if parent.pre_like {
                true
            } else if let Some(display) = display {
                display.sides_sensitive()
            } else {
                !(parent_id == NodeId::DOCUMENT
                    || item.pre_like
                    || parent.script_like
                    || !parent.display.edges_sensitive())
            };
        // The line break HTML drops after `<pre>` is not content
        if leading
            && sensitive
            && neighbor.is_none()
            && t.element(parent_id).is_some_and(|el| el.ignores_first_lf)
        {
            return t.is_interpolation(id);
        }
        sensitive
    }

    /// The node a whitespace run meets at `id`'s start (`start`) or end, looking into blocks:
    /// a block's first (last) child that is not whitespace-only;
    /// `None` for a block without content or a hidden element, which the run passes.
    fn flow_edge(&self, id: NodeId, start: bool) -> Option<NodeId> {
        if self.is_hidden(id) {
            return None;
        }
        if !self.tree.is_block(id) {
            return Some(id);
        }
        let mut edges = self
            .tree
            .root
            .children(id)
            .filter(|(_, node)| !is_whitespace_text(node))
            .filter_map(|(child, _)| self.flow_edge(child, start));
        if start { edges.next() } else { edges.last() }
    }

    /// The node a whitespace run meets from `sibling` on, going forward (`forward`) or back, past empty blocks.
    fn flow_past(&self, sibling: Option<NodeId>, forward: bool) -> Option<NodeId> {
        let t = &self.tree;
        std::iter::successors(sibling, |&id| if forward { t[id].next } else { t[id].prev })
            .find_map(|id| self.flow_edge(id, forward))
    }

    /// An element with `display: none` renders nothing, so the whitespace on both its sides is one run
    /// between the content around it (DIVERGENCES.md#script-display-none).
    /// Not `Tree::is_transparent`: a block's children are read before they are built.
    #[inline]
    fn is_hidden(&self, id: NodeId) -> bool {
        self.display_of(id) == Display::None && self.tree.element(id).is_some()
    }

    /// Where hidden siblings join gaps into one run, whitespace in one gap lets every gap of the run take it:
    /// the run renders the one space either way (`a</span> <script></script><span>b` puts the `script` on its own line).
    /// A run that renders nothing keeps its gaps as written, or a line break printed next to a block would spread next time.
    /// A gap's two sides agree on spaces and sensitivity, so a pass each way carries it.
    fn spread_spaces_past_hidden(&mut self, parent: NodeId) {
        let mut next = self.tree[parent].first_child;
        while let Some(id) = next {
            next = self.tree[id].next;
            let facts = &self.tree[id];
            if self.is_hidden(id) && facts.has_leading_spaces && facts.leading_sensitive {
                self.facts(id).has_trailing_spaces = true;
                if let Some(next) = next {
                    self.facts(next).has_leading_spaces = true;
                }
            }
        }
        let mut prev = self.tree[parent].last_child;
        while let Some(id) = prev {
            prev = self.tree[id].prev;
            let facts = &self.tree[id];
            if self.is_hidden(id) && facts.has_trailing_spaces && facts.trailing_sensitive {
                self.facts(id).has_leading_spaces = true;
                if let Some(prev) = prev {
                    self.facts(prev).has_trailing_spaces = true;
                }
            }
        }
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

    /// A node's display, also for one in a block whose children are not built yet.
    fn display_of(&self, id: NodeId) -> Display {
        if self.tree[id].parent.is_some() { self.tree[id].display } else { self.display(id) }
    }

    /// The nearest box: `id`, or the nearest ancestor that is not a block.
    fn flow_parent(&self, mut id: NodeId) -> NodeId {
        while self.tree.is_block(id) {
            id = self.tree.parent(id);
        }
        id
    }
}

fn is_whitespace_text(node: &Node<'_>) -> bool {
    matches!(&node.kind, NodeKind::Text(text) if text.is_whitespace_only())
}

/// CSS `white-space: pre*`.
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
