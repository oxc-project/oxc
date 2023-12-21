mod builder;

use std::{cell::OnceCell, collections::BTreeMap};

pub use builder::JSDocBuilder;
use oxc_span::Span;

use self::parser::JSDocParser;
pub use self::parser::JSDocTag;

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
    pub fn new(docs: BTreeMap<Span, JSDocComment<'a>>) -> Self {
        Self { docs }
    }

    pub fn get_by_span<'b>(&'b self, span: Span) -> Option<JSDocComment<'a>> {
        self.docs.get(&span).cloned()
    }
}

impl<'a> JSDocComment<'a> {
    pub fn new(comment: &'a str) -> JSDocComment<'a> {
        Self { comment, tags: OnceCell::new() }
    }

    pub fn tags<'b>(&'b self) -> &'b Vec<JSDocTag<'a>> {
        self.tags.get_or_init(|| JSDocParser::new(self.comment).parse())
    }
}
