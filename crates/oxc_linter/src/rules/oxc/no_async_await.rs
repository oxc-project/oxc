use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_async_await_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected async/await")
        .with_help("Async/await is not allowed")
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
    restriction
);

impl Rule for NoAsyncAwait {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(func_decl) => {
                if func_decl.r#async {
                    report(node.id(), func_decl.span, ctx);
                }
            }
            AstKind::ArrowFunctionExpression(arrow_expr) => {
                if arrow_expr.r#async {
                    report(node.id(), arrow_expr.span, ctx);
                }
            }
            _ => {}
        }
    }
}

fn report(node_id: NodeId, func_span: Span, ctx: &LintContext<'_>) {
    /// "async".len()
    const ASYNC_LEN: u32 = 5;

    let parent = ctx.nodes().parent_kind(node_id);
    if let Some(AstKind::ObjectProperty(obj_prop)) = parent {
        ctx.diagnostic(no_async_await_diagnostic(Span::sized(obj_prop.span.start, ASYNC_LEN)));
    } else {
        ctx.diagnostic(no_async_await_diagnostic(Span::sized(func_span.start, ASYNC_LEN)));
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
        // FIXME: diagnostics on method `foo` have incorrect spans
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
        // this one is fine
        "
        const obj = {
            async foo() {}
        }
        ",
    ];

    Tester::new(NoAsyncAwait::NAME, pass, fail).test_and_snapshot();
}
