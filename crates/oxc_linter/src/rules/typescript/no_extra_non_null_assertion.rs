use oxc_ast::{
    AstKind,
    ast::{ChainElement, Expression, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::outermost_paren_parent,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_extra_non_null_assertion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("extra non-null assertion")
        .with_help("Remove the redundant non-null assertion operator (`!`).")
        .with_note("The non-null assertion operator in TypeScript, written as `!`, tells the compiler that an expression is definitely not `null` or `undefined` at that point. Chaining multiple non-null assertions on the same expression does not provide any additional safety and is redundant.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoExtraNonNullAssertion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow extra non-null assertions.
    ///
    /// ### Why is this bad?
    ///
    /// The `!` non-null assertion operator in TypeScript is used to assert that a value's type
    /// does not include `null` or `undefined`. Using the operator any more than once on a single value
    /// does nothing.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// const foo: { bar: number } | null = null;
    /// const bar = foo!!!.bar;
    /// ```
    ///
    /// ```ts
    /// function foo(bar: number | undefined) {
    ///   const bar: number = bar!!!;
    /// }
    /// ```
    ///
    /// ```ts
    /// function foo(bar?: { n: number }) {
    ///   return bar!?.n;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const foo: { bar: number } | null = null;
    /// const bar = foo!.bar;
    /// ```
    ///
    /// ```ts
    /// function foo(bar: number | undefined) {
    ///  const bar: number = bar!;
    /// }
    /// ```
    ///
    /// ```ts
    /// function foo(bar?: { n: number }) {
    ///   return bar?.n;
    /// }
    /// ```
    NoExtraNonNullAssertion,
    typescript,
    correctness,
    fix,
    version = "0.0.6",
);

impl Rule for NoExtraNonNullAssertion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSNonNullExpression(non_null_expr) = node.kind() else {
            return;
        };

        let Some(parent) = outermost_paren_parent(node, ctx.semantic()) else {
            return;
        };

        let is_extra_non_null_assertion = match parent.kind() {
            AstKind::TSNonNullExpression(_) => true,
            _ if let Some(member_expr) = parent.kind().as_member_expression_kind() => {
                member_expr.optional()
                    && matches!(
                        member_expr.object().without_parentheses(),
                        Expression::TSNonNullExpression(expr) if expr.span == non_null_expr.span
                    )
            }
            AstKind::CallExpression(expr) if expr.optional => {
                matches!(
                    expr.callee.without_parentheses(),
                    Expression::TSNonNullExpression(expr) if expr.span == non_null_expr.span
                )
            }
            AstKind::ChainExpression(expr) => match &expr.expression {
                chain_element @ match_member_expression!(ChainElement) => {
                    let member_expr = chain_element.to_member_expression();
                    member_expr.optional()
                        && matches!(
                            member_expr.object().without_parentheses(),
                            Expression::TSNonNullExpression(expr) if expr.span == non_null_expr.span
                        )
                }
                ChainElement::CallExpression(expr) if expr.optional => {
                    matches!(
                        expr.callee.without_parentheses(),
                        Expression::TSNonNullExpression(expr) if expr.span == non_null_expr.span
                    )
                }
                _ => false,
            },
            _ => false,
        };

        if is_extra_non_null_assertion {
            let span = Span::sized(non_null_expr.span.end - 1, 1);
            ctx.diagnostic_with_fix(no_extra_non_null_assertion_diagnostic(span), |fixer| {
                fixer.delete_range(span)
            });
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
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
        "const foo: { bar: number } | null = null; const bar = foo!!!.bar;",
        "const foo: { bar: number } | null = null; const bar = foo!!.bar; ",
        "function foo(bar: number | undefined) { const a: number = bar!!; }",
        "function foo(bar?: { n: number }) { return bar!?.n; }",
        "function foo(bar?: { n: number }) { return bar!?.(); }",
        "const foo: { bar: number } | null = null; const bar = (foo!)!.bar;",
        "function foo(bar?: { n: number }) { return (bar!)?.n; }",
        "function foo(bar?: { n: number }) { return (bar)!?.n; }",
        "function foo(bar?: { n: number }) { return (bar!)?.(); }",
    ];

    let fix = vec![
        (
            "
            const foo: { bar: number } | null = null;
            const bar = foo!!!.bar;
                  ",
            "
            const foo: { bar: number } | null = null;
            const bar = foo!!.bar;
                  ",
        ),
        (
            "
            const foo: { bar: number } | null = null;
            const bar = foo!!.bar;
                  ",
            "
            const foo: { bar: number } | null = null;
            const bar = foo!.bar;
                  ",
        ),
        (
            "
            function foo(bar: number | undefined) {
              const a: number = bar!!;
            }
                  ",
            "
            function foo(bar: number | undefined) {
              const a: number = bar!;
            }
                  ",
        ),
        (
            "
            function foo(bar?: { n: number }) {
              return bar!?.n;
            }
                  ",
            "
            function foo(bar?: { n: number }) {
              return bar?.n;
            }
                  ",
        ),
        (
            "
            function foo(bar?: { n: number }) {
              return bar!?.();
            }
                  ",
            "
            function foo(bar?: { n: number }) {
              return bar?.();
            }
                  ",
        ),
        (
            "
            const foo: { bar: number } | null = null;
            const bar = (foo!)!.bar;
                  ",
            "
            const foo: { bar: number } | null = null;
            const bar = (foo)!.bar;
                  ",
        ),
        (
            "
            function foo(bar?: { n: number }) {
              return (bar!)?.n;
            }
                  ",
            "
            function foo(bar?: { n: number }) {
              return (bar)?.n;
            }
                  ",
        ),
        (
            "
            function foo(bar?: { n: number }) {
              return (bar)!?.n;
            }
                  ",
            "
            function foo(bar?: { n: number }) {
              return (bar)?.n;
            }
                  ",
        ),
        (
            "
            function foo(bar?: { n: number }) {
              return (bar!)?.();
            }
                  ",
            "
            function foo(bar?: { n: number }) {
              return (bar)?.();
            }
                  ",
        ),
    ];

    Tester::new(NoExtraNonNullAssertion::NAME, NoExtraNonNullAssertion::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
