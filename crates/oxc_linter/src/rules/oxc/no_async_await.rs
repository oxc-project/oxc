use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_async_await_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("oxc(no-async-await): Unexpected async/await")
        .with_help("Async/await is not allowed")
        .with_label(span0)
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
    restriction
);

impl Rule for NoAsyncAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func_decl) => {
                if func_decl.r#async {
                    if let Some(AstKind::ObjectProperty(obj_prop)) =
                        ctx.nodes().parent_kind(node.id())
                    {
                        ctx.diagnostic(no_async_await_diagnostic(Span::new(
                            obj_prop.span.start,
                            obj_prop.span.start + 5, // "async".len()
                        )));
                    } else {
                        ctx.diagnostic(no_async_await_diagnostic(Span::new(
                            func_decl.span.start,
                            func_decl.span.start + 5,
                        )));
                    }
                }
            }
            AstKind::ArrowFunctionExpression(arrow_expr) => {
                if arrow_expr.r#async {
                    if let Some(AstKind::ObjectProperty(obj_prop)) =
                        ctx.nodes().parent_kind(node.id())
                    {
                        ctx.diagnostic(no_async_await_diagnostic(Span::new(
                            obj_prop.span.start,
                            obj_prop.span.start + 5,
                        )));
                    } else {
                        ctx.diagnostic(no_async_await_diagnostic(Span::new(
                            arrow_expr.span.start,
                            arrow_expr.span.start + 5,
                        )));
                    };
                }
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "function foo () {}",
        "const foo = () => {}",
        "function foo () { return bar(); }",
        "class Foo { foo() {} }",
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
    ];

    Tester::new(NoAsyncAwait::NAME, pass, fail).test_and_snapshot();
}
