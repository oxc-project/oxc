use std::{collections::BTreeMap, rc::Rc};

use oxc_ast::{AstKind, GetSpan, Span, Trivias};

use super::{JSDoc, JSDocComment};

pub struct JSDocBuilder<'a> {
    source_text: &'a str,

    trivias: Rc<Trivias>,

    docs: BTreeMap<Span, JSDocComment<'a>>,
}

impl<'a> JSDocBuilder<'a> {
    pub fn new(source_text: &'a str, trivias: &Rc<Trivias>) -> Self {
        Self { source_text, trivias: Rc::clone(trivias), docs: BTreeMap::default() }
    }

    pub fn build(self) -> JSDoc<'a> {
        JSDoc::new(self.docs)
    }

    /// Save the span if the given kind has a jsdoc comment attached
    pub fn retrieve_jsdoc_comment(&mut self, kind: AstKind<'a>) -> bool {
        if !kind.is_declaration() {
            return false;
        }
        let span = kind.span();
        let comment_text = self.find_jsdoc_comment(span);
        if let Some(comment_text) = comment_text {
            self.docs.insert(span, JSDocComment::new(comment_text));
        }
        comment_text.is_some()
    }

    /// Find the jsdoc doc in frontend this span, a.k.a leading comment
    fn find_jsdoc_comment(&self, span: Span) -> Option<&'a str> {
        let (start, comment) = self.trivias.comments().range(..span.start).next()?;

        if comment.is_single_line() {
            return None;
        }

        let comment_text = Span::new(*start, comment.end()).source_text(self.source_text);

        // Comments beginning with /*, /***, or more than 3 stars will be ignored.
        let mut chars = comment_text.chars();
        if chars.next() != Some('*') {
            return None;
        }
        if chars.next() == Some('*') {
            return None;
        }

        // The comment is the leading comment of this span if there is nothing in between.
        // +2 to skip `*/` ending
        let text_between = Span::new(comment.end() + 2, span.start).source_text(self.source_text);
        if text_between.chars().any(|c| !c.is_whitespace()) {
            return None;
        }

        Some(comment_text)
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_ast::{SourceType, Span};
    use oxc_parser::Parser;

    use crate::{jsdoc::JSDocComment, SemanticBuilder};

    #[allow(clippy::cast_possible_truncation)]
    fn get_jsdoc<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        symbol: &'a str,
    ) -> Option<JSDocComment<'a>> {
        let source_type = SourceType::default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic =
            SemanticBuilder::new(source_text, source_type, &ret.trivias).build(program).semantic;
        let jsdoc = semantic.jsdoc();
        let start = source_text.find(symbol).unwrap() as u32;
        let span = Span::new(start, start + symbol.len() as u32);
        jsdoc.get_by_span(span)
    }

    fn test_jsdoc(source_text: &str, symbol: &str) {
        let allocator = Allocator::default();
        assert!(
            get_jsdoc(&allocator, source_text, symbol).is_some(),
            "{symbol} not found in {source_text}"
        );
    }

    fn test_jsdoc_not_found(source_text: &str, symbol: &str) {
        let allocator = Allocator::default();
        assert!(
            get_jsdoc(&allocator, source_text, symbol).is_none(),
            "{symbol} found in {source_text}"
        );
    }

    #[test]
    fn not_found() {
        let source_texts = [
            "function foo() {}",
            "/* test */function foo() {}",
            "/*** test */function foo() {}",
            "/** test */ ; function foo() {}",
            "/** test */ function foo1() {} function foo() {}",
        ];
        for source_text in source_texts {
            test_jsdoc_not_found(source_text, "function foo() {}");
        }
    }

    #[test]
    fn found() {
        let source_texts = [
            "/** test */function foo() {}",
            "
            /** test */
        function foo() {}",
            "/** test */
                function foo() {}",
            "/**
             * test
             * */
            function foo() {}",
            "/** test */
            function foo() {}",
        ];
        for source_text in source_texts {
            test_jsdoc(source_text, "function foo() {}");
        }
    }
}
