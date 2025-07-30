use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case};

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
    /// ### Examples
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

static ROLE_TO_REQUIRED_ARIA_PROPS: &[(&str, &[&str])] = &[
    ("checkbox", &["aria-checked"]),
    ("combobox", &["aria-controls", "aria-expanded"]),
    ("heading", &["aria-level"]),
    ("menuitemcheckbox", &["aria-checked"]),
    ("menuitemradio", &["aria-checked"]),
    ("option", &["aria-selected"]),
    ("radio", &["aria-checked"]),
    (
        "scrollbar",
        &["aria-valuemax", "aria-valuemin", "aria-valuenow", "aria-orientation", "aria-controls"],
    ),
    ("slider", &["aria-valuemax", "aria-valuemin", "aria-valuenow"]),
    ("tab", &["aria-selected"]),
];

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
                if let Some(props) = ROLE_TO_REQUIRED_ARIA_PROPS.iter().find(|r| r.0 == role) {
                    for prop in props.1 {
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
        ("<Bar baz />", None, None),
        ("<div />", None, None),
        ("<div></div>", None, None),
        ("<div role={role} />", None, None),
        ("<div role={role || 'button'} />", None, None),
        ("<div role={role || 'foobar'} />", None, None),
        ("<div role='row' />", None, None),
        (
            "<span role='checkbox' aria-checked='false' aria-labelledby='foo' tabindex='0'></span>",
            None,
            None,
        ),
        (
            "<input role='checkbox' aria-checked='false' aria-labelledby='foo' tabindex='0' {...props} type='checkbox' />",
            None,
            None,
        ),
        ("<input type='checkbox' role='switch' />", None, None),
        (
            "<MyComponent role='checkbox' aria-checked='false' aria-labelledby='foo' tabindex='0' />",
            None,
            Some(settings()),
        ),
        ("<div role='menuitemradio' aria-checked='false' />", None, None),
        ("<div role='menuitemcheckbox' aria-checked='false' />", None, None),
    ];

    let fail = vec![
        ("<div role='slider' />", None, None),
        ("<div role='slider' aria-valuemax />", None, None),
        ("<div role='slider' aria-valuemax aria-valuemin />", None, None),
        ("<div role='checkbox' />", None, None),
        ("<div role='checkbox' checked />", None, None),
        ("<div role='checkbox' aria-chcked />", None, None),
        ("<span role='checkbox' aria-labelledby='foo' tabindex='0'></span>", None, None),
        ("<div role='combobox' />", None, None),
        ("<div role='combobox' expanded />", None, None),
        ("<div role='combobox' aria-expandd />", None, None),
        ("<div role='scrollbar' />", None, None),
        ("<div role='scrollbar' aria-valuemax />", None, None),
        ("<div role='scrollbar' aria-valuemax aria-valuemin />", None, None),
        ("<div role='scrollbar' aria-valuemax aria-valuenow />", None, None),
        ("<div role='scrollbar' aria-valuemin aria-valuenow />", None, None),
        ("<div role='heading' />", None, None),
        ("<div role='option' />", None, None),
        ("<div role='menuitemradio' />", None, None),
        ("<div role='menuitemcheckbox' />", None, None),
        ("<MyComponent role='combobox' />", None, Some(settings())),
    ];

    Tester::new(RoleHasRequiredAriaProps::NAME, RoleHasRequiredAriaProps::PLUGIN, pass, fail)
        .test_and_snapshot();
}
