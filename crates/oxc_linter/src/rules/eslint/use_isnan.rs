use oxc_ast::{
    AstKind,
    ast::{BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::BinaryOperator;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn comparison_with_nan(span: Span, operator: BinaryOperator) -> OxcDiagnostic {
    let msg = match operator {
        BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
            "Checking inequality with NaN will always return true"
        }
        BinaryOperator::Equality | BinaryOperator::StrictEquality => {
            "Checking equality with NaN will always return false"
        }
        _ => "Comparison with NaN will always return false",
    };
    OxcDiagnostic::warn(msg)
        .with_help("Use the `isNaN` function to compare with NaN.")
        .with_label(span)
}

fn switch_nan(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Checking `switch` discriminant against NaN will never match")
        .with_help("Use the `isNaN` function instead of the switch.")
        .with_label(span)
}

fn case_nan(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Checking for NaN in `case` clause will never match")
        .with_help("Use the `isNaN` function instead of the switch.")
        .with_label(span)
}

fn index_of_nan(method_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "NaN values will never be found by `Array.prototype.{method_name}`"
    ))
    .with_help("Use the `isNaN` function to check for NaN values.")
    .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
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
    ///
    /// Disallows checking against NaN without using `isNaN()` call.
    ///
    /// ### Why is this bad?
    ///
    /// In JavaScript, NaN is a special value of the Number type.
    /// It’s used to represent any of the “not-a-number” values represented
    /// by the double-precision 64-bit format as specified by the IEEE Standard
    /// for Binary Floating-Point Arithmetic.
    ///
    /// Because NaN is unique in JavaScript by not being equal to anything, including itself,
    /// the results of comparisons to NaN are confusing:
    /// - `NaN === NaN` or `NaN == NaN` evaluate to false
    /// - `NaN !== NaN` or `NaN != NaN` evaluate to true
    ///
    /// Therefore, use `Number.isNaN()` or global `isNaN()` functions to test whether a value is NaN.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// foo == NaN;
    /// foo === NaN;
    /// foo <= NaN;
    /// foo > NaN;
    /// ```
    UseIsnan,
    eslint,
    correctness,
    conditional_fix,
    config = UseIsnan,
);

impl Rule for UseIsnan {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(expr) if expr.operator.is_compare() => {
                if is_nan_identifier(&expr.left) {
                    ctx.diagnostic(comparison_with_nan(expr.left.span(), expr.operator));
                }
                if is_nan_identifier(&expr.right) {
                    ctx.diagnostic(comparison_with_nan(expr.right.span(), expr.operator));
                }
            }
            AstKind::BinaryExpression(expr) if expr.operator.is_equality() => {
                if is_nan_identifier(&expr.left) {
                    ctx.diagnostic_with_fix(
                        comparison_with_nan(expr.left.span(), expr.operator),
                        |fixer| fixer.replace(expr.span, make_equality_fix(true, expr, ctx)),
                    );
                }
                if is_nan_identifier(&expr.right) {
                    ctx.diagnostic_with_fix(
                        comparison_with_nan(expr.right.span(), expr.operator),
                        |fixer| fixer.replace(expr.span, make_equality_fix(false, expr, ctx)),
                    );
                }
            }
            AstKind::SwitchCase(case) if self.enforce_for_switch_case => {
                let Some(test) = &case.test else { return };
                if is_nan_identifier(test) {
                    ctx.diagnostic(case_nan(test.span()));
                }
            }
            AstKind::SwitchStatement(switch) if self.enforce_for_switch_case => {
                if is_nan_identifier(&switch.discriminant) {
                    ctx.diagnostic(switch_nan(switch.discriminant.span()));
                }
            }
            AstKind::CallExpression(call) if self.enforce_for_index_of => {
                // Only check calls with 1 or 2 arguments (standard indexOf/lastIndexOf signature)
                if call.arguments.is_empty() || call.arguments.len() > 2 {
                    return;
                }
                // Match target array prototype methods whose first argument is NaN
                let Some(method) = is_target_callee(&call.callee) else { return };
                if let Some(expr) = call.arguments[0].as_expression()
                    && let Some((span, _)) = get_nan_in_expression(expr)
                {
                    ctx.diagnostic(index_of_nan(method, span));
                }
            }
            _ => (),
        }
    }

    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }
}

