use oxc_ast::{
    AstKind,
    ast::{
        ArrayExpressionElement, BinaryOperator, Expression, UnaryOperator, VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{get_declaration_of_variable, is_new_expression},
    context::LintContext,
    rule::Rule,
    utils::is_same_expression,
};

fn no_duplicate_set_values_diagnostic(span: Span, value: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Remove duplicate value `{value}` from the Set.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateSetValues;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow duplicate values in `Set` constructor array literals.
    ///
    /// ### Why is this bad?
    ///
    /// `Set` values are unique, so repeated static values in a `Set` constructor
    /// array literal are redundant.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// new Set([1, 2, 1]);
    /// new Set(["foo", "bar", "foo"]);
    /// new Set([foo.bar, foo.bar]);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// new Set([1, 2, "1"]);
    /// new Set([{}, {}]);
    /// new Set([foo.bar, foo.baz]);
    /// ```
    NoDuplicateSetValues,
    unicorn,
    suspicious,
    none,
    version = "next",
    short_description = "Disallow duplicate values in `Set` constructor array literals.",
);

impl Rule for NoDuplicateSetValues {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        if !is_new_expression(new_expr, &["Set"], Some(1), Some(1)) {
            return;
        }

        let Some(first_arg) = new_expr.arguments.first() else {
            return;
        };
        let Some(Expression::ArrayExpression(array)) = first_arg.as_expression() else {
            return;
        };

        let mut seen: Vec<Option<&Expression<'a>>> = Vec::with_capacity(array.elements.len());

        for element in &array.elements {
            if matches!(element, ArrayExpressionElement::SpreadElement(_)) {
                continue;
            }

            if matches!(element, ArrayExpressionElement::Elision(_)) {
                if seen.iter().any(|prev| is_duplicate_value(*prev, None, ctx)) {
                    ctx.diagnostic(no_duplicate_set_values_diagnostic(element.span(), "undefined"));
                }
                seen.push(None);
                continue;
            }

            let Some(expr) = element.as_expression() else {
                continue;
            };

            if seen.iter().any(|prev| is_duplicate_value(*prev, Some(expr), ctx)) {
                let text = ctx.source_range(expr.span());
                ctx.diagnostic(no_duplicate_set_values_diagnostic(expr.span(), text));
            }
            seen.push(Some(expr));
        }
    }
}

fn is_duplicate_value<'a>(
    left: Option<&Expression<'a>>,
    right: Option<&Expression<'a>>,
    ctx: &LintContext<'a>,
) -> bool {
    let left_static = comparable_static_value(left, ctx);
    let right_static = comparable_static_value(right, ctx);

    if let (Some(left_static), Some(right_static)) = (left_static, right_static) {
        return same_value_zero(&left_static, &right_static);
    }

    // Upstream: if either side is a Literal (and static comparison failed), do not
    // fall back to reference equality.
    if left.is_some_and(is_literal_like) || right.is_some_and(is_literal_like) {
        return false;
    }

    match (left, right) {
        (Some(left), Some(right)) => {
            is_same_expression(left.get_inner_expression(), right.get_inner_expression(), ctx)
        }
        _ => false,
    }
}

fn is_literal_like(expr: &Expression<'_>) -> bool {
    matches!(
        expr.get_inner_expression(),
        Expression::NullLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
    )
}

#[derive(Debug, Clone, PartialEq)]
enum ComparableStatic<'a> {
    Null,
    Undefined,
    Bool(bool),
    Number(f64),
    String(&'a str),
}

fn same_value_zero(left: &ComparableStatic<'_>, right: &ComparableStatic<'_>) -> bool {
    match (left, right) {
        (ComparableStatic::Null, ComparableStatic::Null)
        | (ComparableStatic::Undefined, ComparableStatic::Undefined) => true,
        (ComparableStatic::Bool(a), ComparableStatic::Bool(b)) => a == b,
        (ComparableStatic::String(a), ComparableStatic::String(b)) => a == b,
        (ComparableStatic::Number(a), ComparableStatic::Number(b)) => {
            if a.is_nan() && b.is_nan() {
                true
            } else {
                // SameValueZero: +0 === -0
                *a == *b
            }
        }
        _ => false,
    }
}

fn comparable_static_value<'a>(
    expr: Option<&Expression<'a>>,
    ctx: &LintContext<'a>,
) -> Option<ComparableStatic<'a>> {
    let Some(expr) = expr else {
        return Some(ComparableStatic::Undefined);
    };
    comparable_static_from_expression(expr.get_inner_expression(), ctx)
}

