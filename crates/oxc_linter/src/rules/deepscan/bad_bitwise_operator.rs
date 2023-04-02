use oxc_ast::{
    ast::{BinaryOperator, Expression},
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
                if is_short_circuit_like(node) {
                    ctx.diagnostic(BadBitwiseOperatorDiagnostic("&", "&&", bin_expr.span));
                }
            }
            _ => {}
        }
    }
}

fn is_short_circuit_like(node: &AstNode) -> bool {
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

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var a = obj && obj.a", None),
        ("var a = obj || obj.a", None),
        ("var a = obj1 & obj2.a", None),
    ];

    let fail = vec![("var a = obj & obj.a", None)];

    Tester::new(BadBitwiseOperator::NAME, pass, fail).test_and_snapshot();
}
