use oxc_ast::{
    ast::{AssignmentOperator, BinaryOperator, Expression},
    AstKind, Span,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Bad bitwise operator")]
#[diagnostic(
    severity(warning),
    help("Bitwise operator '{0}' seems unintended. Did you mean logical operator '{1}'?")
)]
struct BadBitwiseOperatorDiagnostic(&'static str, &'static str, #[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("Bad bitwise operator")]
#[diagnostic(
    severity(warning),
    help(
        "Bitwise operator '|=' seems unintended. Consider using non-compound assignment and logical operator '||' instead."
    )
)]
struct BadBitwiseOrOperatorDiagnostic(#[label] pub Span);

/// `https://deepscan.io/docs/rules/bad-bitwise-operator`
#[derive(Debug, Default, Clone)]
pub struct BadBitwiseOperator;

declare_oxc_lint!(
    /// ### What it does
    /// This rule applies when bitwise operators are used where logical operators are expected.
    ///
    /// ### Why is this bad?
    /// Bitwise operators have different results from logical operators and a `TypeError` exception may be thrown because short-circuit evaluation is not applied.
    /// (In short-circuit evaluation, right operand evaluation is skipped according to left operand value, e.g. `x` is `false` in `x && y`.)
    ///
    /// It is obvious that logical operators are expected in the following code patterns:
    /// ```javascript
    /// e && e.x
    /// e || {}
    /// e || ''
    /// ```
    ///
    /// ### Example
    /// ```javascript
    /// if (obj & obj.prop) {
    ///  console.log(obj.prop);
    /// }
    /// options = options | {};
    /// input |= '';
    /// ```
    BadBitwiseOperator,
    correctness
);

impl Rule for BadBitwiseOperator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.get().kind() {
            AstKind::BinaryExpression(bin_expr) => {
                if is_mistype_short_circuit(node) {
                    ctx.diagnostic(BadBitwiseOperatorDiagnostic("&", "&&", bin_expr.span));
                } else if is_mistype_option_fallback(node) {
                    ctx.diagnostic(BadBitwiseOperatorDiagnostic("|", "||", bin_expr.span));
                }
            }
            AstKind::AssignmentExpression(assign_expr) => {
                if assign_expr.operator == AssignmentOperator::BitwiseOR {
                    let has_error = match &assign_expr.right {
                        Expression::NumberLiteral(_)
                        | Expression::NullLiteral(_)
                        | Expression::Identifier(_) => false,
                        Expression::BinaryExpression(expr) => {
                            contains_string_literal(&expr.left)
                                || contains_string_literal(&expr.right)
                        }
                        Expression::ParenthesizedExpression(expr) => {
                            contains_string_literal(&expr.expression)
                        }
                        expr => !expr.is_undefined(),
                    };

                    if has_error {
                        ctx.diagnostic(BadBitwiseOrOperatorDiagnostic(assign_expr.span));
                    }
                }
            }
            _ => {}
        }
    }
}

fn is_mistype_option_fallback(node: &AstNode) -> bool {
    match node.get().kind() {
        AstKind::BinaryExpression(bin_expr) => {
            if bin_expr.operator != BinaryOperator::BitwiseOR {
                return false;
            }

            if is_number_or_nullish_expr(&bin_expr.left) || is_number_or_nullish_expr(&bin_expr.right) {
                return false;
            }

             // TODO
            // need type inference
            if let Expression::Identifier(_) = &bin_expr.left 
            && let Expression::Identifier(_) = &bin_expr.right {
                return false
            }

            true
        }
        _ => false,
    }
}

fn is_mistype_short_circuit(node: &AstNode) -> bool {
    match node.get().kind() {
        AstKind::BinaryExpression(bin_expr) => {
            if bin_expr.operator != BinaryOperator::BitwiseAnd {
                return false;
            }

            let left_ident = match &bin_expr.left {
                Expression::Identifier(ident) => ident,
                _ => return false,
            };

            if let Expression::MemberExpression(member_expr) = &bin_expr.right {
                if let Expression::Identifier(ident) = member_expr.object() {
                    return ident.name == left_ident.name;
                }
            }

            false
        }
        _ => false,
    }
}

fn is_number_or_nullish_expr(expr: &Expression) -> bool {
    match expr {
        Expression::NumberLiteral(_)
        | Expression::UnaryExpression(_)
        | Expression::NullLiteral(_) => true,
        Expression::BinaryExpression(bin_expr) => {
            is_number_or_nullish_expr(&bin_expr.left) && is_number_or_nullish_expr(&bin_expr.right)
        }
        Expression::ParenthesizedExpression(paren_expr) => {
            is_number_or_nullish_expr(&paren_expr.expression)
        }
        _ => expr.is_undefined(),
    }
}

fn contains_string_literal(expr: &Expression) -> bool {
    match expr {
        Expression::StringLiteral(_) => true,
        Expression::BinaryExpression(bin_expr) => {
            contains_string_literal(&bin_expr.left) || contains_string_literal(&bin_expr.right)
        }
        Expression::ParenthesizedExpression(paren_expr) => {
            contains_string_literal(&paren_expr.expression)
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var a = obj && obj.a", None),
        ("var a = obj || obj.a", None),
        ("var a = obj1 & obj2.a", None),
        ("var a = options || {}", None),
        ("var a = options | 1", None),
        ("var a = options | undefined", None),
        ("var a = options | null", None),
        ("var a = options | void 0", None),
        ("var a = 1 | {}", None),
        ("var a = 1 | ''", None),
        ("var a = 1 | true", None),
        ("var a = 1 | false", None),
        ("var a = {} | 1", None),
        ("var a = '' | 1", None),
        ("var a = true | 1", None),
        ("var a = false | 1", None),
        ("var a = options | (1 + 2 + (3 + 4))", None),
        ("var a = options | (1 + 2 + (3 + !4))", None),
        ("var a = options | (1 + 2 + (3 + !''))", None),
        ("input |= 1", None),
        ("input |= undefined", None),
        ("input |= null", None),
        ("input |= (1 + 1)", None),
        ("input |= (1 + true)", None),
        ("input |= a", None),
        // TODO
        // ("var a = 1; input |= a", None),
    ];

    let fail = vec![
        ("var a = obj & obj.a", None),
        ("var a = options | {}", None),
        ("var a = options | ''", None),
        ("var a = options | true", None),
        ("var a = options | false", None),
        ("var a = options | (1 + 2 + (3 + ''))", None),
        ("a = input |= ''", None),
        ("a = input |= (1 + '')", None),
        // TODO
        // ("var a = '1'; input |= a", None),
    ];

    Tester::new(BadBitwiseOperator::NAME, pass, fail).test_and_snapshot();
}
