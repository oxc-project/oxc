use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeName},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use std::collections::HashSet;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsx-a11y(aria-props):")]
#[diagnostic(severity(warning), help("{1}"))]
struct AriaPropsDiagnostic(#[label] pub Span, String);

#[derive(Debug, Default, Clone)]
pub struct AriaProps;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that elements do not use invalid ARIA attributes.
    ///
    /// ### Why is this bad?
    /// Using invalid ARIA attributes can mislead screen readers and other assistive technologies.
    /// It may cause the accessibility features of the website to fail, making it difficult
    /// for users with disabilities to use the site effectively.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <input aria-labeledby="address_label" />
    ///
    /// // Good
    /// <input aria-labelledby="address_label" />
    /// ```
    AriaProps,
    correctness
);

impl Rule for AriaProps {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let valid_aria_props = create_valid_aria_props();

        if let AstKind::JSXOpeningElement(jsx_el) = node.kind() {
            for attr in &jsx_el.attributes {
                if let JSXAttributeItem::Attribute(attr) = attr {
                    let name = get_attribute_name(&attr.name).to_lowercase();
                    if name.starts_with("aria-") && !valid_aria_props.contains(&name) {
                        let error_message =
                            format!("{}: This attribute is an invalid ARIA attribute.", name);
                        ctx.diagnostic(AriaPropsDiagnostic(attr.span, error_message));
                    }
                }
            }
        }
    }
}

fn get_attribute_name(attr: &JSXAttributeName) -> String {
    match attr {
        JSXAttributeName::Identifier(ident) => ident.name.to_string(),
        JSXAttributeName::NamespacedName(namespaced_name) => {
            format!("{}:{}", namespaced_name.namespace.name, namespaced_name.property.name)
        }
    }
}

/// Creates a set of valid ARIA properties from the WAI-ARIA 1.1 specifications.
///
/// The properties list is based on W3C's WAI-ARIA standards.
/// Reference: <https://www.w3.org/TR/wai-aria/#state_prop_def>
///
/// # Returns
/// `HashSet<String>` containing valid ARIA properties.
fn create_valid_aria_props() -> HashSet<String> {
    let aria_props = [
        "aria-activedescendant",
        "aria-atomic",
        "aria-autocomplete",
        "aria-busy",
        "aria-checked",
        "aria-colcount",
        "aria-colindex",
        "aria-colspan",
        "aria-controls",
        "aria-current",
        "aria-describedby",
        "aria-details",
        "aria-disabled",
        "aria-dropeffect",
        "aria-errormessage",
        "aria-expanded",
        "aria-flowto",
        "aria-grabbed",
        "aria-haspopup",
        "aria-hidden",
        "aria-invalid",
        "aria-keyshortcuts",
        "aria-label",
        "aria-labelledby",
        "aria-level",
        "aria-live",
        "aria-modal",
        "aria-multiline",
        "aria-multiselectable",
        "aria-orientation",
        "aria-owns",
        "aria-placeholder",
        "aria-posinset",
        "aria-pressed",
        "aria-readonly",
        "aria-relevant",
        "aria-required",
        "aria-roledescription",
        "aria-rowcount",
        "aria-rowindex",
        "aria-rowspan",
        "aria-selected",
        "aria-setsize",
        "aria-sort",
        "aria-valuemax",
        "aria-valuemin",
        "aria-valuenow",
        "aria-valuetext",
    ];

    aria_props.into_iter().map(|s| s.to_string()).collect()
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<div />"#,
        r#"<div></div>"#,
        r#"<div aria="wee"></div>"#,
        r#"<div abcARIAdef="true"></div>"#,
        r#"<div fooaria-foobar="true"></div>"#,
        r#"<div fooaria-hidden="true"></div>"#,
        r#"<Bar baz />"#,
        r#"<input type="text" aria-errormessage="foobar" />"#,
    ];

    let fail = vec![
        r#"<div aria-="foobar" />"#,
        r#"<div aria-labeledby="foobar" />"#,
        r#"<div aria-skldjfaria-klajsd="foobar" />"#,
    ];

    Tester::new_without_config(AriaProps::NAME, pass, fail).test_and_snapshot();
}
