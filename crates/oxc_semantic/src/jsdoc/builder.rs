use std::{collections::BTreeMap, rc::Rc};

use oxc_ast::{AstKind, TriviasMap};
use oxc_span::{GetSpan, Span};

use super::{JSDoc, JSDocComment};

pub struct JSDocBuilder<'a> {
    source_text: &'a str,

    trivias: Rc<TriviasMap>,

    docs: BTreeMap<Span, JSDocComment<'a>>,
}

impl<'a> JSDocBuilder<'a> {
    pub fn new(source_text: &'a str, trivias: &Rc<TriviasMap>) -> Self {
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

    /// Find the jsdoc doc in front of this span, a.k.a leading comment
    fn find_jsdoc_comment(&self, span: Span) -> Option<&'a str> {
        for (start, comment) in self.trivias.comments().range(..span.start).rev() {
            if comment.kind().is_single_line() {
                continue;
            }

            let comment_span = Span::new(*start, comment.end());
            let comment_text = comment_span.source_text(self.source_text);

            // Comments beginning with /*, /***, or more than 3 stars will be ignored.
            let mut chars = comment_text.chars();
            if chars.next() != Some('*') {
                continue;
            }
            if chars.next() == Some('*') {
                continue;
            }

            // The comment is the leading comment of this span if there is nothing in between.
            // +2 to skip `*/` ending
            // TODO: Allow comments too
            let text_between =
                Span::new(comment.end() + 2, span.start).source_text(self.source_text);
            if text_between.chars().any(|c| !c.is_whitespace()) {
                return None;
            }

            return Some(comment_text);
        }
        None
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::{SourceType, Span};

    use crate::{jsdoc::JSDocComment, SemanticBuilder};

    #[allow(clippy::cast_possible_truncation)]
    fn get_jsdoc<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        symbol: &'a str,
        source_type: Option<SourceType>,
    ) -> Option<JSDocComment<'a>> {
        let source_type = source_type.unwrap_or_default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .build(program)
            .semantic;
        let jsdoc = semantic.jsdoc();
        let start = source_text.find(symbol).unwrap() as u32;
        let span = Span::new(start, start + symbol.len() as u32);
        jsdoc.get_by_span(span)
    }

    fn test_jsdoc(source_text: &str, symbol: &str, source_type: Option<SourceType>) {
        let allocator = Allocator::default();
        assert!(
            get_jsdoc(&allocator, source_text, symbol, source_type).is_some(),
            "{symbol} not found in {source_text}"
        );
    }

    fn test_jsdoc_not_found(source_text: &str, symbol: &str) {
        let allocator = Allocator::default();
        assert!(
            get_jsdoc(&allocator, source_text, symbol, None).is_none(),
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
            test_jsdoc(source_text, "function foo() {}", None);
        }
    }

    #[test]
    fn found_with_nearest() {
        let source_texts = [
            "
            /** @type {number} */
            /** @type {string} */
            let str;
            ",
            "
            /** @type {number} */
            let num;
            /** @type {string} */
            let str;
            ",
            // TODO: Make these work
            // "
            // /** @type {string} */
            // // ignore me
            // let str;
            // ",
            // "
            // /** @type {string} */
            // /* ignore me */
            // let str;
            // ",
        ];
        for source_text in source_texts {
            let allocator = Allocator::default();
            let symbol = "let str;";
            assert_eq!(
                get_jsdoc(&allocator, source_text, symbol, None),
                Some(JSDocComment::new("* @type {string} ")),
                "`{symbol}` not associated with the nearest jsdoc comment in {source_text}"
            );
        }
    }

    #[test]
    fn found_on_property_definition() {
        let source = "class Foo {
            /** jsdoc */
            bar: string;
        }";
        let source_type = SourceType::default().with_typescript(true);
        test_jsdoc(source, "bar: string;", Some(source_type));
    }
}
