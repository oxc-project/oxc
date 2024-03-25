use super::jsdoc_tag::JSDocTag;
use super::parse::parse_jsdoc;
use std::cell::OnceCell;

#[derive(Debug, Clone)]
pub struct JSDoc<'a> {
    raw: &'a str,
    /// Cached+parsed JSDoc comment and tags
    cached: OnceCell<(String, Vec<JSDocTag<'a>>)>,
}

impl<'a> JSDoc<'a> {
    /// comment_content: Inside of /*HERE*/, not include `/*` and `*/`, the 1st char is `*`
    pub fn new(comment_content: &'a str) -> JSDoc<'a> {
        debug_assert!(comment_content.starts_with('*'));
        Self { raw: comment_content, cached: OnceCell::new() }
    }

    pub fn comment(&self) -> &str {
        &self.parse().0
    }

    pub fn tags(&self) -> &Vec<JSDocTag<'a>> {
        &self.parse().1
    }

    fn parse(&self) -> &(String, Vec<JSDocTag<'a>>) {
        self.cached.get_or_init(|| parse_jsdoc(self.raw))
    }
}
