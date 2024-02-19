use std::collections::BTreeMap;

use oxc_ast::Comment;
use oxc_span::Span;

use super::{JSDoc, JSDocComment};

pub struct JSDocBuilder<'a> {
    source_text: &'a str,

    docs: BTreeMap<Span, Vec<JSDocComment<'a>>>,
}

impl<'a> JSDocBuilder<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text, docs: BTreeMap::default() }
    }

    pub fn build(self) -> JSDoc<'a> {
        JSDoc::new(self.docs)
    }

    pub fn save_jsdoc_comments(&mut self, span: Span, comments: &[(&u32, &Comment)]) -> bool {
        let jsdoc_comments = comments
            .iter()
            .filter(|(_, comment)| comment.is_multi_line())
            .filter_map(|(start, comment)| {
                let comment_span = Span::new(**start, comment.end());
                // Inside of marker: /*_CONTENT_*/
                let comment_content = comment_span.source_text(self.source_text);
                // Should start with "*": /**_CONTENT_*/
                if !comment_content.starts_with('*') {
                    return None;
                }
                Some(comment_content)
            })
            .map(|comment_content| {
                // Remove the very first `*`?
                // Remove the first `*` and whitespaces in each line?
                JSDocComment::new(comment_content)
            })
            .collect::<Vec<_>>();

        if !jsdoc_comments.is_empty() {
            self.docs.insert(span, jsdoc_comments);
            return true;
        }

        false
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
    ) -> Option<Vec<JSDocComment<'a>>> {
        let source_type = source_type.unwrap_or_default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .build(program)
            .semantic;
        let start = source_text.find(symbol).unwrap() as u32;
        let span = Span::new(start, start + symbol.len() as u32);
        semantic.jsdoc().get_by_span(span)
    }

    fn test_jsdoc_found(source_text: &str, symbol: &str, source_type: Option<SourceType>) {
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
            "// single
            function foo() {}",
            "/* test */function foo() {}",
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
            "/*** test */function foo() {}",
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
            "/** test */
            // noop
            function foo() {}",
            "/** test */
            /*noop*/
            function foo() {}",
            "/** foo1 */ function foo1() {} /** test */ function foo() {}",
        ];
        for source_text in source_texts {
            test_jsdoc_found(source_text, "function foo() {}", None);
        }
    }

    #[test]
    fn found_on_property_definition() {
        let source = "class Foo {
            /** jsdoc */
            bar: string;
        }";
        let source_type = SourceType::default().with_typescript(true);
        test_jsdoc_found(source, "bar: string;", Some(source_type));
    }
}
