use crate::{
    context::LintContext,
    rule::Rule,
    utils::{has_jsx_prop_lowercase, parse_jsx_value},
    AstNode,
};
use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXElementName, JSXIdentifier, JSXOpeningElement},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsx-a11y(aria-role): `role` must be a valid value.")]
#[diagnostic(severity(warning), help("Modify the `role` attribute to be a valid value."))]
struct AriaRoleDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct AriaRole;
declare_oxc_lint!(
    /// ### What it does
    /// Enforces that `role` attribute is a valid value.
    ///
    /// ### Why is this bad?
    /// An invalid `role` attribute can lead to confusion or unexpected behavior for screen reader users.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <div role="invalidrole" />
    ///
    /// // Good
    /// <div role="button" />
    ///
    AriaRole,
    correctness
);

impl Rule for AriaRole {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else { return };
        if let Some(role_prop) = has_jsx_prop_lowercase(jsx_el, "role") {
            if is_invalid_role(role_prop) {
                if let JSXAttributeItem::Attribute(boxed_attr) = role_prop {
                    ctx.diagnostic(AriaRoleDiagnostic(boxed_attr.span));
                }
            }
        }
    }
}

fn is_invalid_role(attr: &JSXAttributeItem) -> bool {
    let valid_roles = vec![
        "button",
        "checkbox",
        "dialog",
        "gridcell",
        "link",
        "menuitem",
        "menuitemcheckbox",
        "menuitemradio",
        "option",
        "progressbar",
        "radio",
        "scrollbar",
        "searchbox",
        "slider",
        "spinbutton",
        "switch",
        "tab",
        "tabpanel",
        "textbox",
        "treeitem",
        "combobox",
    ];
    match attr {
        JSXAttributeItem::Attribute(attr) => match &attr.value {
            Some(JSXAttributeValue::StringLiteral(s))
                if !valid_roles.contains(&s.value.as_str()) =>
            {
                true
            }
            _ => false,
        },
        JSXAttributeItem::SpreadAttribute(_) => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass =
        vec!["<div role=\"button\" />;", "<div role=\"checkbox\" />;", "<div role=\"dialog\" />;"];
    let fail = vec![r#"<div role="invalidrole" />;"#, r#"<div role="notarole" />;"#];
    Tester::new_without_config(AriaRole::NAME, pass, fail).test_and_snapshot();
}
