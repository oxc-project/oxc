use std::collections::BTreeMap;
use std::rc::Rc;

use super::parser::JSDoc;
use crate::jsdoc::JSDocFinder;
use oxc_ast::{AstKind, Comment, TriviasMap};
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

pub struct JSDocBuilder<'a> {
    source_text: &'a str,
    trivias: Rc<TriviasMap>,
    attached_docs: BTreeMap<Span, Vec<JSDoc<'a>>>,
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

    pub fn build(self) -> JSDocFinder<'a> {
        let not_attached_docs = self
            .trivias
            .comments()
            .iter()
            .filter(|(start, _)| !self.leading_comments_seen.contains(start))
            .filter_map(|(start, comment)| self.parse_if_jsdoc_comment(*start, *comment))
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
    pub fn retrieve_attached_jsdoc(&mut self, kind: &AstKind<'a>) -> bool {
        if !should_attach_jsdoc(kind) {
            return false;
        }

        let span = kind.span();
        let mut leading_comments = vec![];
        // May be better to set range start for perf?
        // But once I tried, coverage tests start failing...
        for (start, comment) in self.trivias.comments().range(..span.start) {
            if self.leading_comments_seen.contains(start) {
                continue;
            }

            leading_comments.push((start, comment));
            self.leading_comments_seen.insert(*start);
        }

        let leading_jsdoc_comments = leading_comments
            .iter()
            .filter_map(|(start, comment)| self.parse_if_jsdoc_comment(**start, **comment))
            .collect::<Vec<_>>();

        if !leading_jsdoc_comments.is_empty() {
            self.attached_docs.insert(span, leading_jsdoc_comments);
            return true;
        }

        false
    }

    fn parse_if_jsdoc_comment(&self, span_start: u32, comment: Comment) -> Option<JSDoc<'a>> {
        if !comment.is_multi_line() {
            return None;
        }

        let comment_span = Span::new(span_start, comment.end());
        // Inside of marker: /*CONTENT*/ => CONTENT
        let comment_content = comment_span.source_text(self.source_text);
        // Should start with "*"
        if !comment_content.starts_with('*') {
            return None;
        }

        // Remove the very first `*`
        Some(JSDoc::new(&comment_content[1..]))
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

    #[allow(clippy::cast_possible_truncation)]
    fn get_jsdoc<'a>(
        allocator: &'a Allocator,
        source_text: &'a str,
        symbol: &'a str,
        source_type: Option<SourceType>,
    ) -> Option<Vec<JSDoc<'a>>> {
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
            ("/** test */ (() => {})", "() => {}"),
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
            ("/** test */ 1", "1"),
            ("/** test */ (1)", "(1)"),
            ("/** test */ (() => {})", "(() => {})"),
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
        assert!(c1.comment().contains("c1"));
        let _c2 = iter.next().unwrap();
        let c3 = iter.next().unwrap();
        assert!(c3.comment().contains("c3"));
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
