use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn missing_parameters(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing parameters.").with_label(span)
}

fn missing_radix(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing radix parameter.").with_label(span)
}

fn redundant_radix(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Redundant radix parameter.").with_label(span)
}

fn invalid_radix(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid radix parameter, must be an integer between 2 and 36.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct Radix {
    radix_type: RadixType,
}

// doc: https://github.com/eslint/eslint/blob/v9.9.1/docs/src/rules/radix.md
// code: https://github.com/eslint/eslint/blob/v9.9.1/lib/rules/radix.js
// test: https://github.com/eslint/eslint/blob/v9.9.1/tests/lib/rules/radix.js

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
    eslint,
    pedantic
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
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        match call_expr.callee.without_parentheses() {
            Expression::Identifier(ident) => {
                if ident.is_global_reference_name("parseInt", ctx.symbols()) {
                    Self::check_arguments(self, call_expr, ctx);
                }
            }
            Expression::StaticMemberExpression(member_expr) => {
                if let Expression::Identifier(ident) = member_expr.object.without_parentheses() {
                    if ident.is_global_reference_name("Number", ctx.symbols())
                        && member_expr.property.name == "parseInt"
                    {
                        Self::check_arguments(self, call_expr, ctx);
                    }
                }
            }
            Expression::ChainExpression(chain_expr) => {
                if let Some(member_expr) = chain_expr.expression.as_member_expression() {
                    if let Expression::Identifier(ident) = member_expr.object() {
                        if ident.is_global_reference_name("Number", ctx.symbols())
                            && member_expr.static_property_name() == Some("parseInt")
                        {
                            Self::check_arguments(self, call_expr, ctx);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

impl Radix {
    fn check_arguments(&self, call_expr: &CallExpression, ctx: &LintContext) {
        match call_expr.arguments.len() {
            0 => ctx.diagnostic(missing_parameters(call_expr.span)),
            1 => {
                if matches!(&self.radix_type, RadixType::Always) {
                    ctx.diagnostic(missing_radix(call_expr.span));
                }
            }
            _ => {
                let radix_arg = &call_expr.arguments[1];
                if matches!(&self.radix_type, RadixType::AsNeeded) && is_default_radix(radix_arg) {
                    ctx.diagnostic(redundant_radix(radix_arg.span()));
                } else if !is_valid_radix(radix_arg) {
                    ctx.diagnostic(invalid_radix(radix_arg.span()));
                }
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
    use serde_json::json;

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
        (r#"parseInt("10", 10);"#, Some(json!(["always"]))),
        (r#"parseInt("10");"#, Some(json!(["as-needed"]))),
        (r#"parseInt("10", 8);"#, Some(json!(["as-needed"]))),
        (r#"parseInt("10", foo);"#, Some(json!(["as-needed"]))),
        ("parseInt", None),
        ("Number.foo();", None),
        ("Number[parseInt]();", None),
        ("class C { #parseInt; foo() { Number.#parseInt(); } }", None),
        ("class C { #parseInt; foo() { Number.#parseInt(foo); } }", None),
        ("class C { #parseInt; foo() { Number.#parseInt(foo, 'bar'); } }", None),
        ("class C { #parseInt; foo() { Number.#parseInt(foo, 10); } }", Some(json!(["as-needed"]))),
        ("var parseInt; parseInt();", None),
        ("var parseInt; parseInt(foo);", Some(json!(["always"]))),
        ("var parseInt; parseInt(foo, 10);", Some(json!(["as-needed"]))),
        ("var Number; Number.parseInt();", None),
        ("var Number; Number.parseInt(foo);", Some(json!(["always"]))),
        ("var Number; Number.parseInt(foo, 10);", Some(json!(["as-needed"]))),
        // ("/* globals parseInt:off */ parseInt(foo);", Some(json!(["always"]))),
        // ("Number.parseInt(foo, 10);", Some(json!(["as-needed"]))), // { globals: { Number: "off" } }
        (r#"function *f(){ yield(Number).parseInt("10", foo) }"#, None), // { "ecmaVersion": 6 },
    ];

    let fail = vec![
        ("parseInt();", Some(json!(["as-needed"]))),
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
        ("Number.parseInt();", Some(json!(["as-needed"]))),
        (r#"Number.parseInt("10");"#, None),
        (r#"Number.parseInt("10", 1);"#, None),
        (r#"Number.parseInt("10", 37);"#, None),
        (r#"Number.parseInt("10", 10.5);"#, None),
        (r#"parseInt("10", 10);"#, Some(json!(["as-needed"]))),
        (r#"parseInt?.("10");"#, None),
        (r#"Number.parseInt?.("10");"#, None),
        (r#"Number?.parseInt("10");"#, None),
        (r#"(Number?.parseInt)("10");"#, None),
        ("function *f(){ yield(Number).parseInt() }", None), // { "ecmaVersion": 6 },
        ("{ let parseInt; } parseInt();", None),
        ("{ let Number; } Number.parseInt();", None),
        ("{ let Number; } (Number?.parseInt)();", None),
    ];

    Tester::new(Radix::NAME, Radix::PLUGIN, pass, fail).test_and_snapshot();
}
