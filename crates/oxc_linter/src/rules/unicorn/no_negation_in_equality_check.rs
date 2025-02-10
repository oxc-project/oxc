use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_negation_in_equality_check_diagnostic(
    span: Span,
    current_operator: BinaryOperator,
    suggested_operator: BinaryOperator,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("Negated expression is not allowed in equality check.")
        .with_help(format!(
            "Remove the negation operator and use '{}' instead of '{}'.",
            suggested_operator.as_str(),
            current_operator.as_str()
        ))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNegationInEqualityCheck;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow negated expressions on the left of (in)equality checks.
    ///
    /// ### Why is this bad?
    ///
    /// A negated expression on the left of an (in)equality check is likely a mistake from trying to negate the whole condition.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (!foo === bar) {}
    ///
    /// if (!foo !== bar) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (foo !== bar) {}
    ///
    /// if (!(foo === bar)) {}
    /// ```
    NoNegationInEqualityCheck,
    unicorn,
    pedantic,
    pending
);

impl Rule for NoNegationInEqualityCheck {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::BinaryExpression(binary_expr) = node.kind() {
            let Expression::UnaryExpression(left_unary_expr) = &binary_expr.left else {
                return;
            };

            if left_unary_expr.operator != UnaryOperator::LogicalNot {
                return;
            }

            if let Expression::UnaryExpression(left_nested_unary_expr) = &left_unary_expr.argument {
                if left_nested_unary_expr.operator == UnaryOperator::LogicalNot {
                    return;
                }
            }

            if !binary_expr.operator.is_equality() {
                return;
            }

            let Some(suggested_operator) = binary_expr.operator.equality_inverse_operator() else {
                return;
            };

            ctx.diagnostic(no_negation_in_equality_check_diagnostic(
                left_unary_expr.span,
                binary_expr.operator,
                suggested_operator,
            ));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "!foo instanceof bar",
        "+foo === bar",
        "!(foo === bar)",
        "!!foo === bar",
        "!!!foo === bar",
        "foo === !bar",
    ];

    let fail = vec![
        "!foo === bar",
        "!foo !== bar",
        "!foo == bar",
        "!foo != bar",
        "
						function x() {
							return!foo === bar;
						}
					",
        "
						function x() {
							return!
								foo === bar;
							throw!
								foo === bar;
						}
					",
        "
						foo
						!(a) === b
					",
        "
						foo
						![a, b].join('') === c
					",
        "
						foo
						! [a, b].join('') === c
					",
        "
						foo
						!/* comment */[a, b].join('') === c
					",
    ];

    Tester::new(NoNegationInEqualityCheck::NAME, NoNegationInEqualityCheck::PLUGIN, pass, fail)
        .test_and_snapshot();
}
