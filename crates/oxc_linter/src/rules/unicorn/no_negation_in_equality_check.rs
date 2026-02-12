use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::identifier::is_identifier_start;
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{AstNode, ast_util, context::LintContext, rule::Rule};

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
    /// ### Examples
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
    suggestion,
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

            if let Expression::UnaryExpression(left_nested_unary_expr) = &left_unary_expr.argument
                && left_nested_unary_expr.operator == UnaryOperator::LogicalNot
            {
                return;
            }

            if !binary_expr.operator.is_equality() {
                return;
            }

            let Some(suggested_operator) = binary_expr.operator.equality_inverse_operator() else {
                return;
            };

            ctx.diagnostic_with_suggestion(
                no_negation_in_equality_check_diagnostic(
                    left_unary_expr.span,
                    binary_expr.operator,
                    suggested_operator,
                ),
                |fixer| {
                    let source_text = ctx.source_text();
                    let unary_start = left_unary_expr.span.start as usize;

                    let argument_text = ctx.source_range(left_unary_expr.argument.span());
                    let right_text = ctx.source_range(binary_expr.right.span());
                    let first_char = argument_text.chars().next();

                    let is_asi_hazard = ast_util::could_be_asi_hazard(node, ctx)
                        && matches!(first_char, Some('(' | '['));

                    let char_before = if unary_start > 0 {
                        source_text[..unary_start].chars().next_back()
                    } else {
                        None
                    };
                    let needs_space_before = char_before.is_some_and(is_identifier_start);

                    let fixed_expr =
                        format!("{argument_text} {} {right_text}", suggested_operator.as_str());

                    if is_asi_hazard {
                        // Add semicolon before to prevent ASI issues
                        fixer.replace(binary_expr.span, format!(";{fixed_expr}"))
                    } else if needs_space_before {
                        // Add space before (e.g., `return!foo` -> `return foo`)
                        fixer.replace(binary_expr.span, format!(" {fixed_expr}"))
                    } else {
                        fixer.replace(binary_expr.span, fixed_expr)
                    }
                },
            );
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

    let fix = vec![
        ("!foo === bar", "foo !== bar"),
        ("!foo !== bar", "foo === bar"),
        ("!foo == bar", "foo != bar"),
        ("!foo != bar", "foo == bar"),
        (
            "
						function x() {
							return!foo === bar;
						}
					",
            "
						function x() {
							return foo !== bar;
						}
					",
        ),
        (
            "
						function x() {
							return!
								foo === bar;
							throw!
								foo === bar;
						}
					",
            "
						function x() {
							return foo !== bar;
							throw foo !== bar;
						}
					",
        ),
        (
            "
						foo
						!(a) === b
					",
            "
						foo
						;(a) !== b
					",
        ),
        (
            "
						foo
						![a, b].join('') === c
					",
            "
						foo
						;[a, b].join('') !== c
					",
        ),
        (
            "
						foo
						! [a, b].join('') === c
					",
            "
						foo
						;[a, b].join('') !== c
					",
        ),
        (
            "
						foo
						!/* comment */[a, b].join('') === c
					",
            "
						foo
						;[a, b].join('') !== c
					",
        ),
    ];

    Tester::new(NoNegationInEqualityCheck::NAME, NoNegationInEqualityCheck::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
