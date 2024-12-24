use rustc_hash::FxHashMap;

use oxc_span::{GetSpan, Span};

use crate::AstNode;

use super::parser::JSDoc;

#[derive(Debug, Default)]
pub struct JSDocFinder<'a> {
    /// JSDocs by Span
    attached: FxHashMap<u32, Vec<JSDoc<'a>>>,
    not_attached: Vec<JSDoc<'a>>,
}

impl<'a> JSDocFinder<'a> {
    pub fn new(attached: FxHashMap<u32, Vec<JSDoc<'a>>>, not_attached: Vec<JSDoc<'a>>) -> Self {
        Self { attached, not_attached }
    }

    pub fn get_one_by_node<'b>(&'b self, node: &AstNode<'a>) -> Option<JSDoc<'a>> {
        let jsdocs = self.get_all_by_node(node)?;

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
        self.attached.get(&span.start).cloned()
    }

    pub fn iter_all<'b>(&'b self) -> impl Iterator<Item = &'b JSDoc<'a>> + 'b {
        self.attached.values().flatten().chain(self.not_attached.iter())
    }
}
