use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

fn object_comparison(span: Span, const_result: bool) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected object literal comparison.")
        .with_help(format!(
            "This comparison will always return {const_result:?} as object literals are never equal to each other. Consider using `Object.entries()` of `Object.keys()` and comparing their lengths."
        ))
        .with_label(span)
}

fn array_comparison(span: Span, const_result: bool) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected array literal comparison.")
        .with_help(format!("This comparison will always return {const_result:?} as array literals are never equal to each other. Consider using `Array.length` if empty checking was intended."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct BadObjectLiteralComparison;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks for comparisons between object and array literals.
    ///
    /// ### Why is this bad?
    ///
    /// Comparing a variable to an object or array literal will always return false as object and array literals are never equal to each other.
    ///
    /// If you want to check if an object or array is empty, use `Object.entries()` or `Object.keys()` and their lengths.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (x === {}) { }
    /// if (arr !== []) { }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (typeof x === 'object' && Object.keys(x).length === 0) { }
    /// if (Array.isArray(x) && x.length === 0) { }
    /// ```
    BadObjectLiteralComparison,
    oxc,
    correctness
);

impl Rule for BadObjectLiteralComparison {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary_expression) = node.kind() else {
            return;
        };

        if !matches!(
            binary_expression.operator,
            BinaryOperator::StrictEquality
                | BinaryOperator::StrictInequality
                | BinaryOperator::Equality
                | BinaryOperator::Inequality,
        ) {
            return;
        }

        if is_empty_object_expression(&binary_expression.left)
            || is_empty_object_expression(&binary_expression.right)
        {
            ctx.diagnostic(object_comparison(
                binary_expression.span,
                matches!(
                    binary_expression.operator,
                    BinaryOperator::StrictInequality | BinaryOperator::Inequality
                ),
            ));
        }

        if is_empty_array_expression(&binary_expression.left)
            || is_empty_array_expression(&binary_expression.right)
        {
            ctx.diagnostic(array_comparison(
                binary_expression.span,
                matches!(
                    binary_expression.operator,
                    BinaryOperator::StrictInequality | BinaryOperator::Inequality
                ),
            ));
        }
    }
}

fn is_empty_object_expression(maybe_empty_obj_expr: &Expression) -> bool {
    if let Expression::ObjectExpression(object_expression) = maybe_empty_obj_expr {
        object_expression.properties.is_empty()
    } else {
        false
    }
}

fn is_empty_array_expression(maybe_empty_array_expr: &Expression) -> bool {
    if let Expression::ArrayExpression(array_expression) = maybe_empty_array_expr {
        array_expression.elements.is_empty()
    } else {
        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"if (x === null) { }",
        r"if (typeof arr === 'string') { }",
        r"if (item === 5) { }",
        r"if (data != 'example') { }",
        r"if (typeof person === 'number') { }",
        r"if (list === undefined) { }",
        r"if (response !== true) { }",
        r"if (user == 'admin') { }",
        r"if (typeof product === 'boolean' && product === false) { }",
        r"if (config != 42) { }",
    ];

    let fail = vec![
        r"if (y === {}) { }",
        r"if (arr !== []) { }",
        r"if (typeof item == 'object' && item == {}) { }",
        r"if (data === []) { }",
        r"if (typeof person != 'object' || person != {}) { }",
        r"if (list === {}) { }",
        r"if (typeof response == 'object' && response != {}) { }",
        r"if (user !== []) { }",
        r"if (typeof product == 'object' && product === {}) { }",
        r"if (config != []) { }",
    ];

    Tester::new(BadObjectLiteralComparison::NAME, BadObjectLiteralComparison::PLUGIN, pass, fail)
        .test_and_snapshot();
}
