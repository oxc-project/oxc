use oxc_ast::{
    AstKind,
    ast::{Expression, UnaryOperator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode, ast_util::is_method_call, context::LintContext, rule::Rule, utils::is_same_expression,
};

fn negative_index_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid using a negative index with `Array#with()`.")
        .with_note("`Array#with()` interprets a negative index as an offset from the end.")
        .with_help("Use a non-negative index to make the intended position explicit.")
        .with_label(span)
}

fn length_index_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid using `.length` as the index in `Array#with()`.")
        .with_note("An array's `.length` is one past its last valid index.")
        .with_help("Use `.length - 1` to replace the last element.")
        .with_label(span)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfusingWithIndex {
    Negative,
    Length,
}

#[derive(Debug, Default, Clone)]
pub struct NoConfusingArrayWith;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows confusing uses of [`Array#with()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/with).
    ///
    /// ### Why is this bad?
    ///
    /// `Array#with()` treats a negative index as an offset from the end of the array, unlike
    /// methods such as `slice()` or `splice()`. Using a negative static index is usually a mistake.
    /// Using `.length` as the index always produces `undefined`, since valid indices are
    /// `0 .. length - 1`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// array.with(-1, value);
    /// array.with(array.length, value);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// array.with(array.length - 1, value);
    /// array.with(index, value);
    /// ```
    NoConfusingArrayWith,
    unicorn,
    suspicious,
    version = "next",
    short_description = "Disallow confusing uses of `Array#with()`.",
);

impl Rule for NoConfusingArrayWith {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional || !is_method_call(call_expr, None, Some(&["with"]), Some(1), None) {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        if member_expr.is_computed() || member_expr.optional() {
            return;
        }

        let Some(index_argument) = call_expr.arguments.first().and_then(|arg| arg.as_expression())
        else {
            return;
        };

        let object = member_expr.object();
        let Some(confusing_index) = get_confusing_with_index(index_argument, object, ctx) else {
            return;
        };

        let Some((property_span, _)) = member_expr.static_property_info() else {
            return;
        };

        let diagnostic = match confusing_index {
            ConfusingWithIndex::Negative => negative_index_diagnostic(property_span),
            ConfusingWithIndex::Length => length_index_diagnostic(property_span),
        };
        ctx.diagnostic(diagnostic);
    }
}

fn get_confusing_with_index<'a>(
    index: &Expression<'a>,
    object: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> Option<ConfusingWithIndex> {
    let index = index.without_parentheses();

    if let Some(value) = get_static_number_value(index)
        && value.is_finite()
        && value.trunc() < 0.0
    {
        return Some(ConfusingWithIndex::Negative);
    }

    if is_length_member_for(index, object, ctx) {
        return Some(ConfusingWithIndex::Length);
    }

    None
}

fn get_static_number_value(expression: &Expression) -> Option<f64> {
    let expression = expression.get_inner_expression();

    match expression {
        Expression::NumericLiteral(literal) => Some(literal.value),
        Expression::UnaryExpression(unary)
            if matches!(
                unary.operator,
                UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
            ) =>
        {
            let value = get_static_number_value(&unary.argument)?;
            Some(if unary.operator == UnaryOperator::UnaryNegation { -value } else { value })
        }
        _ => None,
    }
}

fn is_length_member_for<'a>(
    index: &Expression<'a>,
    object: &Expression<'a>,
    ctx: &LintContext<'a>,
) -> bool {
    let index = index.get_inner_expression();
    let object = object.get_inner_expression();

    let Some(member) = index.get_member_expr() else {
        return false;
    };

    if member.is_computed() || member.optional() {
        return false;
    }

    if member.static_property_name() != Some("length") {
        return false;
    }

    is_same_expression(member.object().get_inner_expression(), object.get_inner_expression(), ctx)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "array.with(index, value)",
        "array.with(0, value)",
        "array.with(1, value)",
        "array.with(+1, value)",
        "array.with(-0, value)",
        "array.with(-0.5, value)",
        "array.with(array.length - 1, value)",
        "array.with(otherArray.length, value)",
        "object.items.with(other.items.length, value)",
        "array.with(-1e400, value)",
        "array.with(- -1, value)",
        "array.with(-(-(1)), value)",
        "array.with(index)",
        "array.with(index, value, extra)",
        "array['with'](-1, value)",
        "array?.with(-1, value)",
        "array.with?.(-1, value)",
        "with_(-1, value)",
        "array.with(0 as const, value)",
        // Type assertions are not supported in TSX.
        // "array.with(<number>0, value)",
        " array.with(0, value)",
        "array.with(0!, value)",
        "array.with(0 satisfies number, value)",
    ];

    let fail = vec![
        "array.with(-1, value)",
        "array.with(-2, value)",
        "array.with(-1.5, value)",
        "array.with(- 1, value)",
        "array.with(- - -1, value)",
        "array.with(-(-(-1)), value)",
        "array.with(-1)",
        "array.with(-1, value, extra)",
        "array.with(array.length, value)",
        "array.with(array.length)",
        "array.with(array.length, value, extra)",
        "object.items.with(object.items.length, value)",
        "array.with(-1 as const, value)",
        // Type assertions are not supported in TSX.
        // "array.with(<number>-1, value)",
        "array.with( -1, value)",
        "array.with(-1!, value)",
        "array.with(-1 satisfies number, value)",
        "array.with(array.length as number, value)",
        // Type assertions are not supported in TSX.
        // "array.with(<number>array.length, value)",
        "array.with( array.length, value)",
        "array.with(array.length!, value)",
        "array.with(array.length satisfies number, value)",
        "array.with((array satisfies number[]).length, value)",
        "(array satisfies number[]).with(array.length, value)",
        "object.items.with((object satisfies {items: unknown[]}).items.length, value)",
        "(object satisfies {items: unknown[]}).items.with(object.items.length, value)",
    ];

    Tester::new(NoConfusingArrayWith::NAME, NoConfusingArrayWith::PLUGIN, pass, fail)
        .test_and_snapshot();
}
