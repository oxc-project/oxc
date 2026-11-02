use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, Expression, LogicalOperator, MemberExpression, UnaryOperator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::JSStr;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    AstNode, ast_util::get_declaration_of_variable, context::LintContext, rule::Rule,
    utils::is_same_expression,
};

fn no_impossible_length_comparison_diagnostic(
    span: Span,
    property: &str,
    result: bool,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "This comparison is always {result} because `.{property}` is always a non-negative integer."
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoImpossibleLengthComparison;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow impossible comparisons against `.length` or `.size`.
    ///
    /// ### Why is this bad?
    ///
    /// The `.length` of strings and arrays and the `.size` of `Map`s and `Set`s are never
    /// negative. Comparing them against zero or a negative number in a way that can never
    /// change is almost always a mistake, such as checking `length < 0` instead of
    /// `length === 0`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (array.length < 0) {}
    /// if (set.size >= 0) {}
    /// if (string.length === -1) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (array.length === 0) {}
    /// if (set.size > 0) {}
    /// if (string.indexOf(foo) === -1) {}
    /// ```
    NoImpossibleLengthComparison,
    unicorn,
    suspicious,
    version = "next",
    short_description = "Disallow impossible comparisons against `.length` or `.size`.",
);

impl Rule for NoImpossibleLengthComparison {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary) = node.kind() else {
            return;
        };
        let Some((member, operator, value)) = get_comparison(binary) else {
            return;
        };
        let Some(result) = get_constant_result(operator, value) else {
            return;
        };
        let Some(property) = member.static_property_name().and_then(JSStr::as_str) else {
            return;
        };
        if is_excluded_receiver(member, ctx) || has_shape_property_check(node, member, ctx) {
            return;
        }
        ctx.diagnostic(no_impossible_length_comparison_diagnostic(binary.span, property, result));
    }
}

/// Returns the `.length`/`.size` member, the operator with it on the left, and the value.
fn get_comparison<'a, 'b>(
    binary: &'b BinaryExpression<'a>,
) -> Option<(&'b MemberExpression<'a>, BinaryOperator, f64)> {
    let left = binary.left.get_inner_expression();
    let right = binary.right.get_inner_expression();
    if let Some(member) = as_length_or_size(left) {
        return Some((member, binary.operator, get_number(right)?));
    }
    let member = as_length_or_size(right)?;
    let operator = match binary.operator {
        BinaryOperator::LessThan => BinaryOperator::GreaterThan,
        BinaryOperator::LessEqualThan => BinaryOperator::GreaterEqualThan,
        BinaryOperator::GreaterThan => BinaryOperator::LessThan,
        BinaryOperator::GreaterEqualThan => BinaryOperator::LessEqualThan,
        operator => operator,
    };
    Some((member, operator, get_number(left)?))
}

fn as_length_or_size<'a, 'b>(expression: &'b Expression<'a>) -> Option<&'b MemberExpression<'a>> {
    let Expression::StaticMemberExpression(member) = expression else {
        return None;
    };
    if member.optional || !matches!(member.property.name.as_str(), "length" | "size") {
        return None;
    }
    expression.as_member_expression()
}

/// A numeric literal, optionally negated.
fn get_number(expression: &Expression) -> Option<f64> {
    match expression {
        Expression::NumericLiteral(literal) => Some(literal.value),
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation => {
            match unary.argument.get_inner_expression() {
                Expression::NumericLiteral(literal) => Some(-literal.value),
                _ => None,
            }
        }
        _ => None,
    }
}

fn get_constant_result(operator: BinaryOperator, value: f64) -> Option<bool> {
    if !value.is_finite() {
        return None;
    }
    match operator {
        BinaryOperator::LessThan if value <= 0.0 => Some(false),
        BinaryOperator::GreaterEqualThan if value <= 0.0 => Some(true),
        BinaryOperator::LessEqualThan
        | BinaryOperator::StrictEquality
        | BinaryOperator::Equality
            if value < 0.0 =>
        {
            Some(false)
        }
        BinaryOperator::GreaterThan
        | BinaryOperator::StrictInequality
        | BinaryOperator::Inequality
            if value < 0.0 =>
        {
            Some(true)
        }
        _ => None,
    }
}

