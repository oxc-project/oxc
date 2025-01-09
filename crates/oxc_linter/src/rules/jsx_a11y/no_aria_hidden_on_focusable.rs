use oxc_ast::{
    ast::{JSXAttributeItem, JSXAttributeValue, JSXOpeningElement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, has_jsx_prop_ignore_case, parse_jsx_value},
    AstNode,
};

fn no_aria_hidden_on_focusable_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`aria-hidden` must not be true on focusable elements.")
        .with_help("Remove `aria-hidden=\"true\"` from focusable elements or modify the element to be not focusable.")
        .with_label(span)
}

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
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <div aria-hidden="true" tabIndex="0" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div aria-hidden="true" />
    /// ```
    NoAriaHiddenOnFocusable,
    jsx_a11y,
    correctness,
    fix
);

impl Rule for NoAriaHiddenOnFocusable {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };
        if let Some(aria_hidden_prop) = has_jsx_prop_ignore_case(jsx_el, "aria-hidden") {
            if is_aria_hidden_true(aria_hidden_prop) && is_focusable(ctx, jsx_el) {
                if let JSXAttributeItem::Attribute(boxed_attr) = aria_hidden_prop {
                    ctx.diagnostic_with_fix(
                        no_aria_hidden_on_focusable_diagnostic(boxed_attr.span),
                        |fixer| fixer.delete(&boxed_attr.span),
                    );
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
fn is_focusable<'a>(ctx: &LintContext<'a>, element: &JSXOpeningElement<'a>) -> bool {
    let tag_name = get_element_type(ctx, element);

    if let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(element, "tabIndex") {
        if let Some(attr_value) = &attr.value {
            return parse_jsx_value(attr_value).is_ok_and(|num| num >= 0.0);
        }
    }

    match tag_name.as_ref() {
        "a" | "area" => has_jsx_prop_ignore_case(element, "href").is_some(),
        "button" | "input" | "select" | "textarea" => {
            has_jsx_prop_ignore_case(element, "disabled").is_none()
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

    let fix = vec![
        (r#"<div aria-hidden="true" tabIndex="0" />;"#, r#"<div  tabIndex="0" />;"#),
        (r#"<input aria-hidden="true" />;"#, "<input  />;"),
        (r#"<a href="/" aria-hidden="true" />"#, r#"<a href="/"  />"#),
        (r#"<button aria-hidden="true" />"#, "<button  />"),
        (r#"<textarea aria-hidden="true" />"#, "<textarea  />"),
        (r#"<p tabIndex="0" aria-hidden="true">text</p>;"#, r#"<p tabIndex="0" >text</p>;"#),
    ];

    Tester::new(NoAriaHiddenOnFocusable::NAME, NoAriaHiddenOnFocusable::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
