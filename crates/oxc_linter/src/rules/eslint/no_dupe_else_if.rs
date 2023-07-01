use oxc_ast::{
    ast::{Expression, Statement},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::LogicalOperator;

use crate::{ast_util::calculate_hash, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-dupe-else-if): duplicate conditions in if-else-if chains")]
#[diagnostic(
    severity(warning),
    help(
        "This branch can never execute. Its condition is a duplicate or covered by previous conditions in the if-else-if chain"
    )
)]
struct NoDupeElseIfDiagnostic(#[label] pub Span, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoDupeElseIf;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplicate conditions in if-else-if chains
    ///
    /// ### Why is this bad?
    ///
    /// if-else-if chains are commonly used when there is a need to execute only one branch (or at most one branch) out of several possible branches, based on certain conditions.
    /// Two identical test conditions in the same chain are almost always a mistake in the code. Unless there are side effects in the expressions,
    /// a duplicate will evaluate to the same true or false value as the identical expression earlier in the chain, meaning that its branch can never execute.
    ///
    ///
    /// ### Example
    /// ```javascript
    /// if (a) {
    /// foo();
    /// } else if (b) {
    ///     bar();
    /// } else if (b) {
    ///     baz();
    /// }
    /// ```
    NoDupeElseIf,
    correctness
);

impl Rule for NoDupeElseIf {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // if (a) {} else if (a) {}
        //                ^^ get this if statement
        let if_stmt = if let AstKind::IfStatement(if_stmt) = node.kind()
            && let Some(AstKind::IfStatement(parent_if_stmt)) = ctx.nodes().parent_kind(node.id())
            && let Some(Statement::IfStatement(child_if_stmt)) = &parent_if_stmt.alternate
            && child_if_stmt.span == if_stmt.span {
            if_stmt
        } else {
            return
        };

        let mut conditions_to_check = vec![&if_stmt.test];

        if let Expression::LogicalExpression(expr) = &if_stmt.test
            && expr.operator == LogicalOperator::And {
            conditions_to_check.extend(split_by_and(&if_stmt.test));
        }

        let mut list_to_check: Vec<Vec<Vec<_>>> = conditions_to_check
            .iter()
            .map(|expr| split_by_or(expr).into_iter().map(split_by_and).collect())
            .collect();

        let mut current_node = node;
        while let Some(parent_node) = ctx.nodes().parent_node(current_node.id()) {
            let AstKind::IfStatement(stmt) = parent_node.kind() else { break };

            if !stmt
                .alternate
                .as_ref()
                .is_some_and(|stmt| stmt.span() == current_node.kind().span())
            {
                break;
            }

            current_node = parent_node;

            let current_or_operands: Vec<_> =
                split_by_or(&stmt.test).into_iter().map(split_by_and).collect();

            list_to_check = list_to_check
                .into_iter()
                .map(|or_operands| {
                    or_operands
                        .into_iter()
                        .filter(|or_operand| {
                            !current_or_operands
                                .iter()
                                .any(|current_or_operand| is_subset(current_or_operand, or_operand))
                        })
                        .collect()
                })
                .collect();

            if list_to_check.iter().any(Vec::is_empty) {
                ctx.diagnostic(NoDupeElseIfDiagnostic(if_stmt.test.span(), stmt.test.span()));
                break;
            }
        }
    }
}

fn split_by_or<'a, 'b>(expr: &'a Expression<'b>) -> Vec<&'a Expression<'b>> {
    split_by_logical_operator(expr, LogicalOperator::Or)
}

fn split_by_and<'a, 'b>(expr: &'a Expression<'b>) -> Vec<&'a Expression<'b>> {
    split_by_logical_operator(expr, LogicalOperator::And)
}

fn split_by_logical_operator<'a, 'b>(
    expr: &'a Expression<'b>,
    operator: LogicalOperator,
) -> Vec<&'a Expression<'b>> {
    match expr {
        Expression::LogicalExpression(expr) if expr.operator == operator => [
            split_by_logical_operator(&expr.left, operator),
            split_by_logical_operator(&expr.right, operator),
        ]
        .concat(),
        Expression::ParenthesizedExpression(expr) => {
            split_by_logical_operator(&expr.expression, operator)
        }
        _ => vec![expr],
    }
}

fn is_subset<'a, 'b>(a: &'a [&'a Expression<'b>], b: &'a [&'a Expression<'b>]) -> bool {
    a.iter().all(|expr_a| b.iter().any(|expr_b| is_equal(expr_a, expr_b)))
}

