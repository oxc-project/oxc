use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_unreadable_iife_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("IIFE with parenthesized arrow function body is considered unreadable.")
        .with_help("Rewrite the IIFE to avoid having a parenthesized arrow function body.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnreadableIife;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows IIFEs with a parenthesized arrow function body.
    ///
    /// ### Why is this bad?
    ///
    /// IIFEs with a parenthesized arrow function body are unreadable.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = (bar => (bar ? bar.baz : baz))(getBar());
    ///
    /// const foo = ((bar, baz) => ({bar, baz}))(bar, baz);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const bar = getBar();
    /// const foo = bar ? bar.baz : baz;
    ///
    /// const getBaz = bar => (bar ? bar.baz : baz);
    /// const foo = getBaz(getBar());
    ///
    /// const foo = (bar => {
    ///     return bar ? bar.baz : baz;
    /// })(getBar());
    /// ```
    NoUnreadableIife,
    unicorn,
    pedantic,
    conditional_suggestion,
    version = "0.0.19",
    short_description = "This rule disallows IIFEs with a parenthesized arrow function body.",
);

impl Rule for NoUnreadableIife {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Expression::ArrowFunctionExpression(arrow_expr) =
            &call_expr.callee.without_parentheses()
        else {
            return;
        };

        if !arrow_expr.is_expression() {
            return;
        }
        let Some(Expression::ParenthesizedExpression(parenthesized)) = arrow_expr.get_expression()
        else {
            return;
        };

        let body = parenthesized.expression.without_parentheses();
        let parenthesized_span = parenthesized.span;
        let body_span = body.span();

        ctx.diagnostic_with_suggestion(
            no_unreadable_iife_diagnostic(parenthesized_span),
            |fixer| {
                let has_comments_around_body = ctx
                    .has_comments_between(Span::new(parenthesized_span.start, body_span.start))
                    || ctx.has_comments_between(Span::new(body_span.end, parenthesized_span.end));

                if has_comments_around_body {
                    return fixer.noop();
                }

                fixer
                    .replace(
                        parenthesized_span,
                        format!("{{ return {}; }}", body_span.source_text(ctx.source_text())),
                    )
                    .with_message("Use a block statement body.")
            },
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = (bar => bar)();",
        "const foo = (() => {
                return a ? b : c
            })();",
    ];

    let fail = vec![
        "const foo = (() => (a ? b : c))();",
        "const foo = (() => (
                a ? b : c
            ))();",
        "const foo = (
                () => (
                    a ? b : c
                )
            )();",
        "const foo = (() => (/* comment */ a ? b : c))();",
        "const foo = (() => (
                a, b
            ))();",
        "const foo = (() => ({
                a: b,
            }))();",
        "const foo = (bar => (bar))();",
        "(async () => ({
                bar,
            }))();",
        "const foo = (async (bar) => ({
                bar: await baz(),
            }))();",
        "(async () => (( {bar} )))();",
    ];

    let fix = vec![
        ("const foo = (() => (a ? b : c))();", "const foo = (() => { return a ? b : c; })();"),
        (
            "const foo = (() => (
                a ? b : c
            ))();",
            "const foo = (() => { return a ? b : c; })();",
        ),
        (
            "const foo = (
                () => (
                    a ? b : c
                )
            )();",
            "const foo = (
                () => { return a ? b : c; }
            )();",
        ),
        (
            "const foo = (() => (/* comment */ a ? b : c))();",
            "const foo = (() => (/* comment */ a ? b : c))();",
        ),
        (
            "const foo = (() => (
                a, b
            ))();",
            "const foo = (() => { return a, b; })();",
        ),
        (
            "const foo = (() => ({
                a: b,
            }))();",
            "const foo = (() => { return {
                a: b,
            }; })();",
        ),
        ("const foo = (bar => (bar))();", "const foo = (bar => { return bar; })();"),
        (
            "(async () => ({
                bar,
            }))();",
            "(async () => { return {
                bar,
            }; })();",
        ),
        (
            "const foo = (async (bar) => ({
                bar: await baz(),
            }))();",
            "const foo = (async (bar) => { return {
                bar: await baz(),
            }; })();",
        ),
        ("(async () => (( {bar} )))();", "(async () => { return {bar}; })();"),
    ];

    Tester::new(NoUnreadableIife::NAME, NoUnreadableIife::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
