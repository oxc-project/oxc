use itertools::concat;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::{BinaryOperator, LogicalOperator};
use std::fmt::Debug;

use oxc_ast::{
    ast::{Expression, LogicalExpression, MemberExpression},
    AstKind,
};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum NoUselessLengthCheckDiagnostic {
    #[error("eslint-plugin-unicorn(no-useless-length-check)")]
    #[diagnostic(
        severity(warning),
        help(
            "The non-empty check is useless as `Array#some()` returns `false` for an empty array."
        )
    )]
    Some(#[label] Span),
    #[error("eslint-plugin-unicorn(no-useless-length-check)")]
    #[diagnostic(
        severity(warning),
        help("The empty check is useless as `Array#every()` returns `true` for an empty array.")
    )]
    Every(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct NoUselessLengthCheck;

declare_oxc_lint!(
    /// ### What it does
    /// It checks for an unnecessary array length check in a logical expression
    /// The cases are:
    ///  array.length === 0 || array.every(Boolean) (array.every returns true if array is has elements)
    ///  array.length > 0 && array.some(Boolean) (array.some returns false if array is empty)
    ///
    /// ### Why is this bad?
    /// An extra unnecessary length check is done
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoUselessLengthCheck,
    correctness
);

struct ConditionDTO<T: ToString> {
    property_name: T,
    binary_operators: Vec<BinaryOperator>,
}

fn get_static_member_property_name<'a>(expr: &'a MemberExpression<'a>) -> Option<&'a str> {
    match expr {
        MemberExpression::StaticMemberExpression(expr) => Some(expr.property.name.as_str()),
        _ => None,
    }
}

fn is_useless_check<'a>(
    left: &'a Expression<'a>,
    right: &'a Expression<'a>,
    operator: LogicalOperator,
) -> Option<NoUselessLengthCheckDiagnostic> {
    let left_condition = ConditionDTO {
        property_name: "every",
        binary_operators: vec![BinaryOperator::StrictEquality],
    };
    let right_condition = ConditionDTO {
        property_name: "some",
        binary_operators: vec![BinaryOperator::StrictInequality, BinaryOperator::GreaterThan],
    };

    let mut name: &str = "";
    let active_condition = {
        if operator == LogicalOperator::Or {
            left_condition
        } else {
            right_condition
        }
    };
    let mut binary_expression_span: Span = Span::default();

    let l = match left.without_parenthesized() {
        Expression::BinaryExpression(expr) => {
            let Expression::MemberExpression(left_expr) = expr.left.get_inner_expression() else {
                return None;
            };
            let id = left_expr.object().get_identifier_reference()?;
            name = id.name.as_str();

            binary_expression_span = expr.span;

            active_condition.binary_operators.contains(&expr.operator)
                && left_expr.is_specific_member_access(name, "length")
                && expr.right.is_raw("0")
        }
        Expression::CallExpression(expr) => {
            name = expr
                .callee
                .get_member_expr()
                .unwrap()
                .object()
                .get_identifier_reference()
                .unwrap()
                .name
                .as_str();
            let property_name =
                get_static_member_property_name(expr.callee.get_member_expr().unwrap())?;
            let is_same_method = property_name == active_condition.property_name;
            let is_optional = expr.optional;

            is_same_method && !is_optional
        }
        _ => false,
    };

    let r = match right.without_parenthesized() {
        Expression::BinaryExpression(expr) => {
            let Expression::MemberExpression(left_expr) = expr.left.get_inner_expression() else {
                return None;
            };
            let id = left_expr.object().get_identifier_reference()?;
            binary_expression_span = expr.span;

            active_condition.binary_operators.contains(&expr.operator)
                && left_expr.is_specific_member_access(name, "length")
                && expr.right.is_raw("0")
                && id.name.as_str() == name
        }
        Expression::CallExpression(expr) => {
            let is_same_name = expr
                .callee
                .get_member_expr()
                .unwrap()
                .object()
                .get_identifier_reference()
                .unwrap()
                .name
                .as_str()
                == name;
            let property_name =
                get_static_member_property_name(expr.callee.get_member_expr().unwrap())?;
            let is_same_method = property_name == active_condition.property_name;
            let is_optional = expr.optional;

            is_same_method && !is_optional && is_same_name
        }
        _ => false,
    };

    if l && r {
        Some(if active_condition.property_name == "every" {
            NoUselessLengthCheckDiagnostic::Every(binary_expression_span)
        } else {
            NoUselessLengthCheckDiagnostic::Some(binary_expression_span)
        })
    } else {
        None
    }
}

impl Rule for NoUselessLengthCheck {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let log_expr = match node.kind() {
            AstKind::LogicalExpression(log) => Some(log),
            _ => None,
        };
        if let Some(expr) = log_expr {
            if ![LogicalOperator::And, LogicalOperator::Or].contains(&expr.operator) {
                return;
            }
            let flat_expr = flat_logical_expression(expr);
            for i in 0..flat_expr.len() - 1 {
                if let Some(diag) = is_useless_check(flat_expr[i], flat_expr[i + 1], expr.operator)
                {
                    ctx.diagnostic(diag);
                }
            }
        }
    }
}

fn flat_logical_expression<'a>(node: &'a LogicalExpression<'a>) -> Vec<&'a Expression<'a>> {
    let left = match &node.left.without_parenthesized() {
        Expression::LogicalExpression(le) => {
            if le.operator == node.operator {
                flat_logical_expression(le)
            } else {
                vec![&node.left]
            }
        }
        _ => vec![&node.left],
    };

    let right = match &node.right.without_parenthesized() {
        Expression::LogicalExpression(le) => {
            if le.operator == node.operator {
                flat_logical_expression(le)
            } else {
                vec![&node.right]
            }
        }
        _ => vec![&node.right],
    };

    concat(vec![left, right])
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

    Tester::new_without_config(NoUselessLengthCheck::NAME, pass, fail).test_and_snapshot();
}
