use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_node_value_not_dom_node,
};

fn prefer_query_selector_diagnostic(
    good_method: &str,
    bad_method: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `.{good_method}()` over `.{bad_method}()`."))
        .with_help("It's better to use the same method to query DOM elements. This helps keep consistency and it lends itself to future improvements (e.g. more specific selectors).")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferQuerySelector {
    /// When set to `true`, allows using `.getElementById()` and `.getElementsByClassName()` when called with a variable or expression.
    /// This avoids the need to manually compose a CSS selector string, which can be less readable.
    allow_with_variables: bool,
}

fn get_preferred_identifier_name(ident_name: &str) -> Option<&'static str> {
    match ident_name {
        "getElementById" => Some("querySelector"),
        "getElementsByClassName" | "getElementsByTagName" | "getElementsByName" => {
            Some("querySelectorAll")
        }
        _ => None,
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `.querySelector()` over `.getElementById()`. And prefer `.querySelectorAll()`
    /// over `.getElementsByClassName()`, `.getElementsByTagName()`, and `.getElementsByName()`.
    ///
    /// ### Why is this bad?
    ///
    /// - Using `.querySelector()` and `.querySelectorAll()` is more flexible and allows for more specific selectors.
    /// - It's better to use the same method to query DOM elements. This helps keep consistency and it lends itself to future improvements (e.g. more specific selectors).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// document.getElementById('foo');
    /// document.getElementsByClassName('foo bar');
    /// document.getElementsByTagName('main');
    /// document.getElementsByClassName(fn());
    /// document.getElementsByName('foo');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// document.querySelector('#foo');
    /// document.querySelector('.bar');
    /// document.querySelector('main #foo .bar');
    /// document.querySelectorAll('.foo.bar');
    /// document.querySelectorAll('li a');
    /// document.querySelector('li').querySelectorAll('a');
    /// ```
    ///
    /// Examples of **correct** code for this rule with `{ "allowWithVariables": true }`:
    /// ```javascript
    /// document.getElementById(someId);
    /// document.getElementsByClassName(someClass);
    /// document.getElementsByClassName(`${someClass}`);
    /// ```
    PreferQuerySelector,
    unicorn,
    pedantic,
    conditional_fix,
    config = PreferQuerySelector,
    version = "0.0.15",
    short_description = "Prefer `.querySelector()` over `.getElementById()`, and `.querySelectorAll()` over `.getElementsByClassName()` and `.getElementsByTagName()`.",
);

impl Rule for PreferQuerySelector {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional || call_expr.arguments.len() != 1 {
            return;
        }

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        if member_expr.optional()
            || member_expr.is_computed()
            || is_node_value_not_dom_node(member_expr.object())
        {
            return;
        }

        let Some(argument_expr) = call_expr.arguments[0].as_expression() else {
            return;
        };

        let Some((property_span, property_name)) = member_expr.static_property_info() else {
            return;
        };
        let Some(property_name) = property_name.as_str() else { return };

        if self.allow_with_variables
            && matches!(property_name, "getElementById" | "getElementsByClassName")
            && is_non_literal_argument(argument_expr)
        {
            return;
        }

        if let Some(preferred_selector) = get_preferred_identifier_name(property_name) {
            let diagnostic =
                prefer_query_selector_diagnostic(preferred_selector, property_name, property_span);

            if argument_expr.is_null() {
                return ctx.diagnostic_with_fix(diagnostic, |fixer| {
                    fixer.replace(property_span, preferred_selector)
                });
            }

            let literal_value = match argument_expr {
                Expression::StringLiteral(literal) => literal.value.as_str().map(str::trim),
                Expression::TemplateLiteral(literal) => {
                    if literal.expressions.is_empty() {
                        literal
                            .quasis
                            .first()
                            .unwrap()
                            .value
                            .cooked
                            .and_then(JSStr::as_str)
                            .map(str::trim)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(literal_value) = literal_value {
                return ctx.diagnostic_with_fix(diagnostic, |fixer| {
                    if literal_value.is_empty() {
                        return fixer.replace(property_span, preferred_selector);
                    }

                    let source_text = fixer.source_range(argument_expr.span());
                    let quotes_symbol = source_text.chars().next().unwrap();
                    let argument = match property_name {
                        "getElementById" => format!("#{literal_value}"),
                        "getElementsByClassName" => {
                            // `getElementsByClassName` matches elements having ALL the listed
                            // classes, so multiple classes become a compound selector
                            // (`.foo.bar`), not a descendant selector (`.foo .bar`).
                            format!(
                                ".{}",
                                literal_value.split_whitespace().collect::<Vec<_>>().join(".")
                            )
                        }
                        "getElementsByName" => {
                            let inner_quote = if quotes_symbol == '\'' { '"' } else { '\'' };
                            format!("[name={inner_quote}{literal_value}{inner_quote}]")
                        }
                        _ => literal_value.to_string(),
                    };
                    let span = property_span.merge(argument_expr.span());
                    fixer.replace(
                        span,
                        format!("{preferred_selector}({quotes_symbol}{argument}{quotes_symbol}"),
                    )
                });
            }

            // For non-literal arguments, we can still auto-fix `getElementById(id)` -> `querySelector(`#${id}`)
            // Only apply this fix for simple identifiers so we avoid nested template literals
            // and complex expressions like member/call expressions or template literals
            if property_name == "getElementById"
                && matches!(argument_expr, Expression::Identifier(_))
            {
                return ctx.diagnostic_with_fix(diagnostic, |fixer| {
                    let source_text = fixer.source_range(argument_expr.span());
                    let span = property_span.merge(argument_expr.span());
                    fixer.replace(span, format!("{preferred_selector}(`#${{{source_text}}}`"))
                });
            }

            ctx.diagnostic(diagnostic);
        }
    }
}

fn is_non_literal_argument(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::NullLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BinaryExpression(_) => false,
        Expression::TemplateLiteral(template) => {
            !template.expressions.is_empty()
                && template.quasis.iter().all(|quasi| {
                    quasi.value.cooked.is_none_or(|cooked| {
                        cooked.as_str().is_some_and(|cooked| cooked.trim().is_empty())
                    })
                })
        }
        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("new document.getElementById(foo);", None),
        ("getElementById(foo);", None),
        ("document['getElementById'](bar);", None),
        ("document[getElementById](bar);", None),
        ("document.foo(bar);", None),
        ("document.getElementById();", None),
        ("document?.getElementById('foo');", None),
        ("document.getElementById?.('foo');", None),
        (r#"document.getElementsByClassName("foo", "bar");"#, None),
        (r#"document.getElementById(...["id"]);"#, None),
        (r##"document.querySelector("#foo");"##, None),
        (r#"document.querySelector(".bar");"#, None),
        (r#"document.querySelector("main #foo .bar");"#, None),
        (r#"document.querySelectorAll(".foo .bar");"#, None),
        (r#"document.querySelectorAll("li a");"#, None),
        (r#"document.querySelector("li").querySelectorAll("a");"#, None),
        ("document.getElementsByName();", None),
        (
            "document.getElementById(someId);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(someClass);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(fn());",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(`${someClass}`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementById(`${someId}`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementById(obj.id);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementById(someId as string);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
    ];

    let fail = vec![
        (r#"document.getElementById("foo");"#, None),
        (r#"document.getElementsByClassName("foo");"#, None),
        (r#"document.getElementsByClassName("foo bar");"#, None),
        (r#"document.getElementsByTagName("foo");"#, None),
        (r#"document.getElementById("");"#, None),
        ("document.getElementById('foo');", None),
        ("document.getElementsByClassName('foo');", None),
        ("document.getElementsByClassName('foo bar');", None),
        ("document.getElementsByTagName('foo');", None),
        ("document.getElementsByClassName('');", None),
        ("document.getElementById(`foo`);", None),
        ("document.getElementsByClassName(`foo`);", None),
        ("document.getElementsByClassName(`foo bar`);", None),
        ("document.getElementsByTagName(`foo`);", None),
        ("document.getElementsByTagName(``);", None),
        ("document.getElementsByClassName(`${fn()}`);", None),
        ("document.getElementsByClassName(`foo ${undefined}`);", None),
        ("document.getElementsByClassName(null);", None),
        ("document.getElementsByTagName(null);", None),
        ("document.getElementsByClassName(fn());", None),
        (r#"document.getElementsByClassName("foo" + fn());"#, None),
        (r#"document.getElementsByClassName(foo + "bar");"#, None),
        (
            r#"for (const div of document.body.getElementById("id").getElementsByClassName("class")) {
                console.log(div.getElementsByTagName("div"));
            }"#,
            None,
        ),
        ("e.getElementById(3)", None),
        (r#"document.getElementsByName("foo");"#, None),
        ("document.getElementsByName('foo');", None),
        ("document.getElementsByName(`foo`);", None),
        ("document.getElementsByName(`${'foo'}`);", None),
        ("document.getElementsByName(null);", None),
        (r#"document.getElementsByName("");"#, None),
        (r#"document.getElementsByName(foo + "bar");"#, None),
        (r#"document.getElementsByName("multiple name should be fixable");"#, None),
        (
            "document.getElementById(someId);",
            Some(serde_json::json!([{ "allowWithVariables": false }])),
        ),
        (
            r#"document.getElementById("foo");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementById("foo" as string);"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName("foo");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName("foo"!);"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByTagName("foo");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByName("foo");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByTagName(someTag);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByName(someName);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(null);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(`foo`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName(variable + "x");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName("foo" + fn());"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r#"document.getElementsByClassName(foo + "bar");"#,
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementById(`x${someId}`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            "document.getElementsByClassName(`foo ${someClass}`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
        (
            r"document.getElementById(`\uD800${someId}`);",
            Some(serde_json::json!([{ "allowWithVariables": true }])),
        ),
    ];

    let fix = vec![
        ("document.getElementsByTagName('foo');", "document.querySelectorAll('foo');"),
        ("document.getElementsByClassName(`foo bar`);", "document.querySelectorAll(`.foo.bar`);"),
        ("document.getElementsByClassName(null);", "document.querySelectorAll(null);"),
        ("document.getElementsByTagName(`   `);", "document.querySelectorAll(`   `);"),
        ("document.getElementById(123);", "document.getElementById(123);"),
        ("document.getElementById(`id`);", "document.querySelector(`#id`);"),
        ("document.getElementById(obj.id);", "document.getElementById(obj.id);"),
        ("document.getElementById(getId());", "document.getElementById(getId());"),
        ("document.getElementById(`${foo}`);", "document.getElementById(`${foo}`);"),
        ("document.getElementById(searchInputId);", "document.querySelector(`#${searchInputId}`);"),
        (r#"document.getElementsByName("foo");"#, r#"document.querySelectorAll("[name='foo']");"#),
        ("document.getElementsByName('foo');", r#"document.querySelectorAll('[name="foo"]');"#),
        ("document.getElementsByName(`foo`);", "document.querySelectorAll(`[name='foo']`);"),
        ("document.getElementsByName(null);", "document.querySelectorAll(null);"),
        (r#"document.getElementsByName("");"#, r#"document.querySelectorAll("");"#),
        (
            r#"document.getElementsByName("multiple name should be fixable");"#,
            r#"document.querySelectorAll("[name='multiple name should be fixable']");"#,
        ),
        // We do not fix these.
        (
            "document.getElementsByClassName(foo + \"bar\");",
            "document.getElementsByClassName(foo + \"bar\");",
        ),
        ("document.getElementsByClassName(fn());", "document.getElementsByClassName(fn());"),
        ("document.getElementsByName(`${'foo'}`);", "document.getElementsByName(`${'foo'}`);"),
        (
            r#"document.getElementsByName(foo + "bar");"#,
            r#"document.getElementsByName(foo + "bar");"#,
        ),
    ];

    Tester::new(PreferQuerySelector::NAME, PreferQuerySelector::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
