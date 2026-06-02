use oxc_ast::{
    AstKind,
    ast::{Expression, JSXAttributeItem, JSXAttributeValue, JSXOpeningElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    globals::HTML_TAG,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        get_element_type, get_string_literal_prop_value, has_jsx_prop, has_jsx_prop_ignore_case,
    },
};

fn no_autofocus_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `autoFocus` attribute is found here, which can cause usability issues for sighted and non-sighted users.")
        .with_help("Remove the `autoFocus` attribute.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NoAutofocus {
    /// Determines if developer-created components are checked.
    #[serde(rename = "ignoreNonDOM")]
    ignore_non_dom: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that `autoFocus` prop is not used on elements.
    ///
    /// ### Why is this bad?
    ///
    /// Autofocusing elements can cause usability issues for sighted and
    /// non-sighted users alike. It can be disorienting when focus is shifted
    /// without user input and can interfere with assistive technologies.
    /// Users should control when and where focus moves on a page.
    ///
    /// ### Exceptions
    ///
    /// `<dialog>` elements, elements with `role="dialog"`, and elements with the
    /// `popover` attribute (and their descendants) are exempt, since directing
    /// focus into them when opened is the expected behavior.
    /// See [MDN: `<dialog>` accessibility](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/dialog#accessibility).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div autoFocus />
    /// <div autoFocus="true" />
    /// <div autoFocus="false" />
    /// <div autoFocus={undefined} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div />
    /// <dialog><input autoFocus /></dialog>
    /// <div role="dialog"><input autoFocus /></div>
    /// <div popover><input autoFocus /></div>
    /// ```
    NoAutofocus,
    jsx_a11y,
    correctness,
    suggestion,
    config = NoAutofocus,
    version = "0.0.19",
);

impl Rule for NoAutofocus {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(jsx_el) = node.kind() else {
            return;
        };
        let Some(autofocus) = has_jsx_prop(&jsx_el.opening_element, "autoFocus") else {
            return;
        };

        let JSXAttributeItem::Attribute(attr) = autofocus else {
            return;
        };

        if attr.value.as_ref().is_some_and(is_false_attribute_value) {
            return;
        }

        if ctx.nodes().ancestor_kinds(node.id()).any(|kind| {
                matches!(kind, AstKind::JSXElement(ancestor) if is_dialog_or_popover(ctx, &ancestor.opening_element))
            })
        {
            return;
        }

        if self.ignore_non_dom {
            let element_type = get_element_type(ctx, &jsx_el.opening_element);

            if HTML_TAG.contains(element_type.as_ref()) {
                ctx.diagnostic_with_suggestion(no_autofocus_diagnostic(attr.span), |fixer| {
                    fixer.delete(&attr.span)
                });
            }
            return;
        }

        ctx.diagnostic_with_suggestion(no_autofocus_diagnostic(attr.span), |fixer| {
            fixer.delete(&attr.span)
        });
    }
}

fn is_dialog_or_popover<'a>(ctx: &LintContext<'a>, opening: &JSXOpeningElement<'a>) -> bool {
    if get_element_type(ctx, opening) == "dialog" {
        return true;
    }
    if has_jsx_prop_ignore_case(opening, "popover").is_some() {
        return true;
    }
    if let Some(role_attr) = has_jsx_prop_ignore_case(opening, "role")
        && get_string_literal_prop_value(role_attr).is_some_and(|value| value == "dialog")
    {
        return true;
    }
    false
}

