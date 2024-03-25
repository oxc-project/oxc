use std::collections::BTreeMap;
use std::rc::Rc;

use super::parser::JSDoc;
use crate::jsdoc::JSDocFinder;
use oxc_ast::{AstKind, CommentKind, Trivias};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

pub struct JSDocBuilder<'a> {
    source_text: &'a str,
    trivias: Rc<Trivias>,
    attached_docs: BTreeMap<Span, Vec<JSDoc<'a>>>,
    leading_comments_seen: FxHashSet<u32>,
}

impl<'a> JSDocBuilder<'a> {
    pub fn new(source_text: &'a str, trivias: &Rc<Trivias>) -> Self {
        Self {
            source_text,
            trivias: Rc::clone(trivias),
            attached_docs: BTreeMap::default(),
            leading_comments_seen: FxHashSet::default(),
        }
    }

    pub fn build(self) -> JSDocFinder<'a> {
        let not_attached_docs = self
            .trivias
            .comments()
            .filter(|(_, span)| !self.leading_comments_seen.contains(&span.start))
            .filter_map(|(kind, span)| self.parse_if_jsdoc_comment(kind, span))
            .collect::<Vec<_>>();

        JSDocFinder::new(self.attached_docs, not_attached_docs)
    }

    // ## Current architecture
    //
    // - 1) At semantic build time, visit each node and flag it if 1 or more JSDoc comments found
    // - 2) At runtime (usecases like oxlint), reference that flag from the visited node
    //
    // Basically, this speeds up the runtime usecases, but there is a trade-off.
    //
    // ## Only certain nodes can have a JSDoc
    //
    // For perf reasons, not every node is checked.
    // The benchmark says that perf actually drops by -3~4% if we check every kind.
    //
    // This means that some JSDoc comments may not be parsed as originally written.
    // (In the first place, comments can be written anywhere,
    //  although some may already be inconsistent when converted from Token to AST nodes).
    //
    // Check the `should_attach_jsdoc()` function below to see which nodes are listed.
    //
    // ## Usecase matters
    //
    // "Where to write comments and what meaning you want them to have" depends entirely on the usecase.
    //
    // Consider the following common example and some usecases.
    //
    // ```js
    // /** @param {string} x */
    // function foo(x) {}
    // ```
    //
    // In the current implementation, this JSDoc is attached to the `FunctionDeclaration'.
    //
    // - How to validate parameter `x` should have `@param` JSDoc?
    //
    // In this plugin-jsdoc usecase,
    //  visit `FunctionDeclaration`, find `params.items`, get attached JSDoc, and ... OK.
    //
    // Then how about this?
    //
    // ```js
    // /** @param {string} x */
    // const bar = (x) => {}
    // ```
    //
    // We might want to validate this by visiting `ArrowFunctionExpression`.
    // But this JSDoc will be attached to the `VariableDeclaration'.
    //
    // More examples...
    //
    // ```js
    // /** @param {string} x */
    // const a = ((x) => {}), // extra `ParenthesizedExpression`
    //   /** @param {string} x */
    //   b = (x) => {} // `VariableDeclarator` has JSDoc
    // ```
    //
    // So we need extra work to find+ask parent (or sibling?) node until desired JSDoc is found.
    //
    // - How to get type information when visiting `FormalParameter`(or its `Identifier`)?
    //
    // This is another example, but it's also necessary to find+ask parent.
    //
    // Anyway, extra work at runtime seems to be necessary in many cases,
    //  especially for `JSDoc.tags` related things.
    //
    // ## To make the runtime logic consistent
    //
    // The semantic side needs to be versatile, intuitive and expectable.
    // And we also want to avoid having 2 tuning points.
    //
    // Therefore, the `should_attach_jsdoc()` function and its candidates should be carefully listed.
    //
    // As many reasonable types as possible should be listed, as long as it does not affect performance...!
    //
    // If one day we want to add a performance-affecting kind,
    // we might as well give up pre-flagging architecture itself?
    pub fn retrieve_attached_jsdoc(&mut self, kind: &AstKind<'a>) -> bool {
        if !should_attach_jsdoc(kind) {
            return false;
        }

        let span = kind.span();
        let mut leading_comments = vec![];
        // May be better to set range start for perf?
        // But once I tried, coverage tests start failing...
        for (start, comment) in self.trivias.comments_range(..span.start) {
            if self.leading_comments_seen.contains(start) {
                continue;
            }

            leading_comments.push((start, comment));
            self.leading_comments_seen.insert(*start);
        }

        let leading_jsdoc_comments = leading_comments
            .into_iter()
            .filter_map(|(start, comment)| {
                self.parse_if_jsdoc_comment(comment.kind, Span::new(*start, comment.end))
            })
            .collect::<Vec<_>>();

        if !leading_jsdoc_comments.is_empty() {
            self.attached_docs.insert(span, leading_jsdoc_comments);
            return true;
        }

        false
    }

    fn parse_if_jsdoc_comment(&self, kind: CommentKind, comment_span: Span) -> Option<JSDoc<'a>> {
        if !kind.is_multi_line() {
            return None;
        }

        // Inside of marker: /*CONTENT*/ => CONTENT
        let comment_content = comment_span.source_text(self.source_text);
        // Should start with "*"
        if !comment_content.starts_with('*') {
            return None;
        }

        // Remove the very first `*`
        let jsdoc_span = Span::new(comment_span.start + 1, comment_span.end);
        Some(JSDoc::new(&comment_content[1..], jsdoc_span))
    }
}

