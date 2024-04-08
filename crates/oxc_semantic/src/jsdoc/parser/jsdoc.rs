use super::jsdoc_parts::JSDocCommentPart;
use super::jsdoc_tag::JSDocTag;
use super::parse::parse_jsdoc;
use oxc_span::Span;
use std::cell::OnceCell;

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
    use crate::{Semantic, SemanticBuilder};
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn build_semantic<'a>(allocator: &'a Allocator, source_text: &'a str) -> Semantic<'a> {
        let source_type = SourceType::default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .build(program)
            .semantic;
        semantic
    }

    #[test]
    fn jsdoc_span() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            /** single line */
            /**
             * multi
             * line
             */
            /**
multi
line
             */
            ",
        );
        let mut jsdocs = semantic.jsdoc().iter_all();

        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(jsdoc.span.source_text(semantic.source_text), " single line ");
        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(
            jsdoc.span.source_text(semantic.source_text),
            "\n             * multi\n             * line\n             "
        );
        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(jsdoc.span.source_text(semantic.source_text), "\nmulti\nline\n             ");
    }

    #[test]
    fn jsdoc_comment() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            /** single line @k1 c1 @k2 */
            /**
             * multi
             * line
             * @k1 c1a
             * c1b
             * @k2 c2
             * @k3 c3a
             * c3b
             */
            /** * list */
            ",
        );
        let mut jsdocs = semantic.jsdoc().iter_all();

        let jsdoc = jsdocs.next().unwrap();
        let comment = jsdoc.comment();
        assert_eq!(comment.parsed(), "single line");
        assert_eq!(comment.span.source_text(semantic.source_text), " single line ");

        let jsdoc = jsdocs.next().unwrap();
        let comment = jsdoc.comment();
        assert_eq!(comment.parsed(), "multi\nline");
        assert_eq!(
            comment.span.source_text(semantic.source_text),
            "\n             * multi\n             * line\n             * "
        );

        let jsdoc = jsdocs.next().unwrap();
        let comment = jsdoc.comment();
        assert_eq!(comment.parsed(), "* list");
        assert_eq!(
            comment.span.source_text(semantic.source_text),
            " * list "
        );
    }

    #[test]
    fn jsdoc_tags() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            /** single line @k1 c1 @k2 */
            /**
             * multi
             * line
             * @k1 c1a
             * c1b
             * @k2 c2
             * @k3 c3a
             * c3b
             */
            ",
        );
        let mut jsdocs = semantic.jsdoc().iter_all();

        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(jsdoc.tags().len(), 2);

        let jsdoc = jsdocs.next().unwrap();
        assert_eq!(jsdoc.tags().len(), 3);
    }
}
