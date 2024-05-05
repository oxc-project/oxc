use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(radix): Missing parameters.")]
#[diagnostic(severity(warning))]
struct MissingParametersDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(radix): Missing radix parameter.")]
#[diagnostic(severity(warning))]
struct MissingRadixDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(radix): Redundant radix parameter.")]
#[diagnostic(severity(warning))]
struct RedundantRadixDiagnostic(#[label] pub Span);

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(radix): Invalid radix parameter, must be an integer between 2 and 36.")]
#[diagnostic(severity(warning))]
struct InvalidRadixDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct Radix {
    radix_type: RadixType,
}

// doc: https://github.com/eslint/eslint/blob/main/docs/src/rules/radix.md
// code: https://github.com/eslint/eslint/blob/main/lib/rules/radix.js
// test: https://github.com/eslint/eslint/blob/main/tests/lib/rules/radix.js

declare_oxc_lint!(
    /// ### What it does
    /// Enforce the consistent use of the radix argument when using `parseInt()`.
    ///
    /// ### Why is this bad?
    /// Using the `parseInt()` function without specifying the radix can lead to unexpected results.
    ///
    /// ### Example
    /// ```javascript
    /// // error
    /// var num = parseInt("071");      // 57
    ///
    /// // success
    /// var num = parseInt("071", 10);  // 71
    /// ```
    Radix,
    correctness
);

impl Rule for Radix {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self {
            radix_type: obj
                .and_then(serde_json::Value::as_str)
                .map(RadixType::from)
                .unwrap_or_default(),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind() {
            match &call_expr.callee.without_parenthesized() {
                Expression::Identifier(ident) if ident.name == "parseInt" => {
                    check_arguments(&self.radix_type, call_expr, ctx)
                }
                Expression::StaticMemberExpression(member_expr) => {
                    if let Expression::Identifier(ident) = &member_expr.object {
                        if ident.name == "Number" && member_expr.property.name == "parseInt" {
                            check_arguments(&self.radix_type, call_expr, ctx)
                        }
                    }
                }
                Expression::ChainExpression(chain_expr) => {
                    if let Some(member_expr) = chain_expr.expression.as_member_expression() {
                        if let Expression::Identifier(ident) = &member_expr.object() {
                            if ident.name == "Number"
                                && member_expr.static_property_name() == Some("parseInt")
                            {
                                check_arguments(&self.radix_type, call_expr, ctx)
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
enum RadixType {
    #[default]
    Always,
    AsNeeded,
}

impl RadixType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "as-needed" => Self::AsNeeded,
            _ => Self::Always,
        }
    }
}

fn check_arguments(radix_type: &RadixType, call_expr: &CallExpression, ctx: &LintContext) {
    match call_expr.arguments.len() {
        0 => ctx.diagnostic(MissingParametersDiagnostic(Span::new(
            call_expr.span.start,
            call_expr.span.end,
        ))),
        1 => {
            if matches!(radix_type, RadixType::Always) {
                ctx.diagnostic(MissingRadixDiagnostic(Span::new(
                    call_expr.span.start,
                    call_expr.span.end,
                )));
            }
        }
        _ => {
            if matches!(radix_type, RadixType::AsNeeded)
                && is_default_radix(&call_expr.arguments[1])
            {
                ctx.diagnostic(RedundantRadixDiagnostic(Span::new(
                    call_expr.span.start,
                    call_expr.span.end,
                )));
            } else if !is_valid_radix(&call_expr.arguments[1]) {
                ctx.diagnostic(InvalidRadixDiagnostic(Span::new(
                    call_expr.span.start,
                    call_expr.span.end,
                )));
            }
        }
    }
}

fn is_default_radix(node: &Argument) -> bool {
    node.to_expression().is_specific_raw_number_literal("10")
}

fn is_valid_radix(node: &Argument) -> bool {
    let expr = node.to_expression();

    if let Expression::NumericLiteral(lit) = expr {
        return lit.value.fract() == 0.0 && lit.value >= 2.0 && lit.value <= 36.0;
    }

    if let Expression::Identifier(_) = expr {
        return !expr.is_undefined();
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"parseInt("10", 10);"#, None),
        (r#"parseInt("10", 2);"#, None),
        (r#"parseInt("10", 36);"#, None),
        (r#"parseInt("10", 0x10);"#, None),
        (r#"parseInt("10", 1.6e1);"#, None),
        (r#"parseInt("10", 10.0);"#, None),
        (r#"parseInt("10", foo);"#, None),
        (r#"Number.parseInt("10", foo);"#, None),
        (r#"parseInt("10", 10);"#, Some(serde_json::json!(["always"]))),
        (r#"parseInt("10");"#, Some(serde_json::json!(["as-needed"]))),
        (r#"parseInt("10", 8);"#, Some(serde_json::json!(["as-needed"]))),
        (r#"parseInt("10", foo);"#, Some(serde_json::json!(["as-needed"]))),
        ("parseInt", None),
        ("Number.foo();", None),
        ("Number[parseInt]();", None),
        // ("class C { #parseInt; foo() { Number.#parseInt(); } }", None),
        // ("class C { #parseInt; foo() { Number.#parseInt(foo); } }", None),
        // ("class C { #parseInt; foo() { Number.#parseInt(foo, 'bar'); } }", None),
        // (
        //     "class C { #parseInt; foo() { Number.#parseInt(foo, 10); } }",
        //     Some(serde_json::json!(["as-needed"])),
        // ),
        // ("var parseInt; parseInt();", None),
        // ("var parseInt; parseInt(foo);", Some(serde_json::json!(["always"]))),
        // ("var parseInt; parseInt(foo, 10);", Some(serde_json::json!(["as-needed"]))),
        // ("var Number; Number.parseInt();", None),
        // ("var Number; Number.parseInt(foo);", Some(serde_json::json!(["always"]))),
        // ("var Number; Number.parseInt(foo, 10);", Some(serde_json::json!(["as-needed"]))),
        // ("/* globals parseInt:off */ parseInt(foo);", Some(serde_json::json!(["always"]))),
        // ("Number.parseInt(foo, 10);", Some(serde_json::json!(["as-needed"]))), // { globals: { Number: "off" } }
    ];

    let fail = vec![
        ("parseInt();", Some(serde_json::json!(["as-needed"]))),
        ("parseInt();", None),
        (r#"parseInt("10");"#, None),
        (r#"parseInt("10",);"#, None),
        (r#"parseInt((0, "10"));"#, None),
        (r#"parseInt((0, "10"),);"#, None),
        (r#"parseInt("10", null);"#, None),
        (r#"parseInt("10", undefined);"#, None),
        (r#"parseInt("10", true);"#, None),
        (r#"parseInt("10", "foo");"#, None),
        (r#"parseInt("10", "123");"#, None),
        (r#"parseInt("10", 1);"#, None),
        (r#"parseInt("10", 37);"#, None),
        (r#"parseInt("10", 10.5);"#, None),
        ("Number.parseInt();", None),
        ("Number.parseInt();", Some(serde_json::json!(["as-needed"]))),
        (r#"Number.parseInt("10");"#, None),
        (r#"Number.parseInt("10", 1);"#, None),
        (r#"Number.parseInt("10", 37);"#, None),
        (r#"Number.parseInt("10", 10.5);"#, None),
        (r#"parseInt("10", 10);"#, Some(serde_json::json!(["as-needed"]))),
        (r#"parseInt?.("10");"#, None),
        (r#"Number.parseInt?.("10");"#, None),
        (r#"Number?.parseInt("10");"#, None),
        (r#"(Number?.parseInt)("10");"#, None),
    ];

    Tester::new(Radix::NAME, pass, fail).test_and_snapshot();
}
