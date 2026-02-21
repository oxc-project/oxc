use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression, IdentifierReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AstNode, context::LintContext, rule::Rule};

fn missing_parameters(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing parameters.").with_label(span)
}

fn missing_radix(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing radix parameter.")
        .with_help("Add radix parameter `10` for parsing decimal numbers, or specify the appropriate radix for other number formats.")
        .with_label(span)
}

fn invalid_radix(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid radix parameter, must be an integer between 2 and 36.")
        .with_label(span)
}

// `RadixType` has no effect, it is only here for backward compatibility.
// Without it, the linter will report unknown rule configuration error.
#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Radix(RadixType);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RadixType {
    /// Always require the radix parameter when using `parseInt()`.
    #[default]
    #[schemars(skip)]
    Always,
    /// Only require the radix parameter when necessary.
    #[schemars(skip)]
    AsNeeded,
}

// doc: https://github.com/eslint/eslint/blob/v10.0.0/docs/src/rules/radix.md
// code: https://github.com/eslint/eslint/blob/v10.0.0/lib/rules/radix.js
// test: https://github.com/eslint/eslint/blob/v10.0.0/tests/lib/rules/radix.js

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce the consistent use of the radix argument when using `parseInt()`,
    /// which specifies what base to use for parsing the number.
    ///
    /// ### Why is this bad?
    ///
    /// Using the `parseInt()` function without specifying
    /// the radix can lead to unexpected results.
    ///
    /// See the
    /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/parseInt#radix)
    /// for more information on how `parseInt()` handles certain edge-cases.
    ///
    /// ### Configuration
    ///
    /// Note that passing an option to this rule has no effect on its behavior.
    /// In v1.49.0, the config option for this rule was removed and made a no-op.
    /// This matches the behavior change made in ESLint v10, and the rule now
    /// always enforces that a radix parameter is provided to `parseInt()`.
    ///
    /// If you receive new violations due to this change, you may either opt
    /// to disable this rule, or add the radix parameter to all usages of
    /// `parseInt()` in your codebase.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let num = parseInt("071");      // 57
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let num = parseInt("071", 10);  // 71
    /// ```
    Radix,
    eslint,
    pedantic,
    conditional_fix_dangerous,
    config = RadixType,
);

