use super::jsdoc_tag::JSDocTag;
use super::parse::JSDocParser;
use std::cell::OnceCell;

#[derive(Debug, Clone)]
pub struct JSDoc<'a> {
    raw: &'a str,
    /// Cached+parsed JSDoc comment and tags
    cached: OnceCell<(String, Vec<JSDocTag<'a>>)>,
}

impl<'a> JSDoc<'a> {
    /// comment_content: Inside of /**HERE*/, not include `/**` and `*/`
    pub fn new(comment_content: &'a str) -> JSDoc<'a> {
        Self { raw: comment_content, cached: OnceCell::new() }
    }

    pub fn comment(&self) -> &str {
        let cache = self.cached.get_or_init(|| JSDocParser::new(self.raw).parse());
        &cache.0
    }

    pub fn tags<'b>(&'b self) -> &'b Vec<JSDocTag<'a>> {
        let cache = self.cached.get_or_init(|| JSDocParser::new(self.raw).parse());
        &cache.1
    }
}
