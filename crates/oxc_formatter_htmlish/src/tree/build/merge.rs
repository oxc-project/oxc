//! Siblings that print as one text: CDATA sections with the texts around them,
//! and a simple inline element with the texts on both sides.

use oxc_allocator::StringBuilder;
use oxc_htmlish_parser::ast::NodeId;

use super::Builder;

impl Builder<'_> {
    /// Texts and CDATA sections next to each other print as one text;
    /// the source between them, whitespace included, is the merged value.
    pub(super) fn merge_cdata(&mut self, parent: NodeId) {
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

    /// `a<b>x</b>c` prints as one text:
    /// the text before absorbs the element and the text after.
    pub(super) fn merge_simple_elements(&mut self, parent: NodeId) {
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
}
