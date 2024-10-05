use std::fmt::Debug;

use oxc_ast::{
    ast::{Expression, LogicalExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

fn some(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Found a useless array length check")
        .with_help(
            "The non-empty check is useless as `Array#some()` returns `false` for an empty array.",
        )
        .with_label(span)
}

fn every(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Found a useless array length check")
        .with_help(
            "The empty check is useless as `Array#every()` returns `true` for an empty array.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessLengthCheck;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// It checks for an unnecessary array length check in a logical expression.
    ///
    /// The cases are:
    /// - `array.length === 0 || array.every(Boolean)` (`array.every` returns `true` if array is has elements)
    /// - `array.length > 0 && array.some(Boolean)` (`array.some` returns `false` if array is empty)
    ///
    /// ### Why is this bad?
    ///
    /// An extra unnecessary length check is done.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// if (array.length === 0 || array.every(Boolean)) {
    ///    // do something!
    /// }
    /// ```
    NoUselessLengthCheck,
    correctness,
    pending
);

struct ConditionDTO<T: ToString> {
    property_name: T,
    binary_operators: Vec<BinaryOperator>,
}

fn is_useless_check<'a>(
    left: &'a Expression<'a>,
    right: &'a Expression<'a>,
    operator: LogicalOperator,
) -> Option<OxcDiagnostic> {
    let every_condition = ConditionDTO {
        property_name: "every",
        binary_operators: vec![BinaryOperator::StrictEquality],
    };
    let some_condition = ConditionDTO {
        property_name: "some",
        binary_operators: vec![BinaryOperator::StrictInequality, BinaryOperator::GreaterThan],
    };

    let mut array_name: &str = "";
    let active_condition = {
        if operator == LogicalOperator::Or {
            every_condition
        } else {
            some_condition
        }
    };
    let mut binary_expression_span: Option<Span> = None;
    let mut call_expression_span: Option<Span> = None;

    let l = match left.without_parentheses() {
        Expression::BinaryExpression(expr) => {
            let left_expr = expr.left.get_inner_expression().as_member_expression()?;
            array_name = left_expr.object().get_identifier_reference()?.name.as_str();
            binary_expression_span = Some(expr.span);

            active_condition.binary_operators.contains(&expr.operator)
                && left_expr.is_specific_member_access(array_name, "length")
                && expr.right.is_specific_raw_number_literal("0")
        }
        Expression::CallExpression(expr) => {
            array_name =
                expr.callee.get_member_expr()?.object().get_identifier_reference()?.name.as_str();
            let property_name = expr.callee.get_member_expr()?.static_property_name()?;
            call_expression_span = Some(expr.span);

            let is_same_method = property_name == active_condition.property_name;
            let is_optional = expr.optional;

            is_same_method && !is_optional
        }
        _ => false,
    };

    let r = match right.without_parentheses() {
        Expression::BinaryExpression(expr) => {
            let left_expr = expr.left.get_inner_expression().as_member_expression()?;
            let ident_name = left_expr.object().get_identifier_reference()?.name.as_str();
            if binary_expression_span.is_some() {
                return None;
            }
            binary_expression_span = Some(expr.span);

            active_condition.binary_operators.contains(&expr.operator)
                && left_expr.is_specific_member_access(array_name, "length")
                && expr.right.is_specific_raw_number_literal("0")
                && ident_name == array_name
        }
        Expression::CallExpression(expr) => {
            let is_same_name =
                expr.callee.get_member_expr()?.object().get_identifier_reference()?.name.as_str()
                    == array_name;

            if call_expression_span.is_some() {
                return None;
            }
            let property_name = expr.callee.get_member_expr()?.static_property_name()?;
            let is_same_method = property_name == active_condition.property_name;
            let is_optional = expr.optional;

            is_same_method && !is_optional && is_same_name
        }
        _ => false,
    };

    if l && r {
        Some(if active_condition.property_name == "every" {
            every(binary_expression_span?)
        } else {
            some(binary_expression_span?)
        })
    } else {
        None
    }
}

impl Rule for NoUselessLengthCheck {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::LogicalExpression(log_expr) = node.kind() {
            if ![LogicalOperator::And, LogicalOperator::Or].contains(&log_expr.operator) {
                return;
            }
            let mut flat_exprs = Vec::new();
            make_flat_logical_expression(log_expr, &mut flat_exprs);
            for window in flat_exprs.windows(2) {
                if let Some(diag) = is_useless_check(window[0], window[1], log_expr.operator) {
                    ctx.diagnostic(diag);
                }
            }
        };
    }
}

fn make_flat_logical_expression<'a>(
    node: &'a LogicalExpression<'a>,
    result: &mut Vec<&'a Expression<'a>>,
) {
    match &node.left.without_parentheses() {
        Expression::LogicalExpression(le) => {
            if le.operator == node.operator {
                make_flat_logical_expression(le, result);
            } else {
                result.push(&node.left);
            }
        }
        _ => result.push(&node.left),
    };

    match &node.right.without_parentheses() {
        Expression::LogicalExpression(le) => {
            if le.operator == node.operator {
                make_flat_logical_expression(le, result);
            } else {
                result.push(&node.right);
            }
        }
        _ => result.push(&node.right),
    };
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "array.length === 0 ?? array.every(Boolean)",
        "array.length === 0 && array.every(Boolean)",
        "(array.length === 0) + (array.every(Boolean))",
        "array.length === 1 || array.every(Boolean)",
        "array.length === \"0\" || array.every(Boolean)",
        "array.length === 0. || array.every(Boolean)",
        "array.length === 0x0 || array.every(Boolean)",
        "array.length !== 0 || array.every(Boolean)",
        "array.length == 0 || array.every(Boolean)",
        "0 === array.length || array.every(Boolean)",
        "array?.length === 0 || array.every(Boolean)",
        "array.notLength === 0 || array.every(Boolean)",
        "array[length] === 0 || array.every(Boolean)",
        "array.length === 0 || array.every?.(Boolean)",
        "array.length === 0 || array?.every(Boolean)",
        "array.length === 0 || array.every",
        "array.length === 0 || array[every](Boolean)",
        "array1.length === 0 || array2.every(Boolean)",
        "array.length !== 0 ?? array.some(Boolean)",
        "array.length !== 0 || array.some(Boolean)",
        "(array.length !== 0) - (array.some(Boolean))",
        "array.length !== 1 && array.some(Boolean)",
        "array.length !== \"0\" && array.some(Boolean)",
        "array.length !== 0. && array.some(Boolean)",
        "array.length !== 0x0 && array.some(Boolean)",
        "array.length === 0 && array.some(Boolean)",
        "array.length <= 0 && array.some(Boolean)",
        "array.length != 0 && array.some(Boolean)",
        "0 !== array.length && array.some(Boolean)",
        "array?.length !== 0 && array.some(Boolean)",
        "array.notLength !== 0 && array.some(Boolean)",
        "array[length] !== 0 && array.some(Boolean)",
        "array.length !== 0 && array.some?.(Boolean)",
        "array.length !== 0 && array?.some(Boolean)",
        "array.length !== 0 && array.some",
        "array.length !== 0 && array.notSome(Boolean)",
        "array.length !== 0 && array[some](Boolean)",
        "array1.length !== 0 && array2.some(Boolean)",
        "array.length > 0 ?? array.some(Boolean)",
        "array.length > 0 || array.some(Boolean)",
        "(array.length > 0) - (array.some(Boolean))",
        "array.length > 1 && array.some(Boolean)",
        "array.length > \"0\" && array.some(Boolean)",
        "array.length > 0. && array.some(Boolean)",
        "array.length > 0x0 && array.some(Boolean)",
        "array.length >= 0 && array.some(Boolean)",
        "0 > array.length && array.some(Boolean)",
        "0 < array.length && array.some(Boolean)",
        "array?.length > 0 && array.some(Boolean)",
        "array.notLength > 0 && array.some(Boolean)",
        "array.length > 0 && array.some?.(Boolean)",
        "array.length > 0 && array?.some(Boolean)",
        "array.length > 0 && array.some",
        "array.length > 0 && array.notSome(Boolean)",
        "array.length > 0 && array[some](Boolean)",
        "array1.length > 0 && array2.some(Boolean)",
        "(foo && array.length === 0) || array.every(Boolean) && foo",
        "array.length === 0 || (array.every(Boolean) && foo)",
        "(foo || array.length > 0) && array.some(Boolean)",
        "array.length > 0 && (array.some(Boolean) || foo)",
        "array.length === 0 || array.length === 0",
        "array.some(Boolean) && array.some(Boolean)",
    ];

    let fail = vec![
        "array.length === 0 || array.every(Boolean)",
        "array.length > 0 && array.some(Boolean)",
        "array.length !== 0 && array.some(Boolean)",
        "if ((( array.length > 0 )) && array.some(Boolean));",
        "(array.length === 0 || array.every(Boolean)) || foo",
        "foo || (array.length === 0 || array.every(Boolean))",
        "(array.length > 0 && array.some(Boolean)) && foo",
        "foo && (array.length > 0 && array.some(Boolean))",
        "array.every(Boolean) || array.length === 0",
        "array.some(Boolean) && array.length !== 0",
        "array.some(Boolean) && array.length > 0",
        "foo && array.length > 0 && array.some(Boolean)",
        "foo || array.length === 0 || array.every(Boolean)",
        "(foo || array.length === 0) || array.every(Boolean)",
        "array.length === 0 || (array.every(Boolean) || foo)",
        "(foo && array.length > 0) && array.some(Boolean)",
        "array.length > 0 && (array.some(Boolean) && foo)",
        "array.every(Boolean) || array.length === 0 || array.every(Boolean)",
        "array.length === 0 || array.every(Boolean) || array.length === 0",
    ];

    Tester::new(NoUselessLengthCheck::NAME, pass, fail).test_and_snapshot();
}