fn comparable_static_from_expression<'a>(
    expr: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<ComparableStatic<'a>> {
    match expr {
        Expression::NullLiteral(_) => Some(ComparableStatic::Null),
        Expression::BooleanLiteral(b) => Some(ComparableStatic::Bool(b.value)),
        Expression::NumericLiteral(n) => Some(ComparableStatic::Number(n.value)),
        Expression::StringLiteral(s) => Some(ComparableStatic::String(s.value.as_str())),
        Expression::TemplateLiteral(t) => t.single_quasi().map(|q| ComparableStatic::String(q.as_str())),
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation => {
            let Expression::NumericLiteral(n) = unary.argument.get_inner_expression() else {
                return None;
            };
            Some(ComparableStatic::Number(-n.value))
        }
        Expression::BinaryExpression(bin) => {
            let left = comparable_static_from_expression(bin.left.get_inner_expression(), ctx)?;
            let right = comparable_static_from_expression(bin.right.get_inner_expression(), ctx)?;
            match (left, right, bin.operator) {
                (
                    ComparableStatic::Number(a),
                    ComparableStatic::Number(b),
                    BinaryOperator::Addition,
                ) => Some(ComparableStatic::Number(a + b)),
                (
                    ComparableStatic::Number(a),
                    ComparableStatic::Number(b),
                    BinaryOperator::Subtraction,
                ) => Some(ComparableStatic::Number(a - b)),
                (
                    ComparableStatic::Number(a),
                    ComparableStatic::Number(b),
                    BinaryOperator::Multiplication,
                ) => Some(ComparableStatic::Number(a * b)),
                (
                    ComparableStatic::Number(a),
                    ComparableStatic::Number(b),
                    BinaryOperator::Division,
                ) => Some(ComparableStatic::Number(a / b)),
                _ => None,
            }
        }
        Expression::Identifier(ident) => {
            if ident.name == "undefined" {
                return Some(ComparableStatic::Undefined);
            }
            if ident.name == "NaN" {
                return Some(ComparableStatic::Number(f64::NAN));
            }

            let decl = get_declaration_of_variable(ident, ctx)?;
            let AstKind::VariableDeclarator(var_decl) = decl.kind() else {
                return None;
            };
            if var_decl.kind != VariableDeclarationKind::Const {
                return None;
            }
            let init = var_decl.init.as_ref()?;
            comparable_static_from_expression(init.get_inner_expression(), ctx)
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("new Set([1, 2, \"1\"])", None),
        ("new Set([{}, {}])", None),
        ("new Set([[], []])", None),
        ("new Set([/foo/, /foo/])", None),
        ("new Set([Symbol(), Symbol()])", None),
        ("new Set(foo)", None),
        ("Set([1, 1])", None),
        ("new NotSet([1, 1])", None),
        ("new globalThis.Set([1, 1])", None),
        ("new Set([foo(), foo()])", None),
        ("new Set([foo.bar, foo.baz])", None),
        ("new Set([foo, ...bar, baz])", None),
        ("const foo = {}; new Set([foo, {}]);", None),
        ("const foo = 1; const bar = 2; new Set([foo, bar]);", None),
    ];

    let fail = vec![
        ("new Set([1, 2, 1])", None),
        ("new Set([\"foo\", \"bar\", \"foo\"])", None),
        ("new Set([null, null])", None),
        ("new Set([undefined, undefined])", None),
        ("new Set([, undefined])", None),
        ("new Set([undefined, ,])", None),
        ("new Set([,,])", None),
        ("new Set([-1, -1])", None),
        ("new Set([NaN, NaN])", None),
        ("new Set([0, -0])", None),
        ("new Set([1, 1, 2, 2])", None),
        ("new Set([1 + 1, 2])", None),
        ("new Set([`foo`, \"foo\"])", None),
        ("new Set([foo, foo])", None),
        ("new Set([foo.bar, foo.bar])", None),
        ("new Set([foo[\"bar\"], foo.bar])", None),
        ("new Set([this.foo, this.foo])", None),
        ("new Set([foo, ...bar, foo])", None),
        ("const foo = 2; new Set([foo, 2]);", None),
        ("const foo = 'bar'; new Set([foo, 'bar']);", None),
        ("const foo = undefined; new Set([foo, undefined]);", None),
        ("const foo = {}; new Set([foo, foo]);", None),
    ];

    Tester::new(NoDuplicateSetValues::NAME, NoDuplicateSetValues::PLUGIN, pass, fail)
        .test_and_snapshot();
}
