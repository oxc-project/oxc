use oxc_ast::ast::Expression;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoUselessConcat;

fn no_useless_concat_diagnostic(span0: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-useless-concat): Unexpected string concatenation of literals.")
        .with_help("Rewrite into one string literal")
        .with_labels([span0.into()])
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary concatenation of literals or template literals
    ///
    /// ### Why is this bad?
    ///
    /// It’s unnecessary to concatenate two strings together.
    ///
    /// ### Example
    /// ```javascript
    /// var foo = "a" + "b";
    /// ```
    NoUselessConcat,
    suspicious
);

impl Rule for NoUselessConcat {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };

        if !matches!(expr.operator, BinaryOperator::Addition) {
            return;
        }

        if expr.left.is_string_literal() && expr.right.is_string_literal() {
            ctx.diagnostic(no_useless_concat_diagnostic(expr.span));
            return;
        }

        if let Expression::BinaryExpression(left_expr) = &expr.left {
            if left_expr.right.is_string_literal() && expr.right.is_string_literal() {
                ctx.diagnostic(no_useless_concat_diagnostic(expr.span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = 1 + 1;",
        "var a = 1 * '2';",
        "var a = 1 - 2;",
        "var a = foo + bar;",
        "var a = 'foo' + bar;",
        //     "var foo = 'foo' +
        // 'bar';",
        "var string = (number + 1) + 'px';",
        "'a' + 1",
        "1 + '1'",
        "1 + `1`",
        "`1` + 1",
        "(1 + +2) + `b`",
    ];

    let fail = vec![
        "'a' + 'b'",
        "'a' +
			'b' + 'c'",
        "foo + 'a' + 'b'",
        "'a' + 'b' + 'c'",
        "(foo + 'a') + ('b' + 'c')",
        "`a` + 'b'",
        "`a` + `b`",
        "foo + `a` + `b`",
    ];

    Tester::new(NoUselessConcat::NAME, pass, fail).test_and_snapshot();
}
