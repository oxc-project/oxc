use oxc_ast::{
    ast::{Expression, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

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
    pedantic
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

        if !arrow_expr.expression {
            return;
        }
        let Statement::ExpressionStatement(expr_stmt) = &arrow_expr.body.statements[0] else {
            return;
        };
        if matches!(expr_stmt.expression, Expression::ParenthesizedExpression(_)) {
            ctx.diagnostic(no_unreadable_iife_diagnostic(expr_stmt.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = (bar => bar)();",
        "
            const foo = (() => {
                return a ? b : c
            })();
        ",
    ];

    let fail = vec![
        "const foo = (() => (a ? b : c))();",
        "
            const foo = (() => (
                a ? b : c
            ))();
        ",
        "
            const foo = (
                () => (
                    a ? b : c
                )
            )();
        ",
        "
            const foo = (() => (
                a, b
            ))();
        ",
        "
            const foo = (() => ({
                a: b,
            }))();
        ",
        "const foo = (bar => (bar))();",
        "
            (async () => ({
                bar,
            }))();
        ",
        "
            const foo = (async (bar) => ({
                bar: await baz(),
            }))();
        ",
        "(async () => (( {bar} )))();",
    ];

    Tester::new(NoUnreadableIife::NAME, NoUnreadableIife::PLUGIN, pass, fail).test_and_snapshot();
}
