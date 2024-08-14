use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_extra_non_null_assertion_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("extra non-null assertion").with_label(span0)
}

#[derive(Debug, Default, Clone)]
pub struct NoExtraNonNullAssertion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow extra non-null assertions.
    ///
    /// ### Why is this bad?
    /// The `!` non-null assertion operator in TypeScript is used to assert that a value's type does not include null or undefined. Using the operator any more than once on a single value does nothing.
    ///
    /// ### Example
    /// ```ts
    /// const foo: { bar: number } | null = null;
    /// const bar = foo!!!.bar;
    /// ```
    NoExtraNonNullAssertion,
    correctness
);

impl Rule for NoExtraNonNullAssertion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let expr = match node.kind() {
            AstKind::TSNonNullExpression(expr) => {
                if let Expression::TSNonNullExpression(expr) =
                    expr.expression.without_parenthesized()
                {
                    Some(expr)
                } else {
                    None
                }
            }
            AstKind::MemberExpression(expr) if expr.optional() => {
                if let Expression::TSNonNullExpression(expr) = expr.object().without_parenthesized()
                {
                    Some(expr)
                } else {
                    None
                }
            }
            AstKind::CallExpression(expr) if expr.optional => {
                if let Expression::TSNonNullExpression(expr) = expr.callee.without_parenthesized() {
                    Some(expr)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(expr) = expr {
            let end = expr.span.end - 1;
            ctx.diagnostic(no_extra_non_null_assertion_diagnostic(Span::new(end, end)));
        }
    }

    fn should_run(&self, ctx: &LintContext) -> bool {
        ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo: { bar: number } | null = null; const bar = foo!.bar; ",
        "function foo(bar: number | undefined) { const a: number = bar!; }",
        "function foo(bar?: { n: number }) { return bar?.n; }",
        "checksCounter?.textContent!.trim(); ",
        "function foo(key: string | null) { const obj = {}; return obj?.[key!]; }",
    ];

    let fail = vec![
        "const foo: { bar: number } | null = null; const bar = foo!!.bar; ",
        "function foo(bar: number | undefined) { const a: number = bar!!; }",
        "function foo(bar?: { n: number }) { return bar!?.n; }",
        "function foo(bar?: { n: number }) { return bar!?.(); }",
        "const foo: { bar: number } | null = null; const bar = (foo!)!.bar;",
        "function foo(bar?: { n: number }) { return (bar!)?.n; }",
        "function foo(bar?: { n: number }) { return (bar)!?.n; }",
        "function foo(bar?: { n: number }) { return (bar!)?.(); }",
    ];

    Tester::new(NoExtraNonNullAssertion::NAME, pass, fail).test_and_snapshot();
}