/// Receivers whose `.length`/`.size` may be anything: class instances seen through
/// `this`/`super`, optional chains, and variables initialized with an object literal.
fn is_excluded_receiver<'a>(member: &MemberExpression<'a>, ctx: &LintContext<'a>) -> bool {
    let mut object = member.object().get_inner_expression();
    loop {
        match object {
            Expression::ThisExpression(_)
            | Expression::Super(_)
            | Expression::ChainExpression(_) => {
                return true;
            }
            Expression::Identifier(ident) => {
                let Some(declaration) = get_declaration_of_variable(ident, ctx.semantic()) else {
                    return false;
                };
                return matches!(
                    declaration.kind(),
                    AstKind::VariableDeclarator(declarator)
                        if declarator.init.as_ref().is_some_and(|init| {
                            matches!(init.get_inner_expression(), Expression::ObjectExpression(_))
                        })
                );
            }
            _ => {
                let Some(inner) = object.as_member_expression() else {
                    return false;
                };
                if inner.optional() {
                    return true;
                }
                object = inner.object().get_inner_expression();
            }
        }
    }
}

/// `a.width && a.length < 0`: an object with a shape whose `length` is a dimension.
fn has_shape_property_check(node: &AstNode, member: &MemberExpression, ctx: &LintContext) -> bool {
    let mut root = None;
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::ParenthesizedExpression(_) => {}
            AstKind::LogicalExpression(logical) if logical.operator == LogicalOperator::And => {
                root = Some(logical);
            }
            _ => break,
        }
    }
    let Some(root) = root else { return false };
    let mut operands = vec![&root.left, &root.right];
    while let Some(operand) = operands.pop() {
        let operand = operand.get_inner_expression();
        if let Expression::LogicalExpression(logical) = operand
            && logical.operator == LogicalOperator::And
        {
            operands.push(&logical.left);
            operands.push(&logical.right);
            continue;
        }
        if let Expression::StaticMemberExpression(shape) = operand
            && !shape.optional
            && matches!(shape.property.name.as_str(), "depth" | "height" | "width")
            && is_same_expression(&shape.object, member.object(), ctx)
        {
            return true;
        }
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "array.length === 0",
        "array.length > 0",
        "array.length >= 1",
        "array.length < 1",
        "array.length <= 0",
        "array.length !== 0",
        "array.length === 1",
        "array.length < foo",
        "array.length < -Infinity",
        "array.length > NaN",
        "array.length < 0n",
        "array.foo < 0",
        "array[length] < 0",
        "array['length'] < 0",
        "array?.length < 0",
        "array.foo?.bar.length < 0",
        "this.length < 0",
        "this.items.size < 0",
        "super.length < 0",
        "array.length + 0",
        "array.length & -1",
        "const foo = {length: -1}; foo.length < 0",
        "const foo = {}; foo.length < 0",
        "rect.width && rect.length < 0",
        "rect.height && foo && rect.length >= 0",
    ];

    let fail = vec![
        "array.length < 0",
        "array.length <= -1",
        "array.length > -1",
        "array.length >= 0",
        "array.length >= -5",
        "array.length === -1",
        "array.length == -1",
        "array.length !== -1",
        "array.length != -1",
        "set.size < 0",
        "map.size >= 0",
        "0 > array.length",
        "-1 < array.length",
        "0 <= array.length",
        "-1 === array.length",
        "(array.length) < (0)",
        "array.length < -0",
        "foo.bar.length < 0",
        "(array as string[]).length < 0",
        "array.length! >= 0",
        "if (string.length === -1) {}",
        "rect.width && other.length < 0",
        "rect.foo && rect.length < 0",
        "rect.width || rect.length < 0",
        "rect.height > 0 && rect.length >= 0",
        "function foo(array) { return array.length < 0 }",
        "const foo = []; foo.length < 0",
    ];

    Tester::new(
        NoImpossibleLengthComparison::NAME,
        NoImpossibleLengthComparison::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
