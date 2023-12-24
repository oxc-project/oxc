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

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{has_jsx_prop_lowercase, parse_jsx_value},
    AstNode,
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsx-a11y(no-aria-hidden-on-focusable): `aria-hidden` must not be true on focusable elements.")]
#[diagnostic(severity(warning), help("Remove `aria-hidden=\"true\"` from focusable elements or modify the element to be not focusable."))]
struct NoAriaHiddenOnFocusableDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoAriaHiddenOnFocusable;

declare_oxc_lint!(
    /// ### What it does
    /// Enforces that `aria-hidden="true"` is not set on focusable elements.
    ///
    /// ### Why is this bad?
    /// `aria-hidden="true"` on focusable elements can lead to confusion or unexpected behavior for screen reader users.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// <div aria-hidden="true" tabIndex="0" />
    ///
    /// // Good
    /// <div aria-hidden="true" />
    /// ```
    NoAriaHiddenOnFocusable,
    correctness
);

impl Rule for NoAriaHiddenOnFocusable {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else { return };
        if let Some(aria_hidden_prop) = has_jsx_prop_lowercase(jsx_el, "aria-hidden") {
            if is_aria_hidden_true(aria_hidden_prop) && is_focusable(jsx_el) {
                if let JSXAttributeItem::Attribute(boxed_attr) = aria_hidden_prop {
                    ctx.diagnostic(NoAriaHiddenOnFocusableDiagnostic(boxed_attr.span));
                }
            }
        }
    }
}

fn is_aria_hidden_true(attr: &JSXAttributeItem) -> bool {
    match attr {
        JSXAttributeItem::Attribute(attr) => match &attr.value {
            Some(JSXAttributeValue::StringLiteral(s)) if s.value == "true" => true,
            None => true,
            _ => false,
        },
        JSXAttributeItem::SpreadAttribute(_) => false,
    }
}

/// Determines if a JSX element is focusable.
///
/// According to the [W3C's DOM Level 2 HTML specification](https://www.w3.org/TR/DOM-Level-2-HTML/html.html), the elements that are focusable are:
/// - `<a>`, `<area>` with an `href` attribute
/// - `<button>`, `<input>`, `<select>`, `<textarea>` unless they are disabled
/// - Any element with a `tabIndex` of zero or greater
///
/// This function checks the passed `JSXOpeningElement` against these criteria to determine
/// if it is focusable.
///
/// # Arguments
///
/// * `element` - A reference to the JSXOpeningElement to check
///
/// # Returns
///
/// `true` if the element is focusable, `false` otherwise.
fn is_focusable(element: &JSXOpeningElement) -> bool {
    let tag_name = match &element.name {
        JSXElementName::Identifier(JSXIdentifier { name, .. }) => name.as_str(),
        _ => return false,
    };

    if let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_lowercase(element, "tabIndex") {
        if let Some(attr_value) = &attr.value {
            return parse_jsx_value(attr_value).map_or(false, |num| num >= 0.0);
        }
    }

    match tag_name {
        "a" | "area" => has_jsx_prop_lowercase(element, "href").is_some(),
        "button" | "input" | "select" | "textarea" => {
            has_jsx_prop_lowercase(element, "disabled").is_none()
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<div aria-hidden=\"true\" />;",
        "<div onClick={() => void 0} aria-hidden=\"true\" />;",
        "<img aria-hidden=\"true\" />",
        "<a aria-hidden=\"false\" href=\"\" />",
        "<button aria-hidden=\"true\" tabIndex=\"-1\" />",
        "<button />",
        "<a href=\"/\" />",
    ];

    let fail = vec![
        r#"<div aria-hidden="true" tabIndex="0" />;"#,
        r#"<input aria-hidden="true" />;"#,
        r#"<a href="/" aria-hidden="true" />"#,
        r#"<button aria-hidden="true" />"#,
        r#"<textarea aria-hidden="true" />"#,
        r#"<p tabIndex="0" aria-hidden="true">text</p>;"#,
    ];

    Tester::new_without_config(NoAriaHiddenOnFocusable::NAME, pass, fail).test_and_snapshot();
}
