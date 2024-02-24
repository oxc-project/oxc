use std::collections::BTreeMap;
use std::rc::Rc;

use oxc_ast::{AstKind, Comment, TriviasMap};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use super::{JSDoc, JSDocComment};

pub struct JSDocBuilder<'a> {
    source_text: &'a str,
    trivias: Rc<TriviasMap>,
    attached_docs: BTreeMap<Span, Vec<JSDocComment<'a>>>,
    leading_comments_seen: FxHashSet<u32>,
}

impl<'a> JSDocBuilder<'a> {
    pub fn new(source_text: &'a str, trivias: &Rc<TriviasMap>) -> Self {
        Self {
            source_text,
            trivias: Rc::clone(trivias),
            attached_docs: BTreeMap::default(),
            leading_comments_seen: FxHashSet::default(),
        }
    }

    pub fn build(self) -> JSDoc<'a> {
        let not_attached_docs = self
            .trivias
            .comments()
            .iter()
            .filter(|(start, _)| !self.leading_comments_seen.contains(start))
            .filter_map(|(start, comment)| self.parse_if_jsdoc_comment(*start, *comment))
            .collect::<Vec<_>>();

        JSDoc::new(self.attached_docs, not_attached_docs)
    }

    // This process is done in conjunction with the `semantic.build()`.
    // This means that it's done "before" each use case (e.g. execute rule in oxlint) runs.
    //
    // If you need to control this attaching logic (e.g. by rule configurations), one of these would be necessary.
    // - 1. Give up pre-attaching here and leave it for use cases
    // - 2. Attach it more broadly(= loosely) here (may cause performance impact), so that it can be narrowed down later
    //
    // Since there is no reliable spec for JSDoc, there are some naive topics to consider:
    //
    // Q. Which node to attach JSDoc to?
    // A. Each implementation decides according to its own use case.
    //
    // For example, TypeScript tries to target quite broadly nodes.
    // > https://github.com/microsoft/TypeScript/blob/d04e3489b0d8e6bc9a8a9396a633632a5a467328/src/compiler/utilities.ts#L4195
    //
    // In case of `eslint-plugin-jsdoc`, its targets can be freely changed by rule configurations!
    // (But, default is only about function related nodes.)
    // > https://github.com/gajus/eslint-plugin-jsdoc/blob/e948bee821e964a92fbabc01574eca226e9e1252/src/iterateJsdoc.js#L2517-L2536
    //
    // Q. How do we attach JSDoc to that node?
    // A. Also depends on the implementation.
    //
    // In the case of TypeScript (they have specific AST node for JSDoc and an `endOfFileToken`),
    // some AST nodes have the `jsDoc` property and multiple `JSDocComment`s are attached.
    //
    // In the case of `eslint-plugin-jsdoc` (`@es-joy/jsdoccomment` is used)
    // tries to get a only single nearest comment, with some exception handling.
    //
    // It is hard to say how we should behave as OXC Semantic, but the current implementation is,
    // - Intuitive TypeScript-like attaching strategy
    // - Provide `get_one` or `get_all` APIs for each use case
    //
    // Of course, this can be changed in the future.
    pub fn retrieve_attached_jsdoc(&mut self, kind: &AstKind<'a>) -> bool {
        // This may be diffed compare to TypeScript's `canHaveJSDoc()`, should adjust if needed
        if !(kind.is_statement()
            || kind.is_declaration()
            || matches!(kind, AstKind::ParenthesizedExpression(_)))
        {
            return false;
        }

        // 1. Retrieve every kind of leading comments for this node
        let span = kind.span();
        let mut leading_comments = vec![];
        for (start, comment) in self.trivias.comments().range(..span.start) {
            if !self.leading_comments_seen.contains(start) {
                leading_comments.push((start, comment));
            }
            self.leading_comments_seen.insert(*start);
        }

        // 2. Filter and parse JSDoc comments only
        let leading_jsdoc_comments = leading_comments
            .iter()
            .filter_map(|(start, comment)| self.parse_if_jsdoc_comment(**start, **comment))
            .collect::<Vec<_>>();

        // 3. Save and return `true` to mark JSDoc flag
        if !leading_jsdoc_comments.is_empty() {
            self.attached_docs.insert(span, leading_jsdoc_comments);
            return true;
        }

        false
    }

    fn parse_if_jsdoc_comment(
        &self,
        span_start: u32,
        comment: Comment,
    ) -> Option<JSDocComment<'a>> {
        if !comment.is_multi_line() {
            return None;
        }

        let comment_span = Span::new(span_start, comment.end());
        // Inside of marker: /*_CONTENT_*/
        let comment_content = comment_span.source_text(self.source_text);
        // Should start with "*": /**_CONTENT_*/
        if !comment_content.starts_with('*') {
            return None;
        }

        // Should remove the very first `*`?
        Some(JSDocComment::new(comment_content))
    }
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::{SourceType, Span};

    use crate::{jsdoc::JSDocComment, Semantic, SemanticBuilder};

    fn build_semantic<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        source_type: Option<SourceType>,
    ) -> Semantic<'a> {
        let source_type = source_type.unwrap_or_default();
        let ret = Parser::new(allocator, source_text, source_type).parse();
        let program = allocator.alloc(ret.program);
        let semantic = SemanticBuilder::new(source_text, source_type)
            .with_trivias(ret.trivias)
            .build(program)
            .semantic;
        semantic
    }

    #[allow(clippy::cast_possible_truncation)]
    fn get_jsdoc<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        symbol: &'a str,
        source_type: Option<SourceType>,
    ) -> Option<Vec<JSDocComment<'a>>> {
        let semantic = build_semantic(allocator, source_text, source_type);
        let start = source_text.find(symbol).unwrap_or(0) as u32;
        let span = Span::new(start, start + symbol.len() as u32);
        semantic.jsdoc().get_all_by_span(span)
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
            ("function foo() {}", "function foo() {}"),
            ("// test", "function foo() {}"),
            ("function foo() {}", "function foo() {}"),
            ("/* test */function foo() {}", "function foo() {}"),
            ("/** test */ ; function foo() {}", "function foo() {}"),
            ("/** test */ function foo1() {} function foo() {}", "function foo() {}"),
            ("function foo() {} /** test */", "function foo() {}"),
        ];
        for (source_text, target) in source_texts {
            test_jsdoc_not_found(source_text, target);
        }
    }

    #[test]
    fn found() {
        let source_texts = [
            ("/** test */function foo() {}", "function foo() {}"),
            ("/*** test */function foo() {}", "function foo() {}"),
            (
                "
            /** test */
        function foo() {}",
                "function foo() {}",
            ),
            (
                "/** test */


                function foo() {}",
                "function foo() {}",
            ),
            (
                "/**
             * test
             * */
            function foo() {}",
                "function foo() {}",
            ),
            (
                "/** test */
            function foo() {}",
                "function foo() {}",
            ),
            (
                "/** test */
            // noop
            function foo() {}",
                "function foo() {}",
            ),
            (
                "/** test */
            /*noop*/
            function foo() {}",
                "function foo() {}",
            ),
            ("/** foo1 */ function foo1() {} /** test */ function foo() {}", "function foo() {}"),
        ];
        for (source_text, target) in source_texts {
            test_jsdoc_found(source_text, target, None);
        }
    }

    #[test]
    fn found_ts() {
        let source_texts = [(
            "class Foo {
            /** jsdoc */
            bar: string;
        }",
            "bar: string;",
        )];

        let source_type = SourceType::default().with_typescript(true);
        for (source_text, target) in source_texts {
            test_jsdoc_found(source_text, target, Some(source_type));
        }
    }

    #[test]
    fn get_all_by_span_order() {
        let allocator = Allocator::default();
        let source_text = r"
            /**c0*/
            function foo() {}

            /**c1*/
            /* noop */
            /**c2*/
            // noop
            /**c3*/
            const x = () => {};
        ";
        let symbol = "const x = () => {};";
        let jsdocs = get_jsdoc(&allocator, source_text, symbol, None);

        assert!(jsdocs.is_some());
        let jsdocs = jsdocs.unwrap();
        assert_eq!(jsdocs.len(), 3);

        // Should be [farthest, ..., nearest]
        let mut iter = jsdocs.iter();
        let c1 = iter.next().unwrap();
        assert!(c1.comment.contains("c1"));
        let _c2 = iter.next().unwrap();
        let c3 = iter.next().unwrap();
        assert!(c3.comment.contains("c3"));
    }

    #[test]
    fn get_all_jsdoc() {
        let allocator = Allocator::default();
        let semantic = build_semantic(
            &allocator,
            r"
            // noop
            /** 1. ; */
            // noop
            ;
            /** 2. class X {} *//** 3. class X {} */
            class X {
                /** 4. foo */
                foo = /** 5. () */ (() => {});
            }

            /** 6. let x; */
            /* noop */

            let x;

            /** 7. Not attached but collected! */
            ",
            Some(SourceType::default()),
        );
        assert_eq!(semantic.jsdoc().iter_all().count(), 7);
    }
}
