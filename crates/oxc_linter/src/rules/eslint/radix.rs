use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, IdentifierReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn missing_parameters(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing parameters.").with_label(span)
}

fn missing_radix(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing radix parameter.")
        .with_help("Add radix parameter `10` for parsing decimal numbers.")
        .with_label(span)
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
    ///
    /// Enforce the consistent use of the radix argument when using `parseInt()`.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `parseInt()` function without specifying the radix can lead to unexpected results.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var num = parseInt("071");      // 57
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var num = parseInt("071", 10);  // 71
    /// ```
    Radix,
    eslint,
    pedantic,
    conditional_fix_dangerous
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
            Expression::Identifier(ident) if Self::is_global_parse_int_ident(ident, ctx) => {
                Self::check_arguments(self, call_expr, ctx);
            }
            Expression::StaticMemberExpression(member_expr)
                if member_expr.property.name == "parseInt" =>
            {
                if let Expression::Identifier(ident) = member_expr.object.without_parentheses()
                    && Self::is_global_number_ident(ident, ctx)
                {
                    Self::check_arguments(self, call_expr, ctx);
                }
            }
            Expression::ChainExpression(chain_expr) => {
                if let Some(member_expr) = chain_expr.expression.as_member_expression()
                    && let Expression::Identifier(ident) = member_expr.object()
                    && member_expr.static_property_name() == Some("parseInt")
                    && Self::is_global_number_ident(ident, ctx)
                {
                    Self::check_arguments(self, call_expr, ctx);
                }
            }
            _ => {}
        }
    }
}

impl Radix {
    fn is_global_number_ident(ident: &IdentifierReference, ctx: &LintContext) -> bool {
        ident.name == "Number" && ctx.is_reference_to_global_variable(ident)
    }

    fn is_global_parse_int_ident(ident: &IdentifierReference, ctx: &LintContext) -> bool {
        ident.name == "parseInt" && ctx.is_reference_to_global_variable(ident)
    }

