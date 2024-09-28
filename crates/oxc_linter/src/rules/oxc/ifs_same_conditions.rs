use oxc_ast::{
    ast::{Expression, IfStatement, LogicalOperator, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_span::cmp::ContentEq;
use oxc_span::{GetSpan, Span};

fn ifs_same_conditions_diagnostic(span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unreachable if consequent")
        .with_help(
            "This if statement test will always be false, as the previous `if` statement covers the same condition.",
        )
        .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct IfsSameConditions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks for consecutive `if` statements with the same condition.
    ///
    /// ### Why is this bad?
    ///
    /// This is probably a mistake, as the second `if` statement will never be executed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (a) {
    /// } else if (a) {
    /// }
    ///
    /// if (a) {
    /// } else if (b) {
    /// } else if (a) {
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (a) {
    /// } else if (b) {
    /// }
    ///
    /// if (a) {
    /// } else if (b) {
    /// } else {
    /// }
    /// ```
    IfsSameConditions,
    correctness
);

impl Rule for IfsSameConditions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else {
            return;
        };

        if let Some(AstKind::IfStatement(_)) = ctx.nodes().parent_kind(node.id()) {
            return;
        }

        let tests = collect_if_tests(if_stmt);
        if tests.len() < 2 {
            return;
        }

        let mut must_be_falsy: Vec<&Expression> = Vec::new();

        for test in &tests {
            for previous in &must_be_falsy {
                if is_redundant(test, previous) {
                    ctx.diagnostic(ifs_same_conditions_diagnostic(test.span()));
                }
            }
            let mut must_also_be_falsy = Vec::new();
            collect_must_be_falsy(test, &mut must_also_be_falsy);
            must_be_falsy.extend(must_also_be_falsy);
        }
    }
}

fn is_redundant<'a>(current: &'a Expression<'a>, must_be_falsy: &'a Expression<'a>) -> bool {
    match (must_be_falsy.get_inner_expression(), current.get_inner_expression()) {
        (Expression::LogicalExpression(left), Expression::LogicalExpression(right))
            if left.operator == right.operator
                && matches!(left.operator, LogicalOperator::Or | LogicalOperator::And) =>
        {
            (left.left.content_eq(&right.left) && left.right.content_eq(&right.right))
                || (left.left.content_eq(&right.right) && left.right.content_eq(&right.left))
        }
        (left, Expression::LogicalExpression(right))
            if matches!(right.operator, LogicalOperator::Or | LogicalOperator::And) =>
        {
            right.left.content_eq(left) || right.right.content_eq(left)
        }
        (falsy_expr, current) => falsy_expr.content_eq(current),
    }
}

fn collect_must_be_falsy<'a, 'b>(
    expr: &'b Expression<'a>,
    must_be_falsy: &mut Vec<&'b Expression<'a>>,
) {
    match expr.get_inner_expression() {
        Expression::LogicalExpression(logical_expr)
            if matches!(logical_expr.operator, LogicalOperator::Or) =>
        {
            collect_must_be_falsy(&logical_expr.left, must_be_falsy);
            collect_must_be_falsy(&logical_expr.right, must_be_falsy);
        }
        _ => {
            must_be_falsy.push(expr);
        }
    }
}

fn collect_if_tests<'a, 'b>(if_stmt: &'b IfStatement<'a>) -> Vec<&'b Expression<'a>> {
    let mut tests = vec![&if_stmt.test];

    let mut current = &if_stmt.alternate;
    while let Some(Statement::IfStatement(if_stmt)) = current {
        tests.push(&if_stmt.test);
        current = &if_stmt.alternate;
    }

    tests
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Simple chain with distinct conditions
        "if (a) {} else if (b) {} else if (c) {}",
        // Distinct compound conditions without overlap
        "if (a && b) {} else if (c && d) {} else if (e && f) {}",
        // Single condition without chaining
        "if (a) {}",
        // Different conditions using `||` but no overlap with subsequent conditions
        "if (a || b) {} else if (c) {}",
        // Complex distinct conditions
        "if ((a && b) || (c && d)) {} else if (e) {}",
        // Variables with negation, distinct conditions
        "if (a) {} else if (!a) {} else if (b) {}",
        // Logical branching with disjoint conditions
        "if (a && !b) {} else if (b && !a) {}",
        // Conditions using different operators and terms
        "if (x > 5) {} else if (y < 10) {} else if (z == 3) {}",
        // Conditions with different operators and terms
        "if (a && b) {} else if (a && c) {}",
        "if (a && b) {} else if (a) {}",
        "if (p1 && p2) { } else if (p1 || p2) {}",
    ];

    let fail = vec![
        // Exact same condition repeated
        "if (a) {} else if (a) {}",
        // The third condition repeats the first condition
        "if (a) {} else if (b) {} else if (a) {}",
        // Redundant compound conditions
        "if (a && b) {} else if (a && b) {}",
        // `b` is part of `a || b`, making it redundant
        "if (a || b) {} else if (b) {}",
        // Conditions differ but are logically redundant due to `||`
        "if (a || b) {} else if (a) {}",
        // `b && c` is partially covered by `a || b` when `a` is false
        "if (a || b) {} else if (b && c) {}",
        // `!a` implies `a` must be false, making the third condition redundant
        "if (a) {} else if (!a) {} else if (a && b) {}",
        // `b || c` implies at least one of them must be true, making `b` redundant afterwards
        "if (b || c) {} else if (b) {}",
        // `d` is logically covered when `!(a || b || c)` is false
        "if (a || b || c) {} else if (d) {} else if (a) {}",
        // Overlapping terms but logically distinct usage
        "if (a || b) {} else if (a && b) {}",
        // we should only report 1x error here
        "if (false) {} else if (true) {} else if (true) {}",
    ];

    Tester::new(IfsSameConditions::NAME, pass, fail).test_and_snapshot();
}
