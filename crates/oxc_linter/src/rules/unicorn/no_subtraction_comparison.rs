use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, Expression, TSLiteral, TSType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_subtraction_comparison_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Prefer comparing the values directly over comparing the difference with `0`.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSubtractionComparison;

// Ported from:
// <https://github.com/sindresorhus/eslint-plugin-unicorn/blob/v73.0.0/rules/no-subtraction-comparison.js>
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers comparing two values directly instead of subtracting them and
    /// comparing the result with zero.
    ///
    /// ### Why is this bad?
    ///
    /// Subtracting first obscures the comparison's intent and is often left over
    /// from code that previously served as a comparator function.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (a - b > 0) {}
    /// if (0 <= a - b) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (a > b) {}
    /// if (a >= b) {}
    /// ```
    ///
    /// The rule only applies an automatic fix when it can establish that the
    /// rewrite preserves numeric comparison semantics. Otherwise, it offers an
    /// editor suggestion. Comparisons that contain comments are only reported.
    NoSubtractionComparison,
    unicorn,
    nursery,
    conditional_fix_suggestion,
    version = "next",
    short_description = "Prefer comparing values directly over subtracting and comparing to `0`.",
);

impl Rule for NoSubtractionComparison {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(comparison) = node.kind() else {
            return;
        };

        if !(comparison.operator.is_compare() || comparison.operator.is_equality()) {
            return;
        }

        let Some((subtraction, operator)) = get_subtraction_and_operator(comparison) else {
            return;
        };

        let diagnostic = no_subtraction_comparison_diagnostic(comparison.span);
        if ctx.has_comments_between(comparison.span) {
            ctx.diagnostic(diagnostic);
            return;
        }

        let replacement = format!(
            "{} {} {}",
            ctx.source_range(subtraction.left.span()),
            operator.as_str(),
            ctx.source_range(subtraction.right.span())
        );

        let can_fix = if matches!(operator, BinaryOperator::LessThan | BinaryOperator::GreaterThan)
        {
            is_number(&subtraction.left, ctx) && is_number(&subtraction.right, ctx)
        } else {
            is_static_finite_number(&subtraction.left, ctx)
                && is_static_finite_number(&subtraction.right, ctx)
        };

        if can_fix {
            ctx.diagnostic_with_fix(diagnostic, |fixer| {
                fixer.replace(comparison.span, replacement)
            });
        } else {
            ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                fixer.replace(comparison.span, replacement)
            });
        }
    }
}

fn get_subtraction_and_operator<'a>(
    comparison: &'a BinaryExpression<'a>,
) -> Option<(&'a BinaryExpression<'a>, BinaryOperator)> {
    if is_zero(&comparison.right) {
        let subtraction = get_subtraction(&comparison.left)?;
        return Some((subtraction, comparison.operator));
    }

    if is_zero(&comparison.left) {
        let subtraction = get_subtraction(&comparison.right)?;
        return Some((subtraction, invert_operator(comparison.operator)?));
    }

    None
}

fn get_subtraction<'a>(expression: &'a Expression<'a>) -> Option<&'a BinaryExpression<'a>> {
    let Expression::BinaryExpression(binary) = expression.without_parentheses() else {
        return None;
    };
    (binary.operator == BinaryOperator::Subtraction).then_some(binary)
}

fn is_zero(expression: &Expression) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::NumericLiteral(number) if number.value == 0.0
    )
}

fn invert_operator(operator: BinaryOperator) -> Option<BinaryOperator> {
    Some(match operator {
        BinaryOperator::LessThan => BinaryOperator::GreaterThan,
        BinaryOperator::LessEqualThan => BinaryOperator::GreaterEqualThan,
        BinaryOperator::GreaterThan => BinaryOperator::LessThan,
        BinaryOperator::GreaterEqualThan => BinaryOperator::LessEqualThan,
        BinaryOperator::Equality
        | BinaryOperator::Inequality
        | BinaryOperator::StrictEquality
        | BinaryOperator::StrictInequality => operator,
        _ => return None,
    })
}

