use super::parser::JSDoc;
use crate::AstNode;
use oxc_span::{GetSpan, Span};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct JSDocFinder<'a> {
    /// JSDocs by Span
    attached: BTreeMap<Span, Vec<JSDoc<'a>>>,
    not_attached: Vec<JSDoc<'a>>,
}

// NOTE: We may need to provide `get_jsdoc_comments(node)`, and also `get_jsdoc_tags(node)`.
// But, how to get parent here...? Leave it to utils/jsdoc?
// Refs: https://github.com/microsoft/TypeScript/issues/7393#issuecomment-413285773
impl<'a> JSDocFinder<'a> {
    pub fn new(attached: BTreeMap<Span, Vec<JSDoc<'a>>>, not_attached: Vec<JSDoc<'a>>) -> Self {
        Self { attached, not_attached }
    }

    pub fn get_one_by_node<'b>(&'b self, node: &AstNode<'a>) -> Option<JSDoc<'a>> {
        let Some(jsdocs) = self.get_all_by_node(node) else {
            return None;
        };

        // If flagged, at least 1 JSDoc is attached
        // If multiple JSDocs are attached, return the last = nearest
        jsdocs.last().cloned()
    }

    pub fn get_all_by_node<'b>(&'b self, node: &AstNode<'a>) -> Option<Vec<JSDoc<'a>>> {
        if !node.flags().has_jsdoc() {
            return None;
        }

        let span = node.kind().span();
        self.get_all_by_span(span)
    }

    pub fn get_all_by_span<'b>(&'b self, span: Span) -> Option<Vec<JSDoc<'a>>> {
        self.attached.get(&span).cloned()
    }

    pub fn iter_all<'b>(&'b self) -> impl Iterator<Item = &JSDoc<'a>> + 'b {
        self.attached.values().flatten().chain(self.not_attached.iter())
    }
}
