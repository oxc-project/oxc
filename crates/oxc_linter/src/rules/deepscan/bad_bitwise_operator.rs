use oxc_ast::{
    ast::{BinaryExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};

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
    restriction // Restricted because there are false positives for enum bitflags in TypeScript,
                // e.g. in the vscode repo
);

impl Rule for BadBitwiseOperator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(bin_expr) => {
                if is_mistype_short_circuit(node) {
                    ctx.diagnostic(BadBitwiseOperatorDiagnostic("&", "&&", bin_expr.span));
                } else if is_mistype_option_fallback(node) {
                    ctx.diagnostic(BadBitwiseOperatorDiagnostic("|", "||", bin_expr.span));
                }
            }
            AstKind::AssignmentExpression(assign_expr) => {
                if assign_expr.operator == AssignmentOperator::BitwiseOR
                    && !is_numeric_expr(&assign_expr.right, true)
                {
                    ctx.diagnostic(BadBitwiseOrOperatorDiagnostic(assign_expr.span));
                }
            }
            _ => {}
        }
    }
}

fn is_mistype_short_circuit(node: &AstNode) -> bool {
    match node.kind() {
        AstKind::BinaryExpression(bin_expr) => {
            if bin_expr.operator != BinaryOperator::BitwiseAnd {
                return false;
            }

            let Expression::Identifier(left_ident) = &bin_expr.left else { return false };

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

fn is_mistype_option_fallback(node: &AstNode) -> bool {
    match node.kind() {
        AstKind::BinaryExpression(binary_expr) => {
            if binary_expr.operator != BinaryOperator::BitwiseOR {
                return false;
            }

            if let Expression::Identifier(_) = &binary_expr.left
            && !is_numeric_expr(&binary_expr.right, true) {
                return true
            }

            false
        }
        _ => false,
    }
}

fn is_numeric_expr(expr: &Expression, is_outer_most: bool) -> bool {
    match expr {
        Expression::NumberLiteral(_)
        | Expression::NullLiteral(_)
        // TODO: handle type inference
        | Expression::Identifier(_) => true,
        Expression::UnaryExpression(unary_expr) => {
            if is_outer_most {
                unary_expr.operator != UnaryOperator::Typeof && unary_expr.operator != UnaryOperator::LogicalNot
            } else {
                unary_expr.operator != UnaryOperator::Typeof
            }
        }
        Expression::BinaryExpression(binary_expr) => !is_string_concat(binary_expr),
        Expression::ParenthesizedExpression(paren_expr) => {
            is_numeric_expr(&paren_expr.expression, false)
        }
        _ => {
            if is_outer_most {
                expr.is_undefined()
            } else {
                !expr.is_string_literal()
            }
        }
    }
}

fn is_string_concat(binary_expr: &BinaryExpression) -> bool {
    binary_expr.operator == BinaryOperator::Addition
        && (contains_string_literal(&binary_expr.left)
            || contains_string_literal(&binary_expr.right))
}

fn contains_string_literal(expr: &Expression) -> bool {
    match expr {
        Expression::StringLiteral(_) => true,
        Expression::UnaryExpression(unary_expr) => unary_expr.operator == UnaryOperator::Typeof,

        Expression::BinaryExpression(binary_expr) => is_string_concat(binary_expr),
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
        ("var a = b | c", None),
        ("var a = options || {}", None),
        ("var a = options | 1", None),
        ("var a = options | undefined", None),
        ("var a = options | null", None),
        ("var a = options | ~{}", None),
        ("var a = options | (1 + 2 + (3 + !''))", None),
        ("var a = options | (1 + 2 + (3 + !4))", None),
        ("var a = options | (1 + 2 + (3 + +false))", None),
        ("var a = options | (1 + 2 + (3 * '4'))", None),
        ("var a = options | (1 + 2 + (3 * ('4' + 5)))", None),
        ("var a = options | (1 + 2 + (3 + 4))", None),
        ("var a = options | (1 + {})", None),
        ("var a = 1 | {}", None),
        ("var a = 1 | ''", None),
        ("var a = 1 | true", None),
        ("var a = {} | 1", None),
        ("var a = '' | 1", None),
        ("var a = true | 1", None),
        ("var a = b | (1 + 2 + (3 + c))", None),
        ("var a = 1 + '2' - 3", None),
        ("var a = '1' + 2 - 3", None),
        ("input |= 1", None),
        ("input |= undefined", None),
        ("input |= null", None),
        ("input |= (1 + 1)", None),
        ("input |= (1 + true)", None),
        ("input |= (1 + 2 * '3')", None),
        ("input |= (1 + (2 + {}))", None),
        ("input |= ('1' + 2 - 3)", None),
        ("input |= (1 + '2' - 3)", None),
        ("input |= a", None),
        ("input |= ~{}", None),
        // TODO
        // ("var a = 1; input |= a", None),
        // ("var a = 1; var b = a | {}", None),
    ];

    let fail = vec![
        ("var a = obj & obj.a", None),
        ("var a = options | {}", None),
        ("var a = options | !{}", None),
        ("var a = options | typeof {}", None),
        ("var a = options | ''", None),
        ("var a = options | true", None),
        ("var a = options | false", None),
        ("var a = options | (1 + 2 + typeof {})", None),
        ("var a = options | (1 + 2 + (3 + ''))", None),
        ("var a = options | (1 + 2 + (3 + '4'))", None),
        ("input |= ''", None),
        ("input |= (1 + '')", None),
        ("input |= (1 + (3 + '1'))", None),
        ("input |= !{}", None),
        ("input |= typeof {}", None),
        // TODO
        // ("var input; var a = '1'; input |= a", None),
        // ("var a = '1'; var b = a | {}", None),
    ];

    Tester::new(BadBitwiseOperator::NAME, pass, fail).test_and_snapshot();
}
