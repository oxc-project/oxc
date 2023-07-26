use oxc_ast::{
    ast::{Argument, ChainElement, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum UseIsnanDiagnostic {
    #[error("eslint(use-isnan): Requires calls to isNaN() when checking for NaN")]
    #[diagnostic(severity(warning), help("Use the isNaN function to compare with NaN."))]
    ComparisonWithNaN(#[label] Span),
    #[error("eslint(use-isnan): Requires calls to isNaN() when checking for NaN")]
    #[diagnostic(
        severity(warning),
        help(
            "'switch(NaN)' can never match a case clause. Use Number.isNaN instead of the switch."
        )
    )]
    SwitchNaN(#[label] Span),
    #[error("eslint(use-isnan): Requires calls to isNaN() when checking for NaN")]
    #[diagnostic(
        severity(warning),
        help("'case NaN' can never match. Use Number.isNaN before the switch.")
    )]
    CaseNaN(#[label] Span),
    #[error("eslint(use-isnan): Requires calls to isNaN() when checking for NaN")]
    #[diagnostic(severity(warning), help("Array prototype method '{0}' cannot find NaN."))]
    IndexOfNaN(&'static str, #[label] Span),
}

#[derive(Debug, Clone)]
pub struct UseIsnan {
    /// Whether to disallow NaN in switch cases and discriminants
    enforce_for_switch_case: bool,
    /// Whether to disallow NaN as arguments of `indexOf` and `lastIndexOf`
    enforce_for_index_of: bool,
}

impl Default for UseIsnan {
    fn default() -> Self {
        Self { enforce_for_switch_case: true, enforce_for_index_of: false }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallows checking against NaN without using isNaN() call.
    ///
    /// ### Why is this bad?
    /// In JavaScript, NaN is a special value of the Number type.
    /// It’s used to represent any of the “not-a-number” values represented
    /// by the double-precision 64-bit format as specified by the IEEE Standard
    /// for Binary Floating-Point Arithmetic.
    ///
    /// Because NaN is unique in JavaScript by not being equal to anything, including itself,
    /// the results of comparisons to NaN are confusing:
    /// - NaN === NaN or NaN == NaN evaluate to false
    /// - NaN !== NaN or NaN != NaN evaluate to true
    ///
    /// Therefore, use Number.isNaN() or global isNaN() functions to test whether a value is NaN.
    ///
    /// ### Example
    /// ```javascript
    /// foo == NaN;
    /// foo === NaN;
    /// foo <= NaN;
    /// foo > NaN;
    /// ```
    UseIsnan,
    correctness,
);

impl Rule for UseIsnan {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(expr)
                if expr.operator.is_compare() || expr.operator.is_equality() =>
            {
                if is_nan_identifier(&expr.left) {
                    ctx.diagnostic(UseIsnanDiagnostic::ComparisonWithNaN(expr.left.span()));
                }
                if is_nan_identifier(&expr.right) {
                    ctx.diagnostic(UseIsnanDiagnostic::ComparisonWithNaN(expr.right.span()));
                }
            }
            AstKind::SwitchCase(case) if self.enforce_for_switch_case => {
                if let Some(test) = &case.test {
                    if is_nan_identifier(test) {
                        ctx.diagnostic(UseIsnanDiagnostic::CaseNaN(test.span()));
                    }
                }
            }
            AstKind::SwitchStatement(switch) if self.enforce_for_switch_case => {
                if is_nan_identifier(&switch.discriminant) {
                    ctx.diagnostic(UseIsnanDiagnostic::SwitchNaN(switch.discriminant.span()));
                }
            }
            AstKind::CallExpression(call) if self.enforce_for_index_of => {
                // Match target array prototype methods whose only argument is NaN
                if let Some(method) = is_target_callee(&call.callee) {
                    if call.arguments.len() == 1 {
                        if let Some(Argument::Expression(expr)) = &call.arguments.first() {
                            if is_nan_identifier(expr) {
                                ctx.diagnostic(UseIsnanDiagnostic::IndexOfNaN(method, expr.span()));
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let (enforce_for_switch_case, enforce_for_index_of) =
            value.get(0).map_or((true, false), |config| {
                (
                    config
                        .get("enforceForSwitchCase")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true),
                    config
                        .get("enforceForIndexOf")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or_default(),
                )
            });

        Self { enforce_for_switch_case, enforce_for_index_of }
    }
}

fn is_nan_identifier<'a>(expr: &'a Expression<'a>) -> bool {
    expr.is_specific_id("NaN") || expr.is_specific_member_access("Number", "NaN")
}

/// If callee is calling the `indexOf` or `lastIndexOf` function.
fn is_target_callee<'a>(callee: &'a Expression<'a>) -> Option<&'static str> {
    const TARGET_METHODS: [&str; 2] = ["indexOf", "lastIndexOf"];
    let callee = callee.get_inner_expression();
    match callee {
        Expression::MemberExpression(expr) => expr.static_property_name().and_then(|property| {
            TARGET_METHODS.iter().find(|method| **method == property).copied()
        }),
        Expression::ChainExpression(chain) => {
            if let ChainElement::MemberExpression(expr) = &chain.expression {
                expr.static_property_name().and_then(|property| {
                    TARGET_METHODS.iter().find(|method| **method == property).copied()
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var x = NaN;", None),
        ("isNaN(NaN) === true;", None),
        ("isNaN(123) !== true;", None),
        ("Number.isNaN(NaN) === true;", None),
        ("Number.isNaN(123) !== true;", None),
        ("foo(NaN + 1);", None),
        ("foo(1 + NaN);", None),
        ("foo(NaN - 1)", None),
        ("foo(1 - NaN)", None),
        ("foo(NaN * 2)", None),
        ("foo(2 * NaN)", None),
        ("foo(NaN / 2)", None),
        ("foo(2 / NaN)", None),
        ("var x; if (x = NaN) { }", None),
        ("var x = Number.NaN;", None),
        ("isNaN(Number.NaN) === true;", None),
        ("Number.isNaN(Number.NaN) === true;", None),
        ("foo(Number.NaN + 1);", None),
        ("foo(1 + Number.NaN);", None),
        ("foo(Number.NaN - 1)", None),
        ("foo(1 - Number.NaN)", None),
        ("foo(Number.NaN * 2)", None),
        ("foo(2 * Number.NaN)", None),
        ("foo(Number.NaN / 2)", None),
        ("foo(2 / Number.NaN)", None),
        ("var x; if (x = Number.NaN) { }", None),
        ("x === Number[NaN];", None),
        (
            "switch(NaN) { case foo: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": false }])),
        ),
        (
            "switch(foo) { case NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": false }])),
        ),
        (
            "switch(NaN) { case NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": false }])),
        ),
        (
            "switch(foo) { case bar: break; case NaN: break; default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": false }])),
        ),
        ("switch(foo) {}", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        (
            "switch(foo) { case bar: NaN; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { default: NaN; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        ("switch(Nan) {}", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        (
            "switch('NaN') { default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        ("switch(foo(NaN)) {}", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        ("switch(foo.NaN) {}", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        (
            "switch(foo) { case Nan: break }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case 'NaN': break }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case foo(NaN): break }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case foo.NaN: break }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case bar: break; case 1: break; default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(Number.NaN) { case foo: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": false }])),
        ),
        (
            "switch(foo) { case Number.NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": false }])),
        ),
        (
            "switch(NaN) { case Number.NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": false }])),
        ),
        (
            "switch(foo) { case bar: break; case Number.NaN: break; default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": false }])),
        ),
        (
            "switch(foo) { case bar: Number.NaN; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { default: Number.NaN; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        ("switch(Number.Nan) {}", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        (
            "switch('Number.NaN') { default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        ("switch(foo(Number.NaN)) {}", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        ("switch(foo.Number.NaN) {}", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        (
            "switch(foo) { case Number.Nan: break }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case 'Number.NaN': break }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case foo(Number.NaN): break }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case foo.Number.NaN: break }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        ("foo.indexOf(NaN)", None),
        ("foo.lastIndexOf(NaN)", None),
        ("foo.indexOf(Number.NaN)", None),
        ("foo.lastIndexOf(Number.NaN)", None),
        ("foo.indexOf(NaN)", Some(serde_json::json!([{}]))),
        ("foo.lastIndexOf(NaN)", Some(serde_json::json!([{}]))),
        ("foo.indexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": false }]))),
        ("foo.lastIndexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": false }]))),
        ("indexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("lastIndexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("new foo.indexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.bar(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.IndexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo[indexOf](NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo[lastIndexOf](NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("indexOf.foo(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf()", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf()", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf(a)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(Nan)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf(a, NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(NaN, b)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf(a, b)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(NaN, NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf(...NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(NaN())", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf(Number.NaN)", Some(serde_json::json!([{}]))),
        ("foo.lastIndexOf(Number.NaN)", Some(serde_json::json!([{}]))),
        ("foo.indexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": false }]))),
        ("foo.lastIndexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": false }]))),
        ("indexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("lastIndexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("new foo.indexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.bar(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.IndexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo[indexOf](Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo[lastIndexOf](Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("indexOf.foo(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(Number.Nan)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf(a, Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        (
            "foo.lastIndexOf(Number.NaN, b)",
            Some(serde_json::json!([{ "enforceForIndexOf": true }])),
        ),
        (
            "foo.lastIndexOf(Number.NaN, NaN)",
            Some(serde_json::json!([{ "enforceForIndexOf": true }])),
        ),
        ("foo.indexOf(...Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(Number.NaN())", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
    ];

    let fail = vec![
        ("123 == NaN;", None),
        ("123 === NaN;", None),
        ("NaN === \"abc\";", None),
        ("NaN == \"abc\";", None),
        ("123 != NaN;", None),
        ("123 !== NaN;", None),
        ("NaN !== \"abc\";", None),
        ("NaN != \"abc\";", None),
        ("NaN < \"abc\";", None),
        ("\"abc\" < NaN;", None),
        ("NaN > \"abc\";", None),
        ("\"abc\" > NaN;", None),
        ("NaN <= \"abc\";", None),
        ("\"abc\" <= NaN;", None),
        ("NaN >= \"abc\";", None),
        ("\"abc\" >= NaN;", None),
        ("123 == Number.NaN;", None),
        ("123 === Number.NaN;", None),
        ("Number.NaN === \"abc\";", None),
        ("Number.NaN == \"abc\";", None),
        ("123 != Number.NaN;", None),
        ("123 !== Number.NaN;", None),
        ("Number.NaN !== \"abc\";", None),
        ("Number.NaN != \"abc\";", None),
        ("Number.NaN < \"abc\";", None),
        ("\"abc\" < Number.NaN;", None),
        ("Number.NaN > \"abc\";", None),
        ("\"abc\" > Number.NaN;", None),
        ("Number.NaN <= \"abc\";", None),
        ("\"abc\" <= Number.NaN;", None),
        ("Number.NaN >= \"abc\";", None),
        ("\"abc\" >= Number.NaN;", None),
        ("x === Number?.NaN;", None),
        ("x === Number['NaN'];", None),
        ("switch(NaN) { case foo: break; }", None),
        ("switch(foo) { case NaN: break; }", None),
        ("switch(NaN) { case foo: break; }", Some(serde_json::json!([{}]))),
        ("switch(foo) { case NaN: break; }", Some(serde_json::json!([{}]))),
        ("switch(NaN) {}", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        (
            "switch(NaN) { case foo: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(NaN) { default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(NaN) { case foo: break; default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        ("switch(foo) { case NaN: }", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        (
            "switch(foo) { case NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case (NaN): break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case bar: break; case NaN: break; default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case bar: case NaN: default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case bar: break; case NaN: break; case baz: break; case NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(NaN) { case NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        ("switch(Number.NaN) { case foo: break; }", None),
        ("switch(foo) { case Number.NaN: break; }", None),
        ("switch(Number.NaN) { case foo: break; }", Some(serde_json::json!([{}]))),
        ("switch(foo) { case Number.NaN: break; }", Some(serde_json::json!([{}]))),
        ("switch(Number.NaN) {}", Some(serde_json::json!([{ "enforceForSwitchCase": true }]))),
        (
            "switch(Number.NaN) { case foo: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(Number.NaN) { default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(Number.NaN) { case foo: break; default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case Number.NaN: }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case Number.NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case (Number.NaN): break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case bar: break; case Number.NaN: break; default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case bar: case Number.NaN: default: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(foo) { case bar: break; case NaN: break; case baz: break; case Number.NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch(Number.NaN) { case Number.NaN: break; }",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        ("foo.indexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo['indexOf'](NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo['lastIndexOf'](NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo().indexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.bar.lastIndexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf?.(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo?.indexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("(foo?.indexOf)(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo['indexOf'](Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        (
            "foo['lastIndexOf'](Number.NaN)",
            Some(serde_json::json!([{ "enforceForIndexOf": true }])),
        ),
        ("foo().indexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        (
            "foo.bar.lastIndexOf(Number.NaN)",
            Some(serde_json::json!([{ "enforceForIndexOf": true }])),
        ),
        ("foo.indexOf?.(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo?.indexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("(foo?.indexOf)(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
    ];

    Tester::new(UseIsnan::NAME, pass, fail).test_and_snapshot();
}
