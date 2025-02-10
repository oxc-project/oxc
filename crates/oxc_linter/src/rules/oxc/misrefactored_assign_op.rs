// Based on https://github.com/rust-lang/rust-clippy//blob/c9a43b18f11219fa70fe632b29518581fcd589c8/clippy_lints/src/operators/misrefactored_assign_op.rs
use oxc_ast::{
    ast::{match_member_expression, AssignmentTarget, Expression, SimpleAssignmentTarget},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{is_same_expression, is_same_member_expression},
    AstNode,
};

fn misrefactored_assign_op_diagnostic(span: Span, suggestion: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Misrefactored assign op. Variable appears on both sides of an assignment operation",
    )
    .with_help(format!("Did you mean `{suggestion}`?"))
    .with_label(span)
}

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
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// a += a + b;
    /// a -= a - b;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// a += b;
    /// a -= b;
    /// ```
    MisrefactoredAssignOp,
    oxc,
    suspicious,
    pending
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
                ctx.diagnostic(misrefactored_assign_op_diagnostic(
                    assignment_expr.span,
                    &format!(
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
                ctx.diagnostic(misrefactored_assign_op_diagnostic(
                    assignment_expr.span,
                    &format!(
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
    let right_expr = right_expr.get_inner_expression();
    if let Some(simple_assignment_target) = assignment_target.as_simple_assignment_target() {
        return match simple_assignment_target {
            SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                if let Expression::Identifier(right_ident) = right_expr {
                    ident.name == right_ident.name
                } else {
                    false
                }
            }
            match_member_expression!(SimpleAssignmentTarget) => {
                let member_expr = simple_assignment_target.to_member_expression();
                if let Some(right_member_expr) = right_expr.as_member_expression() {
                    is_same_member_expression(member_expr, right_member_expr, ctx)
                } else {
                    false
                }
            }
            SimpleAssignmentTarget::TSAsExpression(ts_expr) => {
                is_same_expression(&ts_expr.expression, right_expr, ctx)
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(ts_expr) => {
                is_same_expression(&ts_expr.expression, right_expr, ctx)
            }
            SimpleAssignmentTarget::TSNonNullExpression(ts_expr) => {
                is_same_expression(&ts_expr.expression, right_expr, ctx)
            }
            SimpleAssignmentTarget::TSTypeAssertion(ts_expr) => {
                is_same_expression(&ts_expr.expression, right_expr, ctx)
            }
            SimpleAssignmentTarget::TSInstantiationExpression(ts_expr) => {
                is_same_expression(&ts_expr.expression, right_expr, ctx)
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
        "a *= a * (a as number);",
        //~^ ERROR: variable appears on both sides of an assignment operation
        "a *= (a as string) * (a as number);",
        //~^ ERROR: variable appears on both sides of an assignment operation
    ];

    Tester::new(MisrefactoredAssignOp::NAME, MisrefactoredAssignOp::PLUGIN, pass, fail)
        .test_and_snapshot();
}