// Mirrors the syntax-based checks used by eslint-plugin-unicorn's `is-number`
// helper. This deliberately avoids assuming that arbitrary identifiers or
// optional-chain results are numbers.
fn is_number<'a>(expression: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let expression = expression.without_parentheses();
    match expression {
        Expression::NumericLiteral(_) => true,
        Expression::TSAsExpression(as_expression) => {
            is_number_type(&as_expression.type_annotation)
                || is_number(&as_expression.expression, ctx)
        }
        Expression::TSSatisfiesExpression(satisfies) => {
            is_number_type(&satisfies.type_annotation) || is_number(&satisfies.expression, ctx)
        }
        Expression::TSTypeAssertion(assertion) => {
            is_number_type(&assertion.type_annotation) || is_number(&assertion.expression, ctx)
        }
        Expression::TSNonNullExpression(non_null) => is_number(&non_null.expression, ctx),
        Expression::UnaryExpression(unary) => match unary.operator {
            UnaryOperator::UnaryPlus => true,
            UnaryOperator::UnaryNegation | UnaryOperator::BitwiseNot => {
                is_number(&unary.argument, ctx)
            }
            _ => false,
        },
        Expression::BinaryExpression(binary) => match binary.operator {
            BinaryOperator::Addition => {
                is_number(&binary.left, ctx) && is_number(&binary.right, ctx)
            }
            BinaryOperator::ShiftRightZeroFill => true,
            BinaryOperator::Subtraction
            | BinaryOperator::Multiplication
            | BinaryOperator::Division
            | BinaryOperator::Remainder
            | BinaryOperator::Exponential
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::BitwiseOR
            | BinaryOperator::BitwiseXOR
            | BinaryOperator::BitwiseAnd => {
                is_number(&binary.left, ctx) || is_number(&binary.right, ctx)
            }
            _ => false,
        },
        Expression::ConditionalExpression(conditional) => {
            is_number(&conditional.consequent, ctx) && is_number(&conditional.alternate, ctx)
        }
        Expression::SequenceExpression(sequence) => {
            sequence.expressions.last().is_some_and(|last| is_number(last, ctx))
        }
        Expression::CallExpression(call) => is_number_call(call, ctx),
        expression if expression.is_member_expression() => is_number_member(expression, ctx),
        _ => false,
    }
}

fn is_number_call<'a>(call: &oxc_ast::ast::CallExpression<'a>, ctx: &LintContext<'a>) -> bool {
    if call.optional {
        return false;
    }

    if let Expression::Identifier(identifier) = call.callee.without_parentheses()
        && matches!(identifier.name.as_str(), "Number" | "parseInt" | "parseFloat")
        && ctx.is_reference_to_global_variable(identifier)
    {
        return true;
    }

    let Some(member) = call.callee.get_member_expr() else {
        return false;
    };
    if member.optional() {
        return false;
    }

    let Some(property) = member.static_property_name() else {
        return false;
    };
    if is_global_member_object(member.object(), "Math", ctx) {
        return matches!(
            property,
            "abs"
                | "acos"
                | "acosh"
                | "asin"
                | "asinh"
                | "atan"
                | "atanh"
                | "atan2"
                | "cbrt"
                | "ceil"
                | "clz32"
                | "cos"
                | "cosh"
                | "exp"
                | "expm1"
                | "floor"
                | "fround"
                | "hypot"
                | "imul"
                | "log"
                | "log1p"
                | "log10"
                | "log2"
                | "max"
                | "min"
                | "pow"
                | "random"
                | "round"
                | "sign"
                | "sin"
                | "sinh"
                | "sqrt"
                | "tan"
                | "tanh"
                | "trunc"
        );
    }

    is_global_member_object(member.object(), "Number", ctx)
        && matches!(property, "parseFloat" | "parseInt")
}

fn is_number_member<'a>(expression: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let member = expression.to_member_expression();
    if member.optional() {
        return false;
    }

    let Some(property) = member.static_property_name() else {
        return false;
    };
    if property == "length" && !member.is_computed() {
        return true;
    }

    if is_global_member_object(member.object(), "Math", ctx) {
        return matches!(
            property,
            "E" | "LN2" | "LN10" | "LOG2E" | "LOG10E" | "PI" | "SQRT1_2" | "SQRT2"
        );
    }

    is_global_member_object(member.object(), "Number", ctx)
        && matches!(
            property,
            "EPSILON"
                | "MAX_SAFE_INTEGER"
                | "MAX_VALUE"
                | "MIN_SAFE_INTEGER"
                | "MIN_VALUE"
                | "NaN"
                | "NEGATIVE_INFINITY"
                | "POSITIVE_INFINITY"
        )
}

fn is_global_member_object<'a>(object: &Expression<'a>, name: &str, ctx: &LintContext<'a>) -> bool {
    let Expression::Identifier(identifier) = object.without_parentheses() else {
        return false;
    };
    identifier.name == name && ctx.is_reference_to_global_variable(identifier)
}

fn is_number_type(type_annotation: &TSType) -> bool {
    match type_annotation {
        TSType::TSNumberKeyword(_) => true,
        TSType::TSLiteralType(literal) => {
            matches!(literal.literal, TSLiteral::NumericLiteral(_))
        }
        _ => false,
    }
}