fn is_false_attribute_value(value: &JSXAttributeValue) -> bool {
    match value {
        JSXAttributeValue::StringLiteral(string_lit) => string_lit.value == "false",
        JSXAttributeValue::ExpressionContainer(expr) => {
            let Some(expression) = expr.expression.as_expression() else {
                return false;
            };

            match expression.get_inner_expression() {
                Expression::BooleanLiteral(bool_lit) => !bool_lit.value,
                Expression::StringLiteral(string_lit) => string_lit.value == "false",
                Expression::TemplateLiteral(template_lit) => {
                    template_lit.quasis.len() == 1
                        && template_lit.expressions.is_empty()
                        && template_lit.quasis[0]
                            .value
                            .cooked
                            .as_ref()
                            .is_some_and(|cooked| cooked == "false")
                }
                _ => false,
            }
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn components_settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Button": "button",
                }
            } }
        })
    }

    fn ignore_non_dom_schema() -> serde_json::Value {
        serde_json::json!([{
            "ignoreNonDOM": true,
        }])
    }

    let pass = vec![
        ("<div />;", None, None),
        ("<div autofocus />;", None, None),
        (r#"<input autofocus="true" />;"#, None, None),
        ("<Foo bar />", None, None),
        ("<Button />", None, None),
        ("<Foo />", Some(ignore_non_dom_schema()), None),
        ("<Foo />", Some(serde_json::json!([{ "ignoreNonDOM": false }])), None),
        ("<Foo autoFocus />", Some(ignore_non_dom_schema()), None),
        ("<Foo autoFocus='true' />", Some(ignore_non_dom_schema()), None),
        ("<div autoFocus={false} />", None, None),
        ("<div autoFocus={(false)} />", None, None),
        (r#"<div autoFocus={("false")} />"#, None, None),
        ("<div autoFocus={(`false`)} />", None, None),
        (r#"<div autoFocus="false" />"#, None, None),
        ("<Foo autoFocus />", Some(ignore_non_dom_schema()), None),
        ("<div><div autofocus /></div>", Some(ignore_non_dom_schema()), None),
        ("<Button />", None, Some(components_settings())),
        ("<Button />", Some(ignore_non_dom_schema()), Some(components_settings())),
        ("<dialog><div autoFocus /></dialog>", None, None),
        ("<dialog><input autoFocus /></dialog>", None, None),
        (r#"<div role="dialog"><input autoFocus /></div>"#, None, None),
        ("<div popover><input autoFocus /></div>", None, None),
        (r#"<div popover="auto"><input autoFocus /></div>"#, None, None),
        ("<dialog><div><input autoFocus /></div></dialog>", None, None),
        (
            r#"<div role="dialog"><section><div><input autoFocus /></div></section></div>"#,
            None,
            None,
        ),
        ("<div popover><section><input autoFocus /></section></div>", None, None),
    ];

    let fail = vec![
        ("<div autoFocus />", None, None),
        ("<div autoFocus={true} />", None, None),
        // the value of ignoreNonDOM should not impact these failing, as div is a dom element.
        ("<div autoFocus={true} />", Some(ignore_non_dom_schema()), None),
        (r#"<div autoFocus={"true"} />"#, Some(ignore_non_dom_schema()), None),
        ("<div autoFocus={`true`} />", Some(ignore_non_dom_schema()), None),
        ("<div autoFocus={(true)} />", Some(ignore_non_dom_schema()), None),
        ("<div autoFocus={true} />", Some(serde_json::json!([{ "ignoreNonDOM": false }])), None),
        ("<div autoFocus={undefined} />", None, None),
        (r#"<div autoFocus="true" />"#, None, None),
        ("<input autoFocus />", None, None),
        ("<Foo autoFocus />", None, None),
        ("<Button autoFocus />", None, Some(components_settings())),
        ("<Button autoFocus />", Some(ignore_non_dom_schema()), Some(components_settings())),
        ("<dialog autoFocus />", None, None),
        (r#"<div role="dialog" autoFocus />"#, None, None),
        ("<div popover autoFocus />", None, None),
        (r#"<div popover="auto" autoFocus />"#, None, None),
    ];

    let fix = vec![
        ("<div autoFocus />", "<div  />"),
        ("<div autoFocus={true} />", "<div  />"),
        ("<div autoFocus='true' />", "<div  />"),
        ("<Button autoFocus='true' />", "<Button  />"),
        ("<input autoFocus />", "<input  />"),
        ("<div autoFocus>foo</div>", "<div >foo</div>"),
        ("<div autoFocus id='lol'>foo</div>", "<div  id='lol'>foo</div>"),
    ];

    Tester::new(NoAutofocus::NAME, NoAutofocus::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
