//! Whitespace sensitivity: which gaps between nodes render, from CSS display and the options,
//! past hidden elements and Angular blocks.

use oxc_htmlish_parser::ast::{Element, Namespace, NodeId};

use super::{Builder, is_pre_like, is_whitespace_text};
use crate::{
    options::WhitespaceSensitivity,
    tree::{Display, display::ua_display},
};

impl Builder<'_> {
    /// The display from the element's table and the option;
    /// a `<!-- display: -->` comment overrides it in `children`.
    pub(super) fn display(&self, id: NodeId) -> Display {
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

    /// A node's display, also for one in a block whose children are not built yet.
    fn display_of(&self, id: NodeId) -> Display {
        if self.tree[id].parent.is_some() { self.tree[id].display } else { self.display(id) }
    }

    /// An SVG `<text>`, or an element in one, under `xml:space="preserve"` on it or an SVG ancestor:
    /// its whitespace renders as written (DIVERGENCES.md#svg-xml-space-preserve).
    pub(super) fn preserves_svg_text(&self, id: NodeId, el: &Element<'_>) -> bool {
        let source = self.tree.source;
        el.namespace == Namespace::Svg
            && (el.key == "text" || self.in_svg_text(id))
            && self.svg_elements(Some(id)).any(|el| {
                el.attributes.iter().any(|attribute| {
                    self.tree.attribute_is(attribute.name.slice(source), "xml:space")
                        && attribute
                            .value
                            .is_some_and(|value| value.span.slice(source) == "preserve")
                })
            })
    }

    /// Whether an SVG element is inside a `<text>`, whose content lays out as inline text.
    fn in_svg_text(&self, id: NodeId) -> bool {
        self.svg_elements(self.tree.root[id].parent).any(|el| el.key == "text")
    }

    /// The SVG elements from `from` up, until one that is not.
    fn svg_elements(&self, from: Option<NodeId>) -> impl Iterator<Item = &Element<'_>> {
        std::iter::successors(from, |&p| self.tree.root[p].parent)
            .map_while(|p| self.tree.element(p).filter(|el| el.namespace == Namespace::Svg))
    }

    /// Each child's leading and trailing sensitivity, once the children are linked
    /// (`any_hidden`: one of them is a hidden element).
    pub(super) fn side_sensitivities(&mut self, parent: NodeId, any_hidden: bool) {
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
    }

    /// Whether whitespace on one side of the node is significant, before the neighbor's side is combined in:
    /// the leading side looks at the previous sibling as `neighbor`, the trailing side at the next one.
    /// Blocks are transparent: a whitespace run continues past their markers, and past hidden elements.
    fn side_sensitive(
        &self,
        id: NodeId,
        parent_id: NodeId,
        neighbor: Option<NodeId>,
        leading: bool,
    ) -> bool {
        if !self.tree.rules.has_blocks {
            return self.side_sensitive_with(id, id, parent_id, neighbor, leading);
        }
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
        self.side_sensitive_with(id, flow_item, self.flow_parent(parent_id), neighbor, leading)
    }

    /// `flow_item` is the node the run meets on `id`'s side (`id` itself, unless it is a block).
    fn side_sensitive_with(
        &self,
        id: NodeId,
        flow_item: NodeId,
        parent_id: NodeId,
        neighbor: Option<NodeId>,
        leading: bool,
    ) -> bool {
        let t = &self.tree;
        let parent = &t[parent_id];
        let text_or_interpolation = |id: NodeId| t.is_text(id) || t.is_interpolation(id);
        let sensitive =
            if text_or_interpolation(flow_item) && neighbor.is_some_and(text_or_interpolation) {
                true
            } else if parent.display == Display::None {
                false
            } else if parent.pre_like {
                true
            } else if let Some(neighbor) = neighbor {
                self.display_of(neighbor).sides_sensitive()
            } else {
                // An HTML `pre`'s own edges, not those of all pre-like content
                !(parent_id == NodeId::DOCUMENT
                    || t.element(id).is_some_and(is_pre_like)
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
    pub(super) fn is_hidden(&self, id: NodeId) -> bool {
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
}