// As noted above, only certain nodes can have JSDoc comments.
// But as many kinds as possible should be added, without affecting performance.
//
// It's a bit hard to explain, but theoretically the more outer ones should be listed.
//
// From a linter point of view, basically only declarations are needed.
// Other kinds, such as statements, act as tie-breakers between them.
#[rustfmt::skip]
fn should_attach_jsdoc(kind: &AstKind) -> bool {
    matches!(kind,
        // This list order comes from oxc_ast/ast_kind.rs
          AstKind::BlockStatement(_)
        | AstKind::BreakStatement(_)
        | AstKind::ContinueStatement(_)
        | AstKind::DebuggerStatement(_)
        | AstKind::DoWhileStatement(_)
        | AstKind::EmptyStatement(_)
        | AstKind::ExpressionStatement(_)
        | AstKind::ForInStatement(_)
        | AstKind::ForOfStatement(_)
        | AstKind::ForStatement(_)
        | AstKind::IfStatement(_)
        | AstKind::LabeledStatement(_)
        | AstKind::ReturnStatement(_)
        | AstKind::SwitchStatement(_)
        | AstKind::ThrowStatement(_)
        | AstKind::TryStatement(_)
        | AstKind::WhileStatement(_)
        | AstKind::WithStatement(_)

        | AstKind::SwitchCase(_)
        | AstKind::CatchClause(_)
        | AstKind::FinallyClause(_)

        | AstKind::VariableDeclaration(_)
        | AstKind::VariableDeclarator(_)

        | AstKind::UsingDeclaration(_)

        // This is slow
        // | AstKind::IdentifierName(_)

        | AstKind::ArrowFunctionExpression(_)
        | AstKind::ObjectExpression(_)
        | AstKind::ParenthesizedExpression(_)

        | AstKind::ObjectProperty(_)

        | AstKind::Function(_)
        | AstKind::FormalParameter(_)

        | AstKind::Class(_)
        | AstKind::MethodDefinition(_)
        | AstKind::PropertyDefinition(_)
        | AstKind::StaticBlock(_)

        | AstKind::Decorator(_)

        | AstKind::ExportAllDeclaration(_)
        | AstKind::ExportDefaultDeclaration(_)
        | AstKind::ExportNamedDeclaration(_)
        | AstKind::ImportDeclaration(_)
        | AstKind::ModuleDeclaration(_)

        // Maybe JSX, TS related kinds should be added?
    )
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::{SourceType, Span};

    use super::JSDoc;
    use crate::{Semantic, SemanticBuilder};

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

    fn get_jsdocs<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        symbol: &'a str,
        source_type: Option<SourceType>,
    ) -> Option<Vec<JSDoc<'a>>> {
        let semantic = build_semantic(allocator, source_text, source_type);
        let start = u32::try_from(source_text.find(symbol).unwrap_or(0)).unwrap();
        let span = Span::new(start, start + u32::try_from(symbol.len()).unwrap());
        semantic.jsdoc().get_all_by_span(span)
    }

    fn test_jsdoc_found(source_text: &str, symbol: &str, source_type: Option<SourceType>) {
        let allocator = Allocator::default();
        assert!(
            get_jsdocs(&allocator, source_text, symbol, source_type).is_some(),
            "JSDoc should found for\n  {symbol} \nin\n  {source_text}"
        );
    }

    fn test_jsdoc_not_found(source_text: &str, symbol: &str) {
        let allocator = Allocator::default();
        assert!(
            get_jsdocs(&allocator, source_text, symbol, None).is_none(),
            "JSDoc should NOT found for\n  {symbol} \nin\n  {source_text}"
        );
    }

    #[test]
    fn not_found() {
        let source_texts = [
            ("function f1() {}", "function f1() {}"),
            ("// test", "function f2() {}"),
            ("/* test */function f3() {}", "function f3() {}"),
            ("/** for 4a */ ; function f4a() {} function f4b() {}", "function f4b() {}"),
            ("function f4a() {} /** for 4b */ ; function f4b() {} ", "function f4a() {}"),
            ("function f5() {} /** test */", "function f5() {}"),
            (
                "/** for o */
                const o = {
                    f6() {}
                };",
                "f6() {}",
            ),
            ("/** for () */ (() => {})", "() => {}"),
            ("/** for ; */ ; let v1;", "let v1;"),
            ("/** for let v2 */ let v2 = () => {};", "() => {}"),
            ("/** for if */ if (true) { let v3; })", "let v3;"),
            (
                "class C1 {
                    /** for m1 */
                    m1() {}
                    m2() {}
                }",
                "m2() {}",
            ),
        ];
        for (source_text, target) in source_texts {
            test_jsdoc_not_found(source_text, target);
        }
    }

    #[test]
    fn found() {
        let source_texts = [
            ("/** test */function f1() {}", "function f1() {}"),
            ("/*** test */function f2() {}", "function f2() {}"),
            (
                "
            /** test */
        function f3() {}",
                "function f3() {}",
            ),
            (
                "/** test */


                function f4() {}",
                "function f4() {}",
            ),
            (
                "/**
             * test
             * */
            function f5() {}",
                "function f5() {}",
            ),
            (
                "/** test */
                // ---
                function f6() {}",
                "function f6() {}",
            ),
            (
                "/** test */
                /* -- */
                function f7() {}",
                "function f7() {}",
            ),
            (
                "/** test */
                /** test2 */
                function f8() {}",
                "function f8() {}",
            ),
            (
                "/** test */ /** test2 */
                function f9() {}",
                "function f9() {}",
            ),
            (
                "/** for f10 */ function f10() {} /** for f11 */ function f11() {}",
                "function f11() {}",
            ),
            (
                "const o = {
                    /** for f12 */
                    f12() {}
                };",
                "f12() {}",
            ),
            ("/** test */ (() => {})", "(() => {})"),
            ("/** test */ let v1 = 1", "let v1 = 1"),
            ("let v2a = 1, /** for v2b */ v2b = 2", "v2b = 2"),
            ("/** for v3a */ const v3a = 1, v3b = 2;", "const v3a = 1, v3b = 2;"),
            ("/** test */ export const e1 = 1;", "export const e1 = 1;"),
            ("/** test */ export default {};", "export default {};"),
            ("/** test */ import 'i1'", "import 'i1'"),
            ("/** test */ import I from 'i2'", "import I from 'i2'"),
            ("/** test */ import { I } from 'i3'", "import { I } from 'i3'"),
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
        let jsdocs = get_jsdocs(&allocator, source_text, symbol, None);

        assert!(jsdocs.is_some());
        let jsdocs = jsdocs.unwrap();
        assert_eq!(jsdocs.len(), 3);

        // Should be [farthest, ..., nearest]
        let mut iter = jsdocs.iter();
        let c1 = iter.next().unwrap();
        assert_eq!(c1.comment(), "c1");
        let c2 = iter.next().unwrap();
        assert_eq!(c2.comment(), "c2");
        let c3 = iter.next().unwrap();
        assert_eq!(c3.comment(), "c3");
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

            /**/ // noop and noop

            /** 7. Not attached but collected! */
            ",
            Some(SourceType::default()),
        );
        assert_eq!(semantic.jsdoc().iter_all().count(), 7);
    }
}
