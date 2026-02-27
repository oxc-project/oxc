use std::cell::OnceCell;

use oxc_span::Span;

use super::{jsdoc_parts::JSDocCommentPart, jsdoc_tag::JSDocTag, parse::parse_jsdoc};

type ParsedJSDoc<'a> = (JSDocCommentPart<'a>, Vec<JSDocTag<'a>>);

#[derive(Debug, Clone)]
pub struct JSDoc<'a> {
    raw: &'a str,
    /// Cached+parsed JSDoc comment and tags
    cached: OnceCell<ParsedJSDoc<'a>>,
    pub span: Span,
}

impl<'a> JSDoc<'a> {
    /// comment_content: Inside of /**HERE*/, not include `/**` and `*/`
    /// span: `Span` for this JSDoc comment, range for `/**HERE*/`
    pub fn new(comment_content: &'a str, span: Span) -> JSDoc<'a> {
        Self { raw: comment_content, cached: OnceCell::new(), span }
    }

    pub fn comment(&self) -> JSDocCommentPart<'a> {
        self.parse().0
    }

    pub fn tags(&self) -> &Vec<JSDocTag<'a>> {
        &self.parse().1
    }

    fn parse(&self) -> &ParsedJSDoc<'a> {
        self.cached.get_or_init(|| parse_jsdoc(self.raw, self.span.start))
    }
}

#[cfg(test)]
mod test {
    use oxc_span::Span;

    #[test]
    fn parses_with_double_backticks() {
        let source = "\
 * Handle whitespace rendering based on meta string
 *
 * `` ```js :whitespace[=all|boundary|trailing] ``
 *
 * @param parser - Code parser instance
 * @param meta - Meta string
 * @param globalOption - Global whitespace option
 *
 * @example
 * ```ts
 * metaWhitespace(parser, ':whitespace=all', true)
 * ```
 ";
        #[expect(clippy::cast_possible_truncation)]
        let jsdoc = super::JSDoc::new(source, Span::new(0, source.len() as u32));
        let tags = jsdoc.tags();
        assert_eq!(tags.len(), 4);
        assert_eq!(tags[0].kind.parsed(), "param");
        assert_eq!(tags[1].kind.parsed(), "param");
        assert_eq!(tags[2].kind.parsed(), "param");
        assert_eq!(tags[3].kind.parsed(), "example");
    }
}