fn is_equal<'a, 'b>(a: &'a Expression<'b>, b: &'a Expression<'b>) -> bool {
    match (a, b) {
        (Expression::LogicalExpression(a), Expression::LogicalExpression(b))
            if matches!(a.operator, LogicalOperator::And | LogicalOperator::Or)
                && a.operator == b.operator =>
        {
            (is_equal(&a.left, &b.left) && is_equal(&a.right, &b.right))
                || (is_equal(&a.left, &b.right) && is_equal(&a.right, &b.left))
        }

        (a, b) => calculate_hash(a) == calculate_hash(b),
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("if (a) {} else if (b) {}", None),
        ("if (a); else if (b); else if (c);", None),
        ("if (true) {} else if (false) {} else {}", None),
        ("if (1) {} else if (2) {}", None),
        ("if (f) {} else if (f()) {}", None),
        ("if (f(a)) {} else if (g(a)) {}", None),
        ("if (f(a)) {} else if (f(b)) {}", None),
        ("if (a === 1) {} else if (a === 2) {}", None),
        ("if (a === 1) {} else if (b === 1) {}", None),
        ("if (a) {}", None),
        ("if (a);", None),
        ("if (a) {} else {}", None),
        ("if (a) if (a) {}", None),
        ("if (a) if (a);", None),
        ("if (a) { if (a) {} }", None),
        ("if (a) {} else { if (a) {} }", None),
        ("if (a) {} if (a) {}", None),
        ("if (a); if (a);", None),
        ("while (a) if (a);", None),
        ("if (a); else a ? a : a;", None),
        ("if (a) { if (b) {} } else if (b) {}", None),
        ("if (a) if (b); else if (a);", None),
        ("if (a) {} else if (!!a) {}", None),
        ("if (a === 1) {} else if (a === (1)) {}", None),
        ("if (a || b) {} else if (c || d) {}", None),
        ("if (a || b) {} else if (a || c) {}", None),
        ("if (a) {} else if (a || b) {}", None),
        ("if (a) {} else if (b) {} else if (a || b || c) {}", None),
        ("if (a && b) {} else if (a) {} else if (b) {}", None),
        ("if (a && b) {} else if (b && c) {} else if (a && c) {}", None),
        ("if (a && b) {} else if (b || c) {}", None),
        ("if (a) {} else if (b && (a || c)) {}", None),
        ("if (a) {} else if (b && (c || d && a)) {}", None),
        ("if (a && b && c) {} else if (a && b && (c || d)) {}", None),
    ];

    let fail = vec![
        ("if (a) {} else if (a) {}", None),
        ("if (a); else if (a);", None),
        ("if (a) {} else if (a) {} else {}", None),
        ("if (a) {} else if (b) {} else if (a) {} else if (c) {}", None),
        ("if (a) {} else if (b) {} else if (a) {}", None),
        ("if (a) {} else if (b) {} else if (c) {} else if (a) {}", None),
        ("if (a) {} else if (b) {} else if (b) {}", None),
        ("if (a) {} else if (b) {} else if (b) {} else {}", None),
        ("if (a) {} else if (b) {} else if (c) {} else if (b) {}", None),
        ("if (a); else if (b); else if (c); else if (b); else if (d); else;", None),
        ("if (a); else if (b); else if (c); else if (d); else if (b); else if (e);", None),
        ("if (a) {} else if (a) {} else if (a) {}", None),
        ("if (a) {} else if (b) {} else if (a) {} else if (b) {} else if (a) {}", None),
        ("if (a) { if (b) {} } else if (a) {}", None),
        ("if (a === 1) {} else if (a === 1) {}", None),
        ("if (1 < a) {} else if (1 < a) {}", None),
        ("if (true) {} else if (true) {}", None),
        ("if (a && b) {} else if (a && b) {}", None),
        ("if (a && b || c)  {} else if (a && b || c) {}", None),
        ("if (f(a)) {} else if (f(a)) {}", None),
        ("if (a === 1) {} else if (a===1) {}", None),
        ("if (a === 1) {} else if (a === /* comment */ 1) {}", None),
        ("if (a === 1) {} else if ((a === 1)) {}", None),
        ("if (a || b) {} else if (a) {}", None),
        ("if (a || b) {} else if (a) {} else if (b) {}", None),
        ("if (a || b) {} else if (b || a) {}", None),
        ("if (a) {} else if (b) {} else if (a || b) {}", None),
        ("if (a || b) {} else if (c || d) {} else if (a || d) {}", None),
        ("if ((a === b && fn(c)) || d) {} else if (fn(c) && a === b) {}", None),
        ("if (a) {} else if (a && b) {}", None),
        ("if (a && b) {} else if (b && a) {}", None),
        ("if (a && b) {} else if (a && b && c) {}", None),
        ("if (a || c) {} else if (a && b || c) {}", None),
        ("if (a) {} else if (b) {} else if (c && a || b) {}", None),
        ("if (a) {} else if (b) {} else if (c && (a || b)) {}", None),
        ("if (a) {} else if (b && c) {} else if (d && (a || e && c && b)) {}", None),
        ("if (a || b && c) {} else if (b && c && d) {}", None),
        ("if (a || b) {} else if (b && c) {}", None),
        ("if (a) {} else if (b) {} else if ((a || b) && c) {}", None),
        ("if ((a && (b || c)) || d) {} else if ((c || b) && e && a) {}", None),
        ("if (a && b || b && c) {} else if (a && b && c) {}", None),
        ("if (a) {} else if (b && c) {} else if (d && (c && e && b || a)) {}", None),
        ("if (a || (b && (c || d))) {} else if ((d || c) && b) {}", None),
        ("if (a || b) {} else if ((b || a) && c) {}", None),
        ("if (a || b) {} else if (c) {} else if (d) {} else if (b && (a || c)) {}", None),
        ("if (a || b || c) {} else if (a || (b && d) || (c && e)) {}", None),
        ("if (a || (b || c)) {} else if (a || (b && c)) {}", None),
        ("if (a || b) {} else if (c) {} else if (d) {} else if ((a || c) && (b || d)) {}", None),
        ("if (a) {} else if (b) {} else if (c && (a || d && b)) {}", None),
        ("if (a) {} else if (a || a) {}", None),
        ("if (a || a) {} else if (a || a) {}", None),
        ("if (a || a) {} else if (a) {}", None),
        ("if (a) {} else if (a && a) {}", None),
        ("if (a && a) {} else if (a && a) {}", None),
        ("if (a && a) {} else if (a) {}", None),
    ];

    Tester::new(NoDupeElseIf::NAME, pass, fail).test_and_snapshot();
}