impl Rule for Radix {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        match call_expr.callee.without_parentheses() {
            Expression::Identifier(ident) if Self::is_global_parse_int_ident(ident, ctx) => {
                Self::check_arguments(call_expr, ctx);
            }
            Expression::StaticMemberExpression(member_expr)
                if member_expr.property.name == "parseInt" =>
            {
                if let Expression::Identifier(ident) = member_expr.object.without_parentheses()
                    && Self::is_global_number_ident(ident, ctx)
                {
                    Self::check_arguments(call_expr, ctx);
                }
            }
            Expression::ChainExpression(chain_expr) => {
                if let Some(member_expr) = chain_expr.expression.as_member_expression()
                    && let Expression::Identifier(ident) = member_expr.object()
                    && member_expr.static_property_name() == Some("parseInt")
                    && Self::is_global_number_ident(ident, ctx)
                {
                    Self::check_arguments(call_expr, ctx);
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

    fn check_arguments(call_expr: &CallExpression, ctx: &LintContext) {
        match call_expr.arguments.len() {
            0 => ctx.diagnostic(missing_parameters(call_expr.span)),
            1 => {
                ctx.diagnostic_with_dangerous_fix(missing_radix(call_expr.span), |fixer| {
                    let first_arg = &call_expr.arguments[0];
                    let end = call_expr.span.end;
                    let check_span = Span::new(first_arg.span().start, end);
                    let insert_param = ctx
                        .source_range(check_span)
                        .chars()
                        .find_map(|c| if c == ',' { Some(" 10,") } else { None })
                        .unwrap_or(", 10");
                    fixer.insert_text_before_range(Span::empty(end - 1), insert_param)
                });
            }
            _ => {
                let radix_arg = &call_expr.arguments[1];
                if !is_valid_radix(radix_arg) {
                    ctx.diagnostic(invalid_radix(radix_arg.span()));
                }
            }
        }
    }
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
        (r#"parseInt("10", 10);"#, None, None),
        (r#"parseInt("10", 2);"#, None, None),
        (r#"parseInt("10", 36);"#, None, None),
        (r#"parseInt("10", 0x10);"#, None, None),
        (r#"parseInt("10", 1.6e1);"#, None, None),
        (r#"parseInt("10", 10.0);"#, None, None),
        (r#"parseInt("10", foo);"#, None, None),
        (r#"Number.parseInt("10", foo);"#, None, None),
        ("parseInt", None, None),
        ("Number.foo();", None, None),
        ("Number[parseInt]();", None, None),
        ("class C { #parseInt; foo() { Number.#parseInt(); } }", None, None), // { "ecmaVersion": 2022 },
        ("class C { #parseInt; foo() { Number.#parseInt(foo); } }", None, None), // { "ecmaVersion": 2022 },
        ("class C { #parseInt; foo() { Number.#parseInt(foo, 'bar'); } }", None, None), // { "ecmaVersion": 2022 },
        ("var parseInt; parseInt();", None, None),
        ("var Number; Number.parseInt();", None, None),
        // ("/* globals parseInt:off */ parseInt(foo);", None, None),
        ("Number.parseInt(foo);", None, Some(serde_json::json!({"globals": {"Number": "off"} }))),
        (r#"parseInt("10", 10);"#, Some(serde_json::json!(["always"])), None),
        (r#"parseInt("10", 10);"#, Some(serde_json::json!(["as-needed"])), None),
        (r#"parseInt("10", 8);"#, Some(serde_json::json!(["always"])), None),
        (r#"parseInt("10", 8);"#, Some(serde_json::json!(["as-needed"])), None),
        (r#"parseInt("10", foo);"#, Some(serde_json::json!(["always"])), None),
        (r#"parseInt("10", foo);"#, Some(serde_json::json!(["as-needed"])), None),
    ];

    let fail = vec![
        ("parseInt();", None, None),
        (r#"parseInt("10");"#, None, None),
        (r#"parseInt("10",);"#, None, None), // { "ecmaVersion": 2017 },
        (r#"parseInt((0, "10"));"#, None, None),
        (r#"parseInt((0, "10"),);"#, None, None), // { "ecmaVersion": 2017 },
        (r#"parseInt("10", null);"#, None, None),
        (r#"parseInt("10", undefined);"#, None, None),
        (r#"parseInt("10", true);"#, None, None),
        (r#"parseInt("10", "foo");"#, None, None),
        (r#"parseInt("10", "123");"#, None, None),
        (r#"parseInt("10", 1);"#, None, None),
        (r#"parseInt("10", 37);"#, None, None),
        (r#"parseInt("10", 10.5);"#, None, None),
        ("Number.parseInt();", None, None),
        (r#"Number.parseInt("10");"#, None, None),
        (r#"Number.parseInt("10", 1);"#, None, None),
        (r#"Number.parseInt("10", 37);"#, None, None),
        (r#"Number.parseInt("10", 10.5);"#, None, None),
        (r#"parseInt?.("10");"#, None, None), // { "ecmaVersion": 2020 },
        (r#"Number.parseInt?.("10");"#, None, None), // { "ecmaVersion": 2020 },
        (r#"Number?.parseInt("10");"#, None, None), // { "ecmaVersion": 2020 },
        (r#"(Number?.parseInt)("10");"#, None, None), // { "ecmaVersion": 2020 },
        ("parseInt();", Some(serde_json::json!(["always"])), None),
        ("parseInt();", Some(serde_json::json!(["as-needed"])), None),
        (r#"parseInt("10");"#, Some(serde_json::json!(["always"])), None),
        (r#"parseInt("10");"#, Some(serde_json::json!(["as-needed"])), None),
        (r#"parseInt("10", 1);"#, Some(serde_json::json!(["always"])), None),
        (r#"parseInt("10", 1);"#, Some(serde_json::json!(["as-needed"])), None),
        ("Number.parseInt();", Some(serde_json::json!(["always"])), None),
        ("Number.parseInt();", Some(serde_json::json!(["as-needed"])), None),
    ];

    let fix = vec![
        ("parseInt(10)", "parseInt(10, 10)", Some(serde_json::json!(["always"]))),
        ("parseInt(10,)", "parseInt(10, 10,)", Some(serde_json::json!(["always"]))),
        ("parseInt(10      )", "parseInt(10      , 10)", Some(serde_json::json!(["always"]))),
        (
            "parseInt(10,         )",
            "parseInt(10,          10,)",
            Some(serde_json::json!(["always"])),
        ),
        (
            r#"parseInt("123123"     ,       )"#,
            r#"parseInt("123123"     ,        10,)"#,
            Some(serde_json::json!(["always"])),
        ),
        (
            r#"Number.parseInt("10")"#,
            r#"Number.parseInt("10", 10)"#,
            Some(serde_json::json!(["always"])),
        ),
        (
            r#"Number.parseInt("10",)"#,
            r#"Number.parseInt("10", 10,)"#,
            Some(serde_json::json!(["always"])),
        ),
        (
            r#"Number.parseInt?.("10")"#,
            r#"Number.parseInt?.("10", 10)"#,
            Some(serde_json::json!(["always"])),
        ),
        (
            "parseInt(10, /** 213123 */)",
            "parseInt(10, /** 213123 */ 10,)",
            Some(serde_json::json!(["always"])),
        ),
    ];

    Tester::new(Radix::NAME, Radix::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
