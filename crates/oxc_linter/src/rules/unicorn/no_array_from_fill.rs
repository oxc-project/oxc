use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, ObjectPropertyKind, PropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

fn no_array_from_fill_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not chain `.fill()` after `Array.from({length: …})`.")
        .with_help("Use the `Array.from(…, mapFunction)` argument to create values directly.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoArrayFromFill;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows `.fill()` chained after `Array.from({length: …})`.
    ///
    /// ### Why is this bad?
    ///
    /// Calling `.fill()` after `Array.from({length: …})` is usually redundant, since
    /// `Array.from(…, mapFunction)` can create each value directly. Additionally,
    /// filling with an object such as `.fill({})` creates a single object shared by
    /// every element, which is rarely intended.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// Array.from({length: 3}).fill(0);
    /// Array.from({length: 3}).fill().map((_, index) => index);
    /// Array.from({length: 3}).fill({});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// Array.from({length: 3}, () => 0);
    /// Array.from({length: 3}, (_, index) => index);
    /// Array.from({length: 3}, () => ({}));
    /// ```
    NoArrayFromFill,
    unicorn,
    suspicious,
    pending,
    version = "next",
    short_description = "Disallow `.fill()` after `Array.from({length: …})`.",
);

impl Rule for NoArrayFromFill {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        // `.fill()` with at most one non-spread argument, no optional chaining
        if !is_method_call(call_expr, None, Some(&["fill"]), None, Some(1))
            || call_expr.optional
            || call_expr.arguments.iter().any(Argument::is_spread)
        {
            return;
        }
        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };
        if member_expr.optional() {
            return;
        }

        let Expression::CallExpression(from_call) = member_expr.object().get_inner_expression()
        else {
            return;
        };
        if !is_array_from_length_call(from_call, ctx) {
            return;
        }

        let Some((property_span, _)) = member_expr.static_property_info() else {
            return;
        };
        ctx.diagnostic(no_array_from_fill_diagnostic(property_span));
    }
}

/// Matches `Array.from({length: …})` where `Array` is a global reference and the
/// sole argument is an object literal with a single non-computed `length` property.
fn is_array_from_length_call(call_expr: &CallExpression, ctx: &LintContext) -> bool {
    if !is_method_call(call_expr, Some(&["Array"]), Some(&["from"]), Some(1), Some(1))
        || call_expr.optional
    {
        return false;
    }

    let Some(member_expr) = call_expr.callee.get_member_expr() else {
        return false;
    };
    if member_expr.optional() {
        return false;
    }
    let Expression::Identifier(ident) = member_expr.object().get_inner_expression() else {
        return false;
    };
    if !ctx.is_reference_to_global_variable(ident) {
        return false;
    }

    let Some(Expression::ObjectExpression(object)) =
        call_expr.arguments[0].as_expression().map(Expression::get_inner_expression)
    else {
        return false;
    };
    if object.properties.len() != 1 {
        return false;
    }
    let ObjectPropertyKind::ObjectProperty(property) = &object.properties[0] else {
        return false;
    };

    !property.computed
        && !property.method
        && property.kind == PropertyKind::Init
        && property.key.is_specific_static_name("length")
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Array.from({length: 3})",
        "Array.from({length: 3}, (_, index) => index)",
        "Array.from({length: 3}).map((_, index) => index)",
        "Array.from(items).fill(0)",
        r#"Array.from({length: 3, 0: "value"}).fill(0)"#,
        "Array.from({...length}).fill(0)",
        r#"Array.from({["length"]: 3}).fill(0)"#,
        "Array.from({length: 3}).fill(0, 1)",
        "Array.from({length: 3}).fill(0, 1, 2)",
        "Array.from({length: 3}).fill(...value)",
        "Array.from?.({length: 3}).fill(0)",
        "Array.from({length: 3})?.fill(0)",
        "Array.from({length: 3}).fill?.(0)",
        "NotArray.from({length: 3}).fill(0)",
        "Array.notFrom({length: 3}).fill(0)",
        "Array.from({length: 3}).slice().fill(0)",
        "const Array = {from() { return {fill() { return {map() {}}; }}; }}; Array.from({length: 3}).fill().map((_, index) => index)",
        "function unicorn(Array) { return Array.from({length: 3}).fill(0); }",
    ];

    let fail = vec![
        "Array.from({length: 3}).fill(0)",
        "Array.from({length: 3}).fill()",
        "Array.from({length}).fill(null)",
        r#"Array.from({"length": 3}).fill(0)"#,
        "Array.from({length: 3}).fill({})",
        "Array.from({length: 3}).fill(0).map((_, index) => index)",
        "Array.from({length: 3}).fill().map((value, index) => index)",
        "Array.from({length: 3}).fill(0).flatMap((_, index) => [index])",
        "Array.from({length: 3}).fill().flatMap(value => [value])",
        "Array.from({length: 3}).fill(0).filter(Boolean)",
        "Array.from({length: 3})
                .fill(0)
                .map((_, index) => index);",
        "Array.from(
                {length: 3}
            )
                .fill(0)
                .map((_, index) => index);",
    ];

    Tester::new(NoArrayFromFill::NAME, NoArrayFromFill::PLUGIN, pass, fail).test_and_snapshot();
}
