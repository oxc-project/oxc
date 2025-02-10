use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::{phf_map, phf_set};

use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case, AstNode};

fn role_has_required_aria_props_diagnostic(span: Span, role: &str, props: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{role}` role is missing required aria props `{props}`."))
        .with_help(format!("Add missing aria props `{props}` to the element with `{role}` role."))
        .and_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RoleHasRequiredAriaProps;
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that elements with ARIA roles must have all required attributes
    /// for that role.
    ///
    /// ### Why is this bad?
    ///
    /// Certain ARIA roles require specific attributes to express necessary
    /// semantics for assistive technology.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div role="checkbox" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div role="checkbox" aria-checked="false" />
    /// ```
    RoleHasRequiredAriaProps,
    jsx_a11y,
    correctness
);

static ROLE_TO_REQUIRED_ARIA_PROPS: phf::Map<&'static str, phf::Set<&'static str>> = phf_map! {
    "checkbox" => phf_set!{"aria-checked"},
    "radio" => phf_set!{"aria-checked"},
    "combobox" => phf_set!{"aria-controls", "aria-expanded"},
    "tab" => phf_set!{"aria-selected"},
    "slider" => phf_set!{"aria-valuemax", "aria-valuemin", "aria-valuenow"},
    "scrollbar" => phf_set!{"aria-valuemax", "aria-valuemin", "aria-valuenow", "aria-orientation", "aria-controls"},
    "heading" => phf_set!{"aria-level"},
    "option" => phf_set!{"aria-selected"},
};

impl Rule for RoleHasRequiredAriaProps {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            let Some(role_prop) = has_jsx_prop_ignore_case(jsx_el, "role") else {
                return;
            };
            let JSXAttributeItem::Attribute(attr) = role_prop else {
                return;
            };
            let Some(JSXAttributeValue::StringLiteral(role_values)) = &attr.value else {
                return;
            };
            let roles = role_values.value.split_whitespace();
            for role in roles {
                if let Some(props) = ROLE_TO_REQUIRED_ARIA_PROPS.get(role) {
                    for prop in props {
                        if has_jsx_prop_ignore_case(jsx_el, prop).is_none() {
                            ctx.diagnostic(role_has_required_aria_props_diagnostic(
                                attr.span, role, prop,
                            ));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "MyComponent": "div",
                }
            } }
        })
    }

    let pass = vec![
        ("<Bar baz />", None, None, None),
        ("<div />", None, None, None),
        ("<div></div>", None, None, None),
        ("<div role={role} />", None, None, None),
        ("<div role={role || 'button'} />", None, None, None),
        ("<div role={role || 'foobar'} />", None, None, None),
        ("<div role='row' />", None, None, None),
        (
            "<span role='checkbox' aria-checked='false' aria-labelledby='foo' tabindex='0'></span>",
            None,
            None,
            None,
        ),
        (
            "<input role='checkbox' aria-checked='false' aria-labelledby='foo' tabindex='0' {...props} type='checkbox' />",
            None,
            None,
            None,
        ),
        ("<input type='checkbox' role='switch' />", None, None, None),
        (
            "<MyComponent role='checkbox' aria-checked='false' aria-labelledby='foo' tabindex='0' />",
            None,
            Some(settings()),
            None,
        ),
    ];

    let fail = vec![
        ("<div role='slider' />", None, None, None),
        ("<div role='slider' aria-valuemax />", None, None, None),
        ("<div role='slider' aria-valuemax aria-valuemin />", None, None, None),
        ("<div role='checkbox' />", None, None, None),
        ("<div role='checkbox' checked />", None, None, None),
        ("<div role='checkbox' aria-chcked />", None, None, None),
        ("<span role='checkbox' aria-labelledby='foo' tabindex='0'></span>", None, None, None),
        ("<div role='combobox' />", None, None, None),
        ("<div role='combobox' expanded />", None, None, None),
        ("<div role='combobox' aria-expandd />", None, None, None),
        ("<div role='scrollbar' />", None, None, None),
        ("<div role='scrollbar' aria-valuemax />", None, None, None),
        ("<div role='scrollbar' aria-valuemax aria-valuemin />", None, None, None),
        ("<div role='scrollbar' aria-valuemax aria-valuenow />", None, None, None),
        ("<div role='scrollbar' aria-valuemin aria-valuenow />", None, None, None),
        ("<div role='heading' />", None, None, None),
        ("<div role='option' />", None, None, None),
        ("<MyComponent role='combobox' />", None, Some(settings()), None),
    ];

    Tester::new(RoleHasRequiredAriaProps::NAME, RoleHasRequiredAriaProps::PLUGIN, pass, fail)
        .test_and_snapshot();
}