fn is_nan_identifier<'a>(expr: &'a Expression<'a>) -> bool {
    let expr = expr.get_inner_expression();
    expr.is_specific_id("NaN") || expr.is_specific_member_access("Number", "NaN")
}

/// Check if an expression evaluates to NaN, handling sequence expressions.
/// Returns the span of the NaN identifier and the expression itself if found.
fn get_nan_in_expression<'a>(expr: &'a Expression<'a>) -> Option<(Span, &'a Expression<'a>)> {
    let expr = expr.get_inner_expression();

    // Handle sequence expressions like (1, NaN) - the result is the last expression
    if let Expression::SequenceExpression(seq) = expr {
        if let Some(last) = seq.expressions.last() {
            let last = last.get_inner_expression();
            if is_nan_identifier(last) {
                return Some((last.span(), last));
            }
        }
        return None;
    }

    if is_nan_identifier(expr) {
        return Some((expr.span(), expr));
    }

    None
}

/// If callee is calling the `indexOf` or `lastIndexOf` function.
fn is_target_callee<'a>(callee: &'a Expression) -> Option<&'a str> {
    const TARGET_METHODS: [&str; 2] = ["indexOf", "lastIndexOf"];
    let callee = callee.get_inner_expression();

    if let Some(expr) = callee.as_member_expression() {
        return expr
            .static_property_name()
            .and_then(|property| TARGET_METHODS.contains(&property).then_some(property));
    }

    if let Expression::ChainExpression(chain) = callee {
        let expr = chain.expression.as_member_expression()?;
        return expr
            .static_property_name()
            .and_then(|property| TARGET_METHODS.contains(&property).then_some(property));
    }

    None
}

fn make_equality_fix<'a>(
    nan_on_left: bool,
    comparison: &BinaryExpression<'a>,
    ctx: &LintContext<'a>,
) -> String {
    let non_nan = if nan_on_left {
        comparison.right.span().source_text(ctx.source_text())
    } else {
        comparison.left.span().source_text(ctx.source_text())
    };

    let maybe_bang = match comparison.operator {
        BinaryOperator::Equality | BinaryOperator::StrictEquality => "",
        BinaryOperator::Inequality | BinaryOperator::StrictInequality => "!",
        _ => unreachable!(),
    };

    format!("{maybe_bang}isNaN({non_nan})")
}

