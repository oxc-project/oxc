use std::cell::OnceCell;
use super::jsdoc_tag::{JSDocTagParser, JSDocTag};

#[derive(Debug, Clone)]
pub struct JSDoc<'a> {
    comment_raw: &'a str,
    /// Cached JSDocTags
    tags: OnceCell<Vec<JSDocTag<'a>>>,
}

impl<'a> JSDoc<'a> {
    pub fn new(comment_raw: &'a str) -> JSDoc<'a> {
        Self { comment_raw, tags: OnceCell::new() }
    }

    pub fn comment(&'a self) -> &'a str {
        // TODO: parse from start, until `@` or `*/
        self.comment_raw
    }

    pub fn tags<'b>(&'b self) -> &'b Vec<JSDocTag<'a>> {
        self.tags.get_or_init(|| JSDocTagParser::new(self.comment_raw).parse())
    }
}