    fn check_arguments(&self, call_expr: &CallExpression, ctx: &LintContext) {
        match call_expr.arguments.len() {
            0 => ctx.diagnostic(missing_parameters(call_expr.span)),
            1 => {
                if matches!(&self.radix_type, RadixType::Always) {
                    let first_arg = &call_expr.arguments[0];
                    let end = call_expr.span.end;
                    let check_span = Span::new(first_arg.span().start, end);
                    let insert_param = ctx
                        .source_range(check_span)
                        .chars()
                        .find_map(|c| if c == ',' { Some(" 10,") } else { None })
                        .unwrap_or(", 10");

                    ctx.diagnostic_with_dangerous_fix(missing_radix(call_expr.span), |fixer| {
                        fixer.insert_text_before_range(Span::empty(end - 1), insert_param)
                    });
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
        (r#"parseInt("10", 10);"#, None, None),
        (r#"parseInt("10", 2);"#, None, None),
        (r#"parseInt("10", 36);"#, None, None),
        (r#"parseInt("10", 0x10);"#, None, None),
        (r#"parseInt("10", 1.6e1);"#, None, None),
        (r#"parseInt("10", 10.0);"#, None, None),
        (r#"parseInt("10", foo);"#, None, None),
        (r#"Number.parseInt("10", foo);"#, None, None),
        (r#"parseInt("10", 10);"#, Some(json!(["always"])), None),
        (r#"parseInt("10");"#, Some(json!(["as-needed"])), None),
        (r#"parseInt("10", 8);"#, Some(json!(["as-needed"])), None),
        (r#"parseInt("10", foo);"#, Some(json!(["as-needed"])), None),
        ("parseInt", None, None),
        ("Number.foo();", None, None),
        ("Number[parseInt]();", None, None),
        ("class C { #parseInt; foo() { Number.#parseInt(); } }", None, None),
        ("class C { #parseInt; foo() { Number.#parseInt(foo); } }", None, None),
        ("class C { #parseInt; foo() { Number.#parseInt(foo, 'bar'); } }", None, None),
        (
            "class C { #parseInt; foo() { Number.#parseInt(foo, 10); } }",
            Some(json!(["as-needed"])),
            None,
        ),
        ("var parseInt; parseInt();", None, None),
        ("var parseInt; parseInt(foo);", Some(json!(["always"])), None),
        ("var parseInt; parseInt(foo, 10);", Some(json!(["as-needed"])), None),
        ("var Number; Number.parseInt();", None, None),
        ("var Number; Number.parseInt(foo);", Some(json!(["always"])), None),
        ("var Number; Number.parseInt(foo, 10);", Some(json!(["as-needed"])), None),
        // ("/* globals parseInt:off */ parseInt(foo);", Some(json!(["always"]))),
        (
            "Number.parseInt(foo, 10);",
            Some(json!(["as-needed"])),
            Some(serde_json::json!({"globals": {"Number": "off"} })),
        ),
        (r#"function *f(){ yield(Number).parseInt("10", foo) }"#, None, None), // { "ecmaVersion": 6 },
    ];

    let fail = vec![
        ("parseInt();", Some(json!(["as-needed"])), None),
        ("parseInt();", None, None),
        (r#"parseInt("10");"#, None, None),
        (r#"parseInt("10",);"#, None, None),
        (r#"parseInt((0, "10"));"#, None, None),
        (r#"parseInt((0, "10"),);"#, None, None),
        (r#"parseInt("10", null);"#, None, None),
        (r#"parseInt("10", undefined);"#, None, None),
        (r#"parseInt("10", true);"#, None, None),
        (r#"parseInt("10", "foo");"#, None, None),
        (r#"parseInt("10", "123");"#, None, None),
        (r#"parseInt("10", 1);"#, None, None),
        (r#"parseInt("10", 37);"#, None, None),
        (r#"parseInt("10", 10.5);"#, None, None),
        ("Number.parseInt();", None, None),
        ("Number.parseInt();", Some(json!(["as-needed"])), None),
        (r#"Number.parseInt("10");"#, None, None),
        (r#"Number.parseInt("10", 1);"#, None, None),
        (r#"Number.parseInt("10", 37);"#, None, None),
        (r#"Number.parseInt("10", 10.5);"#, None, None),
        (r#"parseInt("10", 10);"#, Some(json!(["as-needed"])), None),
        (r#"parseInt?.("10");"#, None, None),
        (r#"Number.parseInt?.("10");"#, None, None),
        (r#"Number?.parseInt("10");"#, None, None),
        (r#"(Number?.parseInt)("10");"#, None, None),
        ("function *f(){ yield(Number).parseInt() }", None, None), // { "ecmaVersion": 6 },
        ("{ let parseInt; } parseInt();", None, None),
        ("{ let Number; } Number.parseInt();", None, None),
        ("{ let Number; } (Number?.parseInt)();", None, None),
    ];

    let fix = vec![
        ("parseInt(10)", "parseInt(10, 10)", Some(json!(["always"]))),
        ("parseInt(10,)", "parseInt(10, 10,)", Some(json!(["always"]))),
        ("parseInt(10      )", "parseInt(10      , 10)", Some(json!(["always"]))),
        ("parseInt(10,         )", "parseInt(10,          10,)", Some(json!(["always"]))),
        (
            r#"parseInt("123123"     ,       )"#,
            r#"parseInt("123123"     ,        10,)"#,
            Some(json!(["always"])),
        ),
        (r#"Number.parseInt("10")"#, r#"Number.parseInt("10", 10)"#, Some(json!(["always"]))),
        (r#"Number.parseInt("10",)"#, r#"Number.parseInt("10", 10,)"#, Some(json!(["always"]))),
        (r#"Number.parseInt?.("10")"#, r#"Number.parseInt?.("10", 10)"#, Some(json!(["always"]))),
        ("parseInt(10, /** 213123 */)", "parseInt(10, /** 213123 */ 10,)", Some(json!(["always"]))),
    ];

    Tester::new(Radix::NAME, Radix::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