#[test]
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
        ("x === (NaN, 1)", None),
        ("x === (doStuff(), NaN, 1)", None),
        ("x === (doStuff(), Number.NaN, 1)", None),
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
        (
            "switch((NaN, doStuff(), 1)) {}",
            Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        ),
        (
            "switch((Number.NaN, doStuff(), 1)) {}",
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
        ("foo.lastIndexOf(NaN, b, c)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf(a, b)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(NaN, NaN, b)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf(...NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 6 },
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
            "foo.lastIndexOf(Number.NaN, b, c)",
            Some(serde_json::json!([{ "enforceForIndexOf": true }])),
        ),
        (
            "foo.lastIndexOf(Number.NaN, NaN, b)",
            Some(serde_json::json!([{ "enforceForIndexOf": true }])),
        ),
        ("foo.indexOf(...Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 6 },
        ("foo.lastIndexOf(Number.NaN())", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf((NaN, 1))", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf((NaN, 1))", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf((Number.NaN, 1))", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        (
            "foo.lastIndexOf((Number.NaN, 1))",
            Some(serde_json::json!([{ "enforceForIndexOf": true }])),
        ),
    ];

    let fail = vec![
        ("123 == NaN;", None),
        ("123 === NaN;", None),
        (r#"NaN === "abc";"#, None),
        (r#"NaN == "abc";"#, None),
        ("123 != NaN;", None),
        ("123 !== NaN;", None),
        (r#"NaN !== "abc";"#, None),
        (r#"NaN != "abc";"#, None),
        (r#"NaN < "abc";"#, None),
        (r#""abc" < NaN;"#, None),
        (r#"NaN > "abc";"#, None),
        (r#""abc" > NaN;"#, None),
        (r#"NaN <= "abc";"#, None),
        (r#""abc" <= NaN;"#, None),
        (r#"NaN >= "abc";"#, None),
        (r#""abc" >= NaN;"#, None),
        ("123 == Number.NaN;", None),
        ("123 === Number.NaN;", None),
        (r#"Number.NaN === "abc";"#, None),
        (r#"Number.NaN == "abc";"#, None),
        ("123 != Number.NaN;", None),
        ("123 !== Number.NaN;", None),
        (r#"Number.NaN !== "abc";"#, None),
        (r#"Number.NaN != "abc";"#, None),
        (r#"Number.NaN < "abc";"#, None),
        (r#""abc" < Number.NaN;"#, None),
        (r#"Number.NaN > "abc";"#, None),
        (r#""abc" > Number.NaN;"#, None),
        (r#"Number.NaN <= "abc";"#, None),
        (r#""abc" <= Number.NaN;"#, None),
        (r#"Number.NaN >= "abc";"#, None),
        (r#""abc" >= Number.NaN;"#, None),
        ("x === Number?.NaN;", None), // { "ecmaVersion": 2020 },
        ("x !== Number?.NaN;", None), // { "ecmaVersion": 2020 },
        ("x === Number['NaN'];", None),
        (
            "/* just
                adding */ x /* some */ === /* comments */ NaN; // here",
            None,
        ),
        ("(1, 2) === NaN;", None),
        // ("x === (doStuff(), NaN);", None),
        // ("x === (doStuff(), Number.NaN);", None),
        // ("x == (doStuff(), NaN);", None),
        // ("x == (doStuff(), Number.NaN);", None),
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
        // (
        //     "switch((doStuff(), NaN)) {}",
        //     Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        // ),
        // (
        //     "switch((doStuff(), Number.NaN)) {}",
        //     Some(serde_json::json!([{ "enforceForSwitchCase": true }])),
        // ),
        ("foo.indexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.lastIndexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo['indexOf'](NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo[`indexOf`](NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo['lastIndexOf'](NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo().indexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.bar.lastIndexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf?.(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("foo?.indexOf(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("(foo?.indexOf)(NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
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
        ("foo.indexOf?.(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("foo?.indexOf(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("(foo?.indexOf)(Number.NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))),
        ("foo.indexOf((1, NaN))", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("foo.indexOf((1, Number.NaN))", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("foo.lastIndexOf((1, NaN))", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        (
            "foo.lastIndexOf((1, Number.NaN))",
            Some(serde_json::json!([{ "enforceForIndexOf": true }])),
        ), // { "ecmaVersion": 2020 },
        ("foo.indexOf(NaN, 1)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("foo.lastIndexOf(NaN, 1)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("foo.indexOf(NaN, b)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("foo.lastIndexOf(NaN, b)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("foo.indexOf(Number.NaN, b)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        (
            "foo.lastIndexOf(Number.NaN, b)",
            Some(serde_json::json!([{ "enforceForIndexOf": true }])),
        ), // { "ecmaVersion": 2020 },
        ("foo.lastIndexOf(NaN, NaN)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 },
        ("foo.indexOf((1, NaN), 1)", Some(serde_json::json!([{ "enforceForIndexOf": true }]))), // { "ecmaVersion": 2020 }
    ];

    let fix = vec![
        ("1 == NaN", "isNaN(1)", None),
        ("1 === NaN", "isNaN(1)", None),
        ("1 != NaN", "!isNaN(1)", None),
        ("1 !== NaN", "!isNaN(1)", None),
        ("NaN == 'foo'", "isNaN('foo')", None),
        ("NaN === 'foo'", "isNaN('foo')", None),
        ("NaN != 'foo'", "!isNaN('foo')", None),
        ("NaN !== 'foo'", "!isNaN('foo')", None),
        ("1 == Number.NaN", "isNaN(1)", None),
        ("1 === Number.NaN", "isNaN(1)", None),
        ("1 != Number.NaN", "!isNaN(1)", None),
        ("1 !== Number.NaN", "!isNaN(1)", None),
    ];

    Tester::new(UseIsnan::NAME, UseIsnan::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
