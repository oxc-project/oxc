use rustc_hash::FxHashMap;

use oxc_ast::{AstKind, ast::Comment};
use oxc_span::{GetSpan, Span};

use super::parser::JSDoc;

/// Raw data produced by [`JSDocBuilder::build`].
pub struct JSDocBuilderResult<'a> {
    pub attached: FxHashMap<u32, Vec<JSDoc<'a>>>,
    pub not_attached: Vec<JSDoc<'a>>,
}

#[derive(Default)]
pub struct JSDocBuilder<'a> {
    not_attached_docs: FxHashMap<u32, Vec<JSDoc<'a>>>,
    attached_docs: FxHashMap<u32, Vec<JSDoc<'a>>>,
}

impl<'a> JSDocBuilder<'a> {
    pub fn new(source_text: &'a str, comments: &[Comment]) -> Self {
        let mut not_attached_docs: FxHashMap<u32, Vec<_>> = FxHashMap::default();
        for comment in comments.iter().filter(|comment| comment.is_jsdoc()) {
            not_attached_docs
                .entry(comment.attached_to)
                .or_default()
                .push(Self::parse_jsdoc_comment(comment, source_text));
        }
        Self { not_attached_docs, attached_docs: FxHashMap::default() }
    }

    pub fn build(self) -> JSDocBuilderResult<'a> {
        JSDocBuilderResult {
            attached: self.attached_docs,
            not_attached: self.not_attached_docs.into_iter().flat_map(|(_, v)| v).collect(),
        }
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
        if should_attach_jsdoc(kind) {
            let start = kind.span().start;
            if let Some(docs) = self.not_attached_docs.remove(&start) {
                self.attached_docs.insert(start, docs);
                return true;
            }
        }
        false
    }

    fn parse_jsdoc_comment(comment: &Comment, source_text: &'a str) -> JSDoc<'a> {
        let span = comment.content_span();
        // Remove the very first `*`
        let jsdoc_span = Span::new(span.start + 1, span.end);
        let comment_content = jsdoc_span.source_text(source_text);
        JSDoc::new(comment_content, jsdoc_span)
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

        | AstKind::VariableDeclaration(_)
        | AstKind::VariableDeclarator(_)

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

        // Maybe JSX, TS related kinds should be added?
    )
}
