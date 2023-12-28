// Based on https://github.com/rust-lang/rust-clippy//blob/c9a43b18f11219fa70fe632b29518581fcd589c8/clippy_lints/src/operators/misrefactored_assign_op.rs
use oxc_ast::{
    ast::{AssignmentTarget, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_same_member_expression, is_same_reference},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("oxc(misrefactored-assign-op): Misrefactored assign op. Variable appears on both sides of an assignment operation")]
#[diagnostic(severity(warning), help("Did you mean `{1}`?"))]
struct MisrefactoredAssignOpDiagnostic(#[label] pub Span, pub String);

#[derive(Debug, Default, Clone)]
pub struct MisrefactoredAssignOp;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// https://rust-lang.github.io/rust-clippy/master/#/misrefactored_assign_op
    ///
    /// Checks for `a op= a op b` or `a op= b op a` patterns.
    ///
    /// ### Why is this bad?
    ///
    /// Most likely these are bugs where one meant to write `a op= b`.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// a += a + b;
    /// a -= a - b;
    ///
    /// // Good
    /// a += b;
    /// a -= b;
    /// ```
    MisrefactoredAssignOp,
    suspicious,
);

impl Rule for MisrefactoredAssignOp {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assignment_expr) = node.kind() else {
            return;
        };

        if let Expression::BinaryExpression(binary_expr) = &assignment_expr.right {
            if !are_matching_operators(assignment_expr.operator, binary_expr.operator) {
                return;
            }

            // lhs op= l op r
            if assignment_target_eq_expr(&assignment_expr.left, &binary_expr.left, ctx) {
                ctx.diagnostic(MisrefactoredAssignOpDiagnostic(
                    assignment_expr.span,
                    format!(
                        "{} {} {}",
                        assignment_expr.left.span().source_text(ctx.source_text()),
                        assignment_expr.operator.as_str(),
                        binary_expr.right.span().source_text(ctx.source_text())
                    ),
                ));
            }

            // lhs op= l commutative_op r
            if is_commutative_operator(binary_expr.operator)
                && assignment_target_eq_expr(&assignment_expr.left, &binary_expr.right, ctx)
            {
                ctx.diagnostic(MisrefactoredAssignOpDiagnostic(
                    assignment_expr.span,
                    format!(
                        "{} {} {}",
                        assignment_expr.left.span().source_text(ctx.source_text()),
                        assignment_expr.operator.as_str(),
                        binary_expr.left.span().source_text(ctx.source_text())
                    ),
                ));
            }
        }
    }
}

fn assignment_target_eq_expr<'a>(
    assignment_target: &AssignmentTarget<'a>,
    right_expr: &Expression<'_>,
    ctx: &LintContext<'a>,
) -> bool {
    if let AssignmentTarget::SimpleAssignmentTarget(simple_assignment_target) = assignment_target {
        return match simple_assignment_target {
            oxc_ast::ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                if let Expression::Identifier(right_ident) = right_expr {
                    ident.name == right_ident.name
                } else {
                    false
                }
            }
            oxc_ast::ast::SimpleAssignmentTarget::MemberAssignmentTarget(member_expr) => {
                if let Expression::MemberExpression(right_member_expr) = right_expr {
                    is_same_member_expression(member_expr, right_member_expr, ctx)
                } else {
                    false
                }
            }
            oxc_ast::ast::SimpleAssignmentTarget::TSAsExpression(ts_expr) => {
                is_same_reference(&ts_expr.expression, right_expr, ctx)
            }
            oxc_ast::ast::SimpleAssignmentTarget::TSSatisfiesExpression(ts_expr) => {
                is_same_reference(&ts_expr.expression, right_expr, ctx)
            }
            oxc_ast::ast::SimpleAssignmentTarget::TSNonNullExpression(ts_expr) => {
                is_same_reference(&ts_expr.expression, right_expr, ctx)
            }
            oxc_ast::ast::SimpleAssignmentTarget::TSTypeAssertion(ts_expr) => {
                is_same_reference(&ts_expr.expression, right_expr, ctx)
            }
        };
    }

    false
}

fn are_matching_operators(op1: AssignmentOperator, op2: BinaryOperator) -> bool {
    matches!(
        (op1, op2),
        (AssignmentOperator::Addition, BinaryOperator::Addition)
            | (AssignmentOperator::Subtraction, BinaryOperator::Subtraction)
            | (AssignmentOperator::Multiplication, BinaryOperator::Multiplication)
            | (AssignmentOperator::Division, BinaryOperator::Division)
            | (AssignmentOperator::Remainder, BinaryOperator::Remainder)
            | (AssignmentOperator::ShiftLeft, BinaryOperator::ShiftLeft)
            | (AssignmentOperator::ShiftRight, BinaryOperator::ShiftRight)
            | (AssignmentOperator::ShiftRightZeroFill, BinaryOperator::ShiftRightZeroFill)
            | (AssignmentOperator::BitwiseOR, BinaryOperator::BitwiseOR)
            | (AssignmentOperator::BitwiseXOR, BinaryOperator::BitwiseXOR)
            | (AssignmentOperator::BitwiseAnd, BinaryOperator::BitwiseAnd)
            | (AssignmentOperator::Exponential, BinaryOperator::Exponential)
    )
}

fn is_commutative_operator(op: BinaryOperator) -> bool {
    matches!(
        op,
        BinaryOperator::Addition
            | BinaryOperator::Multiplication
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseAnd
    )
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // not `*=`
        "a = a * a * a;",
        // not `*=`
        "a = a * 42 * a;",
        // not `*=`
        "a = a * 2 + a;",
        // not commutative
        "a -= 1 - a;",
        // not commutative
        "a /= 5 / a;",
        // not commutative
        "a %= 42 % a;",
        // not commutative
        "a <<= 6 << a;",
        // different ident
        "a += b + 5;",
        // different member
        "a.b.c += a.b.e + 5;",
        "a += a.b.e + 5;",
        // different operator
        "a += a - 5;",
        // different operator
        "a += a / 5;",
        // different operator
        "a += a % 5;",
    ];

    let fail = vec![
        "a += a + 1;",
        //~^ ERROR: variable appears on both sides of an assignment operation
        "a += 1 + a;",
        //~^ ERROR: variable appears on both sides of an assignment operation
        "a -= a - 1;",
        //~^ ERROR: variable appears on both sides of an assignment operation
        "a *= a * 99;",
        //~^ ERROR: variable appears on both sides of an assignment operation
        "a *= 42 * a;",
        //~^ ERROR: variable appears on both sides of an assignment operation
        "a /= a / 2;",
        //~^ ERROR: variable appears on both sides of an assignment operation
        "a %= a % 5;",
        //~^ ERROR: variable appears on both sides of an assignment operation
        "a &= a & 1;",
        //~^ ERROR: variable appears on both sides of an assignment operation
        "a *= a * a;",
        //~^ ERROR: variable appears on both sides of an assignment operation
    ];

    Tester::new_without_config(MisrefactoredAssignOp::NAME, pass, fail).test_and_snapshot();
}
