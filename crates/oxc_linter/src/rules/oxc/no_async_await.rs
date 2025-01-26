use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_async_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("async is not allowed")
        .with_help("Remove the `async` keyword")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoAsyncAwait;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of async/await.
    ///
    /// ### Example
    /// ```javascript
    /// async function foo() {
    ///    await bar();
    ///    return baz();
    /// }
    /// ```
    NoAsyncAwait,
    oxc,
    restriction
);

impl Rule for NoAsyncAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func_decl) => {
                if func_decl.r#async {
                    let parent_kind = ctx.nodes().parent_kind(node.id());
                    let async_span = match &func_decl.id {
                        // named function like `async function run() {}`
                        Some(id) => Span::new(func_decl.span.start, id.span.end),
                        // anonymous function like `async function() {}`
                        None => match parent_kind {
                            // Actually part of a method definition like:
                            // ```
                            // class Foo {
                            //     async bar() {}
                            // }
                            // ```
                            Some(AstKind::MethodDefinition(method_def)) => {
                                Span::new(method_def.span.start, method_def.key.span().start)
                            }
                            // The function is part of an object property like:
                            // ```
                            // const obj = {
                            //     async foo() {}
                            // };
                            // ```
                            Some(AstKind::ObjectProperty(obj_prop)) => {
                                Span::new(obj_prop.span.start, obj_prop.key.span().start)
                            }
                            _ => func_decl.span,
                        },
                    };
                    report_on_async_span(async_span, ctx);
                }
            }
            AstKind::ArrowFunctionExpression(arrow_expr) => {
                if arrow_expr.r#async {
                    let async_span = Span::new(arrow_expr.span.start, arrow_expr.params.span.start);
                    report_on_async_span(async_span, ctx);
                }
            }
            _ => {}
        }
    }
}

/// "async".len()
const ASYNC_LEN: u32 = 5;

#[allow(clippy::cast_possible_truncation)]
fn report_on_async_span(async_span: Span, ctx: &LintContext<'_>) {
    // find the `async` keyword within the span and report on it
    let Some(async_keyword_offset) = ctx.source_range(async_span).find("async") else {
        return;
    };
    let async_keyword_span = Span::sized(async_span.start + async_keyword_offset as u32, ASYNC_LEN);
    ctx.diagnostic(no_async_diagnostic(async_keyword_span));
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo () {}",
        "const foo = () => {}",
        "function foo () { return bar(); }",
        "class Foo { foo() {} }",
        "class async { }",
        "const async = {};",
        "class async { async() { async(); } }",
    ];

    let fail = vec![
        "async function foo() {}",
        "const foo = async () => {}",
        "async () => {}",
        "const test = async () => {};",
        "
            const test = {
                async test() {}
            };
        ",
        "
        class Foo {
            async foo() {}
        }
        ",
        "
        class Foo {
            public async foo() {}
        }
        ",
        "
        const obj = {
            async foo() {}
        }
        ",
        "
        class async {
            async async() {
                async();
            }
        }
        ",
        "
        class async {
            async async() {
                function async() {
                    const async = {
                        async: async () => {},
                    }
                }
            }
        }
        ",
    ];

    Tester::new(NoAsyncAwait::NAME, NoAsyncAwait::PLUGIN, pass, fail).test_and_snapshot();
}
