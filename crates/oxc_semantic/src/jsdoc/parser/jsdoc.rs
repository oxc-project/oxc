use super::jsdoc_tag::JSDocTag;
use super::parse::parse_jsdoc;
use oxc_span::Span;
use std::cell::OnceCell;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct JSDoc<'a> {
    raw: &'a str,
    /// Cached+parsed JSDoc comment and tags
    cached: OnceCell<(String, BTreeMap<Span, JSDocTag<'a>>)>,
    pub span: Span,
}

impl<'a> JSDoc<'a> {
    /// comment_content: Inside of /**HERE*/, not include `/**` and `*/`
    /// span: `Span` for this JSDoc comment, range for `/**HERE*/`
    pub fn new(comment_content: &'a str, span: Span) -> JSDoc<'a> {
        Self { raw: comment_content, cached: OnceCell::new(), span }
    }

    pub fn comment(&self) -> &str {
        &self.parse().0
    }

    pub fn tags(&self) -> &BTreeMap<Span, JSDocTag<'a>> {
        &self.parse().1
    }

    fn parse(&self) -> &(String, BTreeMap<Span, JSDocTag<'a>>) {
        self.cached.get_or_init(|| parse_jsdoc(self.raw, self.span.start))
    }
}