fn is_static_finite_number<'a>(expression: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    let expression = expression.without_parentheses();
    match expression {
        Expression::NumericLiteral(number) => number.value.is_finite(),
        Expression::UnaryExpression(unary)
            if matches!(
                unary.operator,
                UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
            ) =>
        {
            is_static_finite_number(&unary.argument, ctx)
        }
        expression if expression.is_member_expression() => {
            let member = expression.to_member_expression();
            let Some(property) = member.static_property_name() else {
                return false;
            };
            (is_global_member_object(member.object(), "Math", ctx)
                && matches!(
                    property,
                    "E" | "LN2" | "LN10" | "LOG2E" | "LOG10E" | "PI" | "SQRT1_2" | "SQRT2"
                ))
                || (is_global_member_object(member.object(), "Number", ctx)
                    && matches!(
                        property,
                        "EPSILON"
                            | "MAX_SAFE_INTEGER"
                            | "MAX_VALUE"
                            | "MIN_SAFE_INTEGER"
                            | "MIN_VALUE"
                    ))
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "a > b",
        "a < b",
        "a === b",
        "a - b",
        "a - b > 1",
        "a - b > c",
        "a - b === c",
        "a + b > 0",
        "a * b > 0",
        "a % b > 0",
        "a > 0",
        "0 > a",
        "0 > 0",
        "a - b > 0n",
        "a - b > -0",
        "a - b instanceof c",
    ];

    let fail = vec![
        "if (a - b > 0) {}",
        "if (a - b >= 0) {}",
        "if (a - b < 0) {}",
        "if (a - b <= 0) {}",
        "a - b === 0",
        "a - b !== 0",
        "a - b == 0",
        "a - b != 0",
        "0 < a - b",
        "0 <= a - b",
        "0 > a - b",
        "0 >= a - b",
        "0 === a - b",
        "0 !== a - b",
        "1 - 2 > 0",
        "foo.length - bar.length > 0",
        "Number(a) - Number(b) < 0",
        "Number.POSITIVE_INFINITY - Number.POSITIVE_INFINITY > 0",
        "1 - 2 >= 0",
        "1 - 1 === 0",
        "0 < foo.length - bar.length",
        "Number.POSITIVE_INFINITY - Number.POSITIVE_INFINITY >= 0",
        "Number.POSITIVE_INFINITY - Number.POSITIVE_INFINITY === 0",
        "foo.length - bar.length >= 0",
        "foo.length - bar > 0",
        "Math.round(a) - Math.round(b) > 0",
        "a.length - b.length - c > 0",
        "(foo.length) - (bar.length) > 0",
        "const modes = new Set(['foo']); modes.clear(); (modes.size ? 1 : 'x') - (modes.size ? 1 : 'x') === 0",
        "const modes = new Set(['foo']); modes.clear(); ((modes.size && 1) || value) - 1 === 0",
        "const object = {value: true}; Object.defineProperty(object, 'value', {get() { return false; }}); (object.value ? 1 : value) - 1 === 0",
        "const modes = new Set(['foo']); modes.clear(); ((modes.size ? 1 : value) as number) - 1 === 0", // {"parser": parsers.typescript},
        "(a - b) > 0",
        "a?.b - c?.d > 0",
        "a - /* comment */ b > 0",
        "(a as number) - (b as number) > 0", // {"parser": parsers.typescript},
        "const alias = condition; var condition = true; (alias ? 1 : value) - 1 === 0",
    ];

    let fix = vec![
        ("1 - 2 > 0", "1 > 2", None),
        ("foo.length - bar.length > 0", "foo.length > bar.length", None),
        ("Number(a) - Number(b) < 0", "Number(a) < Number(b)", None),
        (
            "Number.POSITIVE_INFINITY - Number.POSITIVE_INFINITY > 0",
            "Number.POSITIVE_INFINITY > Number.POSITIVE_INFINITY",
            None,
        ),
        ("1 - 2 >= 0", "1 >= 2", None),
        ("1 - 1 === 0", "1 === 1", None),
        ("0 < foo.length - bar.length", "foo.length > bar.length", None),
        ("Math.round(a) - Math.round(b) > 0", "Math.round(a) > Math.round(b)", None),
        ("(foo.length) - (bar.length) > 0", "(foo.length) > (bar.length)", None),
        ("(a as number) - (b as number) > 0", "(a as number) > (b as number)", None),
    ];

    Tester::new(NoSubtractionComparison::NAME, NoSubtractionComparison::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
