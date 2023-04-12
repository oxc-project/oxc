mod builder;

use std::collections::BTreeMap;

pub use builder::JSDocBuilder;
use once_cell::unsync::OnceCell;
use oxc_ast::{GetSpan, Span};

pub use self::parser::JSDocTag;
use self::parser::JSDocParser;
use crate::AstNode;

mod parser;

#[derive(Debug)]
pub struct JSDoc<'a> {
    /// JSDocs by Span
    docs: BTreeMap<Span, JSDocComment<'a>>,
}

#[derive(Debug, Clone)]
pub struct JSDocComment<'a> {
    comment: &'a str,
    /// Cached JSDocTags
    tags: OnceCell<Vec<JSDocTag<'a>>>,
}

impl<'a> JSDoc<'a> {
    #[must_use]
    pub fn new(docs: BTreeMap<Span, JSDocComment<'a>>) -> Self {
        Self { docs }
    }

    #[must_use]
    pub fn get_by_node<'b>(&'b self, node: &AstNode<'a>) -> Option<JSDocComment<'a>> {
        if !node.get().has_jsdoc() {
            return None;
        }
        let span = node.get().kind().span();
        self.get_by_span(span)
    }

    #[must_use]
    pub fn get_by_span<'b>(&'b self, span: Span) -> Option<JSDocComment<'a>> {
        self.docs.get(&span).cloned()
    }
}

impl<'a> JSDocComment<'a> {
    #[must_use]
    pub fn new(comment: &'a str) -> JSDocComment<'a> {
        Self { comment, tags: OnceCell::new() }
    }

    pub fn tags<'b>(&'b self) -> &'b Vec<JSDocTag<'a>> {
        self.tags.get_or_init(|| JSDocParser::new(self.comment).parse())
    }
}
